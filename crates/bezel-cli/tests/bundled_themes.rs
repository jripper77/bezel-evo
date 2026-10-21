//! The themes that ship in `themes/`: each loads with the native store, fits
//! a known panel, reads only sensors the demo catalog has, uses only the
//! bundled fonts and renders at its canvas size without a single problem.
//!
//! Set `BEZEL_THEME_PREVIEWS=<folder>` to also write a PNG of each theme
//! (English and Brazilian Portuguese dates) after a minute of busy sensors;
//! otherwise two frames are enough to check them.
#![allow(clippy::expect_used, clippy::panic)] // helpers of a failing test panic

mod support;

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use bezel_core::app::ThemeRuntime;
use bezel_core::domain::catalog::MODELS;
use bezel_core::domain::clock::Language;
use bezel_core::domain::frame::Frame;
use bezel_core::domain::theme::{AssetRef, ElementKind, TextContent, Theme};
use bezel_core::ports::{SensorSource, ThemeLocation, ThemeStore};
use bezel_sensors::FakeSensors;
use bezel_themes::FsThemeStore;

/// Samples graphs get before a preview is written.
const PREVIEW_FRAMES: usize = 60;

type Assets = BTreeMap<AssetRef, Vec<u8>>;

fn load(dir: &Path) -> (Theme, Assets) {
    FsThemeStore
        .load(&ThemeLocation(dir.display().to_string()))
        .unwrap_or_else(|e| panic!("{}: {e}", dir.display()))
}

fn families(theme: &Theme) -> BTreeSet<String> {
    theme
        .elements
        .iter()
        .filter_map(|e| match &e.kind {
            ElementKind::Text { style, .. } => Some(style.font.family.clone()),
            _ => None,
        })
        .collect()
}

/// Every sensor the visible elements read.
fn sensor_keys(theme: &Theme) -> BTreeSet<String> {
    theme
        .sensor_keys()
        .iter()
        .map(|k| k.as_str().to_string())
        .collect()
}

fn fits_a_panel(theme: &Theme) -> bool {
    MODELS.iter().any(|m| theme.misfit(m.panel).is_none())
}

fn save_png(frame: &Frame, path: &Path) {
    let size = frame.size();
    image::RgbaImage::from_raw(size.width, size.height, frame.as_rgba().to_vec())
        .expect("frame bytes")
        .save(path)
        .expect("preview written");
}

fn render(
    theme: &Theme,
    assets: &Assets,
    language: Language,
    frames: usize,
) -> (Frame, Vec<String>) {
    let mut renderer = support::renderer();
    let mut sensors = support::busy_sensors(frames);
    let mut runtime = ThemeRuntime::new(theme.clone(), assets.clone(), language);
    let mut frame = None;
    for _ in 0..frames {
        frame = Some(
            runtime
                .frame(&mut sensors, &mut renderer, support::TIME)
                .expect("frame"),
        );
    }
    (frame.expect("rendered"), renderer.problems())
}

#[test]
fn every_bundled_theme_renders_cleanly() {
    let themes = support::bundled_themes();
    let count = themes.len();
    let offered: BTreeSet<String> = FakeSensors::demo()
        .catalog()
        .expect("catalog")
        .into_iter()
        .map(|s| s.key.as_str().to_string())
        .collect();
    let previews = std::env::var_os("BEZEL_THEME_PREVIEWS").map(PathBuf::from);
    let (frames, languages) = match previews {
        Some(_) => (
            PREVIEW_FRAMES,
            &[(Language::English, ""), (Language::PortugueseBr, ".pt")][..],
        ),
        None => (2, &[(Language::English, "")][..]),
    };
    for dir in themes {
        let name = dir
            .file_name()
            .expect("name")
            .to_string_lossy()
            .into_owned();
        let (theme, assets) = load(&dir);
        assert!(fits_a_panel(&theme), "{name}: {:?}", theme.canvas);
        for asset in theme.assets() {
            assert!(assets.contains_key(&asset), "{name}: {} missing", asset.0);
        }
        for key in sensor_keys(&theme) {
            let weather_key = theme.elements.iter().any(|e| {
                matches!(&e.kind, ElementKind::Text { content: TextContent::Weather(w), .. }
                    if w.keys().iter().any(|k| k.as_str() == key))
            });
            assert!(
                offered.contains(&key)
                    || weather_key
                    || matches!(key.as_str(), "media.sessions" | "activity.processes"),
                "{name}: {key} is not a demo or configured weather sensor"
            );
        }
        let fonts = families(&theme);
        assert!(
            fonts.iter().all(|f| bezel_render::SHIPPED_FAMILIES
                .iter()
                .any(|(shipped, _)| f == shipped)),
            "{name}: {fonts:?}"
        );
        for (language, suffix) in languages {
            let (frame, problems) = render(&theme, &assets, *language, frames);
            assert_eq!(frame.size(), theme.canvas, "{name}");
            assert!(problems.is_empty(), "{name}: {problems:?}");
            if let Some(dir) = &previews {
                std::fs::create_dir_all(dir).expect("preview folder");
                save_png(&frame, &dir.join(format!("{name}{suffix}.png")));
            }
        }
    }
    assert!(count >= 6, "{count} bundled themes");
}

