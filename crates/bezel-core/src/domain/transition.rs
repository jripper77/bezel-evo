//! Smooth value transitions (D-2026-10-15-studio-redesign-fidelity-18): a
//! ring, a bar, a needle or a sensor value eases from the value it shows to
//! a new reading instead of jumping to it.
//!
//! Each element opts in ([`ValueTransition`] on [`Element`]); without it an
//! element shows its reading as it always did. A new sample retargets the
//! element's [`Tween`] from the value it shows at that moment; an edit of
//! the theme settles it at once (an edit is never animated). The renderer
//! draws [`ShownValues`], the values of the tweens still moving, in place of
//! the readings; everything else (range, formatting, units) stays the
//! renderer's.
//!
//! Frames: while a tween moves, a frame is due every [`MIN_TRANSITION_STEP`]
//! at most ([`MAX_TRANSITION_FPS`]); when it settles, nothing more. A screen
//! slower than that gets fewer frames (each shows the value of its moment),
//! and a screen too slow to draw two frames within a transition ([`Pace`])
//! shows the new value at once.

use std::collections::BTreeMap;
use std::time::Duration;

use super::sensor::Snapshot;
use super::theme::{Element, ElementId, ElementKind, TextContent, Theme};

/// Most frames a second drawn for value transitions.
pub const MAX_TRANSITION_FPS: u32 = 20;

/// Shortest time between two frames drawn for value transitions
/// (1 / [`MAX_TRANSITION_FPS`] s).
pub const MIN_TRANSITION_STEP: Duration =
    Duration::from_nanos(1_000_000_000 / MAX_TRANSITION_FPS as u64);

/// Shortest transition, milliseconds: shorter ones last this long.
pub const MIN_TRANSITION_MS: u32 = 50;

/// Longest transition, milliseconds: longer ones last this long.
pub const MAX_TRANSITION_MS: u32 = 5_000;

/// How an element moves to a new reading.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TransitionMode {
    /// It jumps to the new value (themes without a transition).
    #[default]
    None,
    /// It eases from the value it shows to the new one.
    Smooth,
    /// Critically damped pursuit, retaining momentum when readings change.
    Inertia,
}

/// An element's value transition.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ValueTransition {
    /// None or smooth.
    pub mode: TransitionMode,
    /// How long a smooth transition lasts, milliseconds (kept while the mode
    /// is none, so switching back restores it).
    pub duration_ms: u32,
}

impl ValueTransition {
    /// A smooth transition lasting `duration_ms`.
    pub fn smooth(duration_ms: u32) -> Self {
        Self {
            mode: TransitionMode::Smooth,
            duration_ms,
        }
    }

    /// How long a change lasts: `None` when the value jumps (mode none, or
    /// no time at all), else the duration within
    /// [`MIN_TRANSITION_MS`]..=[`MAX_TRANSITION_MS`].
    pub fn length(&self) -> Option<Duration> {
        match self.mode {
            TransitionMode::Smooth | TransitionMode::Inertia if self.duration_ms > 0 => {
                Some(Duration::from_millis(u64::from(
                    self.duration_ms.clamp(MIN_TRANSITION_MS, MAX_TRANSITION_MS),
                )))
            }
            _ => None,
        }
    }
}

/// The eased share of a transition done at `progress` (0..=1 of its
/// length): ease-out cubic, quick at first so a change shows at once, slow
/// as it lands. Outside 0..=1 it holds the end.
pub fn ease(progress: f64) -> f64 {
    if progress.is_nan() {
        return 1.0;
    }
    let t = progress.clamp(0.0, 1.0);
    1.0 - (1.0 - t).powi(3)
}

