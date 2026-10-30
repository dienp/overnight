//! Watches Claude Code: running `claude` processes (and how busy their process
//! trees are) and session transcripts under ~/.claude/projects. A transcript
//! write or a busy process tree counts as activity. Transcripts also show when
//! a session stopped on a usage limit and whether it picked up after the reset.

use chrono::{DateTime, Local, NaiveTime, TimeDelta, TimeZone};
use serde::Serialize;
use std::{
    collections::{HashMap, HashSet},
    fs,
    io::{Read, Seek, SeekFrom},
    path::{Path, PathBuf},
    time::{Duration, SystemTime},
};
use sysinfo::{Pid, Process, ProcessRefreshKind, ProcessesToUpdate, System, UpdateKind};

/// CPU across a Claude process tree (percent of one core) that counts as working,
/// e.g. a long build or test run that writes nothing to the transcript.
const BUSY_CPU: f32 = 20.0;
/// How far back the session list looks.
const RECENT: Duration = Duration::from_secs(12 * 3600);
const MAX_SESSIONS: usize = 6;
/// Recent sessions checked for a usage limit, newest first.
const MAX_SCANNED: usize = 30;
/// How long after a reset a session has to pick up before it counts as missed.
pub const RESUME_GRACE: TimeDelta = TimeDelta::minutes(10);
/// Missed resets older than this are old news.
const MISSED_FOR: TimeDelta = TimeDelta::hours(2);

/// For pasting into a Claude session when sessions don't continue after a reset.
pub const FIX_PROMPT: &str = "My Claude Code sessions stop when they hit the usage limit and don't continue after it resets, even though my Mac stays awake overnight. Please check and fix this:
1. In ~/.claude/settings.json, make sure \"autoContinueAtUsageLimit\" isn't false (it defaults to true) and that the \"env\" block has \"CLAUDE_CODE_RETRY_WATCHDOG\": \"1\".
2. Look for project or managed settings that turn either of these off.
3. Check that my Claude Code version is 2.1.234 or later, which auto-continue needs.
Show me each change before you save it.";
const TAIL_BYTES: u64 = 256 * 1024;
const HEAD_BYTES: u64 = 64 * 1024;

#[derive(Clone, Debug, Serialize)]
pub struct Session {
    pub title: String,
    pub project: String,
    pub idle_secs: u64,
    pub active: bool,
    pub limit: Option<LimitState>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum LimitState {
    /// Stopped on a usage limit that hasn't reset yet (or only just has).
    Waiting,
    /// The reset passed a while ago and the session didn't continue,
    /// or the limit has no reset time (e.g. a per-model limit).
    Missed,
}

/// Sessions stopped on a usage limit, summed up.
#[derive(Clone, Debug, Serialize)]
pub struct Limit {
    pub state: LimitState,
    pub resets_at: Option<DateTime<Local>>,
    /// What Claude Code showed, e.g. "You've hit your session limit · resets 1:40pm (Europe/London)".
    pub message: String,
    pub sessions: usize,
}

/// A usage-limit error with no normal reply after it.
#[derive(Clone, Debug)]
struct LimitHit {
    resets_at: Option<DateTime<Local>>,
    message: String,
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct ClaudeInfo {
    pub processes: usize,
    pub busy_cpu: f32,
    pub active: bool,
    /// Seconds since the last transcript write or busy process tree.
    pub idle_secs: Option<u64>,
    pub sessions: Vec<Session>,
    pub limit: Option<Limit>,
    /// `autoContinueAtUsageLimit` from ~/.claude/settings.json (on unless set
    /// to false). The desktop app's own checkbox lives elsewhere.
    pub auto_continue: Option<bool>,
    pub fix_prompt: &'static str,
}

#[derive(Clone)]
struct Meta {
    len: u64,
    title: String,
    project: String,
    limit: Option<LimitHit>,
    last_record: Option<SystemTime>,
}

pub struct Watcher {
    /// None if process info isn't available; activity then comes from transcripts alone.
    sys: Option<System>,
    projects: PathBuf,
    settings: PathBuf,
    meta: HashMap<PathBuf, Meta>,
    last_busy: Option<SystemTime>,
}

impl Watcher {
    pub fn new(home: &Path) -> Self {
        Self {
            sys: System::new().ok(),
            projects: home.join(".claude/projects"),
            settings: home.join(".claude/settings.json"),
            meta: HashMap::new(),
            last_busy: None,
        }
    }

