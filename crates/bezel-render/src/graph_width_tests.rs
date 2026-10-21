//! Entire-frame pixel contracts for old themes and opt-in bar widths.
use crate::testkit::{Scene, element, key, render, renderer, theme};
use bezel_core::domain::frame::Rgba;
use bezel_core::domain::sensor::keys;
use bezel_core::domain::theme::{Background, Binding, BoxF, ElementKind, GraphStyle};
use bezel_core::domain::visualization::{VisualStyle, Visualization};

fn assert_bar_theme(width: Option<u32>, line_width: f32) {
    let mut graph = element(
        BoxF::new(0., 0., 80., 80.),
        ElementKind::Graph {
            binding: Binding {
                key: key(keys::CPU_USAGE),
                min: 0.,
                max: 100.,
            },
            history: 4,
            interval_seconds: None,
            style: GraphStyle::Bars,
            color: Rgba::WHITE,
            fill: None,
            line_width,
            autoscale: false,
        },
    );
    if let Some(width) = width {
        graph.visualization = Some(Visualization {
            bar_width: Some(width as f32),
            rounded_bars: false,
            ..Visualization::new(VisualStyle::Rounded)
        });
    }
    let theme = theme(80, 80, Background::Color(Rgba::BLACK), vec![graph]);
    let scene = Scene::empty().history(keys::CPU_USAGE, &[Some(100.), Some(50.), None, Some(75.)]);
    let frame = render(&mut renderer(), &theme, &scene);
    // Historical 20px slots have a 4px gap and 16px bars. This oracle is
    // independent of graph::draw_bars and checks every RGBA pixel, not probes.
    let bar_width = width.unwrap_or(16);
    let inset = (20 - bar_width) / 2;
    let mut expected = Vec::new();
    for y in 0..80 {
        for x in 0..80 {
            let slot = x / 20;
            let inside = x % 20 >= inset && x % 20 < inset + bar_width;
            let top = match slot {
                0 => 0,
                1 => 40,
                3 => 20,
                _ => 80,
            };
            let byte = if inside && y >= top { 255 } else { 0 };
            expected.extend_from_slice(&[byte, byte, byte, 255]);
        }
    }
    assert_eq!(frame.as_rgba(), expected);
}

#[test]
fn an_old_bar_theme_preserves_every_original_pixel_regardless_of_line_width() {
    for line_width in [2., 12.] {
        assert_bar_theme(None, line_width);
    }
}

#[test]
fn explicit_bar_width_reaches_the_renderer_and_changes_only_centered_bar_columns() {
    for width in [4, 12] {
        assert_bar_theme(Some(width), 2.);
    }
}
