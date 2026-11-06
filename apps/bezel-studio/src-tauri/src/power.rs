//! What the studio does when the computer shuts down
//! (D-2026-10-03-power-off-standby-3): each screen gets the choice its user
//! made for it (`app::standby::at_shutdown` in the core), and nothing else
//! reaches any screen from then on.
//!
//! - Windows: `RunEvent::Exit` while the session ends
//!   ([`bezel_power::session_ending`], `false` elsewhere) runs the same
//!   [`shut_down`], waiting for it, [`SESSION_END`] at most ([`at_exit`]);
//!   quitting the app applies nothing.
//!
//! [`shut_down`] first puts the session in its final state: no frame, no
//! reconnection, no link lent or opened, no storage operation
//! ([`Backend::enter_final_state`]). The running storage job is cancelled
//! and waited for; then the catalog is read again (a choice made by the CLI
//! while the studio runs counts) and each rev C screen whose choice is not
//! `keep` gets it: the live one through its open link, the other awake ones
//! opened for it; one asleep (its system off the bus) is left as it is,
//! never woken. The actions run on a thread of their own, so a screen that
//! hangs does not hold the shutdown past its deadline.
//!
//! The same choices apply when the computer goes to sleep, and the
//! monitors going off dim the live screens (D-2026-11-06-display-idle-saver-1
//! to -4): [`watch`] feeds Windows' events through an [`IdleState`] and
//! [`on_power_event`] applies what it says. Screens Bezel Light keeps are
//! Light's: its own watch dims them and drives its runtime, so the studio
//! touches only the links it opened itself and the engine it runs itself.

#[cfg(test)]
pub(crate) mod tests;

use std::sync::{Arc, mpsc};
use std::time::{Duration, Instant};

use bezel_core::app::discover_screens;
use bezel_core::app::standby::{Applied, at_shutdown};
use bezel_core::domain::archive::{Catalog, ScreenKey};
use bezel_core::domain::device::DeviceModel;
use bezel_core::domain::discovery::{Screen, ScreenState};
use bezel_core::domain::idle::{IdleState, PowerEvent, Transition};
use bezel_core::domain::screen::{Brightness, ScreenIdentity};
use bezel_core::domain::standby::{Standby, supports};
use bezel_core::ports::ScreenLink;

use crate::backend::Backend;
use crate::commands::Shared;
use crate::diag::{self, DiagCode};
use crate::studio::ForShutdown;

/// How long the actions may take when Windows ends the session: it ends
/// the process soon after.
pub const SESSION_END: Duration = Duration::from_secs(4);

/// How often the shutdown looks for the live link while it is out.
const LINK_POLL: Duration = Duration::from_millis(10);

/// How long Windows waits for the sleep to be applied
/// ([`bezel_power::display_state::watch`]).
const SUSPEND_WAIT: Duration = Duration::from_secs(2);

/// The part of [`SUSPEND_WAIT`] the shutdown choices may use at sleep.
pub const SUSPEND_BUDGET: Duration = Duration::from_millis(1800);

/// How the app ends, as `RunEvent::Exit` tells it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Exit {
    /// The session is ending: Windows is shutting down, restarting or
    /// signing out.
    SessionEnding,
    /// The app quit (the tray, the window): nothing is applied.
    Quit,
}

impl Exit {
    /// The exit, given whether the session is ending
    /// ([`bezel_power::session_ending`]).
    pub fn of(session_ending: bool) -> Self {
        if session_ending {
            Exit::SessionEnding
        } else {
            Exit::Quit
        }
    }
}

/// How the actions of a shutdown ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ending {
    /// Every choice that could be applied was.
    Done,
    /// The deadline came first (a screen hung, or a job did not stop).
    Deadline,
    /// Their thread did not start: nothing was applied.
    NotStarted,
}

/// What `RunEvent::Exit` does: on a session end, [`shut_down`] with
/// [`SESSION_END`] as the deadline, waited for; on a quit, nothing. `None`
/// when nothing was applied (a quit, or the app has no backend yet).
pub fn at_exit(backend: Option<&Shared>, exit: Exit) -> Option<Ending> {
    match (backend, exit) {
        (Some(backend), Exit::SessionEnding) => {
            Some(shut_down(backend, Instant::now() + SESSION_END))
        }
        _ => None,
    }
}

/// Applies each screen's choice as the computer shuts down: the session's
/// final state first (a caller may have entered it already: entering it
/// again changes nothing), then the actions ([`Backend::apply_choices`]) on
/// a thread of their own. Returns once they are done, or at `deadline`
/// (whatever is left goes on, unwaited for). The final state stays until
/// [`Backend::leave_final_state`].
pub fn shut_down(backend: &Shared, deadline: Instant) -> Ending {
    backend.enter_final_state();
    let (done, finished) = mpsc::channel();
    let worker = Arc::clone(backend);
    let spawned = std::thread::Builder::new()
        .name("bezel-shutdown".into())
        .spawn(move || {
            worker.apply_choices(deadline);
            let _ = done.send(());
        });
    if spawned.is_err() {
        diag::report(DiagCode::ShutdownNotApplied);
        return Ending::NotStarted;
    }
    match finished.recv_timeout(deadline.saturating_duration_since(Instant::now())) {
        Ok(()) => Ending::Done,
        Err(_) => {
            diag::report(DiagCode::ShutdownDeadline);
            Ending::Deadline
        }
    }
}

