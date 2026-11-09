//! Whether a newer published release exists (D-2026-11-09-update-check-1):
//! the release's version, compared with the running one, and how often the
//! check runs. The page offered is always the fixed [`RELEASES_PAGE`], never
//! an address read from the network.

use std::time::Duration;

/// The latest published release, as the GitHub API describes it.
pub const LATEST_RELEASE_API: &str =
    "https://api.github.com/repos/jripper77/bezel-evo/releases/latest";

/// The page opened to download a newer release.
pub const RELEASES_PAGE: &str = "https://github.com/jripper77/bezel-evo/releases/latest";

/// How long after a check the next one runs.
pub const CHECK_EVERY: Duration = Duration::from_secs(24 * 60 * 60);

/// A release version: `major.minor.patch`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Version {
    /// The first number.
    pub major: u32,
    /// The second number.
    pub minor: u32,
    /// The third number.
    pub patch: u32,
}

impl Version {
    /// `0.2.25` or a tag such as `v0.2.25`; anything else (a suffix, a
    /// missing part) is not a version.
    pub fn parse(text: &str) -> Option<Self> {
        let text = text.trim();
        let text = text.strip_prefix('v').unwrap_or(text);
        let mut parts = text.split('.');
        let mut part = || -> Option<u32> {
            let digits = parts.next()?;
            if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
                return None;
            }
            digits.parse().ok()
        };
        let version = Self {
            major: part()?,
            minor: part()?,
            patch: part()?,
        };
        parts.next().is_none().then_some(version)
    }
}

impl core::fmt::Display for Version {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

/// `latest` when it is newer than `current`; nothing when either is not a
/// version.
pub fn newer(current: &str, latest: &str) -> Option<Version> {
    let current = Version::parse(current)?;
    let latest = Version::parse(latest)?;
    (latest > current).then_some(latest)
}

/// Whether a check is due at `now` (seconds since the epoch) after the one
/// made at `last`: never made, a day passed, or the clock went back.
pub fn due(last: Option<u64>, now: u64) -> bool {
    last.is_none_or(|last| now < last || now - last >= CHECK_EVERY.as_secs())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn update_versions_are_read_from_tags_and_plain_numbers() {
        let v = |major, minor, patch| Version {
            major,
            minor,
            patch,
        };
        assert_eq!(Version::parse("v0.2.25"), Some(v(0, 2, 25)));
        assert_eq!(Version::parse(" 1.10.0\n"), Some(v(1, 10, 0)));
        for bad in [
            "",
            "v",
            "0.2",
            "0.2.25.1",
            "0.2.x",
            "0.2.25-rc1",
            "v-1.2.3",
            "0..1",
        ] {
            assert_eq!(Version::parse(bad), None, "{bad:?}");
        }
        assert_eq!(v(0, 2, 25).to_string(), "0.2.25");
    }

    #[test]
    fn update_is_offered_only_for_a_newer_release() {
        assert_eq!(
            newer("0.2.25", "v0.2.26").map(|v| v.to_string()),
            Some("0.2.26".into())
        );
        assert_eq!(
            newer("0.2.9", "v0.2.10").map(|v| v.to_string()),
            Some("0.2.10".into())
        );
        assert_eq!(
            newer("0.2.25", "v1.0.0").map(|v| v.to_string()),
            Some("1.0.0".into())
        );
        assert_eq!(newer("0.2.25", "v0.2.25"), None);
        assert_eq!(
            newer("0.2.25", "v0.2.24"),
            None,
            "a local build ahead of the release"
        );
        assert_eq!(newer("0.2.25", "nightly"), None);
        assert_eq!(newer("dev", "v0.2.26"), None);
    }

    #[test]
    fn update_check_runs_once_a_day() {
        let day = CHECK_EVERY.as_secs();
        assert!(due(None, 1_000));
        assert!(!due(Some(1_000), 1_000));
        assert!(!due(Some(1_000), 1_000 + day - 1));
        assert!(due(Some(1_000), 1_000 + day));
        assert!(due(Some(1_000), 999), "the clock went back");
    }
}
