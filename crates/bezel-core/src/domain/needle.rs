//! The extended needle gauge (D-2026-10-20-studio-ux-v2-08): an arc with
//! coloured zones, scale ticks and numbers, a shaped hand with a tail, a
//! pivot cap, the value written under the pivot and a peak mark that holds
//! the highest reading for a while.
//!
//! A needle without a [`NeedleLook`] keeps the original look (a line from
//! the pivot to the box edge): themes written before it draw the same.

use std::time::Duration;

use super::clock::Language;
use super::frame::Rgba;
use super::sensor::DisplayFormat;
use super::theme::{Binding, Cap, TextStyle};

/// Most colour zones on an arc.
pub const MAX_ZONES: usize = 8;

/// How the arc behind the needle is painted.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Track {
    /// One colour: the marks' colour, faint.
    #[default]
    Single,
    /// Each zone in its colour, the rest as [`Track::Single`].
    Zones,
    /// The zones' colours blending into each other along the arc.
    Gradient,
}

/// Scale ticks inside the arc.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ScaleTicks {
    /// None.
    None,
    /// Every tenth of the scale.
    Major,
    /// Every tenth, and small ones every fiftieth.
    #[default]
    MajorMinor,
}

/// Scale numbers inside the ticks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ScaleNumbers {
    /// None.
    None,
    /// The two ends.
    Ends,
    /// The ends and the middle.
    #[default]
    Halves,
    /// At every major tick.
    Major,
}

/// Shape of the drawn hand.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NeedleShape {
    /// A line of even width.
    Thin,
    /// Wide at the pivot, pointed at the tip.
    #[default]
    Tapered,
    /// A line with an arrowhead.
    Arrow,
    /// A line ending in a dot.
    Dot,
}

/// What covers the pivot.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PivotCap {
    /// Nothing.
    None,
    /// A disc in the cap colour.
    Filled,
    /// A disc in the cap colour ringed in the needle's colour.
    #[default]
    Ring,
}

/// A stretch of the scale with its colour, in the sensor's units.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Zone {
    /// Where it starts.
    pub from: f64,
    /// Where it ends.
    pub to: f64,
    /// Its colour.
    pub color: Rgba,
}

/// The value written under the pivot, with a value text's options.
#[derive(Debug, Clone, PartialEq)]
pub struct CenterValue {
    /// How it looks (parts, equal digits, overflow).
    pub style: TextStyle,
    /// Number formatting.
    pub format: DisplayFormat,
    /// Text before the value.
    pub prefix: String,
    /// Text after the value.
    pub suffix: String,
}

/// Everything the extended needle adds.
#[derive(Debug, Clone, PartialEq)]
pub struct NeedleLook {
    /// Arc thickness, pixels (0: no arc).
    pub arc_width: f32,
    /// Arc ends.
    pub arc_cap: Cap,
    /// How the arc is painted.
    pub track: Track,
    /// Colour zones, at most [`MAX_ZONES`].
    pub zones: Vec<Zone>,
    /// The needle takes the colour of the zone its value is in.
    pub color_from_zone: bool,
    /// Scale ticks.
    pub ticks: ScaleTicks,
    /// Scale numbers.
    pub numbers: ScaleNumbers,
    /// Space between the ticks and the numbers, pixels.
    pub number_gap: f32,
    /// Colour of the ticks, the numbers and the single arc.
    pub marks: Rgba,
    /// Shape of the hand.
    pub shape: NeedleShape,
    /// Length of the hand, percent of the arc's radius.
    pub length: f32,
    /// Length of the tail behind the pivot, percent of the radius.
    pub tail: f32,
    /// What covers the pivot.
    pub cap: PivotCap,
    /// Diameter of the cap, pixels.
    pub cap_diameter: f32,
    /// Colour of the cap.
    pub cap_color: Rgba,
    /// The value under the pivot.
    pub center: Option<CenterValue>,
    /// A faint hand stays on the highest reading for this many seconds.
    pub peak_seconds: Option<u32>,
}

