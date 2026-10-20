//! Weather configuration, location keys and localised condition names.
use std::time::Duration;

use super::clock::Language;
use super::sensor::{Reading, SensorKey, Snapshot};

/// Appearance of automatic condition icons.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum IconStyle {
    /// Stroked outlines.
    #[default]
    Outline,
    /// Filled shapes.
    Filled,
    /// Per-component weather colors.
    Colored,
    /// Shaded vectors with depth.
    Dimensional,
}

/// A portable weather object. Coordinates are selected by city search.
#[derive(Debug, Clone, PartialEq)]
pub struct Weather {
    /// Displayed location name.
    pub city: String,
    /// Latitude in degrees.
    pub latitude: f64,
    /// Longitude in degrees.
    pub longitude: f64,
    /// None follows the system language.
    pub language: Option<Language>,
    /// Display Fahrenheit instead of Celsius.
    pub fahrenheit: bool,
    /// Show the condition icon beside the text.
    pub show_icon: bool,
    /// Automatic icon appearance.
    pub icon_style: IconStyle,
    /// Gap to text in pixels; None preserves the automatic legacy spacing.
    pub icon_gap: Option<f32>,
    /// Icon size in pixels; None follows the text size.
    pub icon_size: Option<f32>,
    /// How the icon moves (D-2026-10-20-studio-ux-v2-09); still for themes
    /// written before it.
    pub icon_motion: IconMotion,
    /// Speed of the icon's movement, [`ICON_SPEED`].
    pub icon_speed: f32,
}

/// How a weather icon moves.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum IconMotion {
    /// It does not move: the icon as it always was.
    #[default]
    Still,
    /// Half the movement.
    Gentle,
    /// The whole movement.
    Lively,
}

impl IconMotion {
    /// Share of the movement: 0, 0.5 or 1.
    pub fn amplitude(self) -> f32 {
        match self {
            IconMotion::Still => 0.0,
            IconMotion::Gentle => 0.5,
            IconMotion::Lively => 1.0,
        }
    }

    /// Time between two frames while it moves; none when still.
    pub fn step(self) -> Option<Duration> {
        match self {
            IconMotion::Still => None,
            IconMotion::Gentle => Some(Duration::from_millis(50)),
            IconMotion::Lively => Some(Duration::from_millis(33)),
        }
    }
}

/// Allowed range of [`Weather::icon_speed`].
pub const ICON_SPEED: std::ops::RangeInclusive<f32> = 0.5..=2.0;

/// How long the cross-fade between two condition icons lasts.
pub const ICON_FADE: Duration = Duration::from_millis(300);

/// The pose of a moving weather icon at one moment
/// (`design/weather-motion.md`): offsets in the units of a 64-unit icon,
/// opacities 0..=1. At rest every offset is 0 and every opacity 1.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct IconPose {
    /// Turn of the sun's rays, degrees.
    pub rays: f32,
    /// Growth of the sun's disc, as a share of its radius.
    pub disc: f32,
    /// Sideways shift of the cloud (and of the 3D shadow).
    pub cloud: f32,
    /// Fog strips: sideways shift and opacity.
    pub strips: [(f32, f32); 4],
    /// Rain drops or snowflakes: shift and opacity.
    pub drops: [(f32, f32, f32); 3],
    /// Opacity of the lightning bolt.
    pub bolt: f32,
    /// Opacity of the whole icon (the unavailable one breathes).
    pub whole: f32,
    /// Opacity of the 3D sun's halo (0 without).
    pub halo: f32,
    /// Opacity of the 3D storm's flash over the box (0 without).
    pub flash: f32,
}

impl IconPose {
    /// The icon at rest.
    pub const REST: Self = Self {
        rays: 0.0,
        disc: 0.0,
        cloud: 0.0,
        strips: [(0.0, 1.0); 4],
        drops: [(0.0, 0.0, 1.0); 3],
        bolt: 1.0,
        whole: 1.0,
        halo: 0.0,
        flash: 0.0,
    };
}

/// cubic-bezier(0.45, 0, 0.55, 1) at `x` (0..=1): the «soft» curve.
fn soft(x: f32) -> f32 {
    if x <= 0.0 {
        return 0.0;
    }
    if x >= 1.0 {
        return 1.0;
    }
    let bezier = |t: f32, a: f32, b: f32| {
        let u = 1.0 - t;
        3.0 * u * u * t * a + 3.0 * u * t * t * b + t * t * t
    };
    // Solve x(t) = x by bisection (monotone), then read y(t).
    let (mut low, mut high) = (0.0f32, 1.0f32);
    for _ in 0..24 {
        let mid = (low + high) / 2.0;
        if bezier(mid, 0.45, 0.55) < x {
            low = mid;
        } else {
            high = mid;
        }
    }
    bezier((low + high) / 2.0, 0.0, 1.0)
}

