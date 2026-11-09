//! The last update check, kept in `update.json` beside the settings
//! (D-2026-11-09-update-check-2): Bezel Light writes it after each check,
//! Studio reads it for the Preferences. A file of its own, so Light never
//! writes over Studio's settings.

use std::path::{Path, PathBuf};

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
}