/// Applies `transition` (from [`IdleState`]) at `now`
/// (D-2026-11-06-display-idle-saver-1 to -5): idle dims the studio's live
/// screens and puts its own engine in idle ([`Backend::idle`]); the sleep
/// applies each screen's shutdown choice as a shutdown does
/// ([`shut_down`], [`SUSPEND_BUDGET`] at most, screens asleep never woken);
/// the wake leaves that final state and puts the screens back in idle when
/// the monitors are still off. With the option off [`IdleState`] says
/// [`Transition::Nothing`] and nothing happens.
pub fn on_power_event(backend: &Shared, transition: Transition, now: Instant) {
    match transition {
        Transition::Nothing => {}
        Transition::EnterIdle => backend.idle(true),
        Transition::LeaveIdle => backend.idle(false),
        Transition::Suspend => {
            shut_down(backend, now + SUSPEND_BUDGET);
        }
        Transition::Resume { idle } => {
            backend.leave_final_state();
            backend.idle(idle);
        }
    }
}

/// Feeds `event` through `state`, the option read again from the settings
/// first (Preferences may have changed it: turning it off while idle
/// leaves idle), and applies what it says ([`on_power_event`]).
pub fn on_event(backend: &Shared, state: &mut IdleState, event: PowerEvent) {
    let saver = backend.settings.load().idle_saver.0;
    on_power_event(backend, state.set_enabled(saver), Instant::now());
    on_power_event(backend, state.on(event), Instant::now());
}

/// Watches the monitors and the computer's sleep for the studio
/// ([`bezel_power::display_state::watch`]), the returned watch kept for the
/// app's lifetime: the events go through one [`IdleState`] on a thread of
/// their own ([`on_event`]); the sleep waits for it, [`SUSPEND_WAIT`] at
/// most. `None` when Windows does not report them.
pub fn watch(backend: Shared) -> Option<bezel_power::display_state::Watch> {
    let enabled = backend.settings.load().idle_saver.0;
    let (events, received) = mpsc::channel::<(PowerEvent, Option<mpsc::SyncSender<()>>)>();
    std::thread::Builder::new()
        .name("bezel-studio-power".into())
        .spawn(move || {
            let mut state = IdleState::new(enabled);
            for (event, done) in received {
                on_event(&backend, &mut state, event);
                if let Some(done) = done {
                    let _ = done.send(());
                }
            }
        })
        .ok()?;
    bezel_power::display_state::watch(move |event| {
        if event != PowerEvent::Suspend {
            let _ = events.send((event, None));
            return;
        }
        let (done, applied) = mpsc::sync_channel(1);
        if events.send((event, Some(done))).is_ok() {
            let _ = applied.recv_timeout(SUSPEND_WAIT);
        }
    })
    .ok()
}

/// The level a screen goes back to when idle ends and none was set through
/// the studio: the start level its rev C record keeps, else full
/// brightness.
fn start_level(catalog: Option<&Catalog>, identity: &ScreenIdentity) -> Brightness {
    catalog
        .and_then(|catalog| catalog.screen(&ScreenKey::new(identity.model.id)))
        .and_then(|record| record.start_brightness())
        .unwrap_or(Brightness::MAX)
}

/// What the studio says of the action of a shutdown on one screen
/// (`None`: nothing went wrong): a failure, or an album the screen restarted
/// into whose record was not updated (review W2 of iteration 3), which is
/// not a failed choice.
fn said_of(applied: &bezel_core::Result<Applied>) -> Option<DiagCode> {
    match applied {
        Ok(Applied::AlbumNotRecorded(_)) => Some(DiagCode::ShutdownAlbumNotRecorded),
        Ok(_) => None,
        Err(_) => Some(DiagCode::ShutdownChoiceFailed),
    }
}

/// The choice recorded in `catalog` for a screen of `model` (the studio
/// knows its screens by model: [`ScreenKey::new`]); `keep` without one.
fn choice_for(catalog: &Catalog, model: &DeviceModel) -> Standby {
    catalog
        .screen(&ScreenKey::new(model.id))
        .map(|record| record.standby.clone())
        .unwrap_or_default()
}

/// Whether the shutdown opens `screen` to apply its choice: an awake rev C
/// screen (one asleep is never woken) of a known model whose choice is not
/// `keep`.
fn worth_opening(screen: &Screen, catalog: &Catalog) -> bool {
    screen.state() == ScreenState::Awake
        && screen
            .model()
            .is_some_and(|model| supports(model) && choice_for(catalog, model) != Standby::Keep)
}