/// Where a loop of `period` seconds started `delay` seconds late stands at
/// `t` seconds: 0..1.
fn phase(t: f32, period: f32, delay: f32) -> f32 {
    ((t - delay) / period).rem_euclid(1.0)
}

/// `from → to → from` over a loop at `phase`, on the soft curve.
fn there_and_back(phase: f32, from: f32, to: f32) -> f32 {
    let half = if phase < 0.5 {
        phase * 2.0
    } else {
        2.0 - phase * 2.0
    };
    from + (to - from) * soft(half)
}

/// Linear interpolation through equally spaced `values` at `phase`.
fn through(phase: f32, values: &[f32]) -> f32 {
    let spans = (values.len() - 1) as f32;
    let at = (phase * spans).min(spans - 1e-6).max(0.0);
    let i = at.floor() as usize;
    values[i] + (values[i + 1] - values[i]) * (at - i as f32)
}

/// Linear interpolation through `(time, value)` keys at `phase`.
fn keyed(phase: f32, keys: &[(f32, f32)]) -> f32 {
    for pair in keys.windows(2) {
        let ((a, va), (b, vb)) = (pair[0], pair[1]);
        if phase <= b {
            return if b > a {
                va + (vb - va) * (phase - a) / (b - a)
            } else {
                vb
            };
        }
    }
    keys.last().map_or(0.0, |k| k.1)
}

/// Opacity of a value that breathes between `low` and `high`, its swing
/// scaled by `amplitude` around the middle.
fn breathe(value: f32, low: f32, high: f32, amplitude: f32) -> f32 {
    let middle = (low + high) / 2.0;
    middle + (value - middle) * amplitude
}

/// The pose of family `family`'s icon `t` seconds into its movement at
/// `amplitude` (0..=1) and `speed` (periods divided by it).
pub fn pose(family: usize, t: f32, amplitude: f32, speed: f32) -> IconPose {
    let a = amplitude.clamp(0.0, 1.0);
    if a == 0.0 || !t.is_finite() {
        return IconPose::REST;
    }
    let t = t * speed.clamp(*ICON_SPEED.start(), *ICON_SPEED.end());
    let mut p = IconPose::REST;
    let sun = |p: &mut IconPose| {
        p.rays = phase(t, 24.0, 0.0) * 360.0;
        p.disc = there_and_back(phase(t, 4.0, 0.0), 0.0, 0.12 * a);
        p.halo = breathe(there_and_back(phase(t, 4.0, 0.0), 0.1, 0.3), 0.1, 0.3, a);
    };
    match family {
        0 => sun(&mut p),
        1 => {
            sun(&mut p);
            p.cloud = there_and_back(phase(t, 7.0, 0.0), -3.0, 1.0) * a;
        }
        2 => p.cloud = there_and_back(phase(t, 7.0, 0.0), -2.0, 3.0) * a,
        3 => {
            for (i, strip) in p.strips.iter_mut().enumerate() {
                let at = phase(t, 5.0 + i as f32, 0.6 * i as f32);
                let opacity = there_and_back(at, 0.35, 1.0);
                *strip = (
                    there_and_back(at, -4.0, 4.0) * a,
                    breathe(opacity, 0.35, 1.0, a),
                );
            }
        }
        4 => {
            for (i, drop) in p.drops.iter_mut().enumerate() {
                let at = phase(t, 1.0, 0.33 * i as f32);
                let opacity = through(at, &[0.0, 1.0, 0.0]);
                *drop = (
                    through(at, &[0.0, -2.0]) * a,
                    through(at, &[-4.0, 8.0]) * a,
                    breathe(opacity, 0.0, 1.0, a),
                );
            }
        }
        5 => {
            for (i, flake) in p.drops.iter_mut().enumerate() {
                let at = phase(t, 3.0, i as f32);
                let opacity = through(at, &[0.0, 1.0, 0.0]);
                *flake = (
                    through(at, &[0.0, 2.0, -1.0]) * a,
                    through(at, &[-6.0, 2.0, 10.0]) * a,
                    breathe(opacity, 0.0, 1.0, a),
                );
            }
        }
        6 => {
            let keys = [
                (0.0, 0.0),
                (0.55, 0.0),
                (0.6, 1.0),
                (0.65, 0.2),
                (0.7, 1.0),
                (0.8, 0.0),
                (1.0, 0.0),
            ];
            let bolt = keyed(phase(t, 3.0, 0.0), &keys);
            p.bolt = breathe(bolt, 0.0, 1.0, a);
            p.flash = 0.12 * bolt * a;
        }
        _ => {
            let opacity = there_and_back(phase(t, 4.0, 0.0), 0.45, 0.85);
            p.whole = breathe(opacity, 0.45, 0.85, a);
        }
    }
    p
}

