//! What the studio does when the computer shuts down
//! (D-2026-10-03-power-off-standby-3; DoD rows 2 and 3).
//!

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, PoisonError, mpsc};
use std::time::{Duration, Instant};

use bezel_core::app::discover_screens;
use bezel_core::domain::archive::ScreenKey;
use bezel_core::domain::clock::LocalTime;
use bezel_core::domain::device::ModelId;
use bezel_core::domain::discovery::Screen;
use bezel_core::domain::frame::Frame;
use bezel_core::domain::geometry::Orientation;
use bezel_core::domain::job::Job;
use bezel_core::domain::screen::{Brightness, ScreenIdentity};
use bezel_core::domain::standby::{PlanB, SleepMinutes, Standby, StoredPlanB};
use bezel_core::domain::storage::{
    Confirmed, FileName, RemotePath, Repeat, StartMode, StorageInfo, StorageLocation,
};
use bezel_core::ports::{ArchiveStore, DeviceBus as _, ScreenConnector, ScreenLink, ScreenStorage};
use bezel_core::{BezelError, Result};
use bezel_devices::fake::{FakeStorage, StorageCall};
use bezel_devices::{FakeBus, FakeConnector, FakeHid};
use bezel_media::archive::{DiskArchive, storage_dir};
use bezel_sensors::FakeSensors;

use super::*;
use crate::settings::SettingsFile;
use crate::tests::temp_root;
use crate::{Adapters, Folders, compose};

const TIME: LocalTime = LocalTime {
    year: 2026,
    month: 10,
    day: 3,
    hour: 23,
    minute: 59,
    second: 0,
    weekday: 5,
    millisecond: 0,
};

/// The fake 8.8"'s display (its SoC's port): the key it goes live by.
pub(crate) const DISPLAY: &str = "/dev/ttyACM1";

/// The fake 8.8"'s MCU, the one port of the screen when it sleeps.
const MCU: &str = "/dev/ttyACM0";

/// The 8.8"'s model: the studio knows its screens by model.
const MODEL: ModelId = ModelId("turing-8.8");

/// How long a test waits for what must happen.
const PATIENCE: Duration = Duration::from_secs(10);

/// `off` as the CLI records it (5 minutes of plan B).
pub(crate) fn off() -> Standby {
    Standby::Off(SleepMinutes::SUGGESTED)
}

/// Records `standby` for the 8.8" in the catalog under `data`, as another
/// writer (the CLI) does while the studio runs.
fn record(data: &Path, standby: Standby) {
    record_for(data, MODEL, standby);
}

/// Records `standby` for the screens of `model` in the catalog under
/// `data`.
fn record_for(data: &Path, model: ModelId, standby: Standby) {
    let mut archive = DiskArchive::open(storage_dir(data)).unwrap();
    let mut catalog = archive.load().unwrap();
    catalog.screen_mut(&ScreenKey::new(model)).standby = standby;
    archive.save(&catalog).unwrap();
}

/// The folders of the test `name` (its root, removed when dropped), the
/// 8.8"'s choice `standby` recorded and its display remembered as the live
/// screen (`liveScreen`), as the app leaves them.
pub(crate) fn folders_for(name: &str, standby: Standby) -> (Root, Folders) {
    let root = Root(temp_root(name));
    let folders = Folders::under(&root.0);
    record(&folders.data, standby);
    settings(&folders.config).update(|s| s.live_screen = Some(DISPLAY.into()));
    (root, folders)
}

/// A test's temporary folder, removed when dropped.
pub(crate) struct Root(PathBuf);

