//! Render shipped templates with representative data, including every card face.
use crate::{SkiaRenderer, SystemFonts, testkit};
use bezel_core::{
    domain::{
        frame::Rgba,
        history::{Histories, Tick},
        playback::MediaSession,
        sensor::Reading,
        spectrum::{AudioState, SpectrumFrame},
        theme::{ElementKind, TextContent},
    },
    ports::{FrameRenderer, ThemeLocation, ThemeStore},
};
use std::{path::Path, time::Duration};

#[test]
fn coordinated_templates_load_and_render_with_live_features() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../themes");
    let mut renderer = SkiaRenderer::with_fonts(
        crate::SHIPPED_FONTS.iter().map(|f| f.to_vec()).collect(),
        SystemFonts::Skip,
    );
    for collection in [
        "pulse-gaming",
        "resonance-music",
        "clarity-monitor",
        "synapse-kinetic",
        "prism-kinetic",
        "morph-sound",
    ] {
        for format in ["wide", "round"] {
            let slug = format!("{collection}-{format}");
            let (mut theme, assets) = bezel_themes::FsThemeStore
                .load(&ThemeLocation(root.join(&slug).display().to_string()))
                .unwrap();
            let mut scene = testkit::Scene::empty();
            scene.assets = assets;
            scene.histories = Histories::new(&theme.graph_series());
            for n in 0..90 {
                for (key, base) in [
                    ("cpu.usage", 37.0),
                    ("gpu.usage", 64.0),
                    ("memory.percent", 42.0),
                    ("cpu.temperature", 56.0),
                    ("gpu.temperature", 63.0),
                    ("system.volume", 48.0),
                    ("system.uptime", 86400.0),
                ] {
                    let value = base + (f64::from(n) / 7.0).sin() * 9.0;
                    scene
                        .snapshot
                        .insert(testkit::key(key), Reading::Value(value));
                }
                scene.histories.push(&scene.snapshot, Tick::Untimed);
            }
            scene.snapshot.media_available = true;
            scene.snapshot.media.push(MediaSession {
                source: "Music player".into(),
                title: "A little further".into(),
                artist: "The night session".into(),
                playing: true,
                position: 84.0,
                duration: 224.0,
                cover: testkit::png(256, 256, |x, y| {
                    Rgba::opaque((60 + x / 2) as u8, (40 + y / 3) as u8, 160)
                }),
            });
            for e in &theme.elements {
                assert!(e.frame.x >= 0.0 && e.frame.y >= 0.0);
                assert!(e.frame.x + e.frame.width <= theme.canvas.width as f32);
                assert!(e.frame.y + e.frame.height <= theme.canvas.height as f32);
                if let ElementKind::Text {
                    content: TextContent::Sensor { key, .. },
                    ..
                } = &e.kind
                {
                    assert!(
                        scene.snapshot.get(key).value().is_some(),
                        "unrecognized sensor {key:?}"
                    );
                }
                if let ElementKind::Text {
                    content: TextContent::Spectrum(spectrum),
                    ..
                } = &e.kind
                {
                    assert!(!spectrum.demo, "shipped templates use live audio");
                    let levels = (0..spectrum.bands)
                        .map(|i| 0.15 + ((f32::from(i) * 0.3).sin() + 1.0) * 0.32)
                        .collect::<Vec<_>>();
                    scene.snapshot.spectra.insert(
                        e.id,
                        SpectrumFrame {
                            state: AudioState::Live,
                            peaks: levels.clone(),
                            beat: None,
                            levels,
                        },
                    );
                }
            }
            for face in 0..2 {
                renderer.set_scene(face);
                for e in &mut theme.elements {
                    if let Some(card) = &mut e.card {
                        card.active_face = face as usize;
                    }
                }
                scene.animation = Duration::from_secs(face);
                let frame = testkit::render(&mut renderer, &theme, &scene);
                assert_eq!(frame.size(), theme.canvas);
                if let Some(dir) = std::env::var_os("BEZEL_THEME_PREVIEWS") {
                    std::fs::create_dir_all(&dir).unwrap();
                    image::save_buffer(
                        Path::new(&dir).join(format!("{slug}-{face}.png")),
                        frame.as_rgba(),
                        theme.canvas.width,
                        theme.canvas.height,
                        image::ColorType::Rgba8,
                    )
                    .unwrap();
                }
            }
            let moving = theme
                .elements
                .iter()
                .filter_map(|e| {
                    if let ElementKind::Image { asset, .. } = &e.kind {
                        asset.0.ends_with("motion.gif").then_some(asset)
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>();
            for asset in &moving {
                let timeline = renderer
                    .animation(asset, &scene.assets)
                    .expect("animated asset");
                assert_eq!(timeline.len(), 40);
                assert!(timeline.next_change(Duration::ZERO) >= Duration::from_millis(100));
                assert!(timeline.total() >= Duration::from_secs(4));
            }
            if !moving.is_empty() {
                scene.animation = Duration::ZERO;
                let first = testkit::render(&mut renderer, &theme, &scene);
                scene.animation = Duration::from_millis(125);
                let second = testkit::render(&mut renderer, &theme, &scene);
                assert_ne!(first, second, "kinetic theme must actually change pixels");
                if let Some(dir) = std::env::var_os("BEZEL_KINETIC_PREVIEW_FRAMES") {
                    let folder = Path::new(&dir).join(&slug);
                    std::fs::create_dir_all(&folder).unwrap();
                    for n in 0..40 {
                        scene.animation = Duration::from_millis(n * 120);
                        // Representative audio motion for exported previews only.
                        for spectrum in scene.snapshot.spectra.values_mut() {
                            for (i, level) in spectrum.levels.iter_mut().enumerate() {
                                *level =
                                    0.15 + ((i as f32 * 0.3 + n as f32 * 0.15).sin() + 1.0) * 0.32;
                            }
                            spectrum.peaks.clone_from(&spectrum.levels);
                        }
                        let frame = testkit::render(&mut renderer, &theme, &scene);
                        image::save_buffer(
                            folder.join(format!("{n:02}.png")),
                            frame.as_rgba(),
                            theme.canvas.width,
                            theme.canvas.height,
                            image::ColorType::Rgba8,
                        )
                        .unwrap();
                    }
                }
            }
        }
    }
}
