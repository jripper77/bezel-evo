//! What the app remembers between runs: the last theme and the screen that
//! was showing it, so a start at login picks up where the user left off, and
//! how each screen is used (vertical or horizontal), for its next new theme.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use bezel_core::domain::clock::Language;
use bezel_core::domain::geometry::Orientation;
use bezel_sensors::SensorOptions;
use serde::{Deserialize, Serialize};

use crate::diag::{self, DiagCode};
use crate::dto::{orientation_slug, parse_orientation};
use crate::texts::parse_language;

/// Remembered choices (`settings.json` in the app's config folder).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    /// Local permission to capture the followed app's audio; on unless the
    /// user turned it off. Never read from theme documents.
    pub audio_capture: DefaultOn,
    /// The screens idle (dimmed, moving logo, no audio capture) while
    /// Windows has the monitors off; on unless the user turned it off
    /// (D-2026-11-06-display-idle-saver-4). Bezel Light reads it too.
    pub idle_saver: DefaultOn,
    pub debug: bool,
    pub debug_show_readings: bool,
    pub debug_corner: crate::performance::Corner,
    /// Location of the last theme opened or saved.
    pub last_theme: Option<String>,
    /// Key of the screen that was live when the app last changed it.
    pub live_screen: Option<String>,
    pub selected_screen: Option<String>,
    /// Bezel Light's base theme for each display, changed only by "Use on
    /// Light" (D-2026-11-05-light-studio-split-1).
    pub screen_themes: BTreeMap<String, String>,
    /// The displays Bezel Light shows (Studio's live preview never changes
    /// them; D-2026-11-05-light-studio-split-2).
    pub live_screens: Vec<String>,
    /// The theme Studio last opened or saved for each display. Absent only
    /// in settings written before it existed ([`SettingsFile::load`]
    /// migrates them; Light then stops following Studio's open theme).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub editor_themes: Option<BTreeMap<String, String>>,
    /// The orientation last used with each screen (`portrait`, `landscape`…),
    /// by screen key.
    pub screen_orientations: BTreeMap<String, String>,
    /// The brightness last chosen for each screen (percent), by screen key:
    /// Bezel Light sends it when it opens the screen and restores it when
    /// idle ends.
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub screen_brightness: BTreeMap<String, u8>,
    /// The ffmpeg chosen with the storage tab's Locate button (the program
    /// or its folder); `PATH` is searched after it.
    pub ffmpeg_path: Option<String>,
    /// The language the user chose (`pt-BR` or `en`); without one the app
    /// follows the system's.
    pub language: Option<String>,
    /// The host `net.ping` measures; without one, the sensors' default.
    pub ping_host: Option<String>,
    /// Path to the PresentMon executable; absent means automatic search.
    pub presentmon_path: Option<String>,
    /// Which themes the Themes tab lists ([`THEME_SCOPES`]); without one,
    /// those for the screen in use when one is known.
    pub themes_shown: Option<String>,
    /// The orientation of the themes the Themes tab lists ([`THEME_AXES`]);
    /// without one, both.
    pub themes_axis: Option<String>,
    /// Bezel Light was switched off from its menu: its screens show the
    /// Bezel logo until it is switched on again. Only Light changes it.
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub light_off: bool,
}

/// A choice that starts on: absent from the file, it reads as `true`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct DefaultOn(pub bool);
impl Default for DefaultOn {
    fn default() -> Self {
        Self(true)
    }
}

/// What the Themes tab can list: the themes that fit the screen in use, or
/// all of them.
pub const THEME_SCOPES: [&str; 2] = ["screen", "all"];

/// The orientations the Themes tab can list: both, or one of them.
pub const THEME_AXES: [&str; 3] = ["all", "vertical", "horizontal"];

