//! The last update check, kept in `update.json` beside the settings
//! (D-2026-11-09-update-check-2): Bezel Light writes it after each daily
//! check and Studio after **Check now**; Studio reads it for the
//! Preferences. A file of its own, so Light never writes over Studio's
//! settings. Also the one request both make ([`fetch_latest_tag`]).

use std::path::{Path, PathBuf};
use std::time::Duration;

use bezel_core::domain::update::{LATEST_RELEASE_API, Version};

use serde::{Deserialize, Serialize};

/// What the last check found.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Record {
    /// When it ran, in seconds since the epoch.
    pub checked_at: u64,
    /// The version of the latest published release (`0.2.26`), when the
    /// answer named one.
    pub latest: Option<String>,
}

/// Where the record is, beside `settings` (`settings.json`).
pub fn record_path(settings: &Path) -> PathBuf {
    settings.with_file_name("update.json")
}

/// The record beside `settings`; nothing when it is missing or unreadable.
pub fn read(settings: &Path) -> Option<Record> {
    let bytes = std::fs::read(record_path(settings)).ok()?;
    serde_json::from_slice(&bytes).ok()
}

/// Writes `record` beside `settings`, whole or not at all.
pub fn write(settings: &Path, record: &Record) -> std::io::Result<()> {
    let path = record_path(settings);
    let partial = path.with_extension("json.partial");
    std::fs::write(&partial, serde_json::to_vec_pretty(record)?)?;
    std::fs::rename(&partial, &path)
}

/// Asks `fetch` for the latest release's tag and keeps the answer as a
/// check made at `now` (seconds since the epoch). A failed answer keeps
/// nothing; an answer that is not a version is kept without a release.
pub fn check(
    settings: &Path,
    now: u64,
    fetch: &dyn Fn() -> Result<String, String>,
) -> Result<Record, String> {
    let tag = fetch()?;
    let record = Record {
        checked_at: now,
        latest: Version::parse(&tag).map(|v| v.to_string()),
    };
    write(settings, &record).map_err(|error| error.to_string())?;
    Ok(record)
}

/// The latest published release's tag, from the GitHub API. Sends only
/// `User-Agent: Bezel-Evo/<version>`: HTTPS, no redirects, 10 s, at most
/// 1 MB of answer.
pub fn fetch_latest_tag(version: &str) -> Result<String, String> {
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .https_only(true)
        .max_redirects(0)
        .timeout_global(Some(Duration::from_secs(10)))
        .build()
        .into();
    let mut response = agent
        .get(LATEST_RELEASE_API)
        .header("Accept", "application/vnd.github+json")
        .header("User-Agent", format!("Bezel-Evo/{version}"))
        .call()
        .map_err(|error| error.to_string())?;
    let bytes = response
        .body_mut()
        .with_config()
        .limit(1_048_576)
        .read_to_vec()
        .map_err(|error| error.to_string())?;
    let value: serde_json::Value =
        serde_json::from_slice(&bytes).map_err(|error| error.to_string())?;
    value["tag_name"]
        .as_str()
        .map(str::to_owned)
        .ok_or_else(|| "the answer names no release".into())
}

/// The newer release the record names for a program at `current`, when
/// checking is on.
pub fn available(settings: &Path, current: &str, checking: bool) -> Option<String> {
    let latest = read(settings).filter(|_| checking)?.latest?;
    bezel_core::domain::update::newer(current, &latest).map(|v| v.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn update_record_is_kept_beside_the_settings() {
        let dir = tempfile::tempdir().unwrap();
        let settings = dir.path().join("settings.json");
        assert_eq!(read(&settings), None);
        let record = Record {
            checked_at: 42,
            latest: Some("0.2.26".into()),
        };
        write(&settings, &record).unwrap();
        assert_eq!(read(&settings), Some(record));
        let text = std::fs::read_to_string(dir.path().join("update.json")).unwrap();
        assert!(text.contains("\"checkedAt\": 42"), "{text}");
        assert!(!dir.path().join("update.json.partial").exists());

        assert_eq!(available(&settings, "0.2.25", true), Some("0.2.26".into()));
        assert_eq!(
            available(&settings, "0.2.25", false),
            None,
            "checking is off"
        );
        assert_eq!(available(&settings, "0.2.26", true), None);
        std::fs::write(record_path(&settings), "not json").unwrap();
        assert_eq!(read(&settings), None);
        assert_eq!(available(&settings, "0.2.25", true), None);
    }

    #[test]
    fn a_check_keeps_the_answer_and_a_failure_keeps_nothing() {
        let dir = tempfile::tempdir().unwrap();
        let settings = dir.path().join("settings.json");
        let offline = || -> Result<String, String> { Err("offline".into()) };
        assert_eq!(check(&settings, 10, &offline), Err("offline".into()));
        assert_eq!(read(&settings), None);
        let tag = || -> Result<String, String> { Ok("v0.2.29".into()) };
        let record = check(&settings, 20, &tag).unwrap();
        assert_eq!(
            record,
            Record {
                checked_at: 20,
                latest: Some("0.2.29".into())
            }
        );
        assert_eq!(read(&settings), Some(record));
        let odd = || -> Result<String, String> { Ok("nightly".into()) };
        assert_eq!(check(&settings, 30, &odd).unwrap().latest, None);
        // A failure leaves the last good answer in place.
        assert_eq!(read(&settings).unwrap().checked_at, 30);
        assert!(check(&settings, 40, &offline).is_err());
        assert_eq!(read(&settings).unwrap().checked_at, 30);
    }
}
