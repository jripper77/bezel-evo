//! The extended needle gauge (D-2026-10-20-studio-ux-v2-08): the arc round
//! the pivot with its zones or gradient, scale ticks and numbers, the shaped
//! hand with its tail, the peak mark and the pivot cap. The arc is as large
//! as the box lets it be around the pivot.

use bezel_core::domain::clock::Language;
use bezel_core::domain::frame::Rgba;
use bezel_core::domain::needle::{NeedleLook, NeedleShape, PivotCap, Track, scale_number, share};
use bezel_core::domain::theme::{Binding, BoxF, Cap};
use tiny_skia::{LineCap, PathBuilder, Stroke, StrokeDash};

use crate::analog_clock::{Words, write};
use crate::layer::Layer;
use crate::paint::solid;
use crate::path::{polar, push_arc};

/// Degrees of arc per segment of a gradient.
const GRADIENT_STEP: f32 = 2.0;
/// Opacity of the peak mark.
const PEAK_ALPHA: f32 = 0.35;

/// Where the gauge stands: pivot, arc radius and angles.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Gauge {
    /// Pivot, canvas pixels.
    pub pivot: (f32, f32),
    /// Radius of the middle of the arc.
    pub radius: f32,
    /// Angle of the scale's start, degrees clockwise from 12.
    pub start: f32,
    /// Angle from start to end.
    pub sweep: f32,
}

impl Gauge {
    /// The largest arc round `pivot` (0..=1 of the box) over `start..start +
    /// sweep` that stays `margin` inside `area`.
    pub fn fit(area: BoxF, pivot: (f32, f32), start: f32, sweep: f32, margin: f32) -> Self {
        let px = area.x + pivot.0 * area.width;
        let py = area.y + pivot.1 * area.height;
        let sweep = sweep.clamp(-360.0, 360.0);
        let mut angles = vec![start, start + sweep];
        for cardinal in [
            0.0f32, 90.0, 180.0, 270.0, 360.0, -90.0, -180.0, -270.0, -360.0,
        ] {
            let within = if sweep >= 0.0 {
                cardinal > start && cardinal < start + sweep
            } else {
                cardinal < start && cardinal > start + sweep
            };
            if within {
                angles.push(cardinal);
            }
        }
        let room = |space: f32, reach: f32| {
            if reach > 1e-6 {
                (space - margin).max(0.0) / reach
            } else {
                f32::INFINITY
            }
        };
        let radius = angles
            .iter()
            .map(|a| {
                let (ux, uy) = (a.to_radians().sin(), -a.to_radians().cos());
                room(px - area.x, -ux)
                    .min(room(area.x + area.width - px, ux))
                    .min(room(py - area.y, -uy))
                    .min(room(area.y + area.height - py, uy))
            })
            .fold(f32::INFINITY, f32::min);
        Self {
            pivot: (px, py),
            radius: if radius.is_finite() { radius } else { 0.0 },
            start,
            sweep,
        }
    }

    /// Angle of `share` (0..=1) of the scale.
    pub fn angle(&self, share: f64) -> f32 {
        self.start + self.sweep * share as f32
    }

    /// The point at `radius` and `angle`.
    pub fn at(&self, radius: f32, angle: f32) -> (f32, f32) {
        polar(self.pivot.0, self.pivot.1, radius, angle)
    }
}

/// What the needle shows now.
pub(crate) struct Reading<'a> {
    /// Range of the scale.
    pub binding: &'a Binding,
    /// Value shown (moving or read); `None` rests at the start.
    pub value: Option<f64>,
    /// The peak mark's value, when it shows above the value.
    pub peak: Option<f64>,
    /// Needle colour (after colour rules).
    pub color: Rgba,
    /// Needle width, pixels.
    pub width: f32,
    /// Language of the scale numbers.
    pub language: Language,
}

