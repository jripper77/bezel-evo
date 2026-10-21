//! Temporary inspector inputs. Trials never enter a document or a live scene.
use std::{collections::BTreeSet, time::Duration};

use bezel_core::domain::{
    clock::LocalTime,
    history::{Histories, Tick},
    playback,
    sensor::{Reading, SensorKey, Snapshot},
    theme::{Element, ElementId, ElementKind, TextContent, Theme},
};

/// One second for each condition family; other trials last three seconds.
pub const WEATHER_MS: u32 = 8_000;
pub const DEFAULT_MS: u32 = 3_000;
pub const FRAME_MS: u64 = 33;
/// A canvas trial repeats every 24 s: whole sweeps (3 s) and weather visits (8 s).
pub const SCENE_MS: u32 = 24_000;
/// The weather families a trial visits, one a second.
const WEATHER_CODES: [f64; 8] = [0., 2., 3., 45., 61., 71., 95., 999.];

/// One element whose sensor a canvas trial holds at a share of its scale.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Hold {
    pub element: ElementId,
    pub fraction: f64,
}

/// A detached scene with its own sample and animation clock.
pub struct Trial {
    pub theme: Theme,
    pub snapshot: Snapshot,
    pub histories: Histories,
    pub time: LocalTime,
    pub animation: Duration,
}

/// A smooth trip from the current reading to full scale and back.
pub fn pulse(ms: u32) -> f64 {
    (1. - (std::f64::consts::TAU * f64::from(ms) / f64::from(DEFAULT_MS)).cos()) / 2.
}

impl Trial {
    /// Returns no trial after completion, with reduced motion, or for an
    /// absent element. Callers then use the unchanged live preview path.
    pub fn new(
        theme: &Theme,
        id: ElementId,
        snapshot: &Snapshot,
        mut time: LocalTime,
        ms: Option<u32>,
        motion: bool,
    ) -> Option<Self> {
        let ms = ms.filter(|_| motion)?;
        let element = theme.element(id)?;
        let weather = matches!(
            element.kind,
            ElementKind::Text {
                content: TextContent::Weather(_),
                ..
            }
        );
        if ms >= if weather { WEATHER_MS } else { DEFAULT_MS } {
            return None;
        }
        let mut theme = theme.clone();
        let element = theme.elements.iter_mut().find(|e| e.id == id)?;
        if let Some(card) = &mut element.card {
            card.active_face = (card.active_face + 1) % card.faces.len().max(1);
        }
        let mut snapshot = snapshot.clone();
        let p = pulse(ms);
        match &mut element.kind {
            ElementKind::Bar { binding, .. }
            | ElementKind::Ring { binding, .. }
            | ElementKind::Needle { binding, .. } => {
                let current = snapshot.get(&binding.key).value().unwrap_or(binding.min);
                let current = current.clamp(binding.min, binding.max);
                snapshot.insert(
                    binding.key.clone(),
                    Reading::Value(current + (binding.max - current) * p),
                );
            }
            ElementKind::Graph {
                history,
                interval_seconds,
                ..
            } => {
                *history = 60;
                *interval_seconds = None;
            }
            ElementKind::Text { content, .. } => match content {
                TextContent::Sensor { key, .. } => {
                    let current = snapshot.get(key).value().unwrap_or(0.);
                    snapshot.insert(
                        key.clone(),
                        Reading::Value(current + (current.max(100.) - current) * p),
                    );
                }
                TextContent::Clock { .. } => {
                    let seconds = (u32::from(time.hour) * 3600
                        + u32::from(time.minute) * 60
                        + u32::from(time.second)
                        + ms * 86_400 / DEFAULT_MS)
                        % 86_400;
                    time.hour = (seconds / 3600) as u8;
                    time.minute = (seconds / 60 % 60) as u8;
                    time.second = (seconds % 60) as u8;
                }
                TextContent::Weather(weather) => {
                    let [temperature, code] = weather.keys();
                    let value = snapshot.get(&temperature).value().unwrap_or(20.);
                    snapshot.insert(temperature, Reading::Value(value));
                    snapshot.insert(code, Reading::Value(WEATHER_CODES[(ms / 1000) as usize]));
                }
                TextContent::Player(player) => {
                    let mut media = playback::session(&snapshot, &player.source)
                        .cloned()
                        .unwrap_or_default();
                    if media.source.is_empty() {
                        media.source = player.source.clone();
                    }
                    media.playing = true;
                    media.position = f64::from(ms) / 1000.;
                    if media.duration <= 0. {
                        media.duration = 180.;
                    }
                    snapshot.media = vec![media];
                    snapshot.media_available = true;
                }
                TextContent::Spectrum(spectrum) => spectrum.demo = true,
                TextContent::Static(_) => {}
            },
            _ => {}
        }
        let theme = crate::element_preview::isolate(&theme, id)?;
        let histories = waveform(&theme, &mut snapshot, ms);
        Some(Self {
            theme,
            snapshot,
            histories,
            time,
            animation: Duration::from_millis(u64::from(ms)),
        })
    }
}