impl Default for NeedleLook {
    fn default() -> Self {
        Self {
            arc_width: 5.0,
            arc_cap: Cap::Round,
            track: Track::Single,
            zones: Vec::new(),
            color_from_zone: false,
            ticks: ScaleTicks::MajorMinor,
            numbers: ScaleNumbers::Halves,
            number_gap: 8.0,
            marks: Rgba::opaque(0xae, 0xb5, 0xbd),
            shape: NeedleShape::Tapered,
            length: 92.0,
            tail: 12.0,
            cap: PivotCap::Ring,
            cap_diameter: 9.0,
            cap_color: Rgba::opaque(0x15, 0x19, 0x1c),
            center: None,
            peak_seconds: None,
        }
    }
}

/// Share of the scale `value` stands at, held within 0..=1 (0 for an empty
/// or broken range).
pub fn share(binding: &Binding, value: f64) -> f64 {
    let span = binding.max - binding.min;
    if !span.is_finite() || span == 0.0 || !value.is_finite() {
        return 0.0;
    }
    ((value - binding.min) / span).clamp(0.0, 1.0)
}

impl NeedleLook {
    /// The zone `value` is in (the last one listed when they overlap).
    pub fn zone_at(&self, value: f64) -> Option<&Zone> {
        self.zones
            .iter()
            .take(MAX_ZONES)
            .rev()
            .find(|z| value >= z.from.min(z.to) && value <= z.from.max(z.to))
    }

    /// The needle's colour at `value`: its zone's when it takes it.
    pub fn needle_color(&self, value: Option<f64>, color: Rgba) -> Rgba {
        match value {
            Some(v) if self.color_from_zone => self.zone_at(v).map_or(color, |z| z.color),
            _ => color,
        }
    }

    /// The colour of the arc at `share` of the scale (0..=1) for a
    /// gradient: the zones' colours at their middles, blended in between
    /// and held past the first and the last; the faint marks' colour
    /// without zones.
    pub fn gradient_at(&self, binding: &Binding, at: f64) -> Rgba {
        let mut stops: Vec<(f64, Rgba)> = self
            .zones
            .iter()
            .take(MAX_ZONES)
            .map(|z| (share(binding, (z.from + z.to) / 2.0), z.color))
            .collect();
        stops.sort_by(|a, b| a.0.total_cmp(&b.0));
        let Some(first) = stops.first() else {
            return self.faint();
        };
        if at <= first.0 {
            return first.1;
        }
        for pair in stops.windows(2) {
            let ((a, ca), (b, cb)) = (pair[0], pair[1]);
            if at <= b {
                let t = if b > a { (at - a) / (b - a) } else { 1.0 };
                return mix(ca, cb, t as f32);
            }
        }
        stops.last().map_or(self.faint(), |s| s.1)
    }

    /// The single arc's colour: the marks' colour at 30 %.
    pub fn faint(&self) -> Rgba {
        let mut c = self.marks;
        c.a = (f32::from(c.a) * 0.3).round() as u8;
        c
    }

    /// Shares of the scale where ticks stand: major ones, then minor ones.
    pub fn tick_shares(&self) -> (Vec<f64>, Vec<f64>) {
        let major = || (0..=10).map(|i| f64::from(i) / 10.0);
        match self.ticks {
            ScaleTicks::None => (Vec::new(), Vec::new()),
            ScaleTicks::Major => (major().collect(), Vec::new()),
            ScaleTicks::MajorMinor => (
                major().collect(),
                (0..=50)
                    .filter(|i| i % 5 != 0)
                    .map(|i| f64::from(i) / 50.0)
                    .collect(),
            ),
        }
    }