    pub fn poll(&mut self, idle_after: Duration) -> ClaudeInfo {
        let now = SystemTime::now();
        let since = |t: SystemTime| now.duration_since(t).unwrap_or_default();

        let (processes, busy_cpu) = self.processes().unwrap_or_default();
        // Without a running process there is nothing left to wait for.
        let running = self.sys.is_none() || processes > 0;
        if busy_cpu >= BUSY_CPU {
            self.last_busy = Some(now);
        }
        // A session's activity is its newest timestamped record or subagent
        // write, not the file's modified time (see `last_record_time`).
        let mut recent: Vec<(Meta, SystemTime)> = Vec::new();
        for t in transcripts(&self.projects)
            .into_iter()
            .filter(|t| since(t.written) < RECENT)
            .take(MAX_SCANNED)
        {
            let meta = self.describe(&t.path);
            if let Some(at) = meta.last_record.max(t.subagents).filter(|at| since(*at) < RECENT) {
                recent.push((meta, at));
            }
        }
        recent.sort_by(|a, b| b.1.cmp(&a.1));
        let last = recent.first().map(|r| r.1).max(self.last_busy);
        let idle_secs = last.map(|t| since(t).as_secs());
        let active = running && last.is_some_and(|t| since(t) < idle_after);

        let local_now = Local::now();
        let mut sessions = Vec::new();
        let mut hits = Vec::new();
        for (i, (meta, at)) in recent.into_iter().enumerate() {
            let state = meta.limit.as_ref().and_then(|h| limit_state(h, local_now));
            if state.is_some() {
                hits.extend(meta.limit.clone());
            }
            if i < MAX_SESSIONS {
                sessions.push(Session {
                    title: meta.title,
                    project: meta.project,
                    idle_secs: since(at).as_secs(),
                    active: running && since(at) < idle_after,
                    limit: state,
                });
            }
        }
        let limit = summarize_limits(&hits, local_now);

        let auto_continue = auto_continue(&self.settings);
        ClaudeInfo { processes, busy_cpu, active, idle_secs, sessions, limit, auto_continue, fix_prompt: FIX_PROMPT }
    }

    /// Counts Claude Code processes and sums CPU across them and everything
    /// they started (shells, builds, test runners, MCP servers).
    fn processes(&mut self) -> Option<(usize, f32)> {
        let sys = self.sys.as_mut()?;
        sys.refresh_processes_specifics(
            ProcessesToUpdate::All,
            true,
            ProcessRefreshKind::nothing()
                .with_cpu()
                .with_exe(UpdateKind::OnlyIfNotSet)
                .with_cmd(UpdateKind::OnlyIfNotSet),
        );
        let procs = sys.processes();
        let mut children: HashMap<Pid, Vec<Pid>> = HashMap::new();
        for (pid, p) in procs {
            if let Some(parent) = p.parent() {
                children.entry(parent).or_default().push(*pid);
            }
        }
        let roots: Vec<Pid> = procs
            .iter()
            .filter(|(_, p)| is_claude(p))
            .map(|(pid, _)| *pid)
            .collect();

        let mut seen = HashSet::new();
        let mut stack = roots.clone();
        let mut cpu = 0.0;
        while let Some(pid) = stack.pop() {
            if !seen.insert(pid) {
                continue;
            }
            cpu += procs.get(&pid).map_or(0.0, Process::cpu_usage);
            if let Some(kids) = children.get(&pid) {
                stack.extend(kids);
            }
        }
        Some((roots.len(), cpu))
    }