impl Backend {
    /// The final state of a shutdown starts (D-2026-10-03-power-off-standby-3):
    /// no operation gets the screens and the running job is asked to stop
    /// ([`crate::storage::StorageState::enter_final_state`]), and the
    /// session draws no frame, lends no link, connects nothing again and
    /// puts no screen live ([`crate::studio::Studio::enter_final_state`]).
    pub fn enter_final_state(&self) {
        if let Some(engine) = &self.engine {
            engine.suspend(true);
        }
        self.storage.enter_final_state();
        self.studio().enter_final_state();
    }

    /// The shutdown was cancelled: the final state ends.
    pub fn leave_final_state(&self) {
        if let Some(engine) = &self.engine {
            engine.suspend(false);
        }
        let unwanted = self.studio().leave_final_state();
        drop(unwanted);
        self.storage.leave_final_state();
    }

    /// The monitors went off (`on`) or came back: the engine Studio runs
    /// itself follows ([`crate::engine::EngineBridge::idle`]), and so do the
    /// live screens it opened itself ([`crate::studio::Studio::idle`]),
    /// never one Bezel Light keeps: Light dims those and restores them.
    pub fn idle(&self, on: bool) {
        if let Some(engine) = &self.engine {
            engine.idle(on);
        }
        if self.connector.manages_shutdown() {
            return;
        }
        let catalog = self.storage.archive().load().ok();
        let fallback = |identity: &ScreenIdentity| start_level(catalog.as_ref(), identity);
        // A screen that did not take its level is left as it is: the next
        // idle change or brightness set tries again, and its next frame
        // finds out whether the link failed.
        let _failed = self.idle_studio().idle(on, &fallback);
    }

    /// The actions of a shutdown, in the final state, until `deadline`: the
    /// running job stopped and waited for, then the catalog read again and
    /// each choice applied (the live screen through its link, the other
    /// awake rev C screens opened for it). A job that does not stop in time
    /// leaves every screen as it is.
    fn apply_choices(&self, deadline: Instant) {
        if self
            .engine
            .as_ref()
            .is_some_and(|engine| !engine.wait_idle(deadline))
        {
            return;
        }
        if !self.storage.wait_until_idle(deadline) {
            diag::report(DiagCode::ShutdownJobNotStopped);
            return;
        }
        // Light receives WM_ENDSESSION itself and finishes each retained link
        // once. Direct fallback keeps today's shutdown behavior.
        if self.connector.manages_shutdown() {
            return;
        }
        let mut live_links = Vec::new();
        while let Some(link) = self.live_link_for_shutdown(deadline) {
            live_links.push(link);
        }
        let catalog = match self.storage.archive().load() {
            Ok(catalog) => catalog,
            Err(_) => {
                diag::report(DiagCode::ShutdownCatalogNotRead);
                return;
            }
        };
        let had_live = !live_links.is_empty();
        for mut link in live_links {
            if Instant::now() < deadline {
                self.apply(link.as_mut());
            }
        }
        let screens = discover_screens(self.bus.as_ref()).unwrap_or_else(|_| {
            diag::report(DiagCode::ShutdownScreensNotListed);
            Vec::new()
        });
        for screen in screens {
            if Instant::now() >= deadline {
                break;
            }
            let live = had_live
                && screen
                    .address()
                    .is_some_and(|address| self.studio().is_live(&address.0));
            if live || !worth_opening(&screen, &catalog) {
                continue;
            }
            match self.connector.connect(&screen) {
                Ok(mut link) => self.apply(link.as_mut()),
                Err(_) => diag::report(DiagCode::ShutdownChoiceFailed),
            }
        }
    }

    /// Applies the choice the catalog records for the screen behind `link`
    /// (keyed by its model, [`ScreenKey::new`]), which the core reads again
    /// there ([`at_shutdown`] takes no choice from the studio); `keep` sends
    /// nothing. What went wrong is said ([`said_of`]). The catalog is locked
    /// only while it is read or saved, so a screen that hangs holds no other
    /// command ([`crate::storage::ArchivePerCall`]).
    fn apply(&self, link: &mut dyn ScreenLink) {
        let key = ScreenKey::new(link.identity().model.id);
        let applied = at_shutdown(link, &mut self.storage.archive_per_call(), &key);
        if let Some(code) = said_of(&applied) {
            diag::report(code);
        }
    }

    /// The live link, once it is back from showing a frame, a storage job
    /// or an attempt to connect it again; `None` without one, or at
    /// `deadline`.
    fn live_link_for_shutdown(&self, deadline: Instant) -> Option<Box<dyn ScreenLink>> {
        loop {
            match self.studio().take_for_shutdown() {
                ForShutdown::Link(link) => return Some(link),
                ForShutdown::Nothing => return None,
                ForShutdown::Out => {}
            }
            let now = Instant::now();
            if now >= deadline {
                return None;
            }
            std::thread::sleep(LINK_POLL.min(deadline - now));
        }
    }
}