    /// Shares of the scale that carry a number.
    pub fn number_shares(&self) -> Vec<f64> {
        match self.numbers {
            ScaleNumbers::None => Vec::new(),
            ScaleNumbers::Ends => vec![0.0, 1.0],
            ScaleNumbers::Halves => vec![0.0, 0.5, 1.0],
            ScaleNumbers::Major => (0..=10).map(|i| f64::from(i) / 10.0).collect(),
        }
    }
}

fn mix(a: Rgba, b: Rgba, t: f32) -> Rgba {
    let t = t.clamp(0.0, 1.0);
    let lerp = |x: u8, y: u8| (f32::from(x) + (f32::from(y) - f32::from(x)) * t).round() as u8;
    Rgba {
        r: lerp(a.r, b.r),
        g: lerp(a.g, b.g),
        b: lerp(a.b, b.b),
        a: lerp(a.a, b.a),
    }
}

/// A scale number written for `language`: whole numbers as they are, others
/// with at most two decimals, a decimal comma in Italian and Portuguese.
pub fn scale_number(value: f64, language: Language) -> String {
    if !value.is_finite() {
        return "—".to_string();
    }
    let rounded = (value * 100.0).round() / 100.0;
    let mut text = if rounded.fract() == 0.0 {
        format!("{rounded:.0}")
    } else {
        let mut t = format!("{rounded:.2}");
        while t.ends_with('0') {
            t.pop();
        }
        t
    };
    if text == "-0" {
        text = "0".to_string();
    }
    match language {
        Language::English => text,
        Language::Italian | Language::PortugueseBr => text.replace('.', ","),
    }
}

/// The peak mark of one needle: the highest value since it was last let go,
/// held for a while.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Peak {
    value: f64,
    since: Duration,
}

impl Peak {
    /// Starting at `value` at `now`.
    pub fn new(value: f64, now: Duration) -> Self {
        Self { value, since: now }
    }

    /// A new `value` at `now`: a higher one becomes the peak; after `hold`
    /// without a higher one the mark goes back down to the value.
    pub fn update(&mut self, value: f64, now: Duration, hold: Duration) -> f64 {
        if !value.is_finite() {
            return self.value;
        }
        if value >= self.value || now.saturating_sub(self.since) >= hold || now < self.since {
            *self = Self::new(value, now);
        }
        self.value
    }

    /// The value it marks.
    pub fn value(&self) -> f64 {
        self.value
    }