    /// Session title, project folder and any pending usage limit, cached
    /// until the transcript grows.
    fn describe(&mut self, path: &Path) -> Meta {
        let len = fs::metadata(path).map_or(0, |m| m.len());
        if let Some(m) = self.meta.get(path).filter(|m| m.len == len) {
            return m.clone();
        }
        let head = read_range(path, 0, HEAD_BYTES);
        let mut info = TranscriptInfo {
            // Where the session started; later records follow any `cd`.
            cwd: head.lines().find_map(|l| string_field(l, "cwd")),
            ..Default::default()
        };
        let tail = read_range(path, len.saturating_sub(TAIL_BYTES), TAIL_BYTES);
        info.scan(&tail);
        if info.title().is_none() {
            info.scan(&head);
        }
        let id = path.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
        let title = info
            .title()
            .unwrap_or_else(|| format!("Session {}", id.chars().take(8).collect::<String>()));
        let project = info
            .cwd
            .as_deref()
            .and_then(|c| Path::new(c).file_name())
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| {
                path.parent()
                    .and_then(|p| p.file_name())
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_default()
            });
        let meta = Meta {
            len,
            title,
            project,
            limit: unresumed_limit(&tail),
            last_record: last_record_time(&tail),
        };
        self.meta.insert(path.to_path_buf(), meta.clone());
        meta
    }
}

/// `autoContinueAtUsageLimit` defaults to on; None if settings can't be read.
fn auto_continue(settings: &Path) -> Option<bool> {
    let v: serde_json::Value = match fs::read(settings) {
        Ok(bytes) => serde_json::from_slice(&bytes).ok()?,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Some(true),
        Err(_) => return None,
    };
    Some(v.get("autoContinueAtUsageLimit").and_then(|b| b.as_bool()).unwrap_or(true))
}

/// The newest usage-limit stop in a transcript tail, unless a normal reply
/// came after it (the session continued). Two record shapes:
/// - the session gave up: an assistant record with
///   `"isApiErrorMessage":true,"error":"rate_limit"` and the reset as epoch
///   seconds in `quotaLimits.resetsAt`;
/// - the session is waiting it out (CLAUDE_CODE_RETRY_WATCHDOG): a system
///   `api_error` record with `error.rateLimits.resetsAt`.
fn unresumed_limit(tail: &str) -> Option<LimitHit> {
    for line in tail.lines().rev() {
        if line.contains("\"subtype\":\"api_error\"") && line.contains("\"resetsAt\"") {
            let v: serde_json::Value = serde_json::from_str(line).ok()?;
            let resets_at = v
                .pointer("/error/rateLimits/resetsAt")
                .and_then(|r| r.as_i64())
                .and_then(|r| Local.timestamp_opt(r, 0).single());
            if resets_at.is_some() {
                let message = "Waiting for the usage limit to reset, then retrying.".to_string();
                return Some(LimitHit { resets_at, message });
            }
            continue;
        }
        if !line.contains("\"type\":\"assistant\"") {
            continue;
        }
        if !line.contains("\"isApiErrorMessage\":true") {
            return None; // a normal reply: nothing pending
        }
        if !line.contains("\"error\":\"rate_limit\"") {
            continue; // some other API error; keep looking
        }
        let v: serde_json::Value = serde_json::from_str(line).ok()?;
        let message = v
            .pointer("/message/content/0/text")
            .and_then(|t| t.as_str())
            .unwrap_or("Claude hit a usage limit.")
            .to_string();
        let hit_at = v
            .get("timestamp")
            .and_then(|t| t.as_str())
            .and_then(|t| DateTime::parse_from_rfc3339(t).ok())
            .map(|t| t.with_timezone(&Local));
        let resets_at = v
            .pointer("/quotaLimits/resetsAt")
            .and_then(|r| r.as_i64())
            .and_then(|r| Local.timestamp_opt(r, 0).single())
            .or_else(|| hit_at.and_then(|at| reset_from_text(&message, at)));
        return Some(LimitHit { resets_at, message });
    }
    None
}

/// Fallback for records without `quotaLimits`: reads "resets 3am" or
/// "resets 1:40pm" from the message as the next such local time.
fn reset_from_text(message: &str, hit_at: DateTime<Local>) -> Option<DateTime<Local>> {
    let lower = message.to_lowercase();
    let rest = &lower[lower.rfind("resets ")? + "resets ".len()..];
    let token = rest.split_whitespace().next()?;
    let (num, pm) = match (token.strip_suffix("am"), token.strip_suffix("pm")) {
        (Some(n), _) => (n, false),
        (_, Some(n)) => (n, true),
        _ => return None,
    };
    let (h, m) = match num.split_once(':') {
        Some((h, m)) => (h.parse::<u32>().ok()?, m.parse::<u32>().ok()?),
        None => (num.parse::<u32>().ok()?, 0),
    };
    if !(1..=12).contains(&h) {
        return None;
    }
    let time = NaiveTime::from_hms_opt(h % 12 + if pm { 12 } else { 0 }, m, 0)?;
    let today = hit_at.date_naive().and_time(time).and_local_timezone(Local).earliest()?;
    Some(if today > hit_at { today } else { today + TimeDelta::days(1) })
}

fn limit_state(hit: &LimitHit, now: DateTime<Local>) -> Option<LimitState> {
    match hit.resets_at {
        None => Some(LimitState::Missed),
        Some(at) if now < at + RESUME_GRACE => Some(LimitState::Waiting),
        Some(at) if now - at < MISSED_FOR => Some(LimitState::Missed),
        Some(_) => None,
    }
}

/// Waiting sessions win over missed ones; the latest reset is the one to wait for.
fn summarize_limits(hits: &[LimitHit], now: DateTime<Local>) -> Option<Limit> {
    let with_state: Vec<(&LimitHit, LimitState)> =
        hits.iter().filter_map(|h| Some((h, limit_state(h, now)?))).collect();
    let state = if with_state.iter().any(|(_, s)| *s == LimitState::Waiting) {
        LimitState::Waiting
    } else {
        LimitState::Missed
    };
    let pool: Vec<&LimitHit> = with_state.iter().filter(|(_, s)| *s == state).map(|(h, _)| *h).collect();
    let latest = pool.iter().max_by_key(|h| h.resets_at)?;
    Some(Limit { state, resets_at: latest.resets_at, message: latest.message.clone(), sessions: pool.len() })
}

fn is_claude(p: &Process) -> bool {
    let name = p.name().to_string_lossy();
    if name == "claude" {
        return true;
    }
    let exe = p.exe().map(|e| e.to_string_lossy().into_owned()).unwrap_or_default();
    if exe.ends_with("/claude") || exe.contains("/claude/versions/") {
        return true;
    }
    (name == "node" || name == "bun")
        && p.cmd().iter().any(|a| a.to_string_lossy().contains("claude-code"))
}

struct Transcript {
    path: PathBuf,
    /// The latest write to the transcript or its subagents. Only narrows the
    /// search: the desktop app touches transcripts nobody is working in.
    written: SystemTime,
    /// The latest write by one of its subagents (`<session>/subagents/*.jsonl`),
    /// which hold nothing but conversation, so this counts as activity.
    subagents: Option<SystemTime>,
}

/// Session transcripts, most recently written first.
fn transcripts(projects: &Path) -> Vec<Transcript> {
    let mut latest: HashMap<PathBuf, Transcript> = HashMap::new();
    let mut bump = |path: PathBuf, t: SystemTime, sub: bool| {
        let e = latest
            .entry(path.clone())
            .or_insert(Transcript { path, written: t, subagents: None });
        e.written = e.written.max(t);
        if sub {
            e.subagents = e.subagents.max(Some(t));
        }
    };
    let Ok(dirs) = fs::read_dir(projects) else {
        return Vec::new();
    };
    for project in dirs.flatten() {
        let Ok(entries) = fs::read_dir(project.path()) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let Ok(kind) = entry.file_type() else {
                continue;
            };
            if kind.is_file() && path.extension().is_some_and(|e| e == "jsonl") {
                if let Some(t) = mtime(&path) {
                    bump(path, t, false);
                }
            } else if kind.is_dir() {
                let main = path.with_extension("jsonl");
                if !main.is_file() {
                    continue;
                }
                let Ok(subs) = fs::read_dir(path.join("subagents")) else {
                    continue;
                };
                for sub in subs.flatten() {
                    if let Some(t) = mtime(&sub.path()) {
                        bump(main.clone(), t, true);
                    }
                }
            }
        }
    }
    let mut v: Vec<_> = latest.into_values().collect();
    v.sort_by(|a, b| b.written.cmp(&a.written));
    v
}