/// `text` when it is one of `known`.
fn known(known: &[&'static str], text: Option<&str>) -> Option<&'static str> {
    text.and_then(|t| known.iter().copied().find(|k| *k == t))
}

impl Settings {
    /// The options of the sensors that take settings.
    pub fn sensor_options(&self) -> SensorOptions {
        let defaults = SensorOptions::default();
        SensorOptions {
            ping_host: self.ping_host.clone().unwrap_or(defaults.ping_host),
            presentmon_path: self.presentmon_path.as_ref().map(PathBuf::from),
        }
    }

    /// Which themes the Themes tab lists, if a valid choice was stored.
    pub fn themes_shown(&self) -> Option<&'static str> {
        known(&THEME_SCOPES, self.themes_shown.as_deref())
    }

    /// The orientation of the themes the Themes tab lists (`all` unless a
    /// valid one was stored).
    pub fn themes_axis(&self) -> &'static str {
        known(&THEME_AXES, self.themes_axis.as_deref()).unwrap_or(THEME_AXES[0])
    }

    /// The language the user chose, if a valid one was stored.
    pub fn language(&self) -> Option<Language> {
        self.language.as_deref().and_then(parse_language)
    }

    /// The orientation last used with `screen`, if a valid one was stored.
    pub fn orientation_for(&self, screen: &str) -> Option<Orientation> {
        self.screen_orientations
            .get(screen)
            .and_then(|slug| parse_orientation(slug))
    }

    /// The theme Studio last had open for `screen`.
    pub fn editor_theme(&self, screen: &str) -> Option<&String> {
        self.editor_themes.as_ref()?.get(screen)
    }

    /// Studio's own theme memory, created when missing.
    pub fn editor_themes_mut(&mut self) -> &mut BTreeMap<String, String> {
        self.editor_themes.get_or_insert_with(BTreeMap::new)
    }

    /// Settings of an older version: Studio's open theme was Light's. Studio
    /// keeps remembering it, and every screen Light shows keeps the theme it
    /// showed (the last one opened when it had none of its own).
    fn migrate(&mut self) {
        if self.editor_themes.is_some() {
            return;
        }
        self.editor_themes = Some(self.screen_themes.clone());
        let shown: Vec<String> = if self.live_screens.is_empty() {
            self.live_screen.iter().cloned().collect()
        } else {
            self.live_screens.clone()
        };
        if let Some(last) = &self.last_theme {
            for key in shown {
                self.screen_themes
                    .entry(key)
                    .or_insert_with(|| last.clone());
            }
        }
    }

    /// Remembers `orientation` as the one last used with `screen`.
    pub fn remember_orientation(&mut self, screen: &str, orientation: Orientation) {
        self.screen_orientations.insert(
            screen.to_string(),
            orientation_slug(orientation).to_string(),
        );
    }
}

/// Reads and writes [`Settings`] in one file.
#[derive(Debug, Clone)]
pub struct SettingsFile {
    path: PathBuf,
}

impl SettingsFile {
    /// Settings stored at `path`.
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    /// Where the file is.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// The stored settings; defaults when the file is missing or unreadable.
    pub fn load(&self) -> Settings {
        let mut settings: Settings = std::fs::read(&self.path)
            .ok()
            .and_then(|bytes| serde_json::from_slice(&bytes).ok())
            .unwrap_or_default();
        settings.migrate();
        settings
    }

