//! Persistent screen links owned by Light, independently of editor lifetimes.
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::{Duration, Instant};

use bezel_core::domain::{
    discovery::Screen,
    frame::Frame,
    geometry::Orientation,
    idle::{FADE, FADE_STEP, IDLE_BRIGHTNESS, fade_level},
    job::Job,
    screen::{Brightness, ScreenIdentity},
    standby::PlanB,
    storage::{Confirmed, FileName, RemotePath, Repeat, StorageInfo, StorageLocation},
};
use bezel_core::ports::{ScreenConnector, ScreenLink, ScreenStorage, TransferStats};
use bezel_core::{BezelError, Result};

type Links = BTreeMap<String, Arc<Mutex<Kept>>>;

fn lock<T>(value: &Mutex<T>) -> Result<MutexGuard<'_, T>> {
    value
        .lock()
        .map_err(|_| BezelError::Transport("screen owner lock poisoned".into()))
}

struct Kept {
    link: Box<dyn ScreenLink>,
    orientation: Option<Orientation>,
    video: Option<(RemotePath, Repeat)>,
    edited: bool,
    failed: bool,
    /// Every port the screen answers to (lowercase): its level is
    /// remembered under each, as Studio and Light know it by either.
    addresses: Vec<String>,
    /// The level last sent through this link; unknown until one is.
    shown: Option<Brightness>,
    /// Counts the fades started on this link: a fade stops as soon as a
    /// newer one starts.
    fading: u64,
}

impl Kept {
    fn invalidate(&mut self) {
        let identity = self.link.identity().clone();
        self.link = Box::new(Disconnected(identity));
        self.video = None;
        self.orientation = None;
        self.failed = true;
    }
    fn call<T>(&mut self, op: impl FnOnce(&mut Self) -> Result<T>) -> Result<T> {
        let result = op(self);
        if matches!(
            &result,
            Err(BezelError::Transport(_)
                | BezelError::Timeout(_)
                | BezelError::Hung(_)
                | BezelError::ScreenNotFound(_))
        ) {
            self.invalidate();
        }
        result
    }
}

/// Shared by the pipe server and Light's local runtime. Dropping a client
/// handle never drops or releases the physical connection.
#[derive(Clone)]
pub struct Owner {
    connector: Arc<dyn ScreenConnector + Send + Sync>,
    links: Arc<Mutex<Links>>,
    editing: Arc<AtomicBool>,
    disabled: Arc<AtomicBool>,
    reload: Arc<AtomicBool>,
    idle: Arc<AtomicBool>,
    /// The level chosen for each screen, by port (lowercase): from the
    /// settings and every level set since. Kept when the links close, so
    /// that a screen opened again gets it back.
    levels: Arc<Mutex<BTreeMap<String, Brightness>>>,
    /// How long a change of level takes ([`FADE`]).
    fade: Duration,
}

impl Owner {
    /// The physical connector is reachable only here.
    pub fn new(connector: Arc<dyn ScreenConnector + Send + Sync>) -> Self {
        Self {
            connector,
            links: Arc::default(),
            editing: Arc::default(),
            disabled: Arc::default(),
            reload: Arc::default(),
            idle: Arc::default(),
            levels: Arc::default(),
            fade: FADE,
        }
    }

    /// The same owner with every change of level lasting `fade`
    /// (`Duration::ZERO`: the level jumps).
    #[must_use]
    pub fn with_fade(mut self, fade: Duration) -> Self {
        self.fade = fade;
        self
    }

    /// Remembers `level` as the one chosen for the screen at `address`
    /// (the settings' key, either port of the screen): it is sent when the
    /// screen opens and restored when idle ends.
    pub fn remember_level(&self, address: &str, level: Brightness) {
        if let Ok(mut levels) = self.levels.lock() {
            levels.insert(address.to_lowercase(), level);
        }
    }

    /// The level chosen for the screen with `addresses`, if any.
    fn level_of(&self, addresses: &[String]) -> Option<Brightness> {
        let levels = self.levels.lock().ok()?;
        addresses.iter().find_map(|a| levels.get(a).copied())
    }

    /// Remembers `level` under every port of a screen.
    fn record(&self, addresses: &[String], level: Brightness) {
        if let Ok(mut levels) = self.levels.lock() {
            for address in addresses {
                levels.insert(address.clone(), level);
            }
        }
    }

