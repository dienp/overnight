//! The overnight log: one JSON object per line in events.jsonl.

use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::Path,
};

const MAX_BYTES: u64 = 2 * 1024 * 1024;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Event {
    pub ts: DateTime<Local>,
    pub kind: String,
    pub msg: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub battery: Option<u8>,
}

/// Kinds that end a run started by "arm".
const RUN_ENDS: &[&str] = &["disarm", "guard", "helper_lost"];

#[derive(Clone, Debug, Serialize)]
pub struct Summary {
    pub from: DateTime<Local>,
    /// None while the run is still going.
    pub to: Option<DateTime<Local>>,
    pub held_secs: i64,
    pub claude_secs: i64,
    pub battery_start: Option<u8>,
    pub battery_end: Option<u8>,
    pub ended_by: Option<String>,
}

pub fn append(path: &Path, kind: &str, msg: impl Into<String>, battery: Option<u8>) {
    let event = Event { ts: Local::now(), kind: kind.into(), msg: msg.into(), battery };
    let Ok(line) = serde_json::to_string(&event) else {
        return;
    };
    if fs::metadata(path).is_ok_and(|m| m.len() > MAX_BYTES) {
        trim(path);
    }
    if let Ok(mut f) = OpenOptions::new().create(true).append(true).open(path) {
        let _ = writeln!(f, "{line}");
    }
}

/// Keeps the newer half of the log.
fn trim(path: &Path) {
    let Ok(text) = fs::read_to_string(path) else {
        return;
    };
    let half = text.len() / 2;
    let cut = text.as_bytes()[half..]
        .iter()
        .position(|&b| b == b'\n')
        .map_or(text.len(), |i| half + i + 1);
    let _ = fs::write(path, &text[cut..]);
}

/// Events at or after `since`, oldest first.
pub fn read_since(path: &Path, since: DateTime<Local>) -> Vec<Event> {
    fs::read_to_string(path)
        .unwrap_or_default()
        .lines()
        .filter_map(|l| serde_json::from_str::<Event>(l).ok())
        .filter(|e| e.ts >= since)
        .collect()
}

/// Summarises the most recent run (from "arm" to whatever ended it).
pub fn summarize(events: &[Event], now: DateTime<Local>) -> Option<Summary> {
    let start = events.iter().rposition(|e| e.kind == "arm")?;
    let run = &events[start..];
    let end = run.iter().position(|e| RUN_ENDS.contains(&e.kind.as_str()));
    let run = end.map_or(run, |i| &run[..=i]);
    let to = end.map(|_| run[run.len() - 1].ts);
    let stop = to.unwrap_or(now);
    Some(Summary {
        from: run[0].ts,
        to,
        held_secs: span(run, "hold_on", "hold_off", stop),
        claude_secs: span(run, "claude_active", "claude_idle", stop),
        battery_start: run.iter().find_map(|e| e.battery),
        battery_end: run.iter().rev().find_map(|e| e.battery),
        ended_by: end.map(|_| run[run.len() - 1].msg.clone()),
    })
}

/// Total seconds between `on` and `off` events, counting an open span up to `stop`.
fn span(events: &[Event], on: &str, off: &str, stop: DateTime<Local>) -> i64 {
    let mut total = 0;
    let mut since: Option<DateTime<Local>> = None;
    for e in events {
        if e.kind == on && since.is_none() {
            since = Some(e.ts);
        } else if e.kind == off {
            if let Some(s) = since.take() {
                total += (e.ts - s).num_seconds();
            }
        }
    }
    if let Some(s) = since {
        total += (stop - s).num_seconds();
    }
    total
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn ev(h: u32, m: u32, kind: &str, battery: Option<u8>) -> Event {
        let day = if h >= 12 { 29 } else { 30 };
        Event {
            ts: Local.with_ymd_and_hms(2026, 10, day, h, m, 0).unwrap(),
            kind: kind.into(),
            msg: kind.into(),
            battery,
        }
    }

    #[test]
    fn summarizes_last_run() {
        let events = vec![
            ev(20, 0, "arm", Some(90)), // an older run, ignored
            ev(21, 0, "disarm", Some(80)),
            ev(23, 0, "arm", Some(100)),
            ev(23, 0, "hold_on", None),
            ev(23, 0, "claude_active", None),
            ev(2, 0, "claude_idle", None),
            ev(2, 0, "hold_off", None),
            ev(7, 0, "battery", Some(74)),
            ev(8, 0, "guard", Some(70)),
            ev(9, 0, "lid_opened", Some(70)),
        ];
        let now = Local.with_ymd_and_hms(2026, 10, 30, 10, 0, 0).unwrap();
        let s = summarize(&events, now).unwrap();
        assert_eq!(s.held_secs, 3 * 3600);
        assert_eq!(s.claude_secs, 3 * 3600);
        assert_eq!(s.battery_start, Some(100));
        assert_eq!(s.battery_end, Some(70));
        assert_eq!(s.ended_by.as_deref(), Some("guard"));
        assert_eq!(s.to, Some(events[8].ts));
    }

    #[test]
    fn open_run_counts_up_to_now() {
        let events = vec![ev(23, 0, "arm", None), ev(23, 0, "hold_on", None)];
        let now = Local.with_ymd_and_hms(2026, 10, 30, 1, 30, 0).unwrap();
        let s = summarize(&events, now).unwrap();
        assert_eq!(s.held_secs, 2 * 3600 + 1800);
        assert!(s.to.is_none());
    }
}