impl Drop for Root {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// The app's settings in `config`.
fn settings(config: &Path) -> SettingsFile {
    SettingsFile::new(config.join("settings.json"))
}

/// The machine of the tests: `bus`'s screens, reached through `connector`,
/// and scripted sensors.
pub(crate) fn adapters_over(
    bus: FakeBus,
    connector: impl ScreenConnector + Send + Sync + 'static,
) -> Adapters {
    Adapters {
        bus: Arc::new(bus),
        connector: Arc::new(connector),
        hid: Arc::new(FakeHid::answering(0x88)),
        sensors: Arc::new(|_| Box::new(FakeSensors::demo())),
    }
}

/// The backend the composition root makes on `folders`, over a fake 8.8"
/// (`bus`) reached through `connector`, its catalog on disk.
fn backend_on(
    folders: &Folders,
    bus: FakeBus,
    connector: impl ScreenConnector + Send + Sync + 'static,
) -> Shared {
    Arc::new(compose(folders, adapters_over(bus, connector), false))
}

/// A link that hangs: the error that makes a live screen be connected
/// again.
fn hung() -> BezelError {
    BezelError::Hung("it stopped reading what was sent".into())
}

// --------------------------------------------------- what screens heard --

/// Every call that reached the fake screens, by name, in order (never a
/// frame's pixels), and what a test notes among them.
#[derive(Debug, Clone, Default)]
struct Heard(Arc<Mutex<Vec<&'static str>>>);

impl Heard {
    fn note(&self, call: &'static str) {
        self.0
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .push(call);
    }

    fn all(&self) -> Vec<&'static str> {
        self.0
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }

    /// What was heard after the first `count` calls.
    fn since(&self, count: usize) -> Vec<&'static str> {
        self.all().get(count..).unwrap_or_default().to_vec()
    }

    fn count(&self, call: &str) -> usize {
        self.all().iter().filter(|heard| **heard == call).count()
    }
}

/// How a screen's `turn_off_now` goes.
#[derive(Clone)]
enum Hold {
    /// At once.
    Never,
    /// Once the test says so, or drops its sender: a screen that hangs.
    Until(Arc<Mutex<mpsc::Receiver<()>>>),
}

impl Hold {
    /// Held until `go` says so (or is dropped).
    fn until() -> (mpsc::Sender<()>, Self) {
        let (go, through) = mpsc::channel();
        (go, Hold::Until(Arc::new(Mutex::new(through))))
    }
}

/// A connector over a [`FakeConnector`] whose screens note every call in
/// [`Heard`] (the connection too) and hold `turn_off_now` as [`Hold`] says.
#[derive(Clone)]
struct Recording {
    inner: FakeConnector,
    heard: Heard,
    hold: Hold,
}

impl Recording {
    fn over(inner: FakeConnector, heard: &Heard, hold: Hold) -> Self {
        Self {
            inner,
            heard: heard.clone(),
            hold,
        }
    }
}

impl ScreenConnector for Recording {
    fn connect(&self, screen: &Screen) -> Result<Box<dyn ScreenLink>> {
        self.heard.note("connect");
        let inner = self.inner.connect(screen)?;
        Ok(Box::new(RecordingLink {
            inner,
            heard: self.heard.clone(),
            hold: self.hold.clone(),
        }))
    }

    fn restart(&self, screen: &Screen) -> Result<()> {
        self.heard.note("restart");
        self.inner.restart(screen)
    }
}

/// A [`Recording`] connector's screen.
struct RecordingLink {
    inner: Box<dyn ScreenLink>,
    heard: Heard,
    hold: Hold,
}

impl RecordingLink {
    fn store(&mut self, call: &'static str) -> Result<&mut dyn ScreenStorage> {
        self.heard.note(call);
        self.inner
            .storage()
            .ok_or_else(|| BezelError::Unsupported("no storage".into()))
    }
}

impl ScreenLink for RecordingLink {
    fn identity(&self) -> &ScreenIdentity {
        self.inner.identity()
    }

    fn set_brightness(&mut self, brightness: Brightness) -> Result<()> {
        self.heard.note("set_brightness");
        self.inner.set_brightness(brightness)
    }

    fn set_orientation(&mut self, orientation: Orientation) -> Result<()> {
        self.heard.note("set_orientation");
        self.inner.set_orientation(orientation)
    }

    fn present(&mut self, frame: &Frame) -> Result<()> {
        self.heard.note("present");
        self.inner.present(frame)
    }

    fn screen_off(&mut self) -> Result<()> {
        self.heard.note("screen_off");
        self.inner.screen_off()
    }

