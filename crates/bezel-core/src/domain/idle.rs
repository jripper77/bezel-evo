//! What the screens do while the computer's monitors are off
//! (D-2026-11-06-display-idle-saver-1..5): when Windows turns the monitors
//! off, every screen Bezel shows dims to [`IDLE_BRIGHTNESS`] and shows the
//! logo, moved every [`interval`] so that nothing burns in; when they come
//! back, the level the user had comes back too. The computer's sleep
//! applies the shutdown choice of rev C screens and its wake undoes it.
//!
//! [`IdleState`] turns the power events an adapter reports into the
//! [`Transition`] to apply, idempotently: the same event twice changes
//! nothing the second time. With the option off, nothing happens.

use std::time::Duration;

use super::device::Family;
use super::frame::Rect;
use super::geometry::Size;
use super::screen::Brightness;

/// The backlight level of a screen while the monitors are off
/// (D-2026-11-06-display-idle-saver-1).
pub const IDLE_BRIGHTNESS: Brightness = match Brightness::new(10) {
    Some(level) => level,
    None => Brightness::MAX,
};

/// How long a change of backlight level takes: the level goes from one
/// value to the other through the ones between, instead of jumping.
pub const FADE: Duration = Duration::from_millis(400);

/// How often a fade sends the next level.
pub const FADE_STEP: Duration = Duration::from_millis(40);

/// The level of a fade from `from` to `to` lasting `duration`, `elapsed`
/// after it started: linear, rounded to the nearest percent, `to` once the
/// time is up (at once with a zero `duration`). Driven by the time, not by
/// a count of steps: a step sent late skips ahead instead of slowing the
/// fade down.
pub fn fade_level(
    from: Brightness,
    to: Brightness,
    elapsed: Duration,
    duration: Duration,
) -> Brightness {
    if elapsed >= duration {
        return to;
    }
    let from = i64::from(from.percent());
    let span = i64::from(to.percent()) - from;
    let total = duration.as_micros() as i64;
    let done = elapsed.as_micros() as i64;
    // Rounded half away from zero, whichever way the fade goes.
    let offset = (2 * span * done + span.signum() * total) / (2 * total);
    let percent = u8::try_from(from + offset).unwrap_or(0);
    Brightness::new(percent).unwrap_or(to)
}

/// How often the logo moves on a screen of `family`: 30 s, and 60 s for the
/// families whose links are slow (Turing 3.5" rev A, Kipye), where every
/// move is a whole frame (D-2026-11-06-display-idle-saver-2, -4).
pub const fn interval(family: Family) -> Duration {
    match family {
        Family::TuringRevA | Family::KipyeRevD => Duration::from_secs(60),
        _ => Duration::from_secs(30),
    }
}

/// The logo's side, in percent of the canvas's shorter side.
const LOGO_PERCENT: u32 = 40;

/// The order in which the logo visits the nine places of a 3 x 3 grid:
/// every place once before any repeats, two consecutive ones never in the
/// same row or column more than needed (step 4 of 9: always another cell).
const PLACES: [(u32, u32); 9] = [
    (0, 0),
    (1, 1),
    (2, 2),
    (0, 1),
    (1, 2),
    (2, 0),
    (0, 2),
    (1, 0),
    (2, 1),
];

/// Where the logo goes on `canvas` for its `slot`-th move (the number of
/// [`interval`]s since the screen went idle): a square whose side is
/// [`LOGO_PERCENT`] of the shorter side, always inside the canvas, at one of
/// nine places; two consecutive slots never share a place. Deterministic:
/// the same canvas and slot give the same box everywhere.
pub fn logo_box(canvas: Size, slot: u64) -> Rect {
    let edge = canvas.width.min(canvas.height) * LOGO_PERCENT / 100;
    let free_x = canvas.width - edge;
    let free_y = canvas.height - edge;
    let (column, row) = PLACES[(slot % PLACES.len() as u64) as usize];
    Rect::new(free_x * column / 2, free_y * row / 2, edge, edge)
}