/// When the newest timestamped record in a transcript tail was written.
/// Conversation records (prompts, replies, tool results, retries) carry a
/// timestamp; the bookkeeping the desktop app appends (titles, cost, artifact
/// state) doesn't, so this tells real activity apart from a touched file.
fn last_record_time(tail: &str) -> Option<SystemTime> {
    tail.lines().rev().find_map(|line| {
        if !line.contains("\"timestamp\"") {
            return None;
        }
        let v: serde_json::Value = serde_json::from_str(line).ok()?;
        let ts = v.get("timestamp")?.as_str()?;
        DateTime::parse_from_rfc3339(ts).ok().map(SystemTime::from)
    })
}

fn mtime(path: &Path) -> Option<SystemTime> {
    fs::metadata(path).and_then(|m| m.modified()).ok()
}

fn read_range(path: &Path, start: u64, max: u64) -> String {
    let mut buf = Vec::new();
    if let Ok(mut f) = fs::File::open(path) {
        if f.seek(SeekFrom::Start(start)).is_ok() {
            let _ = f.take(max).read_to_end(&mut buf);
        }
    }
    String::from_utf8_lossy(&buf).into_owned()
}

#[derive(Default)]
struct TranscriptInfo {
    custom: Option<String>,
    agent: Option<String>,
    ai: Option<String>,
    cwd: Option<String>,
}

