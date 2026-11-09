//! Bezel Light's daily look at the latest published release
//! (D-2026-11-09-update-check-1): once a day, unless `updateCheck` is off in
//! the settings, it asks GitHub for the latest release and keeps the answer
//! in `update.json` ([`bezel_runtime::update`]); the tray shows a newer one.
//! Nothing is downloaded and nothing about the user is sent.

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};

use bezel_core::domain::update::{LATEST_RELEASE_API, RELEASES_PAGE, Version, due};
use bezel_runtime::update::{Record, available, read, write};

/// How long after Light starts the first check runs, and how often the
/// settings are looked at again.
const PAUSE: Duration = Duration::from_secs(60);

/// The newer release found, shared with the tray.
pub(super) type Found = Arc<Mutex<Option<String>>>;

/// Whether checking is on (`updateCheck` in the settings; on unless false).
pub(super) fn checking(settings: &Path) -> bool {
    std::fs::read(settings)
        .ok()
        .and_then(|bytes| serde_json::from_slice::<serde_json::Value>(&bytes).ok())
        .is_none_or(|value| value["updateCheck"].as_bool() != Some(false))
}

/// One pass at `now` (seconds since the epoch): asks `fetch` for the latest
/// release's tag when a check is due and keeps it, then names the newer
/// release, if any. A failed answer is not kept, so the next pass retries.
pub(super) fn step(
    settings: &Path,
    current: &str,
    now: u64,
    fetch: &dyn Fn() -> Result<String, String>,
) -> Option<String> {
    if !checking(settings) {
        return None;
    }
    let last = read(settings);
    if due(last.as_ref().map(|record| record.checked_at), now) {
        match fetch() {
            Ok(tag) => {
                let record = Record {
                    checked_at: now,
                    latest: Version::parse(&tag).map(|v| v.to_string()),
                };
                if let Err(error) = write(settings, &record) {
                    eprintln!("bezel light: the update check was not saved: {error}");
                }
            }
            Err(error) => eprintln!("bezel light: update check failed: {error}"),
        }
    }
    available(settings, current, true)
}

/// The latest release's tag, from the GitHub API.
fn fetch() -> Result<String, String> {
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .https_only(true)
        .max_redirects(0)
        .timeout_global(Some(Duration::from_secs(10)))
        .build()
        .into();
    let mut response = agent
        .get(LATEST_RELEASE_API)
        .header("Accept", "application/vnd.github+json")
        .header("User-Agent", format!("Bezel-Evo/{}", bezel_cli::VERSION))
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

/// Starts the checks on a thread of their own; the tray reads `Found`.
pub(super) fn spawn(settings: PathBuf) -> Found {
    let found = Found::default();
    let shared = Arc::clone(&found);
    let started = std::thread::Builder::new()
        .name("bezel-update-check".into())
        .spawn(move || {
            loop {
                std::thread::sleep(PAUSE);
                let now = SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .map_or(0, |since| since.as_secs());
                let newer = step(&settings, bezel_cli::VERSION, now, &fetch);
                if let Ok(mut found) = shared.lock() {
                    *found = newer;
                }
            }
        });
    if let Err(error) = started {
        eprintln!("bezel light: the update check could not start: {error}");
    }
    found
}

/// Opens the releases page in the default browser: always the fixed
/// [`RELEASES_PAGE`], never an address from the answer.
pub(super) fn open_releases() {
    if let Err(error) = std::process::Command::new("rundll32")
        .args(["url.dll,FileProtocolHandler", RELEASES_PAGE])
        .spawn()
    {
        eprintln!("bezel light: the releases page could not be opened: {error}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    #[test]
    fn update_check_asks_once_a_day_and_names_a_newer_release() {
        let dir = tempfile::tempdir().unwrap();
        let settings = dir.path().join("settings.json");
        let asked = Cell::new(0);
        let answer = |tag: &'static str| {
            let asked = &asked;
            move || {
                asked.set(asked.get() + 1);
                Ok(tag.to_owned())
            }
        };
        let day = bezel_core::domain::update::CHECK_EVERY.as_secs();

        assert_eq!(
            step(&settings, "0.2.25", 100, &answer("v0.2.26")),
            Some("0.2.26".into())
        );
        assert_eq!(asked.get(), 1);
        // Within the day the saved answer is used.
        assert_eq!(
            step(&settings, "0.2.25", 200, &answer("v0.2.27")),
            Some("0.2.26".into())
        );
        assert_eq!(asked.get(), 1);
        // Once installed, the same release is no longer offered.
        assert_eq!(step(&settings, "0.2.26", 300, &answer("v0.2.27")), None);
        // A day later it asks again.
        assert_eq!(
            step(&settings, "0.2.26", 100 + day, &answer("v0.2.27")),
            Some("0.2.27".into())
        );
        assert_eq!(asked.get(), 2);
    }

    #[test]
    fn update_check_off_asks_nothing_and_shows_nothing() {
        let dir = tempfile::tempdir().unwrap();
        let settings = dir.path().join("settings.json");
        std::fs::write(&settings, r#"{"updateCheck":false}"#).unwrap();
        let never = || -> Result<String, String> { panic!("asked while off") };
        assert!(!checking(&settings));
        assert_eq!(step(&settings, "0.2.25", 100, &never), None);
        assert!(!dir.path().join("update.json").exists());
        std::fs::write(&settings, r#"{"updateCheck":true}"#).unwrap();
        assert!(checking(&settings));
        std::fs::write(&settings, "{}").unwrap();
        assert!(checking(&settings), "on unless turned off");
    }

    #[test]
    fn update_check_failure_is_retried_and_keeps_nothing() {
        let dir = tempfile::tempdir().unwrap();
        let settings = dir.path().join("settings.json");
        let fail = || -> Result<String, String> { Err("offline".into()) };
        assert_eq!(step(&settings, "0.2.25", 100, &fail), None);
        assert!(read(&settings).is_none(), "the next pass asks again");
        // An answer that is not a version is kept as a check without a release.
        let odd = || -> Result<String, String> { Ok("nightly".into()) };
        assert_eq!(step(&settings, "0.2.25", 200, &odd), None);
        assert_eq!(read(&settings).unwrap().latest, None);
    }
}