/// How long family `family`'s whole movement takes before it repeats, at
/// speed 1 (seconds).
pub fn period(family: usize) -> f32 {
    match family {
        0 | 1 => 168.0,
        2 => 7.0,
        3 => 840.0,
        4 => 1.0,
        5 | 6 => 3.0,
        _ => 4.0,
    }
}

/// Valid coordinates and a bounded city label.
pub fn valid(city: &str, latitude: f64, longitude: f64) -> bool {
    !city.trim().is_empty()
        && city.chars().count() <= 160
        && latitude.is_finite()
        && longitude.is_finite()
        && (-90.0..=90.0).contains(&latitude)
        && (-180.0..=180.0).contains(&longitude)
}

impl Weather {
    /// Location-specific temperature and condition keys.
    pub fn keys(&self) -> [SensorKey; 2] {
        SensorKey::weather(self.latitude, self.longitude)
    }
    /// City, temperature and conditions; missing data is explicit.
    pub fn text(&self, snapshot: &Snapshot, system: Language) -> String {
        let [temperature, code] = self.keys();
        let language = self.language.unwrap_or(system);
        let (Reading::Value(t), Reading::Value(c)) =
            (snapshot.get(&temperature), snapshot.get(&code))
        else {
            let unavailable = match language {
                Language::Italian => "Meteo non disponibile",
                Language::PortugueseBr => "Clima indisponível",
                Language::English => "Weather unavailable",
            };
            return format!("{}\n{unavailable}", self.city);
        };
        let (value, unit) = if self.fahrenheit {
            (t * 1.8 + 32.0, "F")
        } else {
            (t, "C")
        };
        format!(
            "{}\n{value:.0}\u{00b0}{unit}\n{}",
            self.city,
            condition(c as u16, language)
        )
    }
}

/// Condition family used by vector icons and labels.
pub fn family(code: u16) -> usize {
    match code {
        0 => 0,
        1 | 2 => 1,
        3 => 2,
        45 | 48 => 3,
        51..=67 | 80..=82 => 4,
        71..=77 | 85 | 86 => 5,
        95..=99 => 6,
        _ => 7,
    }
}