/// The value an element with a value shows for `snapshot`: the reading of
/// a bar, ring or needle held within its range, the reading of a sensor
/// text as it is. `None` for other elements and readings that are not a
/// number (an unavailable sensor is never eased).
pub fn target(element: &Element, snapshot: &Snapshot) -> Option<f64> {
    match &element.kind {
        ElementKind::Bar { binding, .. }
        | ElementKind::Ring { binding, .. }
        | ElementKind::Needle { binding, .. } => {
            let value = snapshot.get(&binding.key).value()?;
            let (low, high) = (binding.min.min(binding.max), binding.min.max(binding.max));
            if low.is_finite() && high.is_finite() {
                Some(value.clamp(low, high))
            } else {
                Some(value)
            }
        }
        ElementKind::Text {
            content: TextContent::Sensor { key, .. },
            ..
        } => snapshot.get(key).value(),
        _ => None,
    }
}

/// One element moving from a value to another.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Tween {
    mode: TransitionMode,
    velocity: f64,
    from: f64,
    to: f64,
    start: Duration,
    length: Duration,
}

impl Tween {
    /// Resting at `value`.
    pub fn settled(value: f64) -> Self {
        Self {
            mode: TransitionMode::Smooth,
            velocity: 0.,
            from: value,
            to: value,
            start: Duration::ZERO,
            length: Duration::ZERO,
        }
    }

    /// From `from` at `start` to `to`, taking `length`.
    pub fn new(from: f64, to: f64, start: Duration, length: Duration) -> Self {
        Self {
            mode: TransitionMode::Smooth,
            velocity: 0.,
            from,
            to,
            start,
            length,
        }
    }

    /// Where it goes.
    pub fn to(&self) -> f64 {
        self.to
    }

    /// How long it takes.
    pub fn length(&self) -> Duration {
        self.length
    }

    /// When it lands.
    pub fn end(&self) -> Duration {
        self.start + self.length
    }

    /// Whether it still moves at `at`.
    pub fn moving_at(&self, at: Duration) -> bool {
        (self.from != self.to || self.velocity != 0.) && at < self.end()
    }

    /// The value shown at `at`.
    pub fn value_at(&self, at: Duration) -> f64 {
        if !self.moving_at(at) {
            return self.to;
        }
        if at <= self.start {
            return self.from;
        }
        let progress = (at - self.start).as_secs_f64() / self.length.as_secs_f64();
        if self.mode == TransitionMode::Inertia {
            let time = (at - self.start).as_secs_f64();
            let omega = 8. / self.length.as_secs_f64();
            let displacement = self.from - self.to;
            return self.to
                + (displacement + (self.velocity + omega * displacement) * time)
                    * (-omega * time).exp();
        }
        self.from + (self.to - self.from) * ease(progress)
    }

    fn velocity_at(&self, at: Duration) -> f64 {
        if !self.moving_at(at) || self.length.is_zero() {
            return 0.;
        }
        let time = at.saturating_sub(self.start).as_secs_f64();
        let length = self.length.as_secs_f64();
        if self.mode == TransitionMode::Inertia {
            let omega = 8. / length;
            return (self.velocity
                - omega * (self.velocity + omega * (self.from - self.to)) * time)
                * (-omega * time).exp();
        }
        3. * (self.to - self.from) / length * (1. - time / length).powi(2)
    }

    fn retarget(&self, to: f64, now: Duration, length: Duration, mode: TransitionMode) -> Self {
        let mut next = Self::new(self.value_at(now), to, now, length);
        next.mode = mode;
        if mode == TransitionMode::Inertia {
            next.velocity = self.velocity_at(now);
        }
        next
    }
}

/// The values to draw in place of the readings: only elements whose
/// transition still moves.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ShownValues(BTreeMap<ElementId, f64>);

/// Nothing moving: every element shows its reading (a one-off frame).
pub static NOTHING_SHOWN: ShownValues = ShownValues(BTreeMap::new());

impl FromIterator<(ElementId, f64)> for ShownValues {
    fn from_iter<I: IntoIterator<Item = (ElementId, f64)>>(values: I) -> Self {
        Self(values.into_iter().collect())
    }
}

impl ShownValues {
    /// The value element `id` shows, when it is moving.
    pub fn get(&self, id: ElementId) -> Option<f64> {
        self.0.get(&id).copied()
    }

    /// Whether nothing moves.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Number of moving elements.
    pub fn len(&self) -> usize {
        self.0.len()
    }
}