    /// When it goes back down, if it stands above `value`.
    pub fn falls_at(&self, value: f64, hold: Duration) -> Option<Duration> {
        (self.value > value).then_some(self.since + hold)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::sensor::SensorKey;

    fn binding() -> Binding {
        Binding {
            key: SensorKey::new("cpu.usage").expect("key"),
            min: 0.0,
            max: 100.0,
        }
    }

    const GREEN: Rgba = Rgba::opaque(0, 255, 0);
    const YELLOW: Rgba = Rgba::opaque(255, 255, 0);
    const RED: Rgba = Rgba::opaque(255, 0, 0);

    fn zoned() -> NeedleLook {
        NeedleLook {
            zones: vec![
                Zone {
                    from: 0.0,
                    to: 60.0,
                    color: GREEN,
                },
                Zone {
                    from: 60.0,
                    to: 85.0,
                    color: YELLOW,
                },
                Zone {
                    from: 85.0,
                    to: 100.0,
                    color: RED,
                },
            ],
            color_from_zone: true,
            ..NeedleLook::default()
        }
    }

    #[test]
    fn shares_of_the_scale() {
        let b = binding();
        assert_eq!(share(&b, 25.0), 0.25);
        assert_eq!(share(&b, 150.0), 1.0);
        assert_eq!(share(&b, -5.0), 0.0);
        assert_eq!(share(&b, f64::NAN), 0.0);
        let empty = Binding { max: 0.0, ..b };
        assert_eq!(share(&empty, 3.0), 0.0);
    }

    #[test]
    fn the_needle_takes_its_zones_colour() {
        let look = zoned();
        assert_eq!(look.needle_color(Some(30.0), RED), GREEN);
        assert_eq!(look.needle_color(Some(70.0), RED), YELLOW);
        assert_eq!(
            look.needle_color(Some(60.0), RED),
            YELLOW,
            "the later zone wins a boundary"
        );
        assert_eq!(
            look.needle_color(Some(120.0), GREEN),
            GREEN,
            "outside every zone"
        );
        assert_eq!(look.needle_color(None, GREEN), GREEN);
        let plain = NeedleLook {
            color_from_zone: false,
            ..zoned()
        };
        assert_eq!(plain.needle_color(Some(90.0), GREEN), GREEN);
        let reversed = NeedleLook {
            zones: vec![Zone {
                from: 50.0,
                to: 10.0,
                color: RED,
            }],
            ..zoned()
        };
        assert_eq!(reversed.zone_at(20.0).map(|z| z.color), Some(RED));
    }

    #[test]
    fn a_gradient_blends_the_zones() {
        let look = zoned();
        let b = binding();
        assert_eq!(
            look.gradient_at(&b, 0.0),
            GREEN,
            "held before the first middle"
        );
        assert_eq!(look.gradient_at(&b, 1.0), RED, "and after the last");
        let between = look.gradient_at(&b, (0.30 + 0.725) / 2.0);
        assert_eq!((between.r, between.g, between.b), (128, 255, 0));
        let none = NeedleLook::default();
        assert_eq!(none.gradient_at(&b, 0.5), none.faint());
        assert_eq!(none.faint().a, 77);
    }

    #[test]
    fn ticks_and_numbers_on_the_scale() {
        let mut look = NeedleLook::default();
        let (major, minor) = look.tick_shares();
        assert_eq!((major.len(), minor.len()), (11, 40));
        look.ticks = ScaleTicks::Major;
        assert_eq!(look.tick_shares().1.len(), 0);
        look.ticks = ScaleTicks::None;
        assert!(look.tick_shares().0.is_empty());
        assert_eq!(look.number_shares(), vec![0.0, 0.5, 1.0]);
        look.numbers = ScaleNumbers::Ends;
        assert_eq!(look.number_shares(), vec![0.0, 1.0]);
        look.numbers = ScaleNumbers::Major;
        assert_eq!(look.number_shares().len(), 11);
        look.numbers = ScaleNumbers::None;
        assert!(look.number_shares().is_empty());
    }

    #[test]
    fn scale_numbers_follow_the_language() {
        assert_eq!(scale_number(50.0, Language::Italian), "50");
        assert_eq!(scale_number(2.5, Language::Italian), "2,5");
        assert_eq!(scale_number(2.5, Language::English), "2.5");
        assert_eq!(scale_number(1.0 / 3.0, Language::PortugueseBr), "0,33");
        assert_eq!(scale_number(-0.001, Language::English), "0");
        assert_eq!(scale_number(f64::NAN, Language::English), "—");
    }

    #[test]
    fn the_peak_holds_the_highest_value_then_falls() {
        let hold = Duration::from_secs(10);
        let s = Duration::from_secs;
        let mut peak = Peak::new(30.0, s(0));
        assert_eq!(peak.update(80.0, s(1), hold), 80.0);
        assert_eq!(peak.update(40.0, s(5), hold), 80.0, "held");
        assert_eq!(peak.falls_at(40.0, hold), Some(s(11)));
        assert_eq!(
            peak.update(90.0, s(8), hold),
            90.0,
            "a higher one restarts it"
        );
        assert_eq!(peak.update(50.0, s(17), hold), 90.0);
        assert_eq!(peak.update(50.0, s(18), hold), 50.0, "falls after the hold");
        assert_eq!(peak.falls_at(50.0, hold), None);
        assert_eq!(peak.update(f64::NAN, s(19), hold), 50.0);
        assert_eq!(
            peak.update(20.0, s(2), hold),
            20.0,
            "a clock that went back restarts"
        );
        assert_eq!(peak.value(), 20.0);
    }
}