    /// Enters (`true`) or leaves (`false`) idle while the computer's
    /// monitors are off (D-2026-11-06-display-idle-saver-1): every link
    /// kept open fades to [`IDLE_BRIGHTNESS`], then back to the level chosen
    /// for its screen ([`Self::remember_level`] and every level set since),
    /// else to `fallback` of its screen (the start level a rev C record
    /// keeps, or [`Brightness::MAX`]). The screens fade together. Closed
    /// links and screens put to sleep are never opened; asking for the
    /// state already in place sends nothing. Levels set while idle are only
    /// remembered for the restore. The first error is returned after every
    /// screen was tried.
    pub fn idle(
        &self,
        on: bool,
        fallback: &(dyn Fn(&ScreenIdentity) -> Brightness + Sync),
    ) -> Result<()> {
        if self.idle.swap(on, Ordering::SeqCst) == on {
            return Ok(());
        }
        let links: Vec<_> = lock(&self.links)?.values().cloned().collect();
        let results: Vec<Result<()>> = std::thread::scope(|scope| {
            let fades: Vec<_> = links
                .iter()
                .map(|kept| scope.spawn(move || self.idle_one(kept, on, fallback)))
                .collect();
            fades
                .into_iter()
                .map(|fade| fade.join().unwrap_or_else(|_| Err(gone())))
                .collect()
        });
        results.into_iter().find(Result::is_err).unwrap_or(Ok(()))
    }

    /// [`Self::idle`] on one kept link.
    fn idle_one(
        &self,
        kept: &Mutex<Kept>,
        on: bool,
        fallback: &(dyn Fn(&ScreenIdentity) -> Brightness + Sync),
    ) -> Result<()> {
        let target = {
            let k = lock(kept)?;
            if k.failed || !k.link.identity().model.capabilities.brightness {
                return Ok(());
            }
            if on {
                IDLE_BRIGHTNESS
            } else {
                self.level_of(&k.addresses)
                    .unwrap_or_else(|| fallback(k.link.identity()))
            }
        };
        // A later change of idle takes over this fade.
        let idle = Arc::clone(&self.idle);
        self.fade_to(kept, target, &move |_| idle.load(Ordering::SeqCst) == on)
    }

    /// Takes the screen behind `kept` from the level it shows to `to` in
    /// [`Self::with_fade`]'s time ([`fade_level`], one level every
    /// [`FADE_STEP`]), the link locked only to send each one, so that
    /// frames go on meanwhile. Jumps when the level shown is unknown.
    /// Stops, without an error, as soon as a newer fade starts on the link,
    /// the link fails or `still` says no (checked under the link's lock,
    /// also before starting). Sends `to` at least once.
    fn fade_to(
        &self,
        kept: &Mutex<Kept>,
        to: Brightness,
        still: &dyn Fn(&Kept) -> bool,
    ) -> Result<()> {
        let (from, generation) = {
            let mut k = lock(kept)?;
            if k.failed || !still(&k) {
                return Ok(());
            }
            k.fading += 1;
            (k.shown.unwrap_or(to), k.fading)
        };
        let start = Instant::now();
        let mut sent = false;
        loop {
            let level = fade_level(from, to, start.elapsed(), self.fade);
            {
                let mut k = lock(kept)?;
                if k.fading != generation
                    || k.failed
                    || !still(&k)
                    || self.disabled.load(Ordering::SeqCst)
                {
                    return Ok(());
                }
                if k.shown != Some(level) || (level == to && !sent) {
                    k.call(|k| k.link.set_brightness(level))?;
                    k.shown = Some(level);
                    sent = true;
                }
            }
            if level == to {
                return Ok(());
            }
            std::thread::sleep(FADE_STEP);
        }
    }

    /// Whether the screens are dimmed for idle ([`Owner::idle`]).
    pub fn is_idle(&self) -> bool {
        self.idle.load(Ordering::SeqCst)
    }

