//! Phone alerts through ntfy (https://ntfy.sh), an open-source push service
//! with free iPhone and Android apps. Overnight posts the notifications it
//! shows on the Mac to a topic the user subscribes to. They never name a
//! session or project. Sending uses the curl that ships with macOS: a few
//! messages a night don't justify an HTTP stack in the app.

use crate::system;
use std::{
    fs,
    io::{Read, Write},
    process::{Command, Stdio},
    time::Duration,
};

/// No keyboard or mouse input for this long counts as away from the Mac.
const AWAY_AFTER: Duration = Duration::from_secs(5 * 60);

/// A topic on the public ntfy server with a random name. There, the name
/// works like a password: anyone who knows it can read the alerts.
pub fn new_topic_url() -> String {
    // 32 letters and digits, without look-alikes (l, o, 0, 1); 12 of them is 60 bits.
    const ALPHABET: &[u8] = b"abcdefghijkmnpqrstuvwxyz23456789";
    let mut bytes = [0u8; 12];
    let _ = fs::File::open("/dev/urandom").and_then(|mut f| f.read_exact(&mut bytes));
    let name: String = bytes
        .iter()
        .map(|b| ALPHABET[usize::from(*b) % ALPHABET.len()] as char)
        .collect();
    format!("https://ntfy.sh/overnight-{name}")
}

/// True when the lid is closed or nobody has used the keyboard or mouse for a while.
pub fn away(lid_closed: Option<bool>) -> bool {
    lid_closed == Some(true) || system::input_idle().is_some_and(|d| d >= AWAY_AFTER)
}

/// Posts one alert. Blocks for up to 10 s, so call it off the monitor thread.
pub fn send(url: &str, title: &str, body: &str) -> Result<(), String> {
    let (server, topic) =
        split(url).ok_or("That isn't an ntfy topic address. It should look like https://ntfy.sh/your-topic.")?;
    let mut child = Command::new("/usr/bin/curl")
        .args(["-fsS", "-m", "10", "-H", "Content-Type: application/json", "--data-binary", "@-", server])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| e.to_string())?;
    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(payload(topic, title, body).as_bytes())
            .map_err(|e| e.to_string())?;
    }
    let out = child.wait_with_output().map_err(|e| e.to_string())?;
    if out.status.success() {
        return Ok(());
    }
    Err(format!("Couldn't send to {server}: {}.", curl_error(&String::from_utf8_lossy(&out.stderr))))
}

/// "curl: (6) Could not resolve host: x" into "Could not resolve host: x".
fn curl_error(stderr: &str) -> &str {
    let s = stderr.trim().trim_start_matches("curl: ");
    match s.strip_prefix('(').and_then(|r| r.split_once(") ")) {
        Some((code, rest)) if code.chars().all(|c| c.is_ascii_digit()) => rest,
        _ => s,
    }
}

/// "https://ntfy.sh/topic" into the server and the topic. Topics are 1 to 64
/// letters, digits, "-" or "_".
fn split(url: &str) -> Option<(&str, &str)> {
    let url = url.trim().trim_end_matches('/');
    let rest = url.strip_prefix("https://").or_else(|| url.strip_prefix("http://"))?;
    let topic = &rest[rest.rfind('/')? + 1..];
    let valid = (1..=64).contains(&topic.len())
        && topic.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_');
    valid.then(|| (&url[..url.len() - topic.len() - 1], topic))
}

/// ntfy's JSON form, posted to the server root, so titles can hold any text.
fn payload(topic: &str, title: &str, body: &str) -> String {
    serde_json::json!({
        "topic": topic,
        "title": title,
        "message": body,
        "tags": ["crescent_moon"],
    })
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_topic_urls() {
        assert_eq!(split("https://ntfy.sh/overnight-abc"), Some(("https://ntfy.sh", "overnight-abc")));
        assert_eq!(split(" https://push.example.com/ntfy/my_topic/ "), Some(("https://push.example.com/ntfy", "my_topic")));
        assert_eq!(split("https://ntfy.sh/"), None);
        assert_eq!(split("ntfy.sh/topic"), None);
        assert_eq!(split("https://ntfy.sh/has space"), None);
    }

    #[test]
    fn new_topics_are_random_and_valid() {
        let (a, b) = (new_topic_url(), new_topic_url());
        assert_ne!(a, b);
        assert!(split(&a).is_some_and(|(server, topic)| server == "https://ntfy.sh" && topic.len() == 22));
    }

    #[test]
    fn curl_errors_read_plainly() {
        assert_eq!(curl_error("curl: (6) Could not resolve host: x\n"), "Could not resolve host: x");
        assert_eq!(curl_error("curl: (22) The requested URL returned error: 429"), "The requested URL returned error: 429");
        assert_eq!(curl_error("something else"), "something else");
    }

    #[test]
    fn payload_is_json_for_any_text() {
        let p: serde_json::Value = serde_json::from_str(&payload("t", "Claude is idle", "Line \"one\"\nand two")).unwrap();
        assert_eq!(p["topic"], "t");
        assert_eq!(p["message"], "Line \"one\"\nand two");
    }
}
