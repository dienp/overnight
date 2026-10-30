//! Readings from macOS: power, lid, temperature, disk and the sleep setting.
//! They're read in-process (IOKit, Foundation, statvfs) because they run on
//! every tick: spawning `pmset` and `ioreg` cost more CPU than the app itself.

use core_foundation::{
    array::{CFArray, CFArrayRef},
    base::{CFAllocatorRef, CFType, CFTypeRef, TCFType, kCFAllocatorDefault},
    boolean::CFBoolean,
    dictionary::{CFDictionary, CFDictionaryRef, CFMutableDictionaryRef},
    number::CFNumber,
    string::{CFString, CFStringRef},
};
use chrono::{DateTime, Local};
use serde::Serialize;
use std::{
    ffi::{CStr, c_char},
    process::Command,
    time::Duration,
};

#[link(name = "IOKit", kind = "framework")]
unsafe extern "C" {
    fn IOPSCopyPowerSourcesInfo() -> CFTypeRef;
    fn IOPSCopyPowerSourcesList(blob: CFTypeRef) -> CFArrayRef;
    fn IOPSGetPowerSourceDescription(blob: CFTypeRef, ps: CFTypeRef) -> CFDictionaryRef;
    fn IOPSGetProvidingPowerSourceType(blob: CFTypeRef) -> CFStringRef;
    fn IOServiceMatching(name: *const c_char) -> CFMutableDictionaryRef;
    /// Consumes `matching`.
    fn IOServiceGetMatchingService(main_port: u32, matching: CFDictionaryRef) -> u32;
    fn IORegistryEntryCreateCFProperty(entry: u32, key: CFStringRef, allocator: CFAllocatorRef, options: u32) -> CFTypeRef;
    fn IOObjectRelease(object: u32) -> i32;
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct Power {
    pub on_ac: bool,
    /// None on Macs without a battery.
    pub percent: Option<u8>,
    pub charging: bool,
    /// "1:18" while discharging, when macOS has an estimate.
    pub remaining: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct Disk {
    pub free_bytes: u64,
    pub total_bytes: u64,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Thermal {
    #[default]
    Nominal,
    Fair,
    Serious,
    Critical,
}

/// Runs a command and returns stdout when it exits successfully.
pub fn run(cmd: &str, args: &[&str]) -> Option<String> {
    let out = Command::new(cmd).args(args).output().ok()?;
    out.status
        .success()
        .then(|| String::from_utf8_lossy(&out.stdout).into_owned())
}

pub fn sh_quote(s: &str) -> String {
    format!("'{}'", s.replace('\'', r"'\''"))
}

/// What `pmset -g batt` shows, from the same IOKit power-source API.
pub fn power() -> Power {
    // SAFETY: Copy functions hand over a reference (create rule), Get functions
    // lend one (get rule); each pointer is checked for null before wrapping.
    unsafe {
        let blob = IOPSCopyPowerSourcesInfo();
        if blob.is_null() {
            return Power::default();
        }
        let blob = CFType::wrap_under_create_rule(blob);
        let source = IOPSGetProvidingPowerSourceType(blob.as_CFTypeRef());
        let mut p = Power {
            on_ac: source.is_null() || CFString::wrap_under_get_rule(source) == "AC Power",
            ..Default::default()
        };
        let list = IOPSCopyPowerSourcesList(blob.as_CFTypeRef());
        if list.is_null() {
            return p;
        }
        let list: CFArray<CFType> = CFArray::wrap_under_create_rule(list);
        for ps in list.iter() {
            let desc = IOPSGetPowerSourceDescription(blob.as_CFTypeRef(), ps.as_CFTypeRef());
            if desc.is_null() {
                continue;
            }
            let desc: CFDictionary<CFString, CFType> = CFDictionary::wrap_under_get_rule(desc);
            if text(&desc, "Type").as_deref() != Some("InternalBattery") {
                continue;
            }
            let max = number(&desc, "Max Capacity").filter(|m| *m > 0);
            p.percent = number(&desc, "Current Capacity")
                .zip(max)
                .map(|(cur, max)| ((cur * 100 + max / 2) / max).clamp(0, 100) as u8);
            p.charging = flag(&desc, "Is Charging").unwrap_or(false);
            if !p.on_ac {
                p.remaining = number(&desc, "Time to Empty").and_then(hours_minutes);
            }
            break;
        }
        p
    }
}

/// "1:18" for 78 minutes; None while macOS is still estimating (-1).
fn hours_minutes(mins: i64) -> Option<String> {
    (mins > 0).then(|| format!("{}:{:02}", mins / 60, mins % 60))
}

fn value(d: &CFDictionary<CFString, CFType>, key: &'static str) -> Option<CFType> {
    d.find(CFString::from_static_string(key)).map(|v| (*v).clone())
}

fn number(d: &CFDictionary<CFString, CFType>, key: &'static str) -> Option<i64> {
    value(d, key)?.downcast::<CFNumber>()?.to_i64()
}

fn text(d: &CFDictionary<CFString, CFType>, key: &'static str) -> Option<String> {
    Some(value(d, key)?.downcast::<CFString>()?.to_string())
}

fn flag(d: &CFDictionary<CFString, CFType>, key: &'static str) -> Option<bool> {
    Some(value(d, key)?.downcast::<CFBoolean>()?.into())
}

/// A property of the IOKit service of class `class`, as `ioreg` shows it.
fn registry_property(class: &CStr, key: &'static str) -> Option<CFType> {
    // SAFETY: the matching dictionary is consumed by IOServiceGetMatchingService,
    // the entry is released once read, and the property is an owned reference.
    unsafe {
        let matching = IOServiceMatching(class.as_ptr());
        if matching.is_null() {
            return None;
        }
        let entry = IOServiceGetMatchingService(0, matching as CFDictionaryRef);
        if entry == 0 {
            return None;
        }
        let key = CFString::from_static_string(key);
        let v = IORegistryEntryCreateCFProperty(entry, key.as_concrete_TypeRef(), kCFAllocatorDefault, 0);
        IOObjectRelease(entry);
        (!v.is_null()).then(|| CFType::wrap_under_create_rule(v))
    }
}

/// A yes/no property of IOPMrootDomain, the kernel's power manager. This is
/// what `ioreg` and `pmset -g` read.
fn root_domain_flag(key: &'static str) -> Option<bool> {
    registry_property(c"IOPMrootDomain", key)?
        .downcast::<CFBoolean>()
        .map(bool::from)
}

/// How long since the last keyboard, mouse or trackpad input.
pub fn input_idle() -> Option<Duration> {
    let ns = registry_property(c"IOHIDSystem", "HIDIdleTime")?
        .downcast::<CFNumber>()?
        .to_i64()?;
    Some(Duration::from_nanos(ns.max(0) as u64))
}

/// True when `pmset -a disablesleep 1` is in effect.
pub fn sleep_disabled() -> bool {
    root_domain_flag("SleepDisabled").unwrap_or(false)
}

/// The same answer from `pmset -g`. Slower, so it's only asked when the
/// registry seems to disagree with a hold the helper should have applied.
pub fn sleep_disabled_pmset() -> bool {
    run("/usr/bin/pmset", &["-g"]).is_some_and(|s| {
        s.lines().any(|l| {
            let mut it = l.split_whitespace();
            it.next() == Some("SleepDisabled") && it.next() == Some("1")
        })
    })
}

/// None on Macs without a lid.
pub fn lid_closed() -> Option<bool> {
    root_domain_flag("AppleClamshellState")
}

/// A time of day the way macOS shows it, following the user's 12- or 24-hour
/// setting ("2:00 AM" or "02:00"), so Rust's text matches the popover's.
pub fn clock(t: DateTime<Local>) -> String {
    use objc2_foundation::{NSDate, NSDateFormatter, NSDateFormatterStyle};
    let date = NSDate::dateWithTimeIntervalSince1970(t.timestamp() as f64);
    NSDateFormatter::localizedStringFromDate_dateStyle_timeStyle(
        &date,
        NSDateFormatterStyle::NoStyle,
        NSDateFormatterStyle::ShortStyle,
    )
    .to_string()
}

pub fn thermal() -> Thermal {
    let state = objc2_foundation::NSProcessInfo::processInfo().thermalState();
    match state.0 {
        1 => Thermal::Fair,
        2 => Thermal::Serious,
        3 => Thermal::Critical,
        _ => Thermal::Nominal,
    }
}

/// Space on the data volume, where everything the user writes lives.
pub fn disk() -> Disk {
    ["/System/Volumes/Data", "/"]
        .into_iter()
        .find_map(statvfs)
        .unwrap_or_default()
}

fn statvfs(path: &str) -> Option<Disk> {
    let c = std::ffi::CString::new(path).ok()?;
    // SAFETY: statvfs fills the zeroed struct and we only read it on success.
    let mut st: libc::statvfs = unsafe { std::mem::zeroed() };
    if unsafe { libc::statvfs(c.as_ptr(), &mut st) } != 0 {
        return None;
    }
    let block = st.f_frsize as u64;
    Some(Disk {
        free_bytes: st.f_bavail as u64 * block,
        total_bytes: st.f_blocks as u64 * block,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Parses `pmset -g batt`, e.g.
    /// `Now drawing from 'Battery Power'` /
    /// ` -InternalBattery-0 (id=1)\t57%; discharging; 1:18 remaining present: true`.
    /// Only used to check the in-process readings against the command.
    fn parse_batt(s: &str) -> Power {
        let mut p = Power {
            on_ac: s.contains("'AC Power'"),
            ..Default::default()
        };
        let Some(line) = s.lines().find(|l| l.contains("InternalBattery")) else {
            return p;
        };
        if let Some(end) = line.find('%') {
            let start = line[..end]
                .rfind(|c: char| !c.is_ascii_digit())
                .map_or(0, |i| i + 1);
            p.percent = line[start..end].parse().ok();
        }
        let fields: Vec<&str> = line.split(';').map(str::trim).collect();
        p.charging = fields.get(1).is_some_and(|f| *f == "charging");
        p.remaining = fields.get(2).and_then(|f| {
            let tok = f.split_whitespace().next()?;
            (tok.contains(':') && f.contains("remaining")).then(|| tok.to_string())
        });
        p
    }

    #[test]
    fn in_process_readings_match_the_commands() {
        let want = parse_batt(&run("/usr/bin/pmset", &["-g", "batt"]).unwrap());
        let got = power();
        assert_eq!(got.on_ac, want.on_ac);
        assert_eq!(got.charging, want.charging);
        match (got.percent, want.percent) {
            (Some(a), Some(b)) => assert!(a.abs_diff(b) <= 1, "{a}% vs pmset {b}%"),
            (a, b) => assert_eq!(a, b),
        }

        let ioreg = run("/usr/sbin/ioreg", &["-r", "-k", "AppleClamshellState", "-d", "1"]).unwrap_or_default();
        let lid = ioreg
            .lines()
            .find(|l| l.contains("\"AppleClamshellState\""))
            .map(|l| l.trim_end().ends_with("Yes"));
        assert_eq!(lid_closed(), lid);
        assert_eq!(sleep_disabled(), sleep_disabled_pmset());
        // The HID system counts up from the last input on a real Mac; CI's
        // virtual Macs may not have one.
        if std::env::var_os("CI").is_none() {
            assert!(input_idle().is_some());
        }
    }

    #[test]
    fn formats_time_remaining() {
        assert_eq!(hours_minutes(78).as_deref(), Some("1:18"));
        assert_eq!(hours_minutes(5).as_deref(), Some("0:05"));
        assert_eq!(hours_minutes(-1), None);
    }

    #[test]
    fn parses_battery_discharging() {
        let p = parse_batt(
            "Now drawing from 'Battery Power'\n -InternalBattery-0 (id=23593059)\t57%; discharging; 1:18 remaining present: true\n",
        );
        assert!(!p.on_ac);
        assert_eq!(p.percent, Some(57));
        assert!(!p.charging);
        assert_eq!(p.remaining.as_deref(), Some("1:18"));
    }

    #[test]
    fn parses_battery_charging_on_ac() {
        let p = parse_batt(
            "Now drawing from 'AC Power'\n -InternalBattery-0 (id=1)\t100%; charging; (no estimate) present: true\n",
        );
        assert!(p.on_ac);
        assert_eq!(p.percent, Some(100));
        assert!(p.charging);
        assert_eq!(p.remaining, None);
    }

    #[test]
    fn desktop_mac_has_no_battery() {
        let p = parse_batt("Now drawing from 'AC Power'\n");
        assert!(p.on_ac);
        assert_eq!(p.percent, None);
    }

    #[test]
    fn quotes_for_shell() {
        assert_eq!(sh_quote("/a b/it's"), r"'/a b/it'\''s'");
    }
}