    /// Suspend/resume saved-theme device writes; renderer state stays local.
    pub fn set_editing(&self, editing: bool) {
        if editing
            && !self.editing.load(Ordering::SeqCst)
            && let Ok(links) = self.links.lock()
        {
            for kept in links.values() {
                if let Ok(mut kept) = kept.lock() {
                    kept.edited = false;
                }
            }
        }
        if !editing && self.editing.load(Ordering::SeqCst) {
            self.reload.store(true, Ordering::SeqCst);
        }
        self.editing.store(editing, Ordering::SeqCst);
    }

    /// Whether the editor currently supplies the screen's frames.
    pub fn editing(&self) -> bool {
        self.editing.load(Ordering::SeqCst)
    }

    /// The saved runtime must reload settings after an editor session ends.
    pub fn reload_pending(&self) -> bool {
        self.reload.load(Ordering::SeqCst)
    }
    /// Called only after the old saved runtime has stopped.
    pub fn saved_ready(&self) {
        self.reload.store(false, Ordering::SeqCst);
    }

    /// Legacy Studio owns the screen lock: reject new proxy connections.
    pub fn set_enabled(&self, enabled: bool) {
        self.disabled.store(!enabled, Ordering::SeqCst);
    }

    /// Close physical links before releasing the legacy screen lock.
    pub fn clear(&self) -> Result<()> {
        let mut links = lock(&self.links)?;
        for kept in links.values() {
            let mut kept = lock(kept)?;
            kept.invalidate();
        }
        links.clear();
        Ok(())
    }

    /// Finish through retained physical links without opening any screen.
    /// New proxy operations are refused before final shutdown/explicit Quit.
    pub fn finish_until(
        &self,
        deadline: std::time::Instant,
        operation: &mut dyn FnMut(&mut dyn ScreenLink) -> Result<()>,
    ) -> Result<()> {
        self.set_enabled(false);
        let links: Vec<_> = lock(&self.links)?.values().cloned().collect();
        let mut first_error = None;
        for link in links {
            loop {
                if std::time::Instant::now() >= deadline {
                    return Err(BezelError::Timeout("screen owner shutdown".into()));
                }
                match link.try_lock() {
                    Ok(mut kept) => {
                        if !kept.failed {
                            if let Err(error) = operation(kept.link.as_mut()) {
                                first_error.get_or_insert(error);
                            }
                            kept.video = None;
                        }
                        break;
                    }
                    Err(std::sync::TryLockError::WouldBlock) => {
                        std::thread::sleep(std::time::Duration::from_millis(10))
                    }
                    Err(_) => return Err(gone()),
                }
            }
        }
        first_error.map_or(Ok(()), Err)
    }

    /// A connector for the editor (true) or saved-theme runtime (false).
    pub fn client(&self, editor: bool) -> OwnerConnector {
        OwnerConnector {
            owner: self.clone(),
            editor,
        }
    }
}

/// In-process adapter of ScreenConnector, also used by the pipe dispatcher.
#[derive(Clone)]
pub struct OwnerConnector {
    owner: Owner,
    editor: bool,
}

impl ScreenConnector for OwnerConnector {
    fn manages_shutdown(&self) -> bool {
        true
    }
    fn connect(&self, screen: &Screen) -> Result<Box<dyn ScreenLink>> {
        if self.owner.disabled.load(Ordering::SeqCst) {
            return Err(BezelError::Transport(
                "legacy Studio owns the screens".into(),
            ));
        }
        let key = screen
            .wake
            .as_ref()
            .or(screen.display.as_ref())
            .ok_or_else(|| BezelError::InvalidInput("screen has no endpoint".into()))?
            .address
            .0
            .to_lowercase();
        let mut links = lock(&self.owner.links)?;
        if self.owner.disabled.load(Ordering::SeqCst) {
            return Err(gone());
        }
        let existing = links
            .get(&key)
            .filter(|link| lock(link).is_ok_and(|k| !k.failed));
        let seed_allowed = existing.is_none();
        let kept = if let Some(link) = existing {
            Arc::clone(link)
        } else {
            let addresses: Vec<String> = [&screen.display, &screen.wake]
                .into_iter()
                .flatten()
                .map(|endpoint| endpoint.address.0.to_lowercase())
                .collect();
            let mut kept = Kept {
                link: self.owner.connector.connect(screen)?,
                orientation: None,
                video: None,
                edited: false,
                failed: false,
                addresses,
                shown: None,
                fading: 0,
            };
            // A screen opened (again: after a wake, a restart of Light) gets
            // its level at once: the idle one while idle, else the one
            // chosen for it. Without one it keeps the level it has.
            let level = if self.owner.is_idle() {
                Some(IDLE_BRIGHTNESS)
            } else {
                self.owner.level_of(&kept.addresses)
            };
            if let Some(level) = level
                && kept.link.identity().model.capabilities.brightness
            {
                match kept.call(|k| k.link.set_brightness(level)) {
                    Ok(()) => kept.shown = Some(level),
                    Err(error) => tracing::warn!(%error, "level not sent at the opening"),
                }
            }
            let link = Arc::new(Mutex::new(kept));
            links.insert(key, Arc::clone(&link));
            link
        };
        let identity = lock(&kept)?.link.identity().clone();
        Ok(Box::new(OwnedLink {
            kept,
            identity,
            owner: self.owner.clone(),
            editor: self.editor,
            desired_orientation: None,
            desired_video: None,
            seed_allowed,
        }))
    }