    fn turn_off_now(&mut self) -> Result<()> {
        self.heard.note("turn_off_now");
        if let Hold::Until(through) = &self.hold {
            let _ = through
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .recv();
        }
        self.inner.turn_off_now()
    }

    fn release(&mut self) -> Result<()> {
        self.heard.note("release");
        self.inner.release()
    }

    fn storage(&mut self) -> Option<&mut dyn ScreenStorage> {
        if self.inner.storage().is_some() {
            Some(self)
        } else {
            None
        }
    }
}

impl ScreenStorage for RecordingLink {
    fn info(&mut self) -> Result<StorageInfo> {
        self.store("storage.info")?.info()
    }

    fn list(&mut self, location: StorageLocation) -> Result<Vec<FileName>> {
        self.store("storage.list")?.list(location)
    }

    fn size(&mut self, path: &RemotePath) -> Result<Option<u64>> {
        self.store("storage.size")?.size(path)
    }

    fn upload(&mut self, path: &RemotePath, data: &[u8], job: &mut Job<'_>) -> Result<()> {
        self.store("storage.upload")?.upload(path, data, job)
    }

    fn delete(&mut self, path: &RemotePath, confirmed: Confirmed) -> Result<()> {
        self.store("storage.delete")?.delete(path, confirmed)
    }

    fn play_video(&mut self, path: &RemotePath, repeat: Repeat) -> Result<()> {
        self.store("storage.play_video")?.play_video(path, repeat)
    }

    fn play_image(&mut self, path: &RemotePath) -> Result<()> {
        self.store("storage.play_image")?.play_image(path)
    }

    fn stop(&mut self) -> Result<()> {
        self.store("storage.stop")?.stop()
    }

    fn set_options(&mut self, plan: PlanB, confirmed: Confirmed) -> Result<()> {
        self.store("storage.set_options")?
            .set_options(plan, confirmed)
    }

    fn restart(&mut self, confirmed: Confirmed) -> Result<()> {
        self.store("storage.restart")?.restart(confirmed)
    }
}

// ------------------------------------------------------- every system --

#[test]
fn the_exit_says_whether_the_session_ends() {
    assert_eq!(Exit::of(true), Exit::SessionEnding);
    assert_eq!(Exit::of(false), Exit::Quit);
    // The session is not ending while the tests run, on Windows too.
    assert_eq!(Exit::of(bezel_power::session_ending()), Exit::Quit);
}

/// D-2026-10-03-power-off-standby-3 (2) and -7 (DoD row 3): what
/// `RunEvent::Exit` does ([`at_exit`]), on the backend the composition root
/// makes and without Tauri's mock runtime, so it runs on Windows too.
/// Quitting the app sends nothing; the end of the session applies the
/// choice the catalog records (written by another writer, read again
/// then) through the live screen's open link, and nothing else; then the
/// final state holds.
#[test]
fn a_session_end_applies_the_choice_and_a_quit_does_not() {
    let (_root, folders) = folders_for("power-exit", off());
    let heard = Heard::default();
    let fake = FakeConnector::with_storage(FakeStorage::default());
    let connector = Recording::over(fake.clone(), &heard, Hold::Never);
    let backend = backend_on(&folders, FakeBus::turing_88(), connector);
    backend.set_live(true, Some(DISPLAY), TIME).unwrap();
    let live = heard.all();
    assert_eq!(heard.count("present"), 1, "{live:?}");

    assert_eq!(at_exit(Some(&backend), Exit::Quit), None);
    let exit = Exit::of(bezel_power::session_ending());
    assert_eq!(
        at_exit(Some(&backend), exit),
        None,
        "a quit, while the tests run"
    );
    assert_eq!(at_exit(None, Exit::SessionEnding), None, "no backend yet");
    assert_eq!(heard.all(), live, "quitting sent something");
    assert!(!backend.studio().shutting_down());

    let asked = Instant::now();
    assert_eq!(
        at_exit(Some(&backend), Exit::SessionEnding),
        Some(Ending::Done)
    );
    assert!(asked.elapsed() < SESSION_END);
    assert_eq!(
        heard.since(live.len()),
        ["turn_off_now"],
        "through the live link"
    );
    assert_eq!(
        fake.log().storage.calls.last(),
        Some(&StorageCall::TurnOffNow)
    );
    assert!(backend.studio().shutting_down(), "the final state holds");
    assert_eq!(
        backend.studio().live_key(),
        Some(DISPLAY),
        "live mode stays"
    );
    backend.tick_and_wait(TIME, Instant::now() + Duration::from_secs(60));
    assert!(backend.set_live(true, Some(DISPLAY), TIME).is_err());
    assert_eq!(heard.since(live.len()), ["turn_off_now"]);
    assert_eq!(
        settings(&folders.config).load().live_screen.as_deref(),
        Some(DISPLAY)
    );
}