/// How fast frames reach the screen, from the frames it showed.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Pace {
    frame: Option<Duration>,
}

impl Pace {
    /// One frame took `cost` from drawing to shown: averaged with the
    /// frames before it (half each), so one slow frame does not decide.
    pub fn record(&mut self, cost: Duration) {
        self.frame = Some(match self.frame {
            None => cost,
            Some(before) => (before + cost) / 2,
        });
    }

    /// Forgets what was measured (a new screen).
    pub fn forget(&mut self) {
        self.frame = None;
    }

    /// The time a frame takes, when measured.
    pub fn frame(&self) -> Option<Duration> {
        self.frame
    }

    /// Time between two transition frames: [`MIN_TRANSITION_STEP`], or what
    /// a frame takes when the screen is slower.
    pub fn step(&self) -> Duration {
        self.frame
            .map_or(MIN_TRANSITION_STEP, |frame| frame.max(MIN_TRANSITION_STEP))
    }

    /// Whether the screen draws at least two frames within a transition of
    /// `length`; otherwise the transition snaps to its value.
    pub fn keeps_up(&self, length: Duration) -> bool {
        self.frame.is_none_or(|frame| frame * 2 < length)
    }
}

/// Who a frame is for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    /// An editor's preview: eases unless motion is reduced.
    Preview,
    /// The screen: eases at the pace it keeps.
    Screen,
}

/// The transitions of a theme's elements.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Transitions {
    tweens: BTreeMap<ElementId, Tween>,
    pace: Pace,
    still_previews: bool,
}

impl Transitions {
    /// The elements of `theme` that ease, with their transition's length.
    fn eased(theme: &Theme) -> impl Iterator<Item = (&Element, Duration)> {
        theme.elements.iter().filter_map(|element| {
            let length = element.value_transition?.length()?;
            Some((element, length))
        })
    }

    /// A sample: every element that eases goes to its value in `snapshot`,
    /// from the value it shows at `now` (`None`: a sample with no clock, the
    /// values settle at once). A first value, or an element whose reading
    /// is not a number, settles.
    pub fn retarget(&mut self, theme: &Theme, snapshot: &Snapshot, now: Option<Duration>) {
        let mut next = BTreeMap::new();
        for (element, length) in Self::eased(theme) {
            let Some(to) = target(element, snapshot) else {
                continue;
            };
            let mode = element
                .value_transition
                .map_or(TransitionMode::None, |t| t.mode);
            let tween = match (self.tweens.get(&element.id), now) {
                (Some(tween), _) if tween.to == to && tween.mode == mode => *tween,
                (Some(tween), Some(now)) => tween.retarget(to, now, length, mode),
                _ => {
                    let mut tween = Tween::settled(to);
                    tween.mode = mode;
                    tween
                }
            };
            next.insert(element.id, tween);
        }
        self.tweens = next;
    }

    /// An edit: `theme` replaces the theme shown. What it no longer eases is
    /// dropped, and what now shows another value (another sensor, another
    /// range) settles there at once: an edit shows as it is.
    pub fn edited(&mut self, theme: &Theme, snapshot: &Snapshot) {
        let mut next = BTreeMap::new();
        for (element, _) in Self::eased(theme) {
            let Some(to) = target(element, snapshot) else {
                continue;
            };
            let tween = match self.tweens.get(&element.id) {
                Some(tween) if tween.to == to => *tween,
                _ => Tween::settled(to),
            };
            next.insert(element.id, tween);
        }
        self.tweens = next;
    }

    /// Whether previews snap to the readings (reduced motion) instead of
    /// easing.
    pub fn still_previews(&mut self, still: bool) {
        self.still_previews = still;
    }

    /// The screen showed a frame that took `cost` from drawing to shown.
    pub fn screen_pace(&mut self, cost: Duration) {
        self.pace.record(cost);
    }

    /// Forgets the screen's pace (another screen, or none).
    pub fn forget_screen(&mut self) {
        self.pace.forget();
    }

    /// How fast the screen showed its frames.
    pub fn pace(&self) -> Pace {
        self.pace
    }