impl Trial {
    /// The whole scene with trial inputs, for the canvas's trial values: every
    /// linked sensor sweeps its scale from minimum to maximum and back every
    /// three seconds, clocks revolve, the weather visits its families, media
    /// plays and graphs draw a waveform. With `hold` only that element's
    /// sensor changes, held at `fraction` of its scale, over the live
    /// `histories`. Neither the theme nor the readings given change.
    pub fn scene(
        theme: &Theme,
        snapshot: &Snapshot,
        histories: &Histories,
        mut time: LocalTime,
        ms: u32,
        hold: Option<Hold>,
    ) -> Self {
        let ms = ms % SCENE_MS;
        let mut theme = theme.clone();
        let mut snapshot = snapshot.clone();
        let animation = Duration::from_millis(u64::from(ms));
        if let Some(hold) = hold {
            if let Some((key, min, max, _)) = theme.element(hold.element).and_then(scale_of) {
                let fraction = if hold.fraction.is_finite() {
                    hold.fraction.clamp(0., 1.)
                } else {
                    0.
                };
                snapshot.insert(key, Reading::Value(min + (max - min) * fraction));
            }
            return Self {
                theme,
                snapshot,
                histories: histories.clone(),
                time,
                animation,
            };
        }
        let histories = waveform(&theme, &mut snapshot, ms);
        // A gauge's scale wins over a sensor text sharing its key.
        let p = pulse(ms % DEFAULT_MS);
        let mut scales: Vec<_> = theme.elements.iter().filter_map(scale_of).collect();
        scales.sort_by_key(|(_, _, _, text)| *text);
        let mut swept = BTreeSet::new();
        for (key, min, max, _) in scales {
            if swept.insert(key.clone()) {
                snapshot.insert(key, Reading::Value(min + (max - min) * p));
            }
        }
        let seconds = (u64::from(time.hour) * 3600
            + u64::from(time.minute) * 60
            + u64::from(time.second)
            + u64::from(ms) * 86_400 / u64::from(DEFAULT_MS))
            % 86_400;
        time.hour = (seconds / 3600) as u8;
        time.minute = (seconds / 60 % 60) as u8;
        time.second = (seconds % 60) as u8;
        let mut media: Vec<playback::MediaSession> = Vec::new();
        for element in &mut theme.elements {
            let ElementKind::Text { content, .. } = &mut element.kind else {
                continue;
            };
            match content {
                TextContent::Weather(weather) => {
                    let [temperature, code] = weather.keys();
                    let value = snapshot.get(&temperature).value().unwrap_or(20.);
                    snapshot.insert(temperature, Reading::Value(value));
                    snapshot.insert(
                        code,
                        Reading::Value(WEATHER_CODES[(ms / 1000 % 8) as usize]),
                    );
                }
                TextContent::Player(player) => {
                    if media.iter().any(|m| m.source == player.source) {
                        continue;
                    }
                    let mut session = playback::session(&snapshot, &player.source)
                        .cloned()
                        .unwrap_or_default();
                    if session.source.is_empty() {
                        session.source = player.source.clone();
                    }
                    if session.duration <= 0. {
                        session.duration = 180.;
                    }
                    session.playing = true;
                    session.position = (f64::from(ms) / 1000.) % session.duration;
                    media.push(session);
                }
                TextContent::Spectrum(spectrum) => spectrum.demo = true,
                _ => {}
            }
        }
        if !media.is_empty() {
            snapshot.media = media;
            snapshot.media_available = true;
        }
        Self {
            theme,
            snapshot,
            histories,
            time,
            animation,
        }
    }
}

/// The sensor an element shows and its scale (a sensor text: 0 to 100), and
/// whether it is a text, which yields to a gauge's scale.
fn scale_of(element: &Element) -> Option<(SensorKey, f64, f64, bool)> {
    match &element.kind {
        ElementKind::Bar { binding, .. }
        | ElementKind::Ring { binding, .. }
        | ElementKind::Needle { binding, .. } => {
            Some((binding.key.clone(), binding.min, binding.max, false))
        }
        ElementKind::Text {
            content: TextContent::Sensor { key, .. },
            ..
        } => Some((key.clone(), 0., 100., true)),
        _ => None,
    }
}

