//! Where disk space piles up on a Mac running Claude Code, and a button that
//! hands the cleanup to Claude.

use crate::system::sh_quote;
use chrono::{DateTime, Local};
use serde::Serialize;
use std::{
    fs,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    process::Command,
    time::Instant,
};

#[derive(Clone, Debug, Serialize)]
pub struct Item {
    pub label: String,
    pub path: String,
    pub bytes: u64,
}

#[derive(Clone, Debug, Serialize)]
pub struct Scan {
    pub at: DateTime<Local>,
    pub items: Vec<Item>,
    pub secs: f32,
}

/// Places under the home folder that tend to grow. Paths that don't exist are skipped.
const PLACES: &[(&str, &str)] = &[
    ("Xcode DerivedData", "Library/Developer/Xcode/DerivedData"),
    ("iOS simulators", "Library/Developer/CoreSimulator"),
    ("iOS device support", "Library/Developer/Xcode/iOS DeviceSupport"),
    ("Android SDK", "Library/Android/sdk"),
    ("Gradle caches", ".gradle/caches"),
    ("uv cache", ".cache/uv"),
    ("npm cache", ".npm"),
    ("pnpm store", "Library/pnpm/store"),
    ("Playwright browsers", "Library/Caches/ms-playwright"),
    ("Claude worktrees", "Documents/Git/__worktrees"),
    ("Claude Code transcripts", ".claude/projects"),
    ("Claude app data", "Library/Application Support/Claude"),
];

/// Measures each place with `du`. Takes from a few seconds to a minute.
pub fn scan(home: &Path) -> Scan {
    let started = Instant::now();
    let mut items: Vec<Item> = PLACES
        .iter()
        .filter_map(|(label, rel)| {
            let path = home.join(rel);
            if !path.exists() {
                return None;
            }
            Some(Item { label: (*label).into(), path: format!("~/{rel}"), bytes: du(&path)? })
        })
        .collect();
    items.sort_by(|a, b| b.bytes.cmp(&a.bytes));
    Scan { at: Local::now(), items, secs: started.elapsed().as_secs_f32() }
}

/// Size in bytes. `du` still prints a total when some folders are unreadable.
fn du(path: &Path) -> Option<u64> {
    let out = Command::new("/usr/bin/du").arg("-sk").arg(path).output().ok()?;
    let kb: u64 = String::from_utf8_lossy(&out.stdout).split_whitespace().next()?.parse().ok()?;
    Some(kb * 1024)
}

/// Opens Claude Code in Terminal with the cleanup prompt, so the user can
/// watch and answer its questions.
pub fn launch_cleanup(home: &Path, data_dir: &Path, prompt: &str) -> Result<(), String> {
    let claude = find_claude(home)
        .ok_or("Couldn't find the claude command. Install Claude Code or put it on your PATH.")?;
    let script = format!(
        "#!/bin/zsh -l\n# Written by Overnight: asks Claude Code to free up disk space.\ncd \"$HOME\"\nexec {} {}\n",
        sh_quote(&claude.to_string_lossy()),
        sh_quote(prompt)
    );
    let path = data_dir.join("cleanup.command");
    fs::write(&path, script).map_err(|e| e.to_string())?;
    fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).map_err(|e| e.to_string())?;
    let ok = Command::new("/usr/bin/open")
        .args(["-a", "Terminal"])
        .arg(&path)
        .status()
        .map_err(|e| e.to_string())?
        .success();
    if ok { Ok(()) } else { Err("Terminal didn't open.".into()) }
}

fn find_claude(home: &Path) -> Option<PathBuf> {
    [
        home.join(".local/bin/claude"),
        home.join(".claude/local/claude"),
        PathBuf::from("/opt/homebrew/bin/claude"),
        PathBuf::from("/usr/local/bin/claude"),
    ]
    .into_iter()
    .find(|p| p.exists())
}
