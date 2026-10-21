//! Temporary inspector inputs. Trials never enter a document or a live scene.
use std::time::Duration;

use bezel_core::domain::{
    clock::LocalTime,
    history::{Histories, Tick},
    playback,
    sensor::{Reading, Snapshot},
    theme::{ElementId, ElementKind, TextContent, Theme},
};

/// One second for each condition family; other trials last three seconds.
pub const WEATHER_MS: u32 = 8_000;
pub const DEFAULT_MS: u32 = 3_000;
pub const FRAME_MS: u64 = 33;

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
                    snapshot.insert(
                        code,
                        Reading::Value(
                            [0., 2., 3., 45., 61., 71., 95., 999.][(ms / 1000) as usize],
                        ),
                    );
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
        let series = theme.graph_series();
        let mut histories = Histories::new(&series);
        for i in 0..60 {
            for series in &series {
                let binding = theme.elements.iter().find_map(|e| match &e.kind {
                    ElementKind::Graph { binding, .. } if binding.key == series.key => {
                        Some(binding)
                    }
                    _ => None,
                });
                let (min, max) = binding.map_or((0., 100.), |b| (b.min, b.max));
                let fraction = 0.5 + 0.4 * (f64::from(i) * 0.25 + f64::from(ms) / 1000.).sin();
                snapshot.insert(
                    series.key.clone(),
                    Reading::Value(min + (max - min) * fraction),
                );
            }
            histories.push(&snapshot, Tick::Untimed);
        }
        Some(Self {
            theme,
            snapshot,
            histories,
            time,
            animation: Duration::from_millis(u64::from(ms)),
        })
    }
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
}
