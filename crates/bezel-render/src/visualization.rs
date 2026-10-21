//! Native geometry for block matrices, pixel arcs and dial decorations.
use crate::{
    gauges::{BarStyle, RingStyle},
    layer::Layer,
    paint::{paint_for, solid},
    path::{polar, rounded_rect},
};
use bezel_core::domain::{
    gradient::arc_color,
    theme::{BoxF, Direction, Paint},
    visualization::{VisualStyle, Visualization},
};
use tiny_skia::{PathBuilder, Stroke};

pub(crate) fn matrix(
    layer: &mut Layer<'_>,
    area: BoxF,
    value: Option<f64>,
    style: &BarStyle<'_>,
    v: &Visualization,
) {
    if !v.valid() {
        return;
    }
    let columns = usize::from(v.columns);
    let rows = usize::from(v.rows);
    let w = area.width / columns as f32;
    let h = area.height / rows as f32;
    let gap = v.gap.min(w * 0.8).min(h * 0.8);
    let count = columns * rows;
    let lit = (value.unwrap_or(0.).clamp(0., 1.) * count as f64).floor() as usize;
    for i in 0..count {
        let (x, y) = match style.direction {
            Direction::LeftToRight => (i % columns, rows - 1 - i / columns),
            Direction::RightToLeft => (columns - 1 - i % columns, rows - 1 - i / columns),
            Direction::BottomToTop => (i / rows, rows - 1 - i % rows),
            Direction::TopToBottom => (i / rows, i % rows),
        };
        let Some(path) = rounded_rect(
            area.x + x as f32 * w + gap / 2.,
            area.y + y as f32 * h + gap / 2.,
            w - gap,
            h - gap,
            v.radius,
        ) else {
            continue;
        };
        let p = if i < lit {
            Some(style.fill)
        } else {
            style.track
        };
        if let Some(p) = p.and_then(|p| paint_for(p, area)) {
            layer.fill(&path, &p);
        }
    }
}

pub(crate) fn pixel_arc(
    layer: &mut Layer<'_>,
    area: BoxF,
    value: Option<f64>,
    style: &RingStyle<'_>,
    v: &Visualization,
) {
    if !v.valid() {
        return;
    }
    let (cx, cy) = area.center();
    let count = usize::from(v.ticks);
    let side = style
        .thickness
        .min(area.width.min(area.height) / 4.)
        .max(1.);
    let r = (area.width.min(area.height) - side * 1.42) / 2.;
    if r <= 0. {
        return;
    }
    let sweep = style.sweep.clamp(0., 360.);
    let full = sweep >= 360.;
    let denominator = if full { count } else { count - 1 };
    let lit = (value.unwrap_or(0.).clamp(0., 1.) * count as f64).floor() as usize;
    for i in 0..count {
        let position = i as f32 / denominator as f32;
        let angle = style.start_angle + if style.clockwise { 1. } else { -1. } * sweep * position;
        let (x, y) = polar(cx, cy, r, angle);
        let Some(path) = rounded_rect(x - side / 2., y - side / 2., side, side, v.radius) else {
            continue;
        };
        if i < lit {
            let p = match style.fill {
                Paint::Arc {
                    start,
                    end,
                    transition,
                } => Some(solid(arc_color(*start, *end, position, *transition))),
                p => paint_for(p, area),
            };
            if let Some(p) = p {
                layer.fill(&path, &p);
            }
        } else if let Some(p) = style.track.and_then(|p| paint_for(p, area)) {
            layer.fill(&path, &p);
        }
    }
}

pub(crate) fn dial_area(area: BoxF) -> BoxF {
    let inset = area.width.min(area.height) * 0.12;
    BoxF::new(
        area.x + inset,
        area.y + inset,
        area.width - 2. * inset,
        area.height - 2. * inset,
    )
}