    fn restart(&self, screen: &Screen) -> Result<()> {
        // Only this screen's aliases are invalidated before touching its MCU.
        let key = screen
            .wake
            .as_ref()
            .or(screen.display.as_ref())
            .ok_or_else(gone)?
            .address
            .0
            .to_lowercase();
        let mut links = lock(&self.owner.links)?;
        if self.owner.disabled.load(Ordering::SeqCst) {
            return Err(gone());
        }
        if let Some(kept) = links.remove(&key) {
            lock(&kept)?.invalidate();
        }
        // Keep the connection registry locked until the MCU restart finishes:
        // no worker may open the same screen halfway through its restart.
        self.owner.connector.restart(screen)
    }
}

struct OwnedLink {
    kept: Arc<Mutex<Kept>>,
    identity: ScreenIdentity,
    owner: Owner,
    editor: bool,
    desired_orientation: Option<Orientation>,
    desired_video: Option<(RemotePath, Repeat)>,
    seed_allowed: bool,
}

impl OwnedLink {
    fn claim_editor(&self) -> Result<()> {
        if self.editor {
            lock(&self.kept)?.edited = true;
        }
        Ok(())
    }
    fn paused(&self) -> bool {
        self.owner.reload_pending()
            || (self.owner.editing() && lock(&self.kept).map_or(true, |k| k.edited))
    }
    fn allowed(&self) -> bool {
        !self.owner.disabled.load(Ordering::SeqCst)
            && (self.editor || !self.paused() || crate::session_ending())
    }
    fn write(&mut self, op: impl FnOnce(&mut Kept) -> Result<()>) -> Result<()> {
        if !self.allowed() {
            return Ok(());
        }
        let mut kept = lock(&self.kept)?;
        // Recheck after acquiring the physical-link lock: a frame may have
        // queued before shutdown, a legacy takeover or an editor claim.
        if self.owner.disabled.load(Ordering::SeqCst) {
            return Err(gone());
        }
        if !self.editor
            && !crate::session_ending()
            && (self.owner.reload_pending() || (self.owner.editing() && kept.edited))
        {
            return Ok(());
        }
        kept.call(op)
    }
    fn storage_call<T>(
        &mut self,
        op: impl FnOnce(&mut dyn ScreenStorage) -> Result<T>,
    ) -> Result<T> {
        let mut kept = lock(&self.kept)?;
        if self.owner.disabled.load(Ordering::SeqCst) {
            return Err(gone());
        }
        kept.call(|kept| {
            let storage = kept
                .link
                .storage()
                .ok_or_else(|| BezelError::Unsupported("screen storage".into()))?;
            op(storage)
        })
    }
}

