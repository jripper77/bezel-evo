//! One blocking I/O worker per screen. Only its latest pending frame survives.
use bezel_core::{
    BezelError, Result,
    domain::{
        discovery::Screen,
        frame::Frame,
        geometry::Orientation,
        job::Job,
        screen::{Brightness, ScreenIdentity},
        standby::PlanB,
        storage::{Confirmed, FileName, RemotePath, Repeat, StorageInfo, StorageLocation},
    },
    ports::{DeviceBus, ScreenConnector, ScreenLink, ScreenStorage, TransferStats},
};
use bezel_power::diagnostics::{LightEvent, light};
use std::{
    collections::{BTreeMap, VecDeque},
    sync::{Arc, Condvar, Mutex},
    thread,
    time::{Duration, Instant},
};
const PENDING: &str = "asynchronous device query pending";
#[derive(Clone)]
enum Control {
    Orientation(Orientation),
    Size(RemotePath),
    Info,
    Resume(RemotePath, Repeat),
    Stop,
}
struct State {
    paused: bool,
    busy: bool,
    stop: bool,
    commands: VecDeque<Control>,
    frame: Option<Arc<Frame>>,
    opened: Option<(Screen, ScreenIdentity, bool)>,
    error: bool,
    completion: Option<TransferStats>,
    completed: u64,
    sizes: BTreeMap<RemotePath, Option<Result<Option<u64>>>>,
    info: Option<Result<StorageInfo>>,
    info_pending: bool,
}
struct Shared {
    state: Mutex<State>,
    wake: Condvar,
}
pub(crate) struct Presenter {
    shared: Arc<Shared>,
}
pub(crate) struct Link {
    shared: Arc<Shared>,
    identity: ScreenIdentity,
    storage: bool,
    pub pending: bool,
}
pub(crate) struct Update {
    pub opened: Option<(Screen, Link)>,
    pub error: bool,
    pub completed: u64,
}
impl Presenter {
    pub fn start(
        id: u64,
        target: crate::targets::Target,
        bus: Arc<dyn DeviceBus + Send + Sync>,
        connector: Arc<dyn ScreenConnector + Send + Sync>,
        handoff: Option<super::Handoff>,
        theme: bezel_core::domain::theme::Theme,
        wake: crate::Wake,
    ) -> Result<Self> {
        let shared = Arc::new(Shared {
            state: Mutex::new(State {
                paused: false,
                busy: false,
                stop: false,
                commands: VecDeque::new(),
                frame: None,
                opened: None,
                error: false,
                completion: None,
                completed: 0,
                sizes: BTreeMap::new(),
                info: None,
                info_pending: false,
            }),
            wake: Condvar::new(),
        });
        let worker = shared.clone();
        thread::Builder::new()
            .name(format!("bezel-light-present-{id}"))
            .spawn(move || {
                let mut previous: Option<Screen> = None;
                loop {
                    if worker
                        .state
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner)
                        .stop
                    {
                        return;
                    }
                    let opened = match &previous {
                        Some(screen) => {
                            bezel_core::app::reopen_screen(bus.as_ref(), connector.as_ref(), screen)
                        }
                        None => target
                            .resolve(
                                bezel_core::app::discover_screens(bus.as_ref()).unwrap_or_default(),
                            )
                            .and_then(|s| connector.connect(&s).map(|l| (s, l))),
                    };
                    let result = (|| -> Result<()> {
                        let (screen, mut link) = opened?;
                        if theme.misfit(link.identity().model.panel).is_some() {
                            return Err(BezelError::InvalidInput(
                                "theme does not fit screen".into(),
                            ));
                        }
                        let orientation = theme.orientation;
                        link.set_orientation(orientation)?;
                        target.remembered.remember(
                            &bezel_core::app::discover_screens(bus.as_ref()).unwrap_or_default(),
                        );
                        if previous.is_none()
                            && let Some((store, wall)) = &handoff
                            && let Some(address) = screen.address()
                        {
                            let _ = bezel_core::app::handoff::take_over(
                                link.as_mut(),
                                address,
                                orientation,
                                store.as_ref(),
                                wall(),
                            );
                        }
                        let identity = link.identity().clone();
                        let model = identity.model.id.0;
                        let storage = link.storage().is_some();
                        let mut state = worker
                            .state
                            .lock()
                            .unwrap_or_else(std::sync::PoisonError::into_inner);
                        state.sizes.clear();
                        state.info = None;
                        state.info_pending = false;
                        state.commands.clear();
                        state.frame = None;
                        state.opened = Some((screen.clone(), identity, storage));
                        drop(state);
                        bezel_power::diagnostics::device(
                            if previous.is_some() {
                                LightEvent::ScreenReconnected
                            } else {
                                LightEvent::ScreenOpened
                            },
                            id,
                            screen.address().map_or("other", |a| a.0.as_str()),
                            model,
                        );
                        previous = Some(screen);
                        wake.notify();
                        loop {
                            let mut state = worker
                                .state
                                .lock()
                                .unwrap_or_else(std::sync::PoisonError::into_inner);
                            while !state.stop
                                && (state.paused
                                    || (state.commands.is_empty() && state.frame.is_none()))
                            {
                                state = worker
                                    .wake
                                    .wait(state)
                                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                            }
                            if state.stop {
                                return Ok(());
                            }
                            let command = state.commands.pop_front();
                            let frame = if command.is_none() {
                                state.frame.take()
                            } else {
                                None
                            };
                            state.busy = true;
                            drop(state);
                            match command {
                                Some(Control::Orientation(o)) => link.set_orientation(o)?,
                                Some(Control::Info) => {
                                    let result = link
                                        .storage()
                                        .ok_or_else(|| BezelError::Unsupported("storage".into()))?
                                        .info();
                                    if result
                                        .as_ref()
                                        .is_err_and(|e| !matches!(e, BezelError::Unsupported(_)))
                                    {
                                        return result.map(|_| ());
                                    }
                                    let mut state = worker
                                        .state
                                        .lock()
                                        .unwrap_or_else(std::sync::PoisonError::into_inner);
                                    state.info = Some(result);
                                    state.info_pending = false;
                                }
                                Some(Control::Size(path)) => {
                                    let result = link
                                        .storage()
                                        .ok_or_else(|| BezelError::Unsupported("storage".into()))?
                                        .size(&path);
                                    if result
                                        .as_ref()
                                        .is_err_and(|e| !matches!(e, BezelError::Unsupported(_)))
                                    {
                                        return result.map(|_| ());
                                    }
                                    worker
                                        .state
                                        .lock()
                                        .unwrap_or_else(std::sync::PoisonError::into_inner)
                                        .sizes
                                        .insert(path, Some(result));
                                }
                                Some(Control::Resume(path, repeat)) => link
                                    .storage()
                                    .ok_or_else(|| BezelError::Unsupported("storage".into()))?
                                    .resume_video(&path, repeat)?,
                                Some(Control::Stop) => {
                                    if let Some(storage) = link.storage() {
                                        storage.stop()?;
                                    }
                                }
                                None => {
                                    if let Some(frame) = frame {
                                        let start = Instant::now();
                                        link.present(frame.as_ref())?;
                                        let stats = link.last_transfer().unwrap_or(TransferStats {
                                            kind: "unknown",
                                            ..TransferStats::default()
                                        });
                                        bezel_power::diagnostics::present_reason(
                                            id,
                                            stats.kind,
                                            stats.reason,
                                        );
                                        light(
                                            LightEvent::Present,
                                            id,
                                            start.elapsed().as_secs_f64() * 1000.0,
                                            stats.kind,
                                        );
                                        let mut state = worker
                                            .state
                                            .lock()
                                            .unwrap_or_else(std::sync::PoisonError::into_inner);
                                        state.completion = Some(stats);
                                        state.completed += 1;
                                    }
                                }
                            }
                            worker
                                .state
                                .lock()
                                .unwrap_or_else(std::sync::PoisonError::into_inner)
                                .busy = false;
                            wake.notify();
                        }
                    })();
                    if result.is_ok() {
                        return;
                    }
                    light(LightEvent::ScreenError, id, 0.0, "io");
                    let mut state = worker
                        .state
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner);
                    state.error = true;
                    wake.notify();
                    state.busy = false;
                    state.frame = None;
                    state.commands.clear();
                    state.sizes.clear();
                    state.info = None;
                    state.info_pending = false;
                    let deadline = Instant::now() + Duration::from_secs(2);
                    while !state.stop && Instant::now() < deadline {
                        state = worker
                            .wake
                            .wait_timeout(state, deadline.saturating_duration_since(Instant::now()))
                            .unwrap_or_else(std::sync::PoisonError::into_inner)
                            .0;
                    }
                    if state.stop {
                        return;
                    }
                }
            })
            .map_err(|e| BezelError::Transport(e.to_string()))?;
        Ok(Self { shared })
    }
    pub fn poll(&self) -> Update {
        let mut state = self
            .shared
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let opened = state.opened.take().map(|(screen, identity, storage)| {
            (
                screen,
                Link {
                    shared: self.shared.clone(),
                    identity,
                    storage,
                    pending: false,
                },
            )
        });
        Update {
            opened,
            error: std::mem::take(&mut state.error),
            completed: std::mem::take(&mut state.completed),
        }
    }
    pub fn idle(&self) -> bool {
        let state = self
            .shared
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        !state.busy && state.commands.is_empty() && state.frame.is_none() && state.opened.is_none()
    }
    pub fn pause(&self, on: bool) {
        let mut state = self
            .shared
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        state.paused = on;
        if on {
            state.frame = None;
        }
        drop(state);
        self.shared.wake.notify_one();
    }
    pub fn frame(&self, frame: Arc<Frame>) {
        self.shared
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .frame = Some(frame);
        self.shared.wake.notify_one();
    }
}
impl Drop for Presenter {
    fn drop(&mut self) {
        let mut state = self
            .shared
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        state.stop = true;
        state.frame = None;
        state.commands.clear();
        drop(state);
        self.shared.wake.notify_one();
    }
}
impl Link {
    fn enqueue(&mut self, command: Control) -> Result<()> {
        let mut state = self
            .shared
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if state.stop || state.error {
            return Err(BezelError::Transport("screen worker unavailable".into()));
        }
        if state.commands.len() >= 64 {
            return Err(BezelError::Transport("screen command queue full".into()));
        }
        state.commands.push_back(command);
        drop(state);
        self.shared.wake.notify_one();
        Ok(())
    }
}
fn unsupported<T>() -> Result<T> {
    Err(BezelError::Unsupported(
        "operation belongs to the control channel".into(),
    ))
}
impl ScreenLink for Link {
    fn identity(&self) -> &ScreenIdentity {
        &self.identity
    }
    fn set_orientation(&mut self, o: Orientation) -> Result<()> {
        self.enqueue(Control::Orientation(o))
    }
    fn set_brightness(&mut self, _: Brightness) -> Result<()> {
        unsupported()
    }
    fn present(&mut self, _: &Frame) -> Result<()> {
        unsupported()
    }
    fn screen_off(&mut self) -> Result<()> {
        unsupported()
    }
    fn release(&mut self) -> Result<()> {
        unsupported()
    }
    fn storage(&mut self) -> Option<&mut dyn ScreenStorage> {
        self.storage.then_some(self)
    }
}
impl ScreenStorage for Link {
    fn size(&mut self, path: &RemotePath) -> Result<Option<u64>> {
        let mut state = self
            .shared
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(Some(result)) = state.sizes.get(path) {
            return result.clone();
        }
        if !state.sizes.contains_key(path) {
            if state.sizes.len() >= 4096 || state.commands.len() >= 64 {
                return Err(BezelError::InvalidInput("video query limit".into()));
            }
            state.sizes.insert(path.clone(), None);
            state.commands.push_back(Control::Size(path.clone()));
            self.shared.wake.notify_one();
        }
        self.pending = true;
        Err(BezelError::Transport(PENDING.into()))
    }
    fn resume_video(&mut self, path: &RemotePath, repeat: Repeat) -> Result<()> {
        self.enqueue(Control::Resume(path.clone(), repeat))
    }
    fn play_video(&mut self, _: &RemotePath, _: Repeat) -> Result<()> {
        unsupported()
    }
    fn stop(&mut self) -> Result<()> {
        self.enqueue(Control::Stop)
    }
    fn info(&mut self) -> Result<StorageInfo> {
        let mut state = self
            .shared
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(result) = &state.info {
            return result.clone();
        }
        if !state.info_pending {
            if state.commands.len() >= 64 {
                return Err(BezelError::InvalidInput("video query limit".into()));
            }
            state.info_pending = true;
            state.commands.push_back(Control::Info);
            self.shared.wake.notify_one();
        }
        self.pending = true;
        Err(BezelError::Transport(PENDING.into()))
    }
    fn list(&mut self, _: StorageLocation) -> Result<Vec<FileName>> {
        unsupported()
    }
    fn upload(&mut self, _: &RemotePath, _: &[u8], _: &mut Job<'_>) -> Result<()> {
        unsupported()
    }
    fn delete(&mut self, _: &RemotePath, _: Confirmed) -> Result<()> {
        unsupported()
    }
    fn play_image(&mut self, _: &RemotePath) -> Result<()> {
        unsupported()
    }
    fn set_options(&mut self, _: PlanB, _: Confirmed) -> Result<()> {
        unsupported()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bezel_core::domain::{
        device::{ModelId, Transport, UsbId},
        discovery::{DeviceAddress, Endpoint},
        frame::Rgba,
        geometry::Size,
        theme::Theme,
    };
    use bezel_devices::{
        FakeBus,
        driver::{RealTime, turing_rev_a::TuringRevA},
        wire::ScriptedWire,
    };
    struct Connector;
    impl ScreenConnector for Connector {
        fn connect(&self, _: &Screen) -> Result<Box<dyn ScreenLink>> {
            let model = bezel_core::domain::catalog::model_by_id(ModelId("turing-3.5")).unwrap();
            Ok(Box::new(TuringRevA::connect(
                ScriptedWire::default(),
                &RealTime,
                &[model],
            )?))
        }
    }
    #[test]
    fn rev_a_worker_keeps_baseline_and_reports_actual_dirty_rectangles() {
        let bus = Arc::new(FakeBus::new(vec![Endpoint {
            address: DeviceAddress("COM3".into()),
            transport: Transport::Serial,
            usb: UsbId::new(0x1a86, 0x5722),
            serial_number: None,
            manufacturer: None,
            product: None,
            location: None,
        }]));
        let presenter = Presenter::start(
            1,
            crate::targets::Target {
                key: "COM3".into(),
                remembered: Arc::new(crate::targets::Targets::load(
                    std::env::temp_dir().join("bezel-rev-a-addresses.json"),
                )),
            },
            bus,
            Arc::new(Connector),
            None,
            Theme::blank("test", Size::new(320, 480), Orientation::Portrait),
            crate::Wake::default(),
        )
        .unwrap();
        let deadline = Instant::now() + Duration::from_secs(3);
        let mut link = loop {
            if let Some((_, link)) = presenter.poll().opened {
                break link;
            }
            assert!(Instant::now() < deadline);
            thread::sleep(Duration::from_millis(2));
        };
        let mut frame = Frame::filled(Size::new(320, 480), Rgba::BLACK);
        for expected in ["full", "unchanged", "partial"] {
            link.set_orientation(Orientation::Portrait).unwrap();
            if expected == "partial" {
                frame.fill_rect(
                    bezel_core::domain::frame::Rect::new(10, 10, 1, 1),
                    Rgba::WHITE,
                );
            }
            presenter.frame(Arc::new(frame.clone()));
            loop {
                let state = presenter.shared.state.lock().unwrap();
                if state.completed > 0 {
                    assert_eq!(state.completion.unwrap().kind, expected);
                    break;
                }
                drop(state);
                assert!(Instant::now() < deadline);
                thread::sleep(Duration::from_millis(2));
            }
            presenter.poll();
        }
    }
}
