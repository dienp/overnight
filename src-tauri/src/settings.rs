use chrono::NaiveTime;
use serde::{Deserialize, Serialize};
use std::{fs, io, path::Path};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    /// Hold the lid lock only while Claude sessions are busy.
    Claude,
    /// Hold the lid lock until the stop time or a guard trips.
    Night,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub mode: Mode,
    pub stop_enabled: bool,
    /// Local time, "HH:MM".
    pub stop_at: String,
    /// Minutes without session activity before Claude counts as idle.
    pub idle_minutes: u32,
    /// In Claude mode, stay awake through a pending usage-limit reset so
    /// Claude can continue (All night mode always does).
    pub wait_for_reset: bool,
    /// On battery, let the Mac sleep once the charge drops below this percentage.
    pub battery_floor: u8,
    pub thermal_guard: bool,
    pub notify_claude_idle: bool,
    pub storage_warn_gb: u32,
    pub cleanup_prompt: String,
    /// Also send Overnight's notifications to a phone through ntfy.
    pub phone_alerts: bool,
    /// The ntfy topic URL, e.g. "https://ntfy.sh/overnight-k3m9...".
    pub ntfy_url: String,
    /// Send to the phone only while the lid is closed or nobody is using the Mac.
    pub phone_only_when_away: bool,
    /// Has Keep awake tonight ever worked? Until it has, the popover explains
    /// the password prompt before it appears.
    pub armed_before: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            mode: Mode::Claude,
            stop_enabled: true,
            stop_at: "08:00".into(),
            idle_minutes: 15,
            wait_for_reset: true,
            battery_floor: 10,
            thermal_guard: true,
            notify_claude_idle: true,
            storage_warn_gb: 30,
            cleanup_prompt: "Free up disk space on this Mac. Use the disk-cleanup skill if you have it.".into(),
            phone_alerts: false,
            ntfy_url: String::new(),
            phone_only_when_away: true,
            armed_before: false,
        }
    }
}

impl Settings {
    pub fn load(path: &Path) -> Self {
        fs::read(path)
            .ok()
            .and_then(|b| serde_json::from_slice::<Settings>(&b).ok())
            .unwrap_or_default()
            .sanitized()
    }

    pub fn save(&self, path: &Path) -> io::Result<()> {
        let json = serde_json::to_vec_pretty(self).map_err(io::Error::other)?;
        fs::write(path, json)
    }

    pub fn sanitized(mut self) -> Self {
        let d = Settings::default();
        self.idle_minutes = self.idle_minutes.clamp(1, 240);
        self.battery_floor = self.battery_floor.clamp(5, 90);
        self.storage_warn_gb = self.storage_warn_gb.clamp(1, 4000);
        if self.stop_time().is_none() {
            self.stop_at = d.stop_at;
        }
        if self.cleanup_prompt.trim().is_empty() {
            self.cleanup_prompt = d.cleanup_prompt;
        }
        self.ntfy_url = self.ntfy_url.trim().to_string();
        self
    }

    pub fn stop_time(&self) -> Option<NaiveTime> {
        NaiveTime::parse_from_str(&self.stop_at, "%H:%M").ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitizes_out_of_range_values() {
        let s = Settings {
            idle_minutes: 0,
            battery_floor: 1,
            stop_at: "25:99".into(),
            cleanup_prompt: "  ".into(),
            ..Settings::default()
        }
        .sanitized();
        assert_eq!(s.idle_minutes, 1);
        assert_eq!(s.battery_floor, 5);
        assert_eq!(s.stop_at, "08:00");
        assert!(!s.cleanup_prompt.trim().is_empty());
    }

    #[test]
    fn missing_fields_take_defaults() {
        let s: Settings = serde_json::from_str(r#"{"mode":"night"}"#).unwrap();
        assert_eq!(s.mode, Mode::Night);
        assert_eq!(s.idle_minutes, 15);
    }
}