/// Exercise the new templates with measured values, local weather, music/cover,
/// and missing inputs. Optional pictures use exactly these labelled test fixtures.
#[test]
fn studio_ux_v2_templates_render_live_and_unavailable_inputs() {
    use bezel_core::domain::playback::MediaSession;
    use bezel_core::domain::sensor::{Reading, Snapshot};
    use std::io::Cursor;
    let mut cover = Cursor::new(Vec::new());
    image::RgbaImage::from_pixel(320, 320, image::Rgba([124, 92, 255, 255]))
        .write_to(&mut cover, image::ImageFormat::Png)
        .expect("test album cover");
    for name in [
        "pixel-wide",
        "nebula-wide",
        "paper-wide",
        "now-playing-wide",
    ] {
        let (theme, assets) = load(&support::themes_dir().join(name));
        assert_eq!(
            (theme.canvas.width, theme.canvas.height),
            (1920, 480),
            "{name}"
        );
        let ids: BTreeSet<_> = theme.elements.iter().map(|e| e.id).collect();
        assert_eq!(
            ids.len(),
            theme.elements.len(),
            "{name}: unique editable objects"
        );
        for e in &theme.elements {
            assert!(
                e.frame.x >= 0.
                    && e.frame.y >= 0.
                    && e.frame.x + e.frame.width <= 1920.
                    && e.frame.y + e.frame.height <= 480.,
                "{name}: {} outside canvas",
                e.name
            );
        }
        let mut source = support::busy_sensors(60);
        let catalog = source.catalog().expect("catalog");
        let mut measured = Vec::new();
        for _ in 0..60 {
            let mut snapshot = source.sample().expect("sample");
            for e in &theme.elements {
                if let ElementKind::Text {
                    content: TextContent::Weather(w),
                    ..
                } = &e.kind
                {
                    let [temperature, condition] = w.keys();
                    snapshot.insert(temperature, Reading::Value(18.));
                    snapshot.insert(condition, Reading::Value(3.));
                }
            }
            // A measured FFT fixture, not the theme's synthetic-preview switch.
            for e in &theme.elements {
                if let ElementKind::Text {
                    content: TextContent::Spectrum(config),
                    ..
                } = &e.kind
                {
                    use bezel_core::domain::spectrum::{AudioState, SpectrumFrame};
                    snapshot.spectra.insert(
                        e.id,
                        SpectrumFrame {
                            state: AudioState::Live,
                            levels: (0..config.bands)
                                .map(|i| (f32::from(i) * 0.43).sin().abs() * 0.85)
                                .collect(),
                            peaks: Vec::new(),
                        },
                    );
                }
            }
            snapshot.media.push(MediaSession {
                source: "Fixture player".into(),
                title: "A live session".into(),
                artist: "Fixture artist".into(),
                playing: true,
                position: 72.,
                duration: 225.,
                cover: cover.get_ref().clone(),
            });
            measured.push(snapshot);
        }
        let mut sensors = FakeSensors::new(catalog, measured);
        let mut runtime = ThemeRuntime::new(theme.clone(), assets.clone(), Language::Italian);
        let mut renderer = support::renderer();
        let mut frame = None;
        let pictures = std::env::var_os("BEZEL_UX2_THEME_PREVIEWS").map(PathBuf::from);
        for _ in 0..if pictures.is_some() { 60 } else { 2 } {
            frame = Some(
                runtime
                    .frame(&mut sensors, &mut renderer, support::TIME)
                    .expect("measured frame"),
            );
        }
        let live = frame.expect("live frame");
        assert!(
            renderer.problems().is_empty(),
            "{name}: {:?}",
            renderer.problems()
        );
        let mut missing_runtime = ThemeRuntime::new(theme, assets, Language::Italian);
        let mut missing = FakeSensors::new(Vec::new(), vec![Snapshot::default()]);
        let unavailable = missing_runtime
            .frame(&mut missing, &mut renderer, support::TIME)
            .expect("unavailable frame");
        assert!(
            renderer.problems().is_empty(),
            "{name}: {:?}",
            renderer.problems()
        );
        assert_ne!(
            live.as_rgba(),
            unavailable.as_rgba(),
            "{name}: reacts to real inputs"
        );
        if let Some(dir) = pictures {
            std::fs::create_dir_all(&dir).expect("preview directory");
            save_png(&live, &dir.join(format!("{name}.it.png")));
            save_png(
                &unavailable,
                &dir.join(format!("{name}.unavailable.it.png")),
            );
        }
    }
}