pub(crate) fn dial(
    layer: &mut Layer<'_>,
    area: BoxF,
    value: Option<f64>,
    style: &RingStyle<'_>,
    v: &Visualization,
) {
    if !v.valid() || v.style != VisualStyle::Dial {
        return;
    }
    let (cx, cy) = area.center();
    let r = area.width.min(area.height) / 2.;
    let dir = if style.clockwise { 1. } else { -1. };
    let mut ticks = PathBuilder::new();
    let count = usize::from(v.ticks);
    for i in 0..count {
        let angle =
            style.start_angle + dir * style.sweep.clamp(0., 360.) * i as f32 / (count - 1) as f32;
        let (x, y) = polar(cx, cy, r * 0.9, angle);
        let (xx, yy) = polar(cx, cy, r * if i % 5 == 0 { 0.98 } else { 0.95 }, angle);
        ticks.move_to(x, y);
        ticks.line_to(xx, yy);
    }
    if let Some(path) = ticks.finish() {
        let paint = style
            .track
            .or(Some(style.fill))
            .and_then(|p| paint_for(p, area));
        if let Some(p) = paint {
            layer.stroke(
                &path,
                &p,
                &Stroke {
                    width: 1.5,
                    ..Stroke::default()
                },
            );
        }
    }
    if let Some(value) = value.filter(|_| v.marker) {
        let position = value.clamp(0., 1.) as f32;
        let inner = dial_area(area);
        let radius = (inner.width.min(inner.height) - style.thickness) / 2.;
        let (x, y) = polar(
            cx,
            cy,
            radius,
            style.start_angle + dir * style.sweep.clamp(0., 360.) * position,
        );
        let mut pb = PathBuilder::new();
        pb.push_circle(x, y, (style.thickness * 0.6).max(v.radius));
        if let Some(path) = pb.finish() {
            let paint = match style.fill {
                Paint::Arc {
                    start,
                    end,
                    transition,
                } => Some(solid(arc_color(*start, *end, position, *transition))),
                p => paint_for(p, area),
            };
            if let Some(p) = paint {
                if let Some(track) = style.track.and_then(|p| paint_for(p, area)) {
                    layer.fill(&path, &track);
                }
                layer.stroke(
                    &path,
                    &p,
                    &Stroke {
                        width: 2.5,
                        ..Stroke::default()
                    },
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testkit::{self, Scene, count, element, key, render, theme};
    use bezel_core::domain::{
        frame::Rgba,
        history::{Histories, Tick},
        sensor::{Reading, Snapshot},
        theme::{Background, Binding, Cap, ElementKind, GraphStyle},
    };

    fn binding(name: &str) -> Binding {
        Binding {
            key: key(name),
            min: 0.,
            max: 100.,
        }
    }

    #[test]
    fn matrix_lights_only_complete_cells_and_missing_readings_are_empty() {
        let mut e = element(
            BoxF::new(0., 0., 80., 40.),
            ElementKind::Bar {
                binding: binding("a"),
                direction: Direction::LeftToRight,
                fill: Paint::Solid(Rgba::opaque(255, 0, 0)),
                track: Some(Paint::Solid(Rgba::opaque(30, 30, 30))),
                radius: 0.,
                segments: None,
            },
        );
        let mut v = Visualization::new(VisualStyle::Matrix);
        v.columns = 4;
        v.rows = 2;
        v.gap = 2.;
        v.radius = 0.;
        e.visualization = Some(v);
        let t = theme(80, 40, Background::Color(Rgba::BLACK), vec![e]);
        let mut r = testkit::renderer();
        let half = render(&mut r, &t, &Scene::empty().with("a", 50.));
        assert_eq!(count(&half, (0, 0, 80, 20), |p| p.r > 200), 0);
        assert!(count(&half, (0, 20, 80, 40), |p| p.r > 200) > 1000);
        let empty = render(&mut r, &t, &Scene::empty());
        assert_eq!(count(&empty, (0, 0, 80, 40), |p| p.r > 200), 0);
    }

    #[test]
    fn arcs_and_dials_stay_inside_their_box_and_change_with_the_value() {
        let mut r = testkit::renderer();
        for style in [VisualStyle::Pixel, VisualStyle::Dial] {
            let mut e = element(
                BoxF::new(10., 10., 100., 100.),
                ElementKind::Ring {
                    test_full: false,
                    binding: binding("a"),
                    start_angle: -135.,
                    sweep: 270.,
                    thickness: 8.,
                    clockwise: true,
                    fill: Paint::Arc {
                        start: Rgba::opaque(0, 255, 255),
                        end: Rgba::opaque(128, 0, 255),
                        transition: 50.,
                    },
                    track: Some(Paint::Solid(Rgba::opaque(30, 30, 30))),
                    cap: Cap::Round,
                    segments: None,
                },
            );
            e.visualization = Some(Visualization::new(style));
            let t = theme(120, 120, Background::Color(Rgba::BLACK), vec![e]);
            let a = render(&mut r, &t, &Scene::empty().with("a", 0.));
            let b = render(&mut r, &t, &Scene::empty().with("a", 100.));
            assert_ne!(a, b);
            assert_eq!(count(&b, (0, 0, 120, 10), |p| p != Rgba::BLACK), 0);
            assert!(count(&b, (10, 10, 110, 110), |p| p.b > 100) > 50);
        }
    }

    #[test]
    fn mirrored_graph_uses_an_independent_lower_history_and_collects_it() {
        let mut e = element(
            BoxF::new(0., 0., 100., 80.),
            ElementKind::Graph {
                binding: binding("a"),
                history: 4,
                interval_seconds: None,
                style: GraphStyle::Area,
                color: Rgba::opaque(0, 255, 255),
                fill: None,
                line_width: 2.,
                autoscale: true,
            },
        );
        let mut v = Visualization::new(VisualStyle::Mirrored);
        v.secondary = Some(binding("b"));
        e.visualization = Some(v);
        let t = theme(100, 80, Background::Color(Rgba::BLACK), vec![e]);
        assert!(t.sensor_keys().contains(&key("b")));
        assert_eq!(t.history_lengths().len(), 2);
        let mut scene = Scene::empty();
        scene.histories = Histories::new(&t.graph_series());
        for _ in 0..4 {
            let mut s = Snapshot::default();
            s.insert(key("a"), Reading::Value(80.));
            scene.histories.push(&s, Tick::Untimed);
        }
        let mut r = testkit::renderer();
        let missing = render(&mut r, &t, &scene);
        assert!(count(&missing, (0, 0, 100, 39), |p| p.g > 30) > 50);
        assert_eq!(count(&missing, (0, 41, 100, 80), |p| p.r > 30), 0);
        for _ in 0..4 {
            let mut s = Snapshot::default();
            s.insert(key("a"), Reading::Value(80.));
            s.insert(key("b"), Reading::Value(20.));
            scene.histories.push(&s, Tick::Untimed);
        }
        let both = render(&mut r, &t, &scene);
        assert!(count(&both, (0, 41, 100, 80), |p| p.r > 30) > 50);
        let mut single = t.clone();
        single.elements[0].visualization.as_mut().unwrap().style = VisualStyle::Rounded;
        assert!(!single.sensor_keys().contains(&key("b")));
        assert_eq!(single.history_lengths().len(), 1);
        assert_eq!(single.graph_series().len(), 1);
        assert!(
            single.elements[0]
                .visualization
                .as_ref()
                .unwrap()
                .secondary
                .is_some(),
            "settings are retained for switching back"
        );
    }

    #[test]
    fn mirrored_bars_choose_square_or_rounded_corners_and_keep_the_series_gap() {
        let mut e = element(
            BoxF::new(0., 0., 100., 80.),
            ElementKind::Graph {
                binding: binding("a"),
                history: 4,
                interval_seconds: None,
                style: GraphStyle::Bars,
                color: Rgba::WHITE,
                fill: None,
                line_width: 2.,
                autoscale: false,
            },
        );
        let mut visual = Visualization::new(VisualStyle::Mirrored);
        visual.series_gap = 12.;
        visual.rounded_bars = false;
        visual.secondary = Some(binding("b"));
        e.visualization = Some(visual);
        let mut theme = theme(100, 80, Background::Color(Rgba::BLACK), vec![e]);
        let mut scene = Scene::empty();
        scene.histories = Histories::new(&theme.graph_series());
        for _ in 0..4 {
            let mut sample = Snapshot::default();
            sample.insert(key("a"), Reading::Value(80.));
            sample.insert(key("b"), Reading::Value(60.));
            scene.histories.push(&sample, Tick::Untimed);
        }
        let mut renderer = testkit::renderer();
        let square = render(&mut renderer, &theme, &scene);
        theme.elements[0]
            .visualization
            .as_mut()
            .unwrap()
            .rounded_bars = true;
        let rounded = render(&mut renderer, &theme, &scene);
        for image in [&square, &rounded] {
            assert_eq!(count(image, (0, 34, 100, 46), |p| p != Rgba::BLACK), 0);
            assert!(count(image, (0, 0, 100, 34), |p| p.r > 30) > 50);
            assert!(count(image, (0, 46, 100, 80), |p| p.r > 30) > 50);
        }
        assert!(
            count(&square, (0, 0, 100, 80), |p| p.r > 30)
                > count(&rounded, (0, 0, 100, 80), |p| p.r > 30)
        );
    }
}