    /// Changes the stored settings with `edit` (read, edit, write).
    pub fn update(&self, edit: impl FnOnce(&mut Settings)) {
        let mut settings = self.load();
        edit(&mut settings);
        let write = || -> std::io::Result<()> {
            if let Some(parent) = self.path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let json = serde_json::to_vec_pretty(&settings).map_err(std::io::Error::other)?;
            std::fs::write(&self.path, json)
        };
        if write().is_err() {
            diag::report(DiagCode::SettingsNotSaved);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn audio_capture_is_on_unless_turned_off() {
        assert!(Settings::default().audio_capture.0);
        let old: Settings = serde_json::from_str(r#"{"lastTheme":"a.bezeltheme"}"#).unwrap();
        assert!(old.audio_capture.0);
        let off: Settings = serde_json::from_str(r#"{"audioCapture":false}"#).unwrap();
        assert!(!off.audio_capture.0);
        assert_eq!(serde_json::to_value(&off).unwrap()["audioCapture"], false);
    }

    #[test]
    fn idle_saver_is_on_unless_turned_off() {
        assert!(Settings::default().idle_saver.0);
        let old: Settings = serde_json::from_str(r#"{"audioCapture":false}"#).unwrap();
        assert!(old.idle_saver.0);
        let off: Settings = serde_json::from_str(r#"{"idleSaver":false}"#).unwrap();
        assert!(!off.idle_saver.0);
        assert!(off.audio_capture.0, "independent of the audio capture");
        assert_eq!(serde_json::to_value(&off).unwrap()["idleSaver"], false);
    }

    #[test]
    fn keeps_light_switched_off_and_writes_it_only_when_on() {
        let off: Settings = serde_json::from_str(r#"{"lightOff":true}"#).unwrap();
        assert!(off.light_off);
        assert_eq!(serde_json::to_value(&off).unwrap()["lightOff"], true);
        let value = serde_json::to_value(Settings::default()).unwrap();
        assert!(value.get("lightOff").is_none());
    }

    #[test]
    fn keeps_the_brightness_of_each_screen_and_writes_it_only_when_set() {
        let old: Settings = serde_json::from_str(r#"{"lastTheme":"a.bezeltheme"}"#).unwrap();
        assert!(old.screen_brightness.is_empty());
        let value = serde_json::to_value(Settings::default()).unwrap();
        assert!(value.get("screenBrightness").is_none());
        let set: Settings = serde_json::from_str(r#"{"screenBrightness":{"COM6":70}}"#).unwrap();
        assert_eq!(set.screen_brightness.get("COM6"), Some(&70));
        assert_eq!(
            serde_json::to_value(&set).unwrap()["screenBrightness"]["COM6"],
            70
        );
    }

    #[test]
    fn round_trips_and_tolerates_missing_or_broken_files() {
        let dir = std::env::temp_dir().join(format!("bezel-settings-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let file = SettingsFile::new(dir.join("nested/settings.json"));
        let fresh = Settings {
            editor_themes: Some(BTreeMap::new()),
            ..Settings::default()
        };
        assert_eq!(file.load(), fresh);
        file.update(|s| s.last_theme = Some("/t.bezeltheme".into()));
        file.update(|s| s.live_screen = Some("/dev/ttyACM1".into()));
        assert_eq!(
            file.load(),
            Settings {
                last_theme: Some("/t.bezeltheme".into()),
                live_screen: Some("/dev/ttyACM1".into()),
                editor_themes: Some(BTreeMap::new()),
                ..Settings::default()
            }
        );
        std::fs::write(file.path(), b"{ broken").unwrap();
        assert_eq!(file.load(), fresh);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn older_settings_split_studio_themes_from_lights_and_keep_what_light_shows() {
        let dir = tempfile::TempDir::new().unwrap();
        let file = SettingsFile::new(dir.path().join("settings.json"));
        std::fs::write(
            file.path(),
            br#"{"lastTheme":"/open.bezeltheme","liveScreens":["COM9","COM7"],
                "screenThemes":{"COM9":"/light.bezeltheme"}}"#,
        )
        .unwrap();
        let settings = file.load();
        assert_eq!(
            settings.editor_theme("COM9").map(String::as_str),
            Some("/light.bezeltheme")
        );
        assert_eq!(settings.screen_themes["COM9"], "/light.bezeltheme");
        assert_eq!(
            settings.screen_themes["COM7"], "/open.bezeltheme",
            "Light keeps showing what it showed"
        );
        file.update(|s| {
            s.editor_themes_mut()
                .insert("COM9".into(), "/other.bezeltheme".into());
        });
        let written: serde_json::Value =
            serde_json::from_slice(&std::fs::read(file.path()).unwrap()).unwrap();
        assert_eq!(written["editorThemes"]["COM9"], "/other.bezeltheme");
        assert_eq!(written["screenThemes"]["COM9"], "/light.bezeltheme");
    }

    #[test]
    fn legacy_light_on_close_still_loads_and_is_never_written() {
        // Closing Studio always hands over to Light (D-31): the old choice is ignored.
        let old: Settings =
            serde_json::from_str(r#"{"lastTheme":"saved.bezeltheme","lightOnClose":false}"#)
                .unwrap();
        assert_eq!(old.last_theme.as_deref(), Some("saved.bezeltheme"));
        assert!(
            !serde_json::to_string(&old)
                .unwrap()
                .contains("lightOnClose")
        );
    }

    #[test]
    fn remembers_the_orientation_of_each_screen() {
        let dir = std::env::temp_dir().join(format!("bezel-orient-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let file = SettingsFile::new(dir.join("settings.json"));
        assert_eq!(file.load().orientation_for("/dev/ttyACM1"), None);
        file.update(|s| s.remember_orientation("/dev/ttyACM1", Orientation::Landscape));
        file.update(|s| s.remember_orientation("COM3", Orientation::ReversePortrait));
        file.update(|s| s.remember_orientation("/dev/ttyACM1", Orientation::ReverseLandscape));
        let loaded = file.load();
        assert_eq!(
            loaded.orientation_for("/dev/ttyACM1"),
            Some(Orientation::ReverseLandscape)
        );
        assert_eq!(
            loaded.orientation_for("COM3"),
            Some(Orientation::ReversePortrait)
        );
        let json = std::fs::read_to_string(file.path()).unwrap();
        assert!(json.contains("\"screenOrientations\""), "{json}");
        assert!(json.contains("\"reverse-landscape\""), "{json}");

        // A file from an older version, or a value edited by hand, still loads.
        std::fs::write(
            file.path(),
            br#"{"lastTheme": "/t.bezeltheme", "screenOrientations": {"k": "sideways"}}"#,
        )
        .unwrap();
        let old = file.load();
        assert_eq!(old.last_theme.as_deref(), Some("/t.bezeltheme"));
        assert_eq!(old.orientation_for("k"), None);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn remembers_which_themes_the_gallery_lists() {
        let mut settings = Settings::default();
        assert_eq!(settings.themes_shown(), None, "not chosen yet");
        assert_eq!(settings.themes_axis(), "all");
        settings.themes_shown = Some("all".into());
        settings.themes_axis = Some("vertical".into());
        assert_eq!(settings.themes_shown(), Some("all"));
        assert_eq!(settings.themes_axis(), "vertical");
        // Edited by hand: ignored, never an error.
        settings.themes_shown = Some("mine".into());
        settings.themes_axis = Some("diagonal".into());
        assert_eq!(settings.themes_shown(), None);
        assert_eq!(settings.themes_axis(), "all");
    }
}