impl TranscriptInfo {
    /// Reads title records and the working directory, latest lines first.
    /// Only the small title lines are parsed as JSON.
    fn scan(&mut self, text: &str) {
        for line in text.lines().rev() {
            if self.cwd.is_none() {
                self.cwd = string_field(line, "cwd");
            }
            for (key, slot) in [
                ("customTitle", &mut self.custom),
                ("agentName", &mut self.agent),
                ("aiTitle", &mut self.ai),
            ] {
                if slot.is_none() && line.contains(key) {
                    *slot = serde_json::from_str::<serde_json::Value>(line)
                        .ok()
                        .and_then(|v| v.get(key)?.as_str().map(str::to_owned))
                        .filter(|s| !s.trim().is_empty());
                }
            }
        }
    }

    fn title(&self) -> Option<String> {
        self.custom.clone().or_else(|| self.agent.clone()).or_else(|| self.ai.clone())
    }
}

/// Pulls `"key":"value"` out of a JSON line without parsing all of it.
fn string_field(line: &str, key: &str) -> Option<String> {
    let pat = format!("\"{key}\":\"");
    let start = line.find(&pat)? + pat.len();
    let end = line[start..].find('"')?;
    Some(line[start..start + end].to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The desktop app keeps appending untimed bookkeeping to idle sessions,
    /// which made a session that last spoke yesterday look busy.
    #[test]
    fn activity_ignores_untimed_bookkeeping() {
        let tail = concat!(
            "fragment of a cut-off line\n",
            r#"{"type":"assistant","timestamp":"2026-10-29T04:29:58.000Z","message":{"content":[]}}"#, "\n",
            r#"{"type":"queue-operation","operation":"dequeue","timestamp":"2026-10-29T04:30:20.604Z"}"#, "\n",
            r#"{"type":"custom-title","customTitle":"Micro-app platform v1 inventory"}"#, "\n",
            r#"{"type":"cost-state","sessionId":"x"}"#, "\n",
            r#"{"type":"artifact-autoreact-ledger","entries":[{"note":"has a \"timestamp\" inside"}]}"#, "\n",
        );
        let want = DateTime::parse_from_rfc3339("2026-10-29T04:30:20.604Z").unwrap();
        assert_eq!(last_record_time(tail), Some(SystemTime::from(want)));
        assert_eq!(last_record_time(r#"{"type":"custom-title","customTitle":"t"}"#), None);
    }

    #[test]
    fn prefers_custom_title_and_finds_cwd() {
        let mut info = TranscriptInfo::default();
        info.scan(concat!(
            r#"{"type":"user","cwd":"/Users/me/Git/app","message":"hi"}"#, "\n",
            r#"{"type":"ai-title","aiTitle":"Generated","sessionId":"x"}"#, "\n",
            r#"{"type":"custom-title","customTitle":"Mine","sessionId":"x"}"#, "\n",
        ));
        assert_eq!(info.title().as_deref(), Some("Mine"));
        assert_eq!(info.cwd.as_deref(), Some("/Users/me/Git/app"));
    }

    const LIMIT: &str = r#"{"type":"assistant","timestamp":"2026-08-27T06:25:06.782Z","message":{"role":"assistant","content":[{"type":"text","text":"You've hit your session limit · resets 1:40pm (Europe/London)"}]},"error":"rate_limit","isApiErrorMessage":true,"apiErrorStatus":429,"quotaLimits":{"status":"rejected","resetsAt":1787812800,"rateLimitType":"five_hour"}}"#;
    const REPLY: &str = r#"{"type":"assistant","timestamp":"2026-08-27T06:43:00.000Z","message":{"role":"assistant","content":[{"type":"text","text":"Picking up where I left off."}]}}"#;
    const OTHER_ERROR: &str = r#"{"type":"assistant","message":{"content":[{"type":"text","text":"API Error: 529 Overloaded"}]},"error":"overloaded","isApiErrorMessage":true}"#;

    #[test]
    fn finds_pending_limit_with_reset_time() {
        let tail = format!("{REPLY}\n{LIMIT}\n{{\"type\":\"last-prompt\"}}\n{OTHER_ERROR}\n");
        let hit = unresumed_limit(&tail).expect("limit");
        assert_eq!(hit.resets_at, Local.timestamp_opt(1787812800, 0).single());
        assert!(hit.message.contains("session limit"));
    }

    #[test]
    fn retry_wait_counts_as_waiting() {
        let retry = r#"{"type":"system","subtype":"api_error","level":"error","error":{"formatted":"429 ...","rateLimits":{"rateLimitType":"five_hour","resetsAt":1792570800}},"retryInMs":10697801,"retryAttempt":1,"maxRetries":300}"#;
        let hit = unresumed_limit(&format!("{REPLY}\n{retry}\n")).expect("waiting");
        assert_eq!(hit.resets_at, Local.timestamp_opt(1792570800, 0).single());
        assert!(unresumed_limit(&format!("{retry}\n{REPLY}\n")).is_none());
    }

    #[test]
    fn reads_auto_continue_setting() {
        let dir = std::env::temp_dir().join(format!("overnight-settings-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let file = dir.join("settings.json");
        assert_eq!(auto_continue(&file), Some(true)); // no file: default on
        fs::write(&file, r#"{"env":{"CLAUDE_CODE_RETRY_WATCHDOG":"1"}}"#).unwrap();
        assert_eq!(auto_continue(&file), Some(true));
        fs::write(&file, r#"{"autoContinueAtUsageLimit":false}"#).unwrap();
        assert_eq!(auto_continue(&file), Some(false));
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn reply_after_limit_means_resumed() {
        assert!(unresumed_limit(&format!("{LIMIT}\n{REPLY}\n")).is_none());
    }

    #[test]
    fn reset_time_from_text() {
        let at = Local.with_ymd_and_hms(2026, 10, 30, 13, 25, 0).unwrap();
        let msg = "You've hit your session limit · resets 1:40pm (Europe/London)";
        assert_eq!(reset_from_text(msg, at), Local.with_ymd_and_hms(2026, 10, 30, 13, 40, 0).single());
        let late = Local.with_ymd_and_hms(2026, 10, 30, 23, 10, 0).unwrap();
        assert_eq!(reset_from_text("... resets 2am (Europe/London)", late), Local.with_ymd_and_hms(2026, 10, 31, 2, 0, 0).single());
        assert_eq!(reset_from_text("You've reached your Fable limit. Switch to another model to continue.", at), None);
    }

    #[test]
    fn limit_states_over_time() {
        let reset = Local.with_ymd_and_hms(2026, 10, 31, 2, 0, 0).unwrap();
        let hit = LimitHit { resets_at: Some(reset), message: String::new() };
        let mins = |m| reset + TimeDelta::minutes(m);
        assert_eq!(limit_state(&hit, mins(-60)), Some(LimitState::Waiting));
        assert_eq!(limit_state(&hit, mins(5)), Some(LimitState::Waiting)); // grace to pick up
        assert_eq!(limit_state(&hit, mins(15)), Some(LimitState::Missed));
        assert_eq!(limit_state(&hit, mins(180)), None);
        let model = LimitHit { resets_at: None, message: String::new() };
        assert_eq!(limit_state(&model, mins(0)), Some(LimitState::Missed));
        let s = summarize_limits(&[hit.clone(), model], mins(-30)).unwrap();
        assert_eq!((s.state, s.sessions, s.resets_at), (LimitState::Waiting, 1, Some(reset)));
    }

    #[test]
    fn falls_back_to_agent_name() {
        let mut info = TranscriptInfo::default();
        info.scan(r#"{"type":"agent-name","agentName":"Lid sleep","sessionId":"x"}"#);
        assert_eq!(info.title().as_deref(), Some("Lid sleep"));
    }
}