impl ScreenLink for OwnedLink {
    fn manages_shutdown(&self) -> bool {
        true
    }
    fn presentation_paused(&self) -> bool {
        !self.editor && self.paused()
    }
    fn identity(&self) -> &ScreenIdentity {
        &self.identity
    }
    fn last_transfer(&self) -> Option<TransferStats> {
        lock(&self.kept).ok()?.link.last_transfer()
    }
    fn set_brightness(&mut self, value: Brightness) -> Result<()> {
        if !self.allowed() {
            return Ok(());
        }
        if self.owner.disabled.load(Ordering::SeqCst) {
            return Err(gone());
        }
        let addresses = lock(&self.kept)?.addresses.clone();
        self.owner.record(&addresses, value);
        // Checked under the link's lock, like `Owner::idle` fades it: while
        // idle the level waits for the restore, and a paused runtime sends
        // nothing (as in `write`).
        let owner = self.owner.clone();
        let editor = self.editor;
        self.owner.fade_to(&self.kept, value, &move |k| {
            !owner.is_idle()
                && (editor
                    || crate::session_ending()
                    || !(owner.reload_pending() || (owner.editing() && k.edited)))
        })
    }
    fn set_orientation(&mut self, value: Orientation) -> Result<()> {
        self.claim_editor()?;
        self.desired_orientation = Some(value);
        self.write(|k| {
            if k.orientation != Some(value) {
                k.link.set_orientation(value)?;
                k.orientation = Some(value);
            }
            Ok(())
        })
    }
    fn present(&mut self, frame: &Frame) -> Result<()> {
        self.seed_allowed = false;
        self.claim_editor()?;
        if !self.editor && self.allowed() {
            if let Some(orientation) = self.desired_orientation {
                self.set_orientation(orientation)?;
            }
            match self.desired_video.clone() {
                Some((path, repeat)) => self.resume_video(&path, repeat)?,
                None if lock(&self.kept)?.video.is_some() => self.stop()?,
                None => {}
            }
        }
        self.write(|k| k.link.present(frame))
    }
    fn screen_off(&mut self) -> Result<()> {
        self.claim_editor()?;
        self.write(|k| {
            k.video = None;
            k.link.screen_off()?;
            k.invalidate();
            Ok(())
        })
    }
    fn turn_off_now(&mut self) -> Result<()> {
        self.claim_editor()?;
        self.write(|kept| {
            kept.video = None;
            kept.link.turn_off_now()
        })
    }

    fn release(&mut self) -> Result<()> {
        self.claim_editor()?;
        // Light's worker exits can reload the saved theme without releasing
        // the screen; only an explicit editor Return releases it.
        if !self.editor {
            return Ok(());
        }
        self.write(|k| {
            k.video = None;
            k.link.release()
        })
    }
    fn baseline(&self) -> Option<bezel_core::domain::handoff::Baseline> {
        lock(&self.kept).ok()?.link.baseline()
    }
    fn seed_baseline(&mut self, baseline: &bezel_core::domain::handoff::Baseline) -> bool {
        let allowed = self.seed_allowed && self.allowed();
        self.seed_allowed = false;
        allowed && lock(&self.kept).is_ok_and(|mut k| k.link.seed_baseline(baseline))
    }
    fn storage(&mut self) -> Option<&mut dyn ScreenStorage> {
        let supported = lock(&self.kept).ok()?.link.storage().is_some();
        supported.then_some(self)
    }
}