/// D-2026-10-03-power-off-standby-3 (2): when Windows ends the session the
/// sequence is waited for, [`SESSION_END`] at most: a screen that hangs
/// does not hold the app's end longer.
#[test]
fn a_session_end_waits_four_seconds_at_most() {
    let (_root, folders) = folders_for("power-exit-hung", off());
    let heard = Heard::default();
    let (go, hold) = Hold::until();
    let fake = FakeConnector::with_storage(FakeStorage::default());
    let backend = backend_on(
        &folders,
        FakeBus::turing_88(),
        Recording::over(fake, &heard, hold),
    );
    backend.set_live(true, Some(DISPLAY), TIME).unwrap();
    let asked = Instant::now();
    assert_eq!(
        at_exit(Some(&backend), Exit::SessionEnding),
        Some(Ending::Deadline)
    );
    let took = asked.elapsed();
    assert!(
        took >= SESSION_END && took < SESSION_END + Duration::from_secs(1),
        "{took:?}"
    );
    assert_eq!(heard.count("turn_off_now"), 1);
    drop(go);
}

/// Review W1 of iteration 3: the shutdown holds the catalog only while it
/// reads or saves it, never while a screen carries out its choice. A
/// screen that hangs in its action (a write stalls until it gives up),
/// past the deadline and after the shutdown is cancelled, holds no command
/// that reads the catalog.
#[test]
fn a_hung_screen_holds_no_command_on_the_catalog() {
    let (_root, folders) = folders_for("power-hung-catalog", off());
    let heard = Heard::default();
    let (go, hold) = Hold::until();
    let fake = FakeConnector::with_storage(FakeStorage::default());
    let backend = backend_on(
        &folders,
        FakeBus::turing_88(),
        Recording::over(fake, &heard, hold),
    );
    backend.set_live(true, Some(DISPLAY), TIME).unwrap();
    let deadline = Instant::now() + Duration::from_millis(500);
    assert_eq!(shut_down(&backend, deadline), Ending::Deadline);
    assert_eq!(heard.count("turn_off_now"), 1, "the screen hangs in it");
    // Leaving the final state lets commands run again.
    backend.leave_final_state();

    let (said, answer) = mpsc::channel();
    let reader = Arc::clone(&backend);
    std::thread::spawn(move || {
        let _ = said.send(reader.cache_info().is_ok());
    });
    let read = answer.recv_timeout(Duration::from_secs(2));
    drop(go);
    assert_eq!(read, Ok(true), "the catalog waited for the hung screen");
}

/// Review W2 of iteration 3: what the studio says of each screen's action
/// at shutdown is what happened. An album the screen restarted into whose
/// record could not be saved is said with its own fixed code, never as a
/// failed choice; a choice applied says nothing; a failure is a failure.
#[test]
fn an_album_not_recorded_is_not_said_as_a_failed_choice() {
    let full = BezelError::Transport("no space left on the disk".into());
    let video = RemotePath::parse("sd/video/loop.mp4").unwrap();
    for (applied, said) in [
        (
            Ok(Applied::AlbumNotRecorded(full.clone())),
            Some(DiagCode::ShutdownAlbumNotRecorded),
        ),
        (Ok(Applied::Album), None),
        (Ok(Applied::Nothing), None),
        (Ok(Applied::TurnedOff), None),
        (Ok(Applied::Video(video)), None),
        (Err(full), Some(DiagCode::ShutdownChoiceFailed)),
    ] {
        assert_eq!(said_of(&applied), said, "{applied:?}");
    }
    assert_ne!(
        DiagCode::ShutdownAlbumNotRecorded.text(),
        DiagCode::ShutdownChoiceFailed.text()
    );
}

