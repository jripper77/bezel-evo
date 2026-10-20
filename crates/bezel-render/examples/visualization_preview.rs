//! Render a native visualization reference sheet with deterministic histories.
//! Usage: cargo run -p bezel-render --example visualization_preview -- theme.json out.png
use bezel_core::{
    domain::{
        clock::{Language, LocalTime},
        history::{Histories, Tick},
        sensor::{Quantities, Reading, Snapshot},
        transition::ShownValues,
    },
    ports::{Backdrop, FrameRenderer, RenderContext},
};
use std::{collections::BTreeMap, time::Duration};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let input = args.next().ok_or("expected input manifest")?;
    let output = args.next().ok_or("expected output PNG")?;
    let theme = bezel_themes::native::parse_manifest(&std::fs::read(input)?)?;
    let mut histories = Histories::new(&theme.graph_series());
    let mut snapshot = Snapshot::default();
    for i in 0..60 {
        snapshot = Snapshot::default();
        for key in theme.sensor_keys() {
            let offset = key.as_str().bytes().map(u32::from).sum::<u32>() as f64;
            let value = if key.as_str().contains("lower") {
                12. + 8. * (i as f64 * 0.47).sin().abs()
            } else {
                45. + 30. * (i as f64 * 0.23 + offset).sin()
            };
            snapshot.insert(key, Reading::Value(value));
        }
        histories.push(&snapshot, Tick::Untimed);
    }
    for (name, value) in [
        ("cpu.usage", 24.),
        ("memory.percent", 61.),
        ("visual.dial", 38.),
    ] {
        snapshot.insert(
            bezel_core::domain::sensor::SensorKey::new(name).ok_or("invalid preview key")?,
            Reading::Value(value),
        );
    }
    let assets = BTreeMap::new();
    let quantities = Quantities::new();
    let shown = ShownValues::default();
    let mut renderer = bezel_render::SkiaRenderer::new();
    let frame = renderer.render(
        &theme,
        &assets,
        RenderContext {
            snapshot: &snapshot,
            histories: &histories,
            quantities: &quantities,
            time: LocalTime {
                year: 2026,
                month: 10,
                day: 20,
                hour: 12,
                minute: 0,
                second: 0,
                weekday: 2,
                millisecond: 0,
            },
            animation: Duration::ZERO,
            language: Language::English,
            backdrop: Backdrop::Poster,
            shown: &shown,
        },
    )?;
    let size = frame.size();
    image::RgbaImage::from_raw(size.width, size.height, frame.as_rgba().to_vec())
        .ok_or("invalid RGBA size")?
        .save(&output)?;
    println!(
        "Rendered {}x{} native preview: {output}",
        size.width, size.height
    );
    Ok(())
}