impl ScreenStorage for OwnedLink {
    fn info(&mut self) -> Result<StorageInfo> {
        self.storage_call(|s| s.info())
    }
    fn list(&mut self, location: StorageLocation) -> Result<Vec<FileName>> {
        self.storage_call(|s| s.list(location))
    }
    fn size(&mut self, path: &RemotePath) -> Result<Option<u64>> {
        self.storage_call(|s| s.size(path))
    }
    fn upload(&mut self, path: &RemotePath, data: &[u8], job: &mut Job<'_>) -> Result<()> {
        let was_edited = lock(&self.kept)?.edited;
        self.claim_editor()?;
        lock(&self.kept)?.video = None;
        let result = self.storage_call(|s| s.upload(path, data, job));
        if self.editor {
            lock(&self.kept)?.edited = was_edited;
        }
        result
    }
    fn delete(&mut self, path: &RemotePath, confirmed: Confirmed) -> Result<()> {
        {
            let mut kept = lock(&self.kept)?;
            if kept
                .video
                .as_ref()
                .is_some_and(|(playing, _)| playing == path)
            {
                kept.video = None;
            }
        }
        self.storage_call(|s| s.delete(path, confirmed))
    }
    fn play_video(&mut self, path: &RemotePath, repeat: Repeat) -> Result<()> {
        self.claim_editor()?;
        self.desired_video = Some((path.clone(), repeat));
        self.write(|kept| {
            kept.video = None;
            kept.link
                .storage()
                .ok_or_else(|| BezelError::Unsupported("screen storage".into()))?
                .play_video(path, repeat)?;
            kept.video = Some((path.clone(), repeat));
            Ok(())
        })
    }
    fn resume_video(&mut self, path: &RemotePath, repeat: Repeat) -> Result<()> {
        self.claim_editor()?;
        self.desired_video = Some((path.clone(), repeat));
        if !self.allowed() {
            return Ok(());
        }
        if lock(&self.kept)?.video.as_ref() == Some(&(path.clone(), repeat)) {
            return Ok(());
        }
        self.play_video(path, repeat)
    }
    fn play_image(&mut self, path: &RemotePath) -> Result<()> {
        self.claim_editor()?;
        lock(&self.kept)?.video = None;
        self.storage_call(|s| s.play_image(path))
    }
    fn stop(&mut self) -> Result<()> {
        self.claim_editor()?;
        self.desired_video = None;
        if !self.allowed() {
            return Ok(());
        }
        lock(&self.kept)?.video = None;
        self.storage_call(|s| s.stop())
    }
    fn set_options(&mut self, plan: PlanB, confirmed: Confirmed) -> Result<()> {
        self.storage_call(|s| s.set_options(plan, confirmed))
    }
    fn restart(&mut self, confirmed: Confirmed) -> Result<()> {
        lock(&self.kept)?.video = None;
        self.storage_call(|s| s.restart(confirmed))
    }
}

struct Disconnected(ScreenIdentity);
impl ScreenLink for Disconnected {
    fn identity(&self) -> &ScreenIdentity {
        &self.0
    }
    fn set_brightness(&mut self, _: Brightness) -> Result<()> {
        Err(gone())
    }
    fn set_orientation(&mut self, _: Orientation) -> Result<()> {
        Err(gone())
    }
    fn present(&mut self, _: &Frame) -> Result<()> {
        Err(gone())
    }
    fn screen_off(&mut self) -> Result<()> {
        Err(gone())
    }
    fn release(&mut self) -> Result<()> {
        Err(gone())
    }
}
fn gone() -> BezelError {
    BezelError::Transport("screen owner link was closed".into())
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use bezel_core::app::discover_screens;
    use bezel_core::domain::{frame::Rgba, geometry::Size};
    use bezel_devices::fake::FakeStorage;
    use bezel_devices::{FakeBus, FakeConnector};

    #[test]
    fn editor_and_saved_runtime_share_one_link_without_release_or_video_restart() {
        let path = RemotePath::parse("internal/video/earth.mp4").unwrap();
        let physical =
            FakeConnector::with_storage(FakeStorage::default().with_file(path.clone(), vec![1]));
        let owner = Owner::new(Arc::new(physical.clone()));
        let screen = discover_screens(&FakeBus::turing_88()).unwrap().remove(0);
        let mut saved = owner.client(false).connect(&screen).unwrap();
        let path = RemotePath::parse("internal/video/earth.mp4").unwrap();
        saved
            .storage()
            .unwrap()
            .resume_video(&path, Repeat::Loop)
            .unwrap();
        let frame = Frame::filled(Size::new(480, 1920), Rgba::BLACK);
        saved.present(&frame).unwrap();
        owner.set_editing(true);
        let mut editor = owner.client(true).connect(&screen).unwrap();
        editor
            .storage()
            .unwrap()
            .resume_video(&path, Repeat::Loop)
            .unwrap();
        editor.present(&frame).unwrap();
        saved.present(&frame).unwrap(); // suppressed while editing
        drop(editor); // also what an EOF/crashed client does
        owner.set_editing(false);
        owner.saved_ready();
        saved.present(&frame).unwrap();
        saved.release().unwrap(); // saved worker reload must not release
        let log = physical.log();
        assert_eq!(log.connects, 1);
        assert_eq!(log.releases, 0);
        assert_eq!(log.frames.len(), 3);
        assert_eq!(
            log.storage
                .calls
                .iter()
                .filter(|call| matches!(call, bezel_devices::fake::StorageCall::PlayVideo(_, _)))
                .count(),
            1
        );
        assert!(
            !log.storage
                .calls
                .contains(&bezel_devices::fake::StorageCall::Stop)
        );
    }
}
