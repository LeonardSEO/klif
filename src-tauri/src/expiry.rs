use std::time::SystemTime;

use chrono::{DateTime, Duration, Utc};
use idevice::{IdeviceService, misagent::MisagentClient};
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::{
    device::{DeviceInfoMutex, get_provider},
    error::AppError,
};

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ProfileInfo {
    pub uuid: String,
    pub name: String,
    pub app_id_name: Option<String>,
    pub app_identifier: Option<String>,
    pub team_name: Option<String>,
    pub created: Option<DateTime<Utc>>,
    pub expires: DateTime<Utc>,
}

#[tauri::command]
pub async fn list_installed_profiles(
    device_state: State<'_, DeviceInfoMutex>,
) -> Result<Vec<ProfileInfo>, AppError> {
    let device = {
        let device_guard = device_state.lock().unwrap();
        match &*device_guard {
            Some(d) => d.clone(),
            None => return Err(AppError::NoDeviceSelected),
        }
    };
    let provider = get_provider(&device.info).await?;
    let mut misagent = MisagentClient::connect(&provider).await.map_err(|e| {
        AppError::DeviceComsWithMessage("Failed to connect to misagent".into(), e.to_string())
    })?;

    let raw_profiles = misagent.copy_all().await.map_err(|e| {
        AppError::DeviceComsWithMessage(
            "Failed to copy provisioning profiles".into(),
            e.to_string(),
        )
    })?;

    let mut profiles: Vec<ProfileInfo> = raw_profiles
        .iter()
        .filter_map(|raw| match parse_profile(raw) {
            Some(p) => Some(p),
            None => {
                tracing::warn!("Skipping provisioning profile that could not be parsed");
                None
            }
        })
        .collect();
    profiles.sort_by_key(|p| p.expires);
    Ok(profiles)
}

#[tauri::command]
pub async fn export_expiry_calendar(
    device_state: State<'_, DeviceInfoMutex>,
    path: String,
    title_template: String,
    description: String,
) -> Result<usize, AppError> {
    let profiles = list_installed_profiles(device_state).await?;
    let (ics, count) = build_ics(&profiles, Utc::now(), &title_template, &description);
    std::fs::write(&path, ics)
        .map_err(|e| AppError::Filesystem("Failed to write calendar file".into(), e.to_string()))?;
    Ok(count)
}

/// Provisioning profiles are CMS-signed blobs with an XML plist inside.
/// Pull out the plist and read the fields we care about.
fn parse_profile(raw: &[u8]) -> Option<ProfileInfo> {
    let start = find(raw, b"<?xml")?;
    let end = find(&raw[start..], b"</plist>")? + start + b"</plist>".len();
    let value = plist::Value::from_reader_xml(&raw[start..end]).ok()?;
    let dict = value.as_dictionary()?;

    let string = |key: &str| dict.get(key).and_then(|v| v.as_string()).map(str::to_string);
    let date = |key: &str| {
        dict.get(key)
            .and_then(|v| v.as_date())
            .map(|d| DateTime::<Utc>::from(SystemTime::from(d)))
    };

    let app_identifier = dict
        .get("Entitlements")
        .and_then(|v| v.as_dictionary())
        .and_then(|e| e.get("application-identifier"))
        .and_then(|v| v.as_string())
        .map(str::to_string);

    Some(ProfileInfo {
        uuid: string("UUID")?,
        name: string("Name").unwrap_or_default(),
        app_id_name: string("AppIDName"),
        app_identifier,
        team_name: string("TeamName"),
        created: date("CreationDate"),
        expires: date("ExpirationDate")?,
    })
}

fn find(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack.windows(needle.len()).position(|w| w == needle)
}

/// Builds an iCalendar file with one event per profile that has not expired
/// yet, each with reminders 2 days, 1 day and 6 hours before it expires.
/// `title_template` may contain `{name}`. Returns the file and the event count.
fn build_ics(
    profiles: &[ProfileInfo],
    now: DateTime<Utc>,
    title_template: &str,
    description: &str,
) -> (String, usize) {
    let mut lines = vec![
        "BEGIN:VCALENDAR".to_string(),
        "VERSION:2.0".to_string(),
        "PRODID:-//expiry-watch//EN".to_string(),
        "CALSCALE:GREGORIAN".to_string(),
    ];
    let mut count = 0;

    for profile in profiles.iter().filter(|p| p.expires > now) {
        let name = profile.app_id_name.as_deref().unwrap_or(&profile.name);
        let title = title_template.replace("{name}", name);
        let end = profile.expires + Duration::minutes(15);

        lines.push("BEGIN:VEVENT".into());
        lines.push(format!("UID:{}@expiry-watch", profile.uuid));
        lines.push(format!("DTSTAMP:{}", ics_time(now)));
        lines.push(format!("DTSTART:{}", ics_time(profile.expires)));
        lines.push(format!("DTEND:{}", ics_time(end)));
        lines.push(format!("SUMMARY:{}", escape(&title)));
        lines.push(format!("DESCRIPTION:{}", escape(description)));
        for trigger in ["-P2D", "-P1D", "-PT6H"] {
            lines.push("BEGIN:VALARM".into());
            lines.push("ACTION:DISPLAY".into());
            lines.push(format!("DESCRIPTION:{}", escape(&title)));
            lines.push(format!("TRIGGER:{trigger}"));
            lines.push("END:VALARM".into());
        }
        lines.push("END:VEVENT".into());
        count += 1;
    }
    lines.push("END:VCALENDAR".into());

    let mut out = String::new();
    for line in lines {
        out.push_str(&fold(&line));
        out.push_str("\r\n");
    }
    (out, count)
}

