//! Analog clocks (D-2026-10-20-studio-ux-v2-07): a round dial inscribed in
//! the element's box with hour marks (and minute marks), numbers, the date
//! and the hour, minute and second hands. Sizes follow the dial: they are the
//! design's 64-unit dial scaled to the box.

use std::collections::BTreeMap;

use bezel_core::domain::clock::{
    AnalogClock, DialNumbers, DialTicks, HandShape, Language, LocalTime,
};
use bezel_core::domain::frame::Rgba;
use bezel_core::domain::theme::{AssetRef, BoxF, HAlign, Paint, TextStyle, VAlign};
use tiny_skia::{LineCap, PathBuilder, Stroke};

use crate::diagnostics::Diagnostics;
use crate::layer::Layer;
use crate::paint::solid;
use crate::path::polar;
use crate::text::{TextEngine, TextJob};

/// Radius of the dial in design units (of a 64-unit box).
const DIAL: f32 = 28.0;
/// Hour marks run from this radius inward …
const MARK_OUTER: f32 = 26.0;
/// … to this one.
const HOUR_MARK_INNER: f32 = 21.0;
/// Minute marks are shorter.
const MINUTE_MARK_INNER: f32 = 24.0;
/// Where the numbers stand, without and with marks (roman ones, wider, a
/// unit further in).
const NUMBERS_FREE: f32 = 21.0;
const NUMBERS_MARKED: f32 = 15.5;
/// Size of the numbers (roman ones smaller) and of the date.
const NUMBER_SIZE: f32 = 6.5;
const ROMAN_SIZE: f32 = 5.0;
const DATE_SIZE: f32 = 5.0;
/// Where the date stands below the centre, inside the numbers.
const DATE_AT: f32 = 9.0;
/// Hands: length, width, tail.
const HOUR_HAND: (f32, f32, f32) = (11.7, 3.5, 0.0);
const MINUTE_HAND: (f32, f32, f32) = (20.9, 2.5, 0.0);
const SECOND_HAND: (f32, f32, f32) = (23.0, 1.2, 6.0);
/// Radius of the hub over the hands.
const HUB: f32 = 2.5;
/// Opacity of the outline, a fainter version of the marks' colour.
const BORDER_ALPHA: f32 = 0.3;

/// The dial of a box: centre and the scale of a design unit.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Dial {
    /// Centre.
    pub center: (f32, f32),
    /// Pixels per design unit.
    pub unit: f32,
}

impl Dial {
    /// The dial inscribed in `area`.
    pub fn of(area: BoxF) -> Self {
        Self {
            center: area.center(),
            unit: area.width.min(area.height).max(0.0) / 64.0,
        }
    }

    /// The point `radius` design units from the centre at `angle`.
    pub fn at(&self, radius: f32, angle: f32) -> (f32, f32) {
        polar(self.center.0, self.center.1, radius * self.unit, angle)
    }
}

/// The tip and tail ends of a hand of `(length, _, tail)` at `angle`.
pub(crate) fn hand_ends(dial: Dial, hand: (f32, f32, f32), angle: f32) -> ((f32, f32), (f32, f32)) {
    let (length, _, tail) = hand;
    (dial.at(length, angle), dial.at(tail, angle + 180.0))
}

/// Fonts and assets for the numbers and the date.
pub(crate) struct Words<'a> {
    pub engine: &'a mut TextEngine,
    pub assets: &'a BTreeMap<AssetRef, Vec<u8>>,
    pub diagnostics: &'a mut Diagnostics,
    /// The element's text style: its face writes the numbers.
    pub style: &'a TextStyle,
}