/// D-2026-10-03-power-off-standby-3 (1): the final state, entered where
/// every link is opened or lent: once in it, nothing reaches a screen.
/// Neither a refresh nor a reconnection long due (the live screen's link
/// failed), nor any command (live mode on or off, brightness, release, a
/// restart, a storage operation), nor lending the live link, opening a
/// screen or claiming the screens. Live mode and the remembered live
/// screen stay; leaving the final state lets the reconnection go on.
#[test]
fn the_final_state_lets_nothing_reach_a_screen() {
    let (_root, folders) = folders_for("power-final", off());
    let heard = Heard::default();
    let fake = FakeConnector::with_storage(FakeStorage::default()).breaking_after(1, hung());
    let backend = backend_on(
        &folders,
        FakeBus::turing_88(),
        Recording::over(fake, &heard, Hold::Never),
    );
    backend.set_live(true, Some(DISPLAY), TIME).unwrap();
    let later = Instant::now() + Duration::from_secs(60);
    backend.tick_and_wait(TIME, later);
    assert!(
        backend.studio().reconnecting().is_some(),
        "{:?}",
        heard.all()
    );

    backend.enter_final_state();
    let before = heard.all();
    let long_due = later + Duration::from_secs(60);
    for _ in 0..3 {
        backend.tick_and_wait(TIME, long_due);
    }
    assert!(backend.studio().reconnect_due(long_due).is_none());
    assert!(backend.studio().tick(TIME, long_due).unwrap().is_none());
    assert!(backend.set_live(true, Some(DISPLAY), TIME).is_err());
    assert!(backend.set_live(false, None, TIME).is_err());
    assert!(backend.set_brightness(DISPLAY, 50).is_err());
    assert!(backend.release(DISPLAY).is_err());
    assert!(backend.restart_screen(MCU, TIME).is_err());
    assert!(backend.storage_overview(DISPLAY, TIME).is_err());
    assert!(backend.studio().lend_live_link(DISPLAY).is_err());
    assert!(backend.connect(DISPLAY).is_err());
    assert!(backend.storage.claim().is_err());
    assert!(backend.storage.ensure_idle().is_err());
    assert_eq!(
        heard.all(),
        before,
        "a screen heard something in the final state"
    );
    assert_eq!(backend.studio().live_key(), Some(DISPLAY));
    assert_eq!(
        settings(&folders.config).load().live_screen.as_deref(),
        Some(DISPLAY)
    );

    backend.leave_final_state();
    backend.tick_and_wait(TIME, long_due);
    assert_eq!(heard.since(before.len()).first(), Some(&"connect"));
    assert!(backend.storage.claim().is_ok());
}