/// Draws the arc, scale, peak mark, hand (unless an image replaces it) and
/// cap of `look` in `gauge`.
pub(crate) fn draw(
    layer: &mut Layer<'_>,
    gauge: Gauge,
    look: &NeedleLook,
    reading: &Reading<'_>,
    drawn_hand: bool,
    words: &mut Words<'_>,
) {
    if gauge.radius <= 0.0 {
        return;
    }
    arc(layer, gauge, look, reading.binding);
    scale(layer, gauge, look, reading, words);
    let reach = gauge.radius * look.length / 100.0;
    if let Some(peak) = reading.peak {
        let mut ghost = look.marks;
        ghost.a = (f32::from(ghost.a) * PEAK_ALPHA).round() as u8;
        let angle = gauge.angle(share(reading.binding, peak));
        let mut pb = PathBuilder::new();
        pb.move_to(gauge.pivot.0, gauge.pivot.1);
        let tip = gauge.at(reach, angle);
        pb.line_to(tip.0, tip.1);
        if let Some(line) = pb.finish() {
            let dash = (reading.width * 0.8).max(2.0);
            let stroke = Stroke {
                width: (reading.width * 0.5).max(1.0),
                dash: StrokeDash::new(vec![dash, dash], 0.0),
                ..Stroke::default()
            };
            layer.stroke(&line, &solid(ghost), &stroke);
        }
    }
    let color = look.needle_color(reading.value, reading.color);
    if drawn_hand {
        let angle = gauge.angle(reading.value.map_or(0.0, |v| share(reading.binding, v)));
        hand(layer, gauge, look, angle, color, reading.width);
    }
    let radius = look.cap_diameter / 2.0;
    if look.cap != PivotCap::None
        && radius > 0.0
        && let Some(disc) = PathBuilder::from_circle(gauge.pivot.0, gauge.pivot.1, radius)
    {
        layer.fill(&disc, &solid(look.cap_color));
        if look.cap == PivotCap::Ring {
            let stroke = Stroke {
                width: (radius * 0.45).clamp(1.0, 4.0),
                ..Stroke::default()
            };
            layer.stroke(&disc, &solid(color), &stroke);
        }
    }
}

fn arc(layer: &mut Layer<'_>, gauge: Gauge, look: &NeedleLook, binding: &Binding) {
    if look.arc_width <= 0.0 {
        return;
    }
    let stroke = |cap: Cap| Stroke {
        width: look.arc_width,
        line_cap: if cap == Cap::Round {
            LineCap::Round
        } else {
            LineCap::Butt
        },
        ..Stroke::default()
    };
    let piece = |layer: &mut Layer<'_>, from: f64, to: f64, color: Rgba, cap: Cap| {
        let (a, b) = (gauge.angle(from), gauge.angle(to));
        let mut pb = PathBuilder::new();
        push_arc(
            &mut pb,
            (gauge.pivot.0, gauge.pivot.1, gauge.radius),
            a,
            b - a,
        );
        if let Some(path) = pb.finish() {
            layer.stroke(&path, &solid(color), &stroke(cap));
        }
    };
    match look.track {
        Track::Single => piece(layer, 0.0, 1.0, look.faint(), look.arc_cap),
        Track::Zones => {
            piece(layer, 0.0, 1.0, look.faint(), look.arc_cap);
            for zone in look
                .zones
                .iter()
                .take(bezel_core::domain::needle::MAX_ZONES)
            {
                let (a, b) = (share(binding, zone.from), share(binding, zone.to));
                if (a - b).abs() > 1e-9 {
                    piece(layer, a.min(b), a.max(b), zone.color, Cap::Butt);
                }
            }
        }
        Track::Gradient => {
            let steps = ((gauge.sweep.abs() / GRADIENT_STEP).ceil() as usize).max(1);
            for i in 0..steps {
                let (a, b) = (i as f64 / steps as f64, (i + 1) as f64 / steps as f64);
                let cap = if i == 0 || i + 1 == steps {
                    look.arc_cap
                } else {
                    Cap::Butt
                };
                // A sliver of overlap hides the seams between segments.
                let b = (b + 0.002).min(1.0);
                piece(layer, a, b, look.gradient_at(binding, (a + b) / 2.0), cap);
            }
        }
    }
}

/// Length of a major tick for an arc of `radius`.
pub(crate) fn major_tick(radius: f32) -> f32 {
    (radius * 0.1).max(4.0)
}

fn scale(
    layer: &mut Layer<'_>,
    gauge: Gauge,
    look: &NeedleLook,
    reading: &Reading<'_>,
    words: &mut Words<'_>,
) {
    let inner = gauge.radius - look.arc_width / 2.0 - 2.0;
    let (major, minor) = look.tick_shares();
    let paint = solid(look.marks);
    let long = major_tick(gauge.radius);
    for (shares, length, width) in [
        (&major, long, (gauge.radius / 60.0).max(1.5)),
        (&minor, long * 0.5, (gauge.radius / 120.0).max(1.0)),
    ] {
        for s in shares {
            let angle = gauge.angle(*s);
            let (x0, y0) = gauge.at(inner, angle);
            let (x1, y1) = gauge.at(inner - length, angle);
            let mut pb = PathBuilder::new();
            pb.move_to(x0, y0);
            pb.line_to(x1, y1);
            if let Some(line) = pb.finish() {
                let stroke = Stroke {
                    width,
                    ..Stroke::default()
                };
                layer.stroke(&line, &paint, &stroke);
            }
        }
    }
    let shares = look.number_shares();
    if shares.is_empty() {
        return;
    }
    let size = (gauge.radius * 0.12).clamp(8.0, 64.0);
    let ticks = if major.is_empty() { 0.0 } else { long };
    let at = inner - ticks - look.number_gap - size * 0.6;
    let binding = reading.binding;
    for s in shares {
        let value = binding.min + (binding.max - binding.min) * s;
        let text = scale_number(value, reading.language);
        let point = gauge.at(at, gauge.angle(s));
        write(words, layer, &text, point, size, look.marks);
    }
}