fn ics_time(t: DateTime<Utc>) -> String {
    t.format("%Y%m%dT%H%M%SZ").to_string()
}

fn escape(text: &str) -> String {
    text.replace('\\', "\\\\")
        .replace(';', "\\;")
        .replace(',', "\\,")
        .replace("\r\n", "\\n")
        .replace('\n', "\\n")
}

/// RFC 5545 lines must not exceed 75 octets; longer lines continue on the
/// next line after a single space. Never split inside a UTF-8 character.
fn fold(line: &str) -> String {
    let mut out = String::new();
    let mut len = 0;
    for c in line.chars() {
        let width = c.len_utf8();
        if len + width > 75 {
            out.push_str("\r\n ");
            len = 1;
        }
        out.push(c);
        len += width;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn profile(uuid: &str, name: &str, expires: DateTime<Utc>) -> ProfileInfo {
        ProfileInfo {
            uuid: uuid.into(),
            name: name.into(),
            app_id_name: Some(name.into()),
            app_identifier: None,
            team_name: None,
            created: None,
            expires,
        }
    }

    #[test]
    fn parses_profile_inside_cms_blob() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
<key>AppIDName</key><string>NOOP</string>
<key>CreationDate</key><date>2026-10-20T10:00:00Z</date>
<key>Entitlements</key><dict><key>application-identifier</key><string>ABCDE12345.com.example.noop</string></dict>
<key>ExpirationDate</key><date>2026-10-27T10:00:00Z</date>
<key>Name</key><string>iOS Team Provisioning Profile: com.example.noop</string>
<key>TeamName</key><string>Jane Doe</string>
<key>UUID</key><string>1111-2222</string>
</dict></plist>"#;
        let mut raw = vec![0x30, 0x82, 0x01, 0x00];
        raw.extend_from_slice(xml.as_bytes());
        raw.extend_from_slice(&[0xa0, 0x82, 0x00]);

        let p = parse_profile(&raw).expect("profile should parse");
        assert_eq!(p.uuid, "1111-2222");
        assert_eq!(p.app_id_name.as_deref(), Some("NOOP"));
        assert_eq!(p.app_identifier.as_deref(), Some("ABCDE12345.com.example.noop"));
        assert_eq!(p.team_name.as_deref(), Some("Jane Doe"));
        assert_eq!(p.expires, Utc.with_ymd_and_hms(2026, 10, 27, 10, 0, 0).unwrap());
        assert_eq!(p.created, Some(Utc.with_ymd_and_hms(2026, 10, 20, 10, 0, 0).unwrap()));
    }

    #[test]
    fn rejects_blob_without_plist_or_expiry() {
        assert!(parse_profile(b"not a profile").is_none());
        let no_expiry = br#"<?xml version="1.0"?><plist version="1.0"><dict><key>UUID</key><string>x</string></dict></plist>"#;
        assert!(parse_profile(no_expiry).is_none());
    }

    #[test]
    fn ics_skips_expired_profiles_and_adds_reminders() {
        let now = Utc.with_ymd_and_hms(2026, 10, 26, 12, 0, 0).unwrap();
        let profiles = vec![
            profile("old", "Old", now - Duration::days(1)),
            profile("new", "NOOP", Utc.with_ymd_and_hms(2026, 10, 30, 8, 30, 0).unwrap()),
        ];
        let (ics, count) = build_ics(&profiles, now, "{name} expires", "Refresh it");

        assert_eq!(count, 1);
        assert!(ics.starts_with("BEGIN:VCALENDAR\r\n"));
        assert!(ics.ends_with("END:VCALENDAR\r\n"));
        assert!(ics.contains("UID:new@expiry-watch\r\n"));
        assert!(!ics.contains("UID:old@expiry-watch"));
        assert!(ics.contains("DTSTART:20261030T083000Z\r\n"));
        assert!(ics.contains("SUMMARY:NOOP expires\r\n"));
        assert_eq!(ics.matches("BEGIN:VALARM").count(), 3);
        assert!(ics.contains("TRIGGER:-PT6H\r\n"));
    }

    #[test]
    fn escapes_special_characters() {
        assert_eq!(escape("a,b;c\\d\ne"), "a\\,b\\;c\\\\d\\ne");
    }

    #[test]
    fn folds_long_lines_without_splitting_characters() {
        let line = format!("SUMMARY:{}", "é".repeat(60));
        let folded = fold(&line);
        for part in folded.split("\r\n") {
            assert!(part.len() <= 75, "line too long: {}", part.len());
        }
        assert_eq!(folded.replace("\r\n ", ""), line);
    }
}