    /// The tween of element `id`, if it eases.
    pub fn tween(&self, id: ElementId) -> Option<&Tween> {
        self.tweens.get(&id)
    }

    /// The tweens `view` animates among the visible elements of `theme` at
    /// `at`.
    fn moving<'a>(
        &'a self,
        theme: &'a Theme,
        at: Duration,
        view: View,
    ) -> impl Iterator<Item = (ElementId, &'a Tween)> + 'a {
        let animates = match view {
            View::Preview => !self.still_previews,
            View::Screen => true,
        };
        theme
            .elements
            .iter()
            .filter(move |element| animates && theme.is_visible(element))
            .filter_map(move |element| {
                let tween = self.tweens.get(&element.id)?;
                let paced = view == View::Preview || self.pace.keeps_up(tween.length);
                (paced && tween.moving_at(at)).then_some((element.id, tween))
            })
    }

    /// Whether a preview of `theme` eases a visible element (motion not
    /// reduced): it then wants a frame at the next sample, when a
    /// transition starts.
    pub fn eases_in_previews(&self, theme: &Theme) -> bool {
        !self.still_previews && Self::eased(theme).any(|(element, _)| theme.is_visible(element))
    }

    /// What `view` draws at `at` for the visible elements of `theme`.
    pub fn shown(&self, theme: &Theme, at: Duration, view: View) -> ShownValues {
        self.moving(theme, at, view)
            .map(|(id, tween)| (id, tween.value_at(at)))
            .collect()
    }

    /// When `view` needs its next frame after one drawn at `at`: a step
    /// later while a visible tween moves (or when it lands, if sooner);
    /// `None` when nothing moves.
    pub fn next_change(&self, theme: &Theme, at: Duration, view: View) -> Option<Duration> {
        let step = match view {
            View::Preview => MIN_TRANSITION_STEP,
            View::Screen => self.pace.step(),
        };
        self.moving(theme, at, view)
            .map(|(_, tween)| tween.end().min(at + step))
            .min()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::geometry::{Orientation, Size};
    use crate::domain::sensor::{DisplayFormat, Reading, SensorKey};
    use crate::domain::theme::{Binding, BoxF, Direction, Paint, TextStyle};

    fn ms(n: u64) -> Duration {
        Duration::from_millis(n)
    }

    fn key() -> SensorKey {
        SensorKey::new("cpu.usage").expect("key")
    }

    fn snapshot(value: f64) -> Snapshot {
        let mut snapshot = Snapshot::default();
        snapshot.insert(key(), Reading::Value(value));
        snapshot
    }

    fn element(id: u32, kind: ElementKind, transition: Option<ValueTransition>) -> Element {
        Element {
            sensor_motion: None,
            rotation: 0.0,
            is_group: false,
            group_parent: None,
            card: None,
            card_member: None,
            id: ElementId(id),
            name: format!("e{id}"),
            frame: BoxF::new(0.0, 0.0, 100.0, 20.0),
            opacity: 1.0,
            visible: true,
            locked: false,
            color_by_value: None,
            visualization: None,
            value_transition: transition,
            kind,
        }
    }

    fn bar(min: f64, max: f64) -> ElementKind {
        ElementKind::Bar {
            binding: Binding {
                key: key(),
                min,
                max,
            },
            direction: Direction::LeftToRight,
            fill: Paint::solid(crate::domain::frame::Rgba::WHITE),
            track: None,
            radius: 0.0,
            segments: None,
        }
    }

    fn text() -> ElementKind {
        ElementKind::Text {
            content: TextContent::Sensor {
                key: key(),
                format: DisplayFormat::default(),
                prefix: String::new(),
                suffix: String::new(),
            },
            style: TextStyle::default(),
        }
    }

    fn theme(elements: Vec<Element>) -> Theme {
        let mut theme = Theme::blank("t", Size::new(320, 480), Orientation::Portrait);
        theme.elements = elements;
        theme
    }

    #[test]
    fn the_easing_is_quick_first_and_lands_on_the_value() {
        assert_eq!(ease(0.0), 0.0);
        assert_eq!(ease(1.0), 1.0);
        assert_eq!(ease(2.0), 1.0);
        assert_eq!(ease(-1.0), 0.0);
        assert_eq!(ease(f64::NAN), 1.0);
        assert!((ease(0.5) - 0.875).abs() < 1e-12);
        let steps: Vec<f64> = (0..=10).map(|i| ease(f64::from(i) / 10.0)).collect();
        assert!(steps.windows(2).all(|w| w[1] >= w[0]), "never goes back");
    }

    #[test]
    fn inertia_preserves_position_and_velocity_on_retarget_and_settles() {
        let theme = theme(vec![element(
            1,
            bar(0., 100.),
            Some(ValueTransition {
                mode: TransitionMode::Inertia,
                duration_ms: 800,
            }),
        )]);
        let mut transitions = Transitions::default();
        transitions.retarget(&theme, &snapshot(0.), Some(ms(0)));
        transitions.retarget(&theme, &snapshot(100.), Some(ms(1000)));
        let old = transitions.tweens[&ElementId(1)];
        assert_eq!(old.value_at(ms(1000)), 0.);
        assert_eq!(old.velocity_at(ms(1000)), 0.);
        assert!(old.value_at(ms(1100)) > 0. && old.value_at(ms(1100)) < 33.1);
        transitions.retarget(&theme, &snapshot(20.), Some(ms(1100)));
        let next = transitions.tweens[&ElementId(1)];
        assert!((next.value_at(ms(1100)) - old.value_at(ms(1100))).abs() < 1e-9);
        assert!((next.velocity_at(ms(1100)) - old.velocity_at(ms(1100))).abs() < 1e-9);
        assert_eq!(next.value_at(ms(1900)), 20.);
        assert_eq!(next.velocity_at(ms(1900)), 0.);
        assert_eq!(
            transitions.next_change(&theme, ms(1900), View::Preview),
            None
        );
        transitions.still_previews(true);
        assert!(
            transitions
                .shown(&theme, ms(1150), View::Preview)
                .is_empty()
        );
        transitions.retarget(&theme, &Snapshot::default(), Some(ms(1200)));
        assert!(
            transitions
                .shown(&theme, ms(1250), View::Preview)
                .is_empty()
        );
    }

    #[test]
    fn a_transition_lasts_its_duration_within_bounds() {
        assert_eq!(ValueTransition::smooth(400).length(), Some(ms(400)));
        assert_eq!(ValueTransition::smooth(10).length(), Some(ms(50)));
        assert_eq!(ValueTransition::smooth(60_000).length(), Some(ms(5_000)));
        assert_eq!(ValueTransition::smooth(0).length(), None);
        let none = ValueTransition {
            mode: TransitionMode::None,
            duration_ms: 400,
        };
        assert_eq!(none.length(), None);
        assert_eq!(TransitionMode::default(), TransitionMode::None);
    }

    #[test]
    fn a_tween_interpolates_between_its_ends() {
        let tween = Tween::new(10.0, 50.0, ms(1_000), ms(400));
        assert_eq!(tween.value_at(ms(900)), 10.0, "before it starts");
        assert_eq!(tween.value_at(ms(1_000)), 10.0);
        assert!(
            (tween.value_at(ms(1_200)) - 45.0).abs() < 1e-9,
            "eased half"
        );
        assert_eq!(tween.value_at(ms(1_400)), 50.0);
        assert_eq!(tween.value_at(ms(9_000)), 50.0);
        assert!(tween.moving_at(ms(1_399)));
        assert!(!tween.moving_at(ms(1_400)));
        assert_eq!(
            (tween.to(), tween.end(), tween.length()),
            (50.0, ms(1_400), ms(400))
        );
        let settled = Tween::settled(7.0);
        assert!(!settled.moving_at(Duration::ZERO));
        assert_eq!(settled.value_at(ms(5)), 7.0);
    }

    #[test]
    fn targets_are_held_in_range_and_need_a_number() {
        let e = element(1, bar(0.0, 100.0), None);
        assert_eq!(target(&e, &snapshot(150.0)), Some(100.0));
        assert_eq!(target(&e, &snapshot(-3.0)), Some(0.0));
        let reversed = element(1, bar(100.0, 0.0), None);
        assert_eq!(target(&reversed, &snapshot(150.0)), Some(100.0));
        let open = element(1, bar(0.0, f64::INFINITY), None);
        assert_eq!(target(&open, &snapshot(150.0)), Some(150.0));
        assert_eq!(
            target(&element(2, text(), None), &snapshot(150.0)),
            Some(150.0)
        );
        let mut missing = Snapshot::default();
        missing.insert(key(), Reading::Unavailable("no".into()));
        assert_eq!(target(&e, &missing), None);
        let shape = element(
            3,
            ElementKind::Shape {
                image: None,
                video_window: false,
                fade: None,
                shape: crate::domain::theme::ShapeKind::Ellipse,
                fill: None,
                stroke: None,
            },
            None,
        );
        assert_eq!(target(&shape, &snapshot(1.0)), None);
    }

    #[test]
    fn a_new_reading_eases_from_what_is_shown() {
        let t = theme(vec![element(
            1,
            bar(0.0, 100.0),
            Some(ValueTransition::smooth(400)),
        )]);
        let mut tr = Transitions::default();
        tr.retarget(&t, &snapshot(20.0), Some(ms(0)));
        assert!(
            tr.shown(&t, ms(0), View::Screen).is_empty(),
            "a first value settles"
        );
        assert_eq!(tr.next_change(&t, ms(0), View::Screen), None);

        tr.retarget(&t, &snapshot(60.0), Some(ms(1_000)));
        let shown = tr.shown(&t, ms(1_200), View::Screen);
        assert_eq!(shown.len(), 1);
        assert!((shown.get(ElementId(1)).unwrap_or_default() - 55.0).abs() < 1e-9);
        assert_eq!(tr.next_change(&t, ms(1_000), View::Screen), Some(ms(1_050)));
        assert_eq!(
            tr.next_change(&t, ms(1_380), View::Screen),
            Some(ms(1_400)),
            "lands"
        );
        assert_eq!(tr.next_change(&t, ms(1_400), View::Screen), None);
        assert!(tr.shown(&t, ms(1_400), View::Screen).is_empty());

        // Retargeted mid-way: from the value shown then.
        tr.retarget(&t, &snapshot(60.0), Some(ms(1_100)));
        tr.retarget(&t, &snapshot(0.0), Some(ms(1_200)));
        let tween = tr
            .tween(ElementId(1))
            .copied()
            .unwrap_or(Tween::settled(0.0));
        assert!((tween.value_at(ms(1_200)) - 55.0).abs() < 1e-9);
        assert_eq!(tween.to(), 0.0);
    }

    #[test]
    fn steady_readings_and_plain_elements_never_move() {
        let t = theme(vec![
            element(1, bar(0.0, 100.0), Some(ValueTransition::smooth(400))),
            element(2, bar(0.0, 100.0), None),
            element(
                3,
                text(),
                Some(ValueTransition {
                    mode: TransitionMode::None,
                    duration_ms: 400,
                }),
            ),
        ]);
        let mut tr = Transitions::default();
        for n in 0..5 {
            tr.retarget(&t, &snapshot(42.0), Some(ms(n * 1_000)));
            assert!(tr.shown(&t, ms(n * 1_000 + 10), View::Screen).is_empty());
            assert_eq!(tr.next_change(&t, ms(n * 1_000), View::Screen), None);
        }
        assert!(tr.tween(ElementId(2)).is_none());
        assert!(tr.tween(ElementId(3)).is_none());
        // A sample with no clock settles.
        tr.retarget(&t, &snapshot(80.0), None);
        assert!(tr.shown(&t, ms(5_000), View::Screen).is_empty());
    }

    #[test]
    fn an_edit_settles_and_an_unavailable_reading_drops() {
        let mut t = theme(vec![element(
            1,
            bar(0.0, 100.0),
            Some(ValueTransition::smooth(400)),
        )]);
        let mut tr = Transitions::default();
        tr.retarget(&t, &snapshot(20.0), Some(ms(0)));
        tr.retarget(&t, &snapshot(80.0), Some(ms(1_000)));
        tr.edited(&t, &snapshot(80.0));
        assert_eq!(
            tr.shown(&t, ms(1_100), View::Screen).len(),
            1,
            "same target keeps moving"
        );
        // Another range: shows its value at once.
        t.elements[0].kind = bar(0.0, 50.0);
        tr.edited(&t, &snapshot(80.0));
        assert!(tr.shown(&t, ms(1_100), View::Screen).is_empty());
        assert_eq!(tr.tween(ElementId(1)).map(Tween::to), Some(50.0));
        let mut missing = Snapshot::default();
        missing.insert(key(), Reading::Unavailable("gone".into()));
        tr.retarget(&t, &missing, Some(ms(2_000)));
        assert!(tr.tween(ElementId(1)).is_none());
        tr.edited(&t, &missing);
        assert!(tr.tween(ElementId(1)).is_none());
        t.elements[0].value_transition = None;
        tr.edited(&t, &snapshot(10.0));
        assert!(tr.tween(ElementId(1)).is_none());
    }

    #[test]
    fn hidden_elements_and_still_previews_do_not_move() {
        let mut t = theme(vec![element(
            1,
            bar(0.0, 100.0),
            Some(ValueTransition::smooth(400)),
        )]);
        let mut tr = Transitions::default();
        tr.retarget(&t, &snapshot(0.0), Some(ms(0)));
        tr.retarget(&t, &snapshot(100.0), Some(ms(1_000)));
        assert!(tr.eases_in_previews(&t));
        tr.still_previews(true);
        assert!(!tr.eases_in_previews(&t));
        assert!(
            tr.shown(&t, ms(1_100), View::Preview).is_empty(),
            "reduced motion"
        );
        assert_eq!(tr.next_change(&t, ms(1_100), View::Preview), None);
        assert_eq!(
            tr.shown(&t, ms(1_100), View::Screen).len(),
            1,
            "the screen still eases"
        );
        tr.still_previews(false);
        assert_eq!(
            tr.next_change(&t, ms(1_100), View::Preview),
            Some(ms(1_150))
        );
        t.elements[0].visible = false;
        assert!(!tr.eases_in_previews(&t));
        assert!(tr.shown(&t, ms(1_100), View::Screen).is_empty());
        assert_eq!(tr.next_change(&t, ms(1_100), View::Screen), None);
    }

    #[test]
    fn a_slow_screen_gets_fewer_frames_then_snaps() {
        let t = theme(vec![element(
            1,
            bar(0.0, 100.0),
            Some(ValueTransition::smooth(400)),
        )]);
        let mut tr = Transitions::default();
        tr.retarget(&t, &snapshot(0.0), Some(ms(0)));
        tr.retarget(&t, &snapshot(100.0), Some(ms(1_000)));
        tr.screen_pace(ms(120));
        assert_eq!(tr.pace().frame(), Some(ms(120)));
        assert_eq!(
            tr.next_change(&t, ms(1_000), View::Screen),
            Some(ms(1_120)),
            "its pace"
        );
        assert_eq!(
            tr.next_change(&t, ms(1_000), View::Preview),
            Some(ms(1_050))
        );
        tr.screen_pace(ms(400));
        assert_eq!(tr.pace().frame(), Some(ms(260)), "averaged");
        assert!(!tr.pace().keeps_up(ms(400)));
        assert!(tr.shown(&t, ms(1_100), View::Screen).is_empty(), "snaps");
        assert_eq!(
            tr.next_change(&t, ms(1_000), View::Screen),
            None,
            "no frames"
        );
        assert_eq!(
            tr.shown(&t, ms(1_100), View::Preview).len(),
            1,
            "previews still ease"
        );
        tr.forget_screen();
        assert_eq!(tr.pace(), Pace::default());
        assert_eq!(Pace::default().step(), MIN_TRANSITION_STEP);
        assert_eq!(MIN_TRANSITION_STEP, ms(50));
    }
}