/// WMO weather-code description.
pub fn condition(code: u16, language: Language) -> &'static str {
    let labels = match language {
        Language::Italian => [
            "Sereno",
            "Parzialmente nuvoloso",
            "Nuvoloso",
            "Nebbia",
            "Pioggia",
            "Neve",
            "Temporale",
            "Non disponibile",
        ],
        Language::English => [
            "Clear",
            "Partly cloudy",
            "Cloudy",
            "Fog",
            "Rain",
            "Snow",
            "Thunderstorm",
            "Unavailable",
        ],
        Language::PortugueseBr => [
            "Limpo",
            "Parcialmente nublado",
            "Nublado",
            "Nevoeiro",
            "Chuva",
            "Neve",
            "Trovoada",
            "Indisponível",
        ],
    };
    labels[family(code)]
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn coordinates_and_conditions() {
        assert!(!valid("Roma", f64::NAN, 12.0));
        assert!(!valid("", 41.0, 12.0));
        assert!(!valid("Roma", 91.0, 12.0));
        assert!(valid("Roma", 41.9, 12.5));
        assert_eq!(condition(95, Language::Italian), "Temporale");
        assert_eq!(family(999), 7);
    }
    #[test]
    fn unavailable_and_temperature_conversion() {
        let w = Weather {
            city: "Roma".into(),
            latitude: 41.9,
            longitude: 12.5,
            language: Some(Language::Italian),
            fahrenheit: true,
            show_icon: true,
            icon_style: Default::default(),
            icon_gap: None,
            icon_size: None,
            icon_motion: IconMotion::Still,
            icon_speed: 1.0,
        };
        let mut snapshot = Snapshot::default();
        assert!(
            w.text(&snapshot, Language::English)
                .contains("Meteo non disponibile")
        );
        let [t, c] = w.keys();
        snapshot.insert(t, Reading::Value(20.0));
        snapshot.insert(c, Reading::Value(3.0));
        assert_eq!(
            w.text(&snapshot, Language::English),
            "Roma\n68\u{00b0}F\nNuvoloso"
        );
    }

    fn close(a: IconPose, b: IconPose) -> bool {
        let near = |x: f32, y: f32| (x - y).abs() < 1e-3;
        let rays = (a.rays - b.rays).rem_euclid(360.0);
        (near(rays, 0.0) || near(rays, 360.0))
            && near(a.disc, b.disc)
            && near(a.cloud, b.cloud)
            && a.strips
                .iter()
                .zip(b.strips)
                .all(|(x, y)| near(x.0, y.0) && near(x.1, y.1))
            && a.drops
                .iter()
                .zip(b.drops)
                .all(|(x, y)| near(x.0, y.0) && near(x.1, y.1) && near(x.2, y.2))
            && near(a.bolt, b.bolt)
            && near(a.whole, b.whole)
            && near(a.halo, b.halo)
            && near(a.flash, b.flash)
    }

    #[test]
    fn every_movement_repeats_without_a_jump() {
        for family in 0..8 {
            for speed in [0.5f32, 1.0, 2.0] {
                let period = period(family) / speed;
                for t in [0.0f32, 0.37, 1.9, 2.5] {
                    let now = pose(family, t, 1.0, speed);
                    let later = pose(family, t + period, 1.0, speed);
                    assert!(close(now, later), "family {family} speed {speed} at {t}");
                }
            }
            assert_eq!(pose(family, 3.0, 0.0, 1.0), IconPose::REST, "still");
            assert_eq!(pose(family, f32::NAN, 1.0, 1.0), IconPose::REST);
        }
    }

    #[test]
    fn movements_follow_the_specification() {
        let clear = pose(0, 6.0, 1.0, 1.0);
        assert!((clear.rays - 90.0).abs() < 1e-3, "a quarter turn in 6 s");
        assert!(
            (pose(0, 2.0, 1.0, 1.0).disc - 0.12).abs() < 1e-4,
            "widest half way"
        );
        assert!((pose(0, 2.0, 1.0, 1.0).halo - 0.3).abs() < 1e-4);
        assert!(
            (pose(0, 2.0, 0.5, 1.0).halo - 0.25).abs() < 1e-4,
            "gentle halves the swing"
        );
        assert!((pose(1, 3.5, 1.0, 1.0).cloud - 1.0).abs() < 1e-4);
        assert!((pose(1, 0.0, 1.0, 1.0).cloud + 3.0).abs() < 1e-4);
        assert!((pose(1, 3.5, 0.5, 1.0).cloud - 0.5).abs() < 1e-4);
        assert!((pose(2, 3.5, 1.0, 1.0).cloud - 3.0).abs() < 1e-4);
        assert!(
            (pose(2, 1.75, 1.0, 2.0).cloud - 3.0).abs() < 1e-4,
            "twice as fast"
        );
        let fog = pose(3, 2.5, 1.0, 1.0);
        assert!((fog.strips[0].0 - 4.0).abs() < 1e-3 && (fog.strips[0].1 - 1.0).abs() < 1e-3);
        assert!(fog.strips[1].0 != fog.strips[0].0, "strips are out of step");
        let rain = pose(4, 0.5, 1.0, 1.0);
        assert!((rain.drops[0].1 - 2.0).abs() < 1e-3 && (rain.drops[0].2 - 1.0).abs() < 1e-3);
        assert!(
            (pose(4, 0.0, 1.0, 1.0).drops[0].2).abs() < 1e-3,
            "drops fade in"
        );
        let snow = pose(5, 1.5, 1.0, 1.0);
        assert!((snow.drops[0].0 - 2.0).abs() < 1e-3 && (snow.drops[0].1 - 2.0).abs() < 1e-3);
        assert!(
            (pose(6, 1.8, 1.0, 1.0).bolt - 1.0).abs() < 1e-3,
            "first flash at 0.6"
        );
        assert!((pose(6, 1.95, 1.0, 1.0).bolt - 0.2).abs() < 1e-3);
        assert!((pose(6, 1.8, 1.0, 1.0).flash - 0.12).abs() < 1e-3);
        assert!(
            pose(6, 0.3, 1.0, 1.0).bolt.abs() < 1e-3,
            "dark before the flash"
        );
        assert!((pose(7, 2.0, 1.0, 1.0).whole - 0.85).abs() < 1e-3);
        assert!((pose(7, 0.0, 1.0, 1.0).whole - 0.45).abs() < 1e-3);
        assert!(
            (soft(0.5) - 0.5).abs() < 1e-3 && soft(0.0) == 0.0 && (soft(1.0) - 1.0).abs() < 1e-4
        );
        assert_eq!(IconMotion::Still.step(), None);
        assert_eq!(IconMotion::Gentle.step(), Some(Duration::from_millis(50)));
        assert_eq!(IconMotion::Lively.amplitude(), 1.0);
        assert_eq!(keyed(2.0, &[(0.0, 1.0), (1.0, 3.0)]), 3.0);
    }
}
