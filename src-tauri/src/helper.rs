//! Starts and steers the root helper (helper.sh) that flips
//! `pmset disablesleep`. The helper reads the hold/release files in the control
//! directory, so after one password prompt the app can hold and release the
//! lid lock all night without asking again.

use crate::system::sh_quote;
use base64::Engine;
use std::{
    fs, io,
    os::unix::fs::DirBuilderExt,
    path::Path,
    process::Command,
    thread,
    time::{Duration, Instant},
};

const MARKER: &str = "overnight-helper";
const SCRIPT: &str = include_str!("helper.sh");
const PROMPT: &str = "Overnight needs your password to keep the Mac awake with the lid closed. \
It turns this off again when you stop it or quit the app.";

#[derive(Clone, Copy, Debug, Default)]
pub struct Status {
    pub running: bool,
    pub pid: Option<u32>,
}

/// Finds the helper through the pid file it writes when it starts. Listing
/// every process with `ps` each tick cost about 50 ms of CPU; this is a file
/// read and two system calls.
pub fn status(ctl: &Path) -> Status {
    let pid = fs::read_to_string(ctl.join("pid"))
        .ok()
        .and_then(|s| s.trim().parse::<i32>().ok())
        .filter(|&pid| pid > 1);
    match pid {
        Some(pid) if is_root_shell(pid) => Status { running: true, pid: Some(pid as u32) },
        _ => Status::default(),
    }
}

/// The helper is a root shell: a process this user may not signal, running
/// /bin/sh (bash on macOS). That rules out a stale pid file left by a helper
/// that was killed outright, unless its pid went to another root shell.
fn is_root_shell(pid: i32) -> bool {
    // SAFETY: signal 0 only checks for the process; proc_pidpath writes at
    // most `buf.len()` bytes and returns how many.
    let not_ours = unsafe { libc::kill(pid, 0) } == -1 && io::Error::last_os_error().raw_os_error() == Some(libc::EPERM);
    if !not_ours {
        return false;
    }
    let mut buf = [0u8; libc::PROC_PIDPATHINFO_MAXSIZE as usize];
    let n = unsafe { libc::proc_pidpath(pid, buf.as_mut_ptr().cast(), buf.len() as u32) };
    n > 0 && matches!(&buf[..n as usize], b"/bin/sh" | b"/bin/bash" | b"/bin/zsh" | b"/bin/dash")
}

/// Resets the control directory for a new night: no release file, hold on.
pub fn prepare(ctl: &Path) -> io::Result<()> {
    fs::DirBuilder::new().recursive(true).mode(0o700).create(ctl)?;
    remove(&ctl.join("release"))?;
    remove(&ctl.join("pid"))?;
    fs::write(ctl.join("hold"), b"")
}

/// Waits for a helper that was told to release to finish (it polls every 5 s).
pub fn wait_until_stopped(ctl: &Path) -> bool {
    let started = Instant::now();
    while status(ctl).running {
        if started.elapsed() > Duration::from_secs(8) {
            return false;
        }
        thread::sleep(Duration::from_millis(500));
    }
    true
}

/// Shows the macOS admin prompt and starts the helper in the background.
/// Blocks until the user answers the prompt.
pub fn launch(ctl: &Path, app_pid: u32) -> Result<(), String> {
    let osa = do_shell_script(&shell_command(SCRIPT, ctl, app_pid), true);
    let out = Command::new("/usr/bin/osascript")
        .args(["-e", &osa])
        .output()
        .map_err(|e| e.to_string())?;
    if out.status.success() {
        return Ok(());
    }
    let err = String::from_utf8_lossy(&out.stderr);
    if err.contains("-128") {
        Err("Cancelled. The Mac will sleep normally.".into())
    } else {
        Err(err.trim().to_string())
    }
}

pub fn set_hold(ctl: &Path, on: bool) -> io::Result<()> {
    if on {
        fs::write(ctl.join("hold"), b"")
    } else {
        remove(&ctl.join("hold"))
    }
}

/// Tells the helper to restore the original setting and exit.
pub fn release(ctl: &Path) {
    let _ = remove(&ctl.join("hold"));
    let _ = fs::write(ctl.join("release"), b"");
}

/// Clears leftover control files once no helper is running.
pub fn cleanup(ctl: &Path) {
    let _ = remove(&ctl.join("hold"));
    let _ = remove(&ctl.join("release"));
    let _ = remove(&ctl.join("pid"));
}

fn remove(path: &Path) -> io::Result<()> {
    match fs::remove_file(path) {
        Err(e) if e.kind() != io::ErrorKind::NotFound => Err(e),
        _ => Ok(()),
    }
}

/// Runs `script` in the background with the marker as `$0`. Base64 keeps the
/// script out of the quoting and on one line in `ps`.
fn shell_command(script: &str, ctl: &Path, app_pid: u32) -> String {
    let b64 = base64::engine::general_purpose::STANDARD.encode(script);
    format!(
        "/bin/sh -c 'eval \"$(echo {b64} | /usr/bin/base64 -D)\"' {MARKER} {} {app_pid} >/dev/null 2>&1 &",
        sh_quote(&ctl.to_string_lossy())
    )
}

fn do_shell_script(cmd: &str, admin: bool) -> String {
    let mut s = format!("do shell script \"{}\"", applescript_escape(cmd));
    if admin {
        s += &format!(" with prompt \"{}\" with administrator privileges", applescript_escape(PROMPT));
    }
    s
}

fn applescript_escape(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pid_file_must_point_at_a_root_shell() {
        let dir = std::env::temp_dir().join(format!("overnight-pid-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let check = |pid: &str| {
            fs::write(dir.join("pid"), pid).unwrap();
            status(&dir).running
        };
        assert!(!check(&std::process::id().to_string()), "our own process isn't root");
        assert!(!check("1"), "launchd is root but not the helper");
        assert!(!check("garbage"));
        fs::remove_dir_all(&dir).unwrap();
        assert!(!status(&dir).running, "no pid file");
    }

    #[test]
    fn escapes_for_applescript() {
        assert_eq!(applescript_escape(r#"a "b" \c"#), r#"a \"b\" \\c"#);
    }

    /// Runs the real launch command through osascript, minus the admin
    /// prompt, with a harmless script and an awkward control path.
    #[test]
    fn launch_command_survives_quoting() {
        let dir = std::env::temp_dir().join(format!("overnight test 'q' \"{}\"", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let script = r#"printf '%s|%s|%s' "$0" "$1" "$2" > "$1/out.tmp" && mv "$1/out.tmp" "$1/out""#;
        let osa = do_shell_script(&shell_command(script, &dir, 42), false);
        let ok = Command::new("/usr/bin/osascript").args(["-e", &osa]).status().unwrap().success();
        assert!(ok, "osascript failed");
        let out = dir.join("out");
        for _ in 0..60 {
            if out.exists() {
                break;
            }
            thread::sleep(Duration::from_millis(50));
        }
        let got = fs::read_to_string(&out).unwrap_or_default();
        fs::remove_dir_all(&dir).unwrap();
        assert_eq!(got, format!("{MARKER}|{}|42", dir.display()));
    }
}
