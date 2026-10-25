//! Nonblocking client of the separately installed, privileged FPS collector.
use super::{
    FrameMetrics, FrameRateSource, MAX_AGE,
    presentmon::{FrameData, PresentMon, Target, metrics},
    rtss::Rtss,
    service_api::{Batch, Client},
};
use std::{
    collections::VecDeque,
    sync::{Arc, Condvar, Mutex},
    thread::JoinHandle,
    time::{Duration, Instant},
};

const QUEUE: usize = 8192;
const HISTORY: usize = 240_000;
trait Connection: Send {
    fn consume(&mut self) -> Result<Batch, String>;
}
impl Connection for Client {
    fn consume(&mut self) -> Result<Batch, String> {
        Client::consume(self)
    }
}
type Connect = Box<dyn Fn(Target) -> Result<Box<dyn Connection>, String> + Send>;
#[derive(Default)]
struct State {
    stop: bool,
    generation: u64,
    desired: Option<Target>,
    frames: VecDeque<FrameData>,
    error: Option<String>,
    overflow: bool,
}
#[derive(Default)]
struct Shared {
    state: Mutex<State>,
    wake: Condvar,
}
struct Worker {
    shared: Arc<Shared>,
    thread: Option<JoinHandle<()>>,
}
impl Worker {
    fn new(connect: Connect) -> Result<Self, String> {
        let shared = Arc::new(Shared::default());
        let worker = shared.clone();
        let thread = std::thread::Builder::new()
            .name("bezel-fps-service".into())
            .spawn(move || {
                let mut generation = 0;
                let mut client: Option<Box<dyn Connection>> = None;
                let mut retry = Instant::now();
                let mut backoff = Duration::from_secs(2);
                loop {
                    let (next_generation, target) = {
                        let mut state = worker.state.lock().unwrap_or_else(|p| p.into_inner());
                        while !state.stop && state.desired.is_none() {
                            drop(state);
                            drop(client.take());
                            state = worker.state.lock().unwrap_or_else(|p| p.into_inner());
                            if state.stop || state.desired.is_some() {
                                break;
                            }
                            state = worker.wake.wait(state).unwrap_or_else(|p| p.into_inner());
                        }
                        if state.stop {
                            break;
                        }
                        (state.generation, state.desired.clone())
                    };
                    if generation != next_generation {
                        drop(client.take());
                        generation = next_generation;
                        retry = Instant::now();
                        backoff = Duration::from_secs(2);
                    }
                    let mut result = None;
                    if client.is_none()
                        && Instant::now() >= retry
                        && let Some(target) = target
                    {
                        match connect(target) {
                            Ok(opened) => {
                                client = Some(opened);
                            }
                            Err(reason) => {
                                result = Some(Err(reason));
                            }
                        }
                    }
                    if let Some(active) = &mut client {
                        result = Some(active.consume());
                    }
                    let mut failed = false;
                    if let Some(result) = result {
                        let mut state = worker.state.lock().unwrap_or_else(|p| p.into_inner());
                        if state.generation == generation && !state.stop {
                            match result {
                                Ok(batch) => {
                                    if !batch.frames.is_empty() {
                                        backoff = Duration::from_secs(2);
                                    }
                                    state.error = None;
                                    state.overflow |= batch.saturated;
                                    for frame in batch.frames {
                                        if state.frames.len() == QUEUE {
                                            state.frames.pop_front();
                                            state.overflow = true;
                                        }
                                        state.frames.push_back(frame);
                                    }
                                }
                                Err(reason) => {
                                    state.error = Some(reason);
                                    failed = true;
                                    retry = Instant::now() + backoff;
                                    backoff = (backoff * 2).min(Duration::from_secs(60));
                                }
                            }
                        }
                    }
                    if failed {
                        drop(client.take());
                    }
                    let state = worker.state.lock().unwrap_or_else(|p| p.into_inner());
                    if state.stop {
                        break;
                    }
                    if state.generation == generation {
                        drop(
                            worker
                                .wake
                                .wait_timeout(state, Duration::from_millis(50))
                                .unwrap_or_else(|p| p.into_inner()),
                        );
                    }
                }
                // Client RAII stops only this session's tracking; the shared service
                // and other applications' sessions remain owned by Windows.
            })
            .map_err(|error| format!("Cannot start FPS reader: {error}"))?;
        Ok(Self {
            shared,
            thread: Some(thread),
        })
    }
    fn target(&self, target: Option<Target>) {
        let mut state = self.shared.state.lock().unwrap_or_else(|p| p.into_inner());
        if state.desired.as_ref().map(|t| t.pid) == target.as_ref().map(|t| t.pid) {
            return;
        }
        state.desired = target;
        state.generation = state.generation.wrapping_add(1);
        state.frames.clear();
        state.error = None;
        state.overflow = false;
        self.shared.wake.notify_all();
    }
    fn drain(&self) -> (Vec<FrameData>, Option<String>, bool) {
        let mut state = self.shared.state.lock().unwrap_or_else(|p| p.into_inner());
        (
            state.frames.drain(..).collect(),
            state.error.clone(),
            std::mem::take(&mut state.overflow),
        )
    }
}
impl Drop for Worker {
    fn drop(&mut self) {
        self.shared
            .state
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .stop = true;
        self.shared.wake.notify_all();
        // A vendor call must not hang the sampler/application shutdown. A call
        // already in progress closes its client when it returns; never spawn
        // another thread per poll or kill the shared Windows service.
        if let Some(thread) = self.thread.take()
            && thread.is_finished()
        {
            let _ = thread.join();
        }
    }
}