/// A change in the computer's power state, as the adapter watching Windows
/// reports it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PowerEvent {
    /// The monitors were turned off.
    DisplayOff,
    /// The monitors are on.
    DisplayOn,
    /// The monitors were dimmed: still on, nothing changes.
    DisplayDimmed,
    /// The computer is going to sleep.
    Suspend,
    /// The computer woke up.
    Resume,
}

/// What the screens must do after a [`PowerEvent`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Transition {
    /// Nothing changes.
    Nothing,
    /// Dim the screens and show the moving logo.
    EnterIdle,
    /// Restore the screens' level and what they showed.
    LeaveIdle,
    /// The computer goes to sleep: apply the rev C shutdown choice, stop
    /// the audio capture.
    Suspend,
    /// The computer woke up: undo [`Transition::Suspend`]. With `idle`, the
    /// monitors are still off and the screens go back to idle.
    Resume {
        /// The monitors are still off.
        idle: bool,
    },
}

/// The idle state of the screens, fed with every [`PowerEvent`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IdleState {
    enabled: bool,
    display_off: bool,
    suspended: bool,
    idle: bool,
}

impl IdleState {
    /// A state with the monitors on, awake; `enabled` is the user's option
    /// (on by default, D-2026-11-06-display-idle-saver-4).
    pub const fn new(enabled: bool) -> Self {
        Self {
            enabled,
            display_off: false,
            suspended: false,
            idle: false,
        }
    }

    /// The user's option.
    pub const fn enabled(&self) -> bool {
        self.enabled
    }

    /// The screens are idle (dimmed, showing the moving logo).
    pub const fn is_idle(&self) -> bool {
        self.idle
    }

    /// The computer's sleep was applied and its wake not yet.
    pub const fn is_suspended(&self) -> bool {
        self.suspended
    }

    /// Takes `event` and says what to do. The monitors' state is kept even
    /// with the option off, so that turning it on while they are off enters
    /// idle at once ([`Self::set_enabled`]).
    pub fn on(&mut self, event: PowerEvent) -> Transition {
        match event {
            PowerEvent::DisplayOff => {
                self.display_off = true;
                self.enter()
            }
            PowerEvent::DisplayOn => {
                self.display_off = false;
                self.leave()
            }
            PowerEvent::DisplayDimmed => Transition::Nothing,
            PowerEvent::Suspend => self.suspend(),
            PowerEvent::Resume => self.resume(),
        }
    }

    /// Changes the user's option: turning it off while idle leaves idle,
    /// turning it on while the monitors are off enters it. A sleep already
    /// applied is still undone by the wake.
    pub fn set_enabled(&mut self, enabled: bool) -> Transition {
        self.enabled = enabled;
        if enabled {
            self.enter()
        } else if self.idle {
            self.idle = false;
            Transition::LeaveIdle
        } else {
            Transition::Nothing
        }
    }

    fn enter(&mut self) -> Transition {
        if !self.enabled || self.suspended || self.idle || !self.display_off {
            return Transition::Nothing;
        }
        self.idle = true;
        Transition::EnterIdle
    }

    fn leave(&mut self) -> Transition {
        if self.suspended || !self.idle {
            return Transition::Nothing;
        }
        self.idle = false;
        Transition::LeaveIdle
    }

    fn suspend(&mut self) -> Transition {
        if !self.enabled || self.suspended {
            return Transition::Nothing;
        }
        self.suspended = true;
        self.idle = false;
        Transition::Suspend
    }

    fn resume(&mut self) -> Transition {
        if !self.suspended {
            return Transition::Nothing;
        }
        self.suspended = false;
        self.idle = self.enabled && self.display_off;
        Transition::Resume { idle: self.idle }
    }
}

/// The backlight level to restore when idle ends: the last one set through
/// Bezel in this session, kept apart from the idle level itself.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct IdleBrightness {
    last: Option<Brightness>,
}

impl IdleBrightness {
    /// Remembers `level`, set by the user (never [`IDLE_BRIGHTNESS`] sent
    /// by [`crate::app::idle::dim`]).
    pub fn record(&mut self, level: Brightness) {
        self.last = Some(level);
    }

    /// The last level recorded, if any.
    pub const fn last(&self) -> Option<Brightness> {
        self.last
    }