/// Draws `clock` at `time` in `area`; `sweep` lets the second hand sweep.
pub(crate) fn draw(
    layer: &mut Layer<'_>,
    area: BoxF,
    clock: &AnalogClock,
    time: &LocalTime,
    language: Language,
    sweep: bool,
    words: Words<'_>,
) {
    let dial = Dial::of(area);
    let u = dial.unit;
    if u.is_nan() || u <= 0.0 {
        return;
    }
    let (cx, cy) = dial.center;
    if let Some(face) = PathBuilder::from_circle(cx, cy, DIAL * u) {
        layer.fill(&face, &solid(clock.face));
        if clock.border > 0.0 {
            let mut outline = clock.marks;
            outline.a = (f32::from(outline.a) * BORDER_ALPHA).round() as u8;
            let stroke = Stroke {
                width: clock.border,
                ..Stroke::default()
            };
            layer.stroke(&face, &solid(outline), &stroke);
        }
    }
    marks(layer, dial, clock);
    let numerals = clock.numerals();
    let roman = clock.numbers == DialNumbers::Roman;
    let (radius, size) = match (clock.ticks == DialTicks::None, roman) {
        (true, false) => (NUMBERS_FREE, NUMBER_SIZE),
        (true, true) => (NUMBERS_FREE - 1.0, ROMAN_SIZE),
        (false, false) => (NUMBERS_MARKED, NUMBER_SIZE),
        (false, true) => (NUMBERS_MARKED - 1.0, ROMAN_SIZE),
    };
    let mut words = words;
    for (hour, text) in numerals {
        let (x, y) = dial.at(radius, f32::from(hour) * 30.0);
        write(&mut words, layer, &text, (x, y), size * u, clock.marks);
    }
    let date = clock.date_text(time, language);
    if !date.is_empty() {
        let at = (cx, cy + DATE_AT * u);
        write(&mut words, layer, &date, at, DATE_SIZE * u, clock.marks);
    }
    let angles = clock.angles(time, sweep);
    let hands = [
        (clock.hour_hand, HOUR_HAND, angles.hour, clock.hour_color),
        (
            clock.minute_hand,
            MINUTE_HAND,
            angles.minute,
            clock.minute_color,
        ),
        (
            clock.second_hand,
            SECOND_HAND,
            angles.second,
            clock.second_color,
        ),
    ];
    for (shown, hand, angle, color) in hands {
        if shown {
            draw_hand(layer, dial, hand, angle, color, clock.hand_shape);
        }
    }
    let hub = [
        (clock.second_hand, clock.second_color),
        (clock.minute_hand, clock.minute_color),
        (clock.hour_hand, clock.hour_color),
    ]
    .into_iter()
    .find_map(|(shown, color)| shown.then_some(color));
    if let (Some(color), Some(circle)) = (hub, PathBuilder::from_circle(cx, cy, HUB * u)) {
        layer.fill(&circle, &solid(color));
    }
}

fn marks(layer: &mut Layer<'_>, dial: Dial, clock: &AnalogClock) {
    let count = match clock.ticks {
        DialTicks::None => return,
        DialTicks::Hours => 12,
        DialTicks::Minutes => 60,
    };
    let paint = solid(clock.marks);
    for index in 0..count {
        let hourly = count == 12 || index % 5 == 0;
        let (inner, width) = if hourly {
            (HOUR_MARK_INNER, 2.0)
        } else {
            (MINUTE_MARK_INNER, 1.0)
        };
        let angle = index as f32 * 360.0 / count as f32;
        let (x0, y0) = dial.at(MARK_OUTER, angle);
        let (x1, y1) = dial.at(inner, angle);
        let mut pb = PathBuilder::new();
        pb.move_to(x0, y0);
        pb.line_to(x1, y1);
        if let Some(line) = pb.finish() {
            let stroke = Stroke {
                width: width * dial.unit,
                ..Stroke::default()
            };
            layer.stroke(&line, &paint, &stroke);
        }
    }
}