pub(super) struct Service {
    worker: Option<Worker>,
    wanted: bool,
    target: Option<Target>,
    foreground: Option<u32>,
    frames: VecDeque<FrameData>,
    last: Option<Instant>,
    started: Instant,
    idle_until: Option<Instant>,
    incomplete_until: Option<Instant>,
    rtss: Rtss,
    fallback: bool,
}
impl Service {
    pub fn new() -> Self {
        Self {
            worker: None,
            wanted: false,
            target: None,
            foreground: None,
            frames: VecDeque::new(),
            last: None,
            started: Instant::now(),
            idle_until: None,
            incomplete_until: None,
            rtss: Rtss,
            fallback: false,
        }
    }
    fn stop(&mut self) {
        if let Some(worker) = &self.worker {
            worker.target(None);
        }
        self.target = None;
        self.frames.clear();
        self.last = None;
        self.incomplete_until = None;
    }
    fn read_target(
        &mut self,
        foreground: Option<Target>,
        now: Instant,
    ) -> Result<FrameMetrics, String> {
        if !self.wanted {
            return Err("FPS capture is not requested".into());
        }
        let pid = foreground.as_ref().map(|target| target.pid);
        let returned = pid.is_some() && pid != self.foreground;
        self.foreground = pid;
        if returned && self.idle_until.take().is_some() {
            self.started = now;
        }
        if let (Some(current), Some(next)) = (&self.target, &foreground)
            && current.pid != next.pid
        {
            self.stop();
            self.idle_until = None;
        }
        if self.target.is_none() {
            if self.idle_until.is_some_and(|until| now < until) {
                return Err("fps.stale:Waiting to retry an idle game".into());
            }
            let Some(target) = foreground else {
                return Err("fps.stale:Bring a game to the foreground".into());
            };
            if self.worker.is_none() {
                self.worker = Some(Worker::new(Box::new(|target| {
                    Client::open(target).map(|client| Box::new(client) as Box<dyn Connection>)
                }))?);
            }
            if let Some(worker) = &self.worker {
                worker.target(Some(target.clone()));
            }
            self.target = Some(target);
            self.started = now;
        } else if returned
            && self
                .last
                .is_none_or(|last| now.saturating_duration_since(last) > MAX_AGE)
        {
            self.last = None;
            self.started = now;
        }
        let (incoming, error, overflow) = self.worker.as_ref().ok_or("Missing FPS worker")?.drain();
        let pid = self.target.as_ref().ok_or("Missing FPS target")?.pid;
        if overflow {
            self.frames.clear();
            self.incomplete_until = Some(now + Duration::from_secs(30));
        }
        for frame in incoming {
            if frame.process_id != pid
                || frame.timestamp > now
                || now.duration_since(frame.timestamp) > Duration::from_secs(30)
            {
                continue;
            }
            self.last = Some(
                self.last
                    .map_or(frame.timestamp, |last| last.max(frame.timestamp)),
            );
            if self.frames.len() == HISTORY {
                self.frames.pop_front();
                self.incomplete_until = Some(now + Duration::from_secs(30));
            }
            self.frames.push_back(frame);
        }
        self.frames.retain(|frame| {
            now.saturating_duration_since(frame.timestamp) <= Duration::from_secs(30)
        });
        if let Some(reason) = error {
            return Err(reason);
        }
        let grace = if self.last.is_some() {
            Duration::from_secs(60)
        } else {
            Duration::from_secs(10)
        };
        if now.saturating_duration_since(self.last.unwrap_or(self.started)) > grace {
            self.stop();
            self.idle_until = Some(now + Duration::from_secs(60));
            return Err("fps.stale:No recent game frames; tracking paused".into());
        }
        let mut result = metrics(&self.frames, pid, now)?;
        if self.incomplete_until.is_some_and(|until| now < until) {
            result.fps_low_1p = None;
            result.fps_low_01p = None;
            result.fps_min = None;
            result.fps_max = None;
        }
        Ok(result)
    }
}
impl FrameRateSource for Service {
    fn describe(&self) -> String {
        if self.fallback {
            "RivaTuner Statistics Server (PresentMon service fallback)".into()
        } else {
            "Intel PresentMon service API (on demand)".into()
        }
    }
    fn want(&mut self, wanted: bool) {
        if wanted == self.wanted {
            return;
        }
        self.wanted = wanted;
        self.stop();
        self.foreground = None;
        self.idle_until = None;
        self.fallback = false;
    }
    fn read(&mut self) -> Result<FrameMetrics, String> {
        match self.read_target(PresentMon::foreground_target(), Instant::now()) {
            Ok(metrics) => {
                self.fallback = false;
                Ok(metrics)
            }
            Err(reason) => {
                self.fallback = true;
                self.rtss
                    .read()
                    .map_err(|fallback| format!("{reason}; RTSS fallback: {fallback}"))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    struct Fake {
        target: Target,
        dropped: Arc<AtomicUsize>,
    }
    impl Drop for Fake {
        fn drop(&mut self) {
            self.dropped.fetch_add(1, Ordering::SeqCst);
        }
    }
    impl Connection for Fake {
        fn consume(&mut self) -> Result<Batch, String> {
            Ok(Batch {
                frames: vec![FrameData {
                    timestamp: Instant::now(),
                    process_name: self.target.name.clone(),
                    process_id: self.target.pid,
                    swap_chain: 1,
                    qpc_ms: None,
                    frametime_ms: 10.0,
                    cpu_busy: None,
                    cpu_wait: None,
                    gpu_busy: None,
                    gpu_wait: None,
                }],
                saturated: false,
            })
        }
    }
    #[test]
    fn a_blocked_vendor_call_cannot_block_control_or_shutdown() {
        use std::sync::mpsc;
        struct Blocked {
            entered: mpsc::Sender<()>,
            release: mpsc::Receiver<()>,
            dropped: Arc<AtomicUsize>,
        }
        impl Connection for Blocked {
            fn consume(&mut self) -> Result<Batch, String> {
                let _ = self.entered.send(());
                let _ = self.release.recv_timeout(Duration::from_secs(3));
                Ok(Batch {
                    frames: Vec::new(),
                    saturated: false,
                })
            }
        }
        impl Drop for Blocked {
            fn drop(&mut self) {
                self.dropped.fetch_add(1, Ordering::SeqCst);
            }
        }
        let (entered, waiting) = mpsc::channel();
        let (release, paused) = mpsc::channel();
        let dropped = Arc::new(AtomicUsize::new(0));
        let observed = dropped.clone();
        let paused = Mutex::new(Some(paused));
        let worker = Worker::new(Box::new(move |_| {
            Ok(Box::new(Blocked {
                entered: entered.clone(),
                release: paused.lock().unwrap().take().unwrap(),
                dropped: observed.clone(),
            }))
        }))
        .unwrap();
        worker.target(Some(Target {
            pid: 42,
            name: "game.exe".into(),
        }));
        waiting.recv_timeout(Duration::from_secs(3)).unwrap();
        let start = Instant::now();
        worker.target(None);
        assert!(worker.drain().0.is_empty());
        drop(worker);
        assert!(start.elapsed() < Duration::from_millis(500));
        release.send(()).unwrap();
        let deadline = Instant::now() + Duration::from_secs(3);
        while dropped.load(Ordering::SeqCst) == 0 {
            assert!(Instant::now() < deadline);
            std::thread::sleep(Duration::from_millis(2));
        }
    }

    #[test]
    fn an_unwanted_or_idle_source_does_not_open_an_api_connection() {
        let mut source = Service::new();
        assert!(source.read_target(None, Instant::now()).is_err());
        source.want(true);
        assert!(source.read_target(None, Instant::now()).is_err());
        assert!(source.worker.is_none());
        source.want(false);
        assert!(source.worker.is_none());
    }

    #[test]
    fn worker_changes_targets_and_releases_tracking_when_paused() {
        let dropped = Arc::new(AtomicUsize::new(0));
        let observe = dropped.clone();
        let worker = Worker::new(Box::new(move |target| {
            Ok(Box::new(Fake {
                target,
                dropped: observe.clone(),
            }))
        }))
        .unwrap();
        for pid in [42, 43] {
            worker.target(Some(Target {
                pid,
                name: "game.exe".into(),
            }));
            let deadline = Instant::now() + Duration::from_secs(3);
            loop {
                let (frames, error, _) = worker.drain();
                assert!(error.is_none());
                assert!(frames.iter().all(|frame| frame.process_id == pid));
                if !frames.is_empty() {
                    break;
                }
                assert!(Instant::now() < deadline);
                std::thread::sleep(Duration::from_millis(2));
            }
        }
        worker.target(None);
        let deadline = Instant::now() + Duration::from_secs(3);
        while dropped.load(Ordering::SeqCst) < 2 {
            assert!(Instant::now() < deadline);
            std::thread::sleep(Duration::from_millis(2));
        }
        assert!(worker.drain().0.is_empty());
    }
}