    /// The level to restore: the last one recorded, else `fallback` (the
    /// adapter's: the start level a rev C screen keeps, or full brightness).
    pub fn to_restore(&self, fallback: Brightness) -> Brightness {
        self.last.unwrap_or(fallback)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn level(percent: u8) -> Brightness {
        Brightness::new(percent).unwrap()
    }

    #[test]
    fn idle_brightness_is_ten_percent_and_intervals_follow_the_family() {
        assert_eq!(IDLE_BRIGHTNESS.percent(), 10);
        assert_eq!(interval(Family::TuringRevC), Duration::from_secs(30));
        assert_eq!(interval(Family::TuringUsb), Duration::from_secs(30));
        assert_eq!(interval(Family::WeAct), Duration::from_secs(30));
        assert_eq!(interval(Family::TuringRevA), Duration::from_secs(60));
        assert_eq!(interval(Family::KipyeRevD), Duration::from_secs(60));
    }

    #[test]
    fn idle_display_off_and_on_enter_and_leave_once() {
        let mut state = IdleState::new(true);
        assert_eq!(state.on(PowerEvent::DisplayOn), Transition::Nothing);
        assert_eq!(state.on(PowerEvent::DisplayOff), Transition::EnterIdle);
        assert!(state.is_idle());
        assert_eq!(state.on(PowerEvent::DisplayOff), Transition::Nothing);
        assert_eq!(state.on(PowerEvent::DisplayOn), Transition::LeaveIdle);
        assert!(!state.is_idle());
        assert_eq!(state.on(PowerEvent::DisplayOn), Transition::Nothing);
    }

    #[test]
    fn idle_dimmed_monitors_change_nothing() {
        let mut state = IdleState::new(true);
        assert_eq!(state.on(PowerEvent::DisplayDimmed), Transition::Nothing);
        assert!(!state.is_idle());
        state.on(PowerEvent::DisplayOff);
        assert_eq!(state.on(PowerEvent::DisplayDimmed), Transition::Nothing);
        assert!(state.is_idle());
    }

    #[test]
    fn idle_option_off_always_does_nothing() {
        let mut state = IdleState::new(false);
        for event in [
            PowerEvent::DisplayOff,
            PowerEvent::DisplayDimmed,
            PowerEvent::Suspend,
            PowerEvent::Resume,
            PowerEvent::DisplayOn,
        ] {
            assert_eq!(state.on(event), Transition::Nothing, "{event:?}");
        }
        assert!(!state.is_idle());
        assert!(!state.is_suspended());
    }

    #[test]
    fn idle_suspend_and_resume_are_idempotent() {
        let mut state = IdleState::new(true);
        assert_eq!(state.on(PowerEvent::Resume), Transition::Nothing);
        assert_eq!(state.on(PowerEvent::Suspend), Transition::Suspend);
        assert_eq!(state.on(PowerEvent::Suspend), Transition::Nothing);
        assert_eq!(state.on(PowerEvent::DisplayOn), Transition::Nothing);
        assert_eq!(
            state.on(PowerEvent::Resume),
            Transition::Resume { idle: false }
        );
        assert_eq!(state.on(PowerEvent::Resume), Transition::Nothing);
    }

    #[test]
    fn idle_resume_with_the_monitors_still_off_goes_back_to_idle() {
        let mut state = IdleState::new(true);
        assert_eq!(state.on(PowerEvent::DisplayOff), Transition::EnterIdle);
        assert_eq!(state.on(PowerEvent::Suspend), Transition::Suspend);
        assert!(!state.is_idle());
        assert_eq!(state.on(PowerEvent::DisplayOff), Transition::Nothing);
        assert_eq!(
            state.on(PowerEvent::Resume),
            Transition::Resume { idle: true }
        );
        assert!(state.is_idle());
        assert_eq!(state.on(PowerEvent::DisplayOn), Transition::LeaveIdle);
    }

    #[test]
    fn idle_option_follows_the_monitors_when_toggled() {
        let mut state = IdleState::new(false);
        assert_eq!(state.on(PowerEvent::DisplayOff), Transition::Nothing);
        assert_eq!(state.set_enabled(true), Transition::EnterIdle);
        assert_eq!(state.set_enabled(true), Transition::Nothing);
        assert_eq!(state.set_enabled(false), Transition::LeaveIdle);
        assert_eq!(state.set_enabled(false), Transition::Nothing);
        assert_eq!(state.on(PowerEvent::DisplayOn), Transition::Nothing);
        assert_eq!(state.set_enabled(true), Transition::Nothing);
    }

    #[test]
    fn idle_wake_is_undone_even_after_the_option_is_turned_off() {
        let mut state = IdleState::new(true);
        assert_eq!(state.on(PowerEvent::Suspend), Transition::Suspend);
        assert_eq!(state.set_enabled(false), Transition::Nothing);
        assert_eq!(
            state.on(PowerEvent::Resume),
            Transition::Resume { idle: false }
        );
    }

    #[test]
    fn idle_logo_box_stays_inside_and_moves_every_slot() {
        let canvases = [
            Size::new(1920, 480),
            Size::new(480, 1920),
            Size::new(320, 480),
            Size::new(480, 480),
            Size::new(80, 160),
        ];
        for canvas in canvases {
            let mut previous = None;
            for slot in 0..40 {
                let logo = logo_box(canvas, slot);
                assert!(logo.width > 0 && logo.width == logo.height, "{canvas:?}");
                assert!(logo.x + logo.width <= canvas.width, "{canvas:?} {slot}");
                assert!(logo.y + logo.height <= canvas.height, "{canvas:?} {slot}");
                assert_ne!(Some(logo), previous, "{canvas:?} {slot}");
                assert_eq!(logo, logo_box(canvas, slot));
                previous = Some(logo);
            }
        }
    }

    #[test]
    fn idle_logo_box_visits_every_place_before_repeating() {
        let canvas = Size::new(1920, 480);
        let mut seen: Vec<Rect> = (0..9).map(|slot| logo_box(canvas, slot)).collect();
        seen.sort_by_key(|r| (r.x, r.y));
        seen.dedup();
        assert_eq!(seen.len(), 9);
        assert_eq!(logo_box(canvas, 9), logo_box(canvas, 0));
        assert_eq!(logo_box(canvas, u64::MAX), logo_box(canvas, u64::MAX % 9));
    }

    #[test]
    fn idle_fade_goes_through_the_levels_between_and_ends_on_the_target() {
        let ms = Duration::from_millis;
        assert_eq!(fade_level(level(10), level(70), ms(0), FADE), level(10));
        assert_eq!(fade_level(level(10), level(70), ms(200), FADE), level(40));
        assert_eq!(fade_level(level(70), level(10), ms(200), FADE), level(40));
        assert_eq!(fade_level(level(10), level(70), ms(400), FADE), level(70));
        assert_eq!(fade_level(level(10), level(70), ms(9000), FADE), level(70));
        assert_eq!(fade_level(level(0), level(3), ms(200), FADE), level(2));
        assert_eq!(fade_level(level(3), level(0), ms(200), FADE), level(1));
        assert_eq!(
            fade_level(level(10), level(70), ms(0), Duration::ZERO),
            level(70)
        );
        for (from, to) in [(10, 70), (70, 10), (0, 100), (55, 55)] {
            let mut previous = level(from);
            for step in 0..=12 {
                let now = fade_level(level(from), level(to), FADE_STEP * step, FADE);
                let (low, high) = (from.min(to), from.max(to));
                assert!((low..=high).contains(&now.percent()), "{from}->{to}");
                let towards = |a: u8, b: u8| if to >= from { a <= b } else { a >= b };
                assert!(towards(previous.percent(), now.percent()), "{from}->{to}");
                previous = now;
            }
            assert_eq!(previous, level(to));
        }
    }

    #[test]
    fn idle_brightness_restores_the_last_level_or_the_fallback() {
        let mut levels = IdleBrightness::default();
        assert_eq!(levels.last(), None);
        assert_eq!(levels.to_restore(level(70)), level(70));
        levels.record(level(40));
        levels.record(level(55));
        assert_eq!(levels.last(), Some(level(55)));
        assert_eq!(levels.to_restore(Brightness::MAX), level(55));
    }
}