fn draw_hand(
    layer: &mut Layer<'_>,
    dial: Dial,
    hand: (f32, f32, f32),
    angle: f32,
    color: Rgba,
    shape: HandShape,
) {
    let ((tx, ty), (bx, by)) = hand_ends(dial, hand, angle);
    let width = hand.1 * dial.unit;
    let paint = solid(color);
    let (width, cap) = match shape {
        HandShape::Thin => (width * 0.6, LineCap::Butt),
        HandShape::Rounded => (width, LineCap::Round),
        HandShape::Arrow => (width * 0.8, LineCap::Butt),
    };
    let mut pb = PathBuilder::new();
    pb.move_to(bx, by);
    pb.line_to(tx, ty);
    if let Some(line) = pb.finish() {
        let stroke = Stroke {
            width,
            line_cap: cap,
            ..Stroke::default()
        };
        layer.stroke(&line, &paint, &stroke);
    }
    if shape == HandShape::Arrow {
        // A head as long as three widths and two and a half wide.
        let head = (width * 3.0).min(hand.0 * dial.unit * 0.5);
        let base = (
            tx - head * angle.to_radians().sin(),
            ty + head * angle.to_radians().cos(),
        );
        let side = width * 1.25;
        let (nx, ny) = (angle.to_radians().cos(), angle.to_radians().sin());
        let mut pb = PathBuilder::new();
        pb.move_to(tx, ty);
        pb.line_to(base.0 + nx * side, base.1 + ny * side);
        pb.line_to(base.0 - nx * side, base.1 - ny * side);
        pb.close();
        if let Some(arrow) = pb.finish() {
            layer.fill(&arrow, &paint);
        }
    }
}

/// Writes `text` centred on `(x, y)` in the element's face, `size` px.
pub(crate) fn write(
    words: &mut Words<'_>,
    layer: &mut Layer<'_>,
    text: &str,
    (x, y): (f32, f32),
    size: f32,
    color: Rgba,
) {
    if size < 1.0 {
        return;
    }
    let style = TextStyle {
        size,
        paint: Paint::solid(color),
        align: HAlign::Center,
        valign: VAlign::Middle,
        letter_spacing: 0.0,
        tabular_digits: false,
        overflow: None,
        parts: Default::default(),
        ..words.style.clone()
    };
    let area = BoxF::new(x - size * 2.0, y - size, size * 4.0, size * 2.0);
    let job = TextJob {
        text,
        style: &style,
        area,
    };
    words
        .engine
        .draw(layer, &job, words.assets, words.diagnostics);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: (f32, f32), b: (f32, f32)) -> bool {
        (a.0 - b.0).abs() < 1e-3 && (a.1 - b.1).abs() < 1e-3
    }

    #[test]
    fn hands_point_where_the_time_says() {
        let dial = Dial::of(BoxF::new(0.0, 0.0, 128.0, 128.0));
        assert_eq!(dial.unit, 2.0);
        let clock = AnalogClock::default();
        let time = |hour, minute, second| LocalTime {
            year: 2026,
            month: 10,
            day: 20,
            hour,
            minute,
            second,
            weekday: 1,
            millisecond: 0,
        };
        let three = clock.angles(&time(3, 0, 0), true);
        let (tip, _) = hand_ends(dial, HOUR_HAND, three.hour);
        assert!(
            close(tip, (64.0 + 23.4, 64.0)),
            "3:00 points right: {tip:?}"
        );
        let (tip, _) = hand_ends(dial, MINUTE_HAND, three.minute);
        assert!(close(tip, (64.0, 64.0 - 41.8)), "minutes up");

        let half_six = clock.angles(&time(6, 30, 0), true);
        let (tip, _) = hand_ends(dial, MINUTE_HAND, half_six.minute);
        assert!(close(tip, (64.0, 64.0 + 41.8)), "minutes down");
        let (tip, _) = hand_ends(dial, HOUR_HAND, half_six.hour);
        assert!(
            tip.0 < 64.0 && tip.1 > 64.0 + 22.0,
            "between 6 and 7, left of centre: {tip:?}"
        );

        let late = clock.angles(&time(21, 45, 30), true);
        let (tip, tail) = hand_ends(dial, SECOND_HAND, late.second);
        assert!(close(tip, (64.0, 64.0 + 46.0)), "seconds down at :30");
        assert!(close(tail, (64.0, 64.0 - 12.0)), "with a tail");
        let (tip, _) = hand_ends(dial, MINUTE_HAND, late.minute);
        assert!(
            tip.0 < 64.0 - 41.0 && tip.1 < 64.0,
            "just past 45, a little up: {tip:?}"
        );
        let (tip, _) = hand_ends(dial, HOUR_HAND, late.hour);
        assert!(tip.0 < 64.0 - 20.0 && tip.1 < 64.0, "past 9: {tip:?}");
    }
}