/// D-2026-10-03-power-off-standby-3 (1): the session's own gates. In the
/// final state a link opened just before it (a screen going live as the
/// shutdown starts) is closed, never shown; the live link is not lent, not
/// handed out by stopping live mode, not used for the brightness, and no
/// frame is drawn: the shutdown alone takes it, once. Leaving the final
/// state stops live mode, which the caller shows again as at the start.
#[test]
fn the_session_hands_its_live_link_to_the_shutdown_only() {
    let (_root, folders) = folders_for("power-session", Standby::Keep);
    let heard = Heard::default();
    let fake = FakeConnector::with_storage(FakeStorage::default());
    let connector = Recording::over(fake, &heard, Hold::Never);
    let backend = backend_on(&folders, FakeBus::turing_88(), connector.clone());
    backend.set_live(true, Some(DISPLAY), TIME).unwrap();
    let screen = discover_screens(&FakeBus::turing_88()).unwrap().remove(0);
    let opened = connector.connect(&screen).unwrap();
    let mut studio = backend.studio();
    assert!(
        matches!(studio.take_for_shutdown(), ForShutdown::Nothing),
        "outside the final state"
    );

    studio.enter_final_state();
    let before = heard.all();
    assert!(
        !studio.go_live(DISPLAY.into(), opened),
        "a screen went live"
    );
    assert!(studio.lend_live_link(DISPLAY).is_err());
    assert!(studio.stop_live().is_none());
    assert_eq!(studio.live_key(), Some(DISPLAY), "live mode stays");
    let brightness = Brightness::new(40).unwrap();
    assert!(studio.live_brightness(DISPLAY, brightness).is_err());
    assert!(
        studio
            .frame_for_screen(TIME, Instant::now())
            .unwrap()
            .is_none()
    );
    let ForShutdown::Link(link) = studio.take_for_shutdown() else {
        panic!("the shutdown did not get the live link");
    };
    assert_eq!(link.identity().model.id, MODEL);
    assert!(
        matches!(studio.take_for_shutdown(), ForShutdown::Nothing),
        "taken once"
    );
    drop(link);
    assert_eq!(heard.all(), before, "a screen heard something");

    assert!(studio.leave_final_state().is_none());
    assert_eq!(studio.live_key(), None);
    assert!(!studio.shutting_down());
}

/// D-2026-10-03-power-off-standby-3 (1): with no screen live, the shutdown
/// opens each awake rev C screen whose choice is not `keep` and applies it
/// (`album`: OPTIONS start mode 1 and RESTART, with a card); a screen
/// asleep (only its MCU on the bus) is never woken, and nothing is sent
/// to it. The catalog on disk then says the album's plan B was stored
/// last, where a boot media set after the choice had stored start mode 2
/// (review W5 of iteration 2).
#[test]
fn a_shutdown_opens_the_awake_screens_and_wakes_none() {
    let (_root, folders) = folders_for("power-awake", Standby::Album);
    let stored = |data: &Path| {
        let mut archive = DiskArchive::open(storage_dir(data)).unwrap();
        let catalog = archive.load().unwrap();
        catalog
            .screen(&ScreenKey::new(MODEL))
            .and_then(|r| r.stored)
    };
    let video = StoredPlanB {
        plan: PlanB::new(StartMode::Video, 0),
        brightness: None,
    };
    {
        let mut archive = DiskArchive::open(storage_dir(&folders.data)).unwrap();
        let mut catalog = archive.load().unwrap();
        catalog.screen_mut(&ScreenKey::new(MODEL)).stored = Some(video);
        archive.save(&catalog).unwrap();
    }
    let heard = Heard::default();
    let card = FakeStorage::default().with_card(1 << 30);
    let fake = FakeConnector::with_storage(card);
    let connector = Recording::over(fake.clone(), &heard, Hold::Never);
    let backend = backend_on(&folders, FakeBus::turing_88(), connector);
    assert_eq!(shut_down(&backend, Instant::now() + PATIENCE), Ending::Done);
    assert_eq!(
        heard.all(),
        [
            "connect",
            "storage.info",
            "storage.set_options",
            "storage.restart"
        ]
    );
    assert_eq!(
        fake.log().storage.options,
        Some(PlanB::new(StartMode::Image, 0))
    );
    let album = StoredPlanB {
        plan: PlanB::new(StartMode::Image, 0),
        brightness: None,
    };
    assert_eq!(stored(&folders.data), Some(album));

    let (_root, folders) = folders_for("power-asleep", off());
    let heard = Heard::default();
    let fake = FakeConnector::with_storage(FakeStorage::default());
    let only_the_mcu = FakeBus::turing_88()
        .endpoints()
        .unwrap()
        .into_iter()
        .filter(|endpoint| endpoint.address.0 == MCU)
        .collect();
    let backend = backend_on(
        &folders,
        FakeBus::new(only_the_mcu),
        Recording::over(fake, &heard, Hold::Never),
    );
    assert_eq!(shut_down(&backend, Instant::now() + PATIENCE), Ending::Done);
    assert!(heard.all().is_empty(), "{:?}", heard.all());
}