fn hand(
    layer: &mut Layer<'_>,
    gauge: Gauge,
    look: &NeedleLook,
    angle: f32,
    color: Rgba,
    width: f32,
) {
    let tip = gauge.at(gauge.radius * look.length / 100.0, angle);
    let tail = gauge.at(gauge.radius * look.tail / 100.0, angle + 180.0);
    let paint = solid(color);
    let line = |layer: &mut Layer<'_>, width: f32, cap: LineCap| {
        let mut pb = PathBuilder::new();
        pb.move_to(tail.0, tail.1);
        pb.line_to(tip.0, tip.1);
        if let Some(path) = pb.finish() {
            let stroke = Stroke {
                width,
                line_cap: cap,
                ..Stroke::default()
            };
            layer.stroke(&path, &paint, &stroke);
        }
    };
    let (nx, ny) = (angle.to_radians().cos(), angle.to_radians().sin());
    match look.shape {
        NeedleShape::Thin => line(layer, width, LineCap::Butt),
        NeedleShape::Tapered => {
            // Widest at the pivot, a point at the tip, narrower at the tail.
            let (px, py) = gauge.pivot;
            let half = width.max(1.0);
            let mut pb = PathBuilder::new();
            pb.move_to(tip.0, tip.1);
            pb.line_to(px + nx * half, py + ny * half);
            pb.line_to(tail.0 + nx * half * 0.5, tail.1 + ny * half * 0.5);
            pb.line_to(tail.0 - nx * half * 0.5, tail.1 - ny * half * 0.5);
            pb.line_to(px - nx * half, py - ny * half);
            pb.close();
            if let Some(path) = pb.finish() {
                layer.fill(&path, &paint);
            }
        }
        NeedleShape::Arrow => {
            line(layer, width, LineCap::Butt);
            let head = width * 3.5;
            let (dx, dy) = (angle.to_radians().sin(), -angle.to_radians().cos());
            let base = (tip.0 - dx * head, tip.1 - dy * head);
            let side = width * 1.5;
            let mut pb = PathBuilder::new();
            pb.move_to(tip.0 + dx * width, tip.1 + dy * width);
            pb.line_to(base.0 + nx * side, base.1 + ny * side);
            pb.line_to(base.0 - nx * side, base.1 - ny * side);
            pb.close();
            if let Some(path) = pb.finish() {
                layer.fill(&path, &paint);
            }
        }
        NeedleShape::Dot => {
            line(layer, (width * 0.6).max(1.0), LineCap::Butt);
            if let Some(dot) = PathBuilder::from_circle(tip.0, tip.1, (width * 1.5).max(2.0)) {
                layer.fill(&dot, &paint);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: (f32, f32), b: (f32, f32)) -> bool {
        (a.0 - b.0).abs() < 1e-3 && (a.1 - b.1).abs() < 1e-3
    }

    #[test]
    fn the_arc_is_as_large_as_the_box_allows() {
        let area = BoxF::new(0.0, 0.0, 200.0, 200.0);
        let half = Gauge::fit(area, (0.5, 0.5), -90.0, 180.0, 0.0);
        assert_eq!(half.pivot, (100.0, 100.0));
        assert!((half.radius - 100.0).abs() < 1e-3);
        let low = Gauge::fit(area, (0.5, 0.9), -90.0, 180.0, 5.0);
        assert!(
            (low.radius - 95.0).abs() < 1e-3,
            "the sides limit it: {}",
            low.radius
        );
        let wide = Gauge::fit(area, (0.5, 0.9), -120.0, 240.0, 0.0);
        assert!(
            (wide.radius - 40.0).abs() < 1e-3,
            "the arc dips below the pivot: {}",
            wide.radius
        );
        let top = Gauge::fit(area, (0.5, 0.9), -45.0, 90.0, 0.0);
        assert!(
            (top.radius - 141.421).abs() < 0.01,
            "only the corners reach out: {}",
            top.radius
        );
        let reversed = Gauge::fit(area, (0.5, 0.5), 90.0, -180.0, 0.0);
        assert!((reversed.radius - 100.0).abs() < 1e-3);
        assert!(close(half.at(10.0, 90.0), (110.0, 100.0)));
        assert_eq!(half.angle(0.5), 0.0);
        assert_eq!(major_tick(20.0), 4.0);
        assert_eq!(major_tick(100.0), 10.0);
        let flat = Gauge::fit(BoxF::new(0.0, 0.0, 0.0, 0.0), (0.5, 0.5), 0.0, 0.0, 0.0);
        assert_eq!(flat.radius, 0.0);
    }
}