/// Sixty points of a moving wave for every graph of `theme`, within its scale.
fn waveform(theme: &Theme, snapshot: &mut Snapshot, ms: u32) -> Histories {
    let series = theme.graph_series();
    let mut histories = Histories::new(&series);
    for i in 0..60 {
        for series in &series {
            let binding = theme.elements.iter().find_map(|e| match &e.kind {
                ElementKind::Graph { binding, .. } if binding.key == series.key => Some(binding),
                _ => None,
            });
            let (min, max) = binding.map_or((0., 100.), |b| (b.min, b.max));
            let fraction = 0.5 + 0.4 * (f64::from(i) * 0.25 + f64::from(ms) / 1000.).sin();
            snapshot.insert(
                series.key.clone(),
                Reading::Value(min + (max - min) * fraction),
            );
        }
        histories.push(snapshot, Tick::Untimed);
    }
    histories
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use bezel_core::domain::{
        clock::{ClockCase, Language},
        frame::Rgba,
        geometry::{Orientation, Size},
        sensor::SensorKey,
        theme::{Binding, BoxF, Card, Direction, GraphStyle, Paint, TextStyle},
        weather::{IconStyle, Weather, family},
    };

    fn time() -> LocalTime {
        LocalTime {
            year: 2026,
            month: 10,
            day: 20,
            hour: 9,
            minute: 45,
            second: 30,
            weekday: 1,
            millisecond: 0,
        }
    }
    fn theme(kind: ElementKind) -> Theme {
        let mut theme = Theme::blank("trial", Size::new(200, 100), Orientation::Landscape);
        let mut e =
            crate::element_preview::tests::shape(1, BoxF::new(0., 0., 200., 100.), Rgba::WHITE);
        e.kind = kind;
        theme.elements.push(e);
        theme
    }
    fn text(content: TextContent) -> Theme {
        theme(ElementKind::Text {
            content,
            style: TextStyle::default(),
        })
    }
    fn binding() -> Binding {
        Binding {
            key: SensorKey::new("cpu.usage").unwrap(),
            min: 10.,
            max: 110.,
        }
    }
    fn trial(theme: &Theme, snapshot: &Snapshot, ms: u32) -> Trial {
        Trial::new(theme, ElementId(1), snapshot, time(), Some(ms), true).unwrap()
    }

    #[test]
    fn a_value_goes_from_its_reading_to_full_and_back_without_touching_the_source() {
        let b = binding();
        let theme = theme(ElementKind::Bar {
            binding: b.clone(),
            direction: Direction::LeftToRight,
            fill: Paint::solid(Rgba::WHITE),
            track: None,
            radius: 0.,
            segments: None,
        });
        let mut snapshot = Snapshot::default();
        snapshot.insert(b.key.clone(), Reading::Value(35.));
        let original_theme = theme.clone();
        let original_snapshot = snapshot.clone();
        assert_eq!(
            trial(&theme, &snapshot, 0).snapshot.get(&b.key).value(),
            Some(35.)
        );
        assert_eq!(
            trial(&theme, &snapshot, 1500).snapshot.get(&b.key).value(),
            Some(110.)
        );
        assert!(
            trial(&theme, &snapshot, 2999)
                .snapshot
                .get(&b.key)
                .value()
                .unwrap()
                < 35.001
        );
        assert_eq!(pulse(0), 0.);
        assert_eq!(pulse(1500), 1.);
        assert_eq!(pulse(3000), 0.);
        assert_eq!(snapshot, original_snapshot);
        assert_eq!(theme, original_theme);
        assert!(Trial::new(&theme, ElementId(1), &snapshot, time(), None, true).is_none());
        assert!(Trial::new(&theme, ElementId(1), &snapshot, time(), Some(1500), false).is_none());
        assert!(Trial::new(&theme, ElementId(1), &snapshot, time(), Some(3000), true).is_none());
        assert!(Trial::new(&theme, ElementId(99), &snapshot, time(), Some(0), true).is_none());
    }

    #[test]
    fn weather_visits_all_eight_families_without_persisting_any_reading() {
        let w = Weather {
            icon_color: None,
            city: "Local".into(),
            latitude: 45.,
            longitude: 9.,
            language: Some(Language::Italian),
            fahrenheit: false,
            show_icon: true,
            icon_style: IconStyle::Outline,
            icon_gap: None,
            icon_size: None,
            icon_motion: Default::default(),
            icon_speed: 1.0,
        };
        let [temperature, code] = w.keys();
        let theme = text(TextContent::Weather(w));
        let mut snapshot = Snapshot::default();
        snapshot.insert(temperature.clone(), Reading::Value(26.));
        for i in 0..8 {
            let scene = trial(&theme, &snapshot, i * 1000);
            assert_eq!(
                family(scene.snapshot.get(&code).value().unwrap() as u16),
                i as usize
            );
            assert_eq!(scene.snapshot.get(&temperature).value(), Some(26.));
        }
        assert!(snapshot.get(&code).value().is_none());
        assert!(Trial::new(&theme, ElementId(1), &snapshot, time(), Some(8000), true).is_none());
    }

    #[test]
    fn clock_revolves_and_media_restarts_only_in_the_detached_sample() {
        let clock = text(TextContent::Clock {
            pattern: "%H:%M:%S".into(),
            language: None,
            casing: ClockCase::Normal,
            analog: None,
        });
        let snapshot = Snapshot::default();
        let scene = trial(&clock, &snapshot, 1500);
        assert_eq!(
            (scene.time.hour, scene.time.minute, scene.time.second),
            (21, 45, 30)
        );
        assert_eq!(
            (scene.time.year, scene.time.month, scene.time.day),
            (2026, 10, 20)
        );
        let player = text(TextContent::Player(Default::default()));
        let mut snapshot = Snapshot::default();
        snapshot.media.push(playback::MediaSession {
            title: "Current title".into(),
            position: 90.,
            duration: 210.,
            ..Default::default()
        });
        let scene = trial(&player, &snapshot, 500);
        assert_eq!(scene.snapshot.media[0].position, 0.5);
        assert_eq!(scene.snapshot.media[0].title, "Current title");
        assert!(scene.snapshot.media[0].playing);
        assert_eq!(snapshot.media[0].position, 90.);
        let scene = trial(&player, &Snapshot::default(), 0);
        assert_eq!(scene.snapshot.media[0].duration, 180.);
    }

    #[test]
    fn card_tries_the_next_face_and_graph_has_a_detached_waveform() {
        let mut card = text(TextContent::Static("".into()));
        card.elements[0].card = Some(Card {
            faces: vec!["A".into(), "B".into()],
            active_face: 0,
            triggers: vec![],
            rotation_seconds: None,
            transition: None,
        });
        assert_eq!(
            trial(&card, &Snapshot::default(), 0).theme.elements[0]
                .card
                .as_ref()
                .unwrap()
                .active_face,
            1
        );
        assert_eq!(card.elements[0].card.as_ref().unwrap().active_face, 0);
        let graph = theme(ElementKind::Graph {
            binding: binding(),
            style: GraphStyle::Line,
            color: Rgba::WHITE,
            fill: None,
            line_width: 1.,
            history: 200,
            interval_seconds: Some(3),
            autoscale: false,
        });
        let scene = trial(&graph, &Snapshot::default(), 1500);
        let values = scene.histories.get(&binding().key);
        assert_eq!(values.len(), 60);
        assert!(
            values
                .iter()
                .flatten()
                .all(|value| (20. ..=100.).contains(value))
        );
        assert!((values[0].unwrap() - (60. + 40. * 1.5_f64.sin())).abs() < 0.0001);
        let spectrum = text(TextContent::Spectrum(Default::default()));
        assert!(
            matches!(&trial(&spectrum, &Snapshot::default(), 100).theme.elements[0].kind, ElementKind::Text { content: TextContent::Spectrum(s), .. } if s.demo)
        );
    }

    fn gauge(id: u32, key: &str, min: f64, max: f64) -> bezel_core::domain::theme::Element {
        let mut e =
            crate::element_preview::tests::shape(id, BoxF::new(0., 0., 50., 20.), Rgba::WHITE);
        e.kind = ElementKind::Bar {
            binding: Binding {
                key: SensorKey::new(key).unwrap(),
                min,
                max,
            },
            direction: Direction::LeftToRight,
            fill: Paint::solid(Rgba::WHITE),
            track: None,
            radius: 0.,
            segments: None,
        };
        e
    }

    #[test]
    fn a_canvas_trial_sweeps_every_sensor_over_its_scale_and_holds_one_on_request() {
        let mut scene = theme(ElementKind::Text {
            content: TextContent::Sensor {
                key: SensorKey::new("cpu.usage").unwrap(),
                format: Default::default(),
                prefix: String::new(),
                suffix: String::new(),
            },
            style: TextStyle::default(),
        });
        scene.elements.push(gauge(2, "cpu.usage", 10., 110.));
        scene.elements.push(gauge(3, "gpu.temperature", 30., 90.));
        let text_only = SensorKey::new("ram.usage").unwrap();
        let mut text = scene.elements[0].clone();
        text.id = ElementId(4);
        if let ElementKind::Text {
            content: TextContent::Sensor { key, .. },
            ..
        } = &mut text.kind
        {
            *key = text_only.clone();
        }
        scene.elements.push(text);
        let mut snapshot = Snapshot::default();
        snapshot.insert(SensorKey::new("cpu.usage").unwrap(), Reading::Value(35.));
        let original = (scene.clone(), snapshot.clone());
        let histories = Histories::default();
        let at = |ms, hold| Trial::scene(&scene, &snapshot, &histories, time(), ms, hold);
        let value =
            |trial: &Trial, key: &str| trial.snapshot.get(&SensorKey::new(key).unwrap()).value();
        // Minimum at the start, maximum half way: the gauge's scale wins over the text's.
        assert_eq!(value(&at(0, None), "cpu.usage"), Some(10.));
        assert_eq!(value(&at(1500, None), "cpu.usage"), Some(110.));
        assert_eq!(value(&at(1500, None), "gpu.temperature"), Some(90.));
        assert_eq!(value(&at(1500, None), "ram.usage"), Some(100.));
        // The trial repeats: 24 s later it is where it began.
        assert_eq!(value(&at(SCENE_MS + 1500, None), "cpu.usage"), Some(110.));
        let clock = at(1500, None).time;
        assert_eq!((clock.hour, clock.minute, clock.second), (21, 45, 30));
        // Held: only that element's sensor, at its share; the rest as read.
        let held = at(
            0,
            Some(Hold {
                element: ElementId(3),
                fraction: 0.25,
            }),
        );
        assert_eq!(value(&held, "gpu.temperature"), Some(45.));
        assert_eq!(value(&held, "cpu.usage"), Some(35.));
        assert_eq!((held.time.hour, held.time.minute), (9, 45));
        let clamped = at(
            0,
            Some(Hold {
                element: ElementId(3),
                fraction: f64::NAN,
            }),
        );
        assert_eq!(value(&clamped, "gpu.temperature"), Some(30.));
        let stale = at(
            0,
            Some(Hold {
                element: ElementId(99),
                fraction: 1.,
            }),
        );
        assert_eq!(stale.snapshot, snapshot);
        assert_eq!((scene, snapshot), original);
    }

    #[test]
    fn a_canvas_trial_plays_every_player_source_once_and_cycles_the_weather() {
        let mut scene = text(TextContent::Player(Default::default()));
        let mut other = scene.elements[0].clone();
        other.id = ElementId(2);
        scene.elements.push(other);
        let trial = Trial::scene(
            &scene,
            &Snapshot::default(),
            &Histories::default(),
            time(),
            9_500,
            None,
        );
        assert_eq!(trial.snapshot.media.len(), 1);
        assert!(trial.snapshot.media[0].playing);
        assert_eq!(trial.snapshot.media[0].position, 9.5);
        assert!(trial.snapshot.media_available);
        let w = Weather {
            icon_color: None,
            city: "Local".into(),
            latitude: 45.,
            longitude: 9.,
            language: None,
            fahrenheit: false,
            show_icon: true,
            icon_style: IconStyle::Outline,
            icon_gap: None,
            icon_size: None,
            icon_motion: Default::default(),
            icon_speed: 1.0,
        };
        let [temperature, code] = w.keys();
        let weather = text(TextContent::Weather(w));
        let codes: Vec<_> = (0..10)
            .map(|s| {
                let trial = Trial::scene(
                    &weather,
                    &Snapshot::default(),
                    &Histories::default(),
                    time(),
                    s * 1000,
                    None,
                );
                assert_eq!(trial.snapshot.get(&temperature).value(), Some(20.));
                family(trial.snapshot.get(&code).value().unwrap() as u16)
            })
            .collect();
        assert_eq!(codes, vec![0, 1, 2, 3, 4, 5, 6, 7, 0, 1]);
        let spectrum = text(TextContent::Spectrum(Default::default()));
        let trial = Trial::scene(
            &spectrum,
            &Snapshot::default(),
            &Histories::default(),
            time(),
            0,
            None,
        );
        assert!(
            matches!(&trial.theme.elements[0].kind, ElementKind::Text { content: TextContent::Spectrum(s), .. } if s.demo)
        );
    }
}
