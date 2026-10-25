//! One hardware reader independent of rendering and device I/O.
use bezel_core::{
    Result,
    domain::sensor::{SensorInfo, Snapshot, Wanted},
    ports::SensorSource,
};
use bezel_power::diagnostics::{LightEvent, light};
use std::{
    sync::{Arc, Condvar, Mutex},
    thread,
    time::{Duration, Instant},
};
#[derive(Clone)]
pub(crate) struct Reading {
    pub millis: f64,
    pub at: Duration,
    pub scheduled_at: Duration,
    pub sequence: u64,
    pub snapshot: Snapshot,
    pub catalog: Vec<SensorInfo>,
}
struct State {
    latest: Arc<Reading>,
    periods: Vec<(Duration, Duration)>,
    wanted: Wanted,
    replace: Option<Box<dyn SensorSource>>,
    stop: bool,
}
struct Shared {
    state: Mutex<State>,
    wake: Condvar,
}
pub(crate) struct Sampler {
    shared: Arc<Shared>,
    #[cfg(test)]
    manual: Option<Mutex<Box<dyn SensorSource>>>,
}
impl Sampler {
    pub fn new(
        mut source: Box<dyn SensorSource>,
        origin: Instant,
        wake: crate::Wake,
    ) -> Result<Self> {
        let catalog = source.catalog()?;
        let shared = Arc::new(Shared {
            state: Mutex::new(State {
                latest: Arc::new(Reading {
                    millis: 0.0,
                    at: Duration::ZERO,
                    scheduled_at: Duration::ZERO,
                    sequence: 0,
                    snapshot: Snapshot::default(),
                    catalog,
                }),
                periods: Vec::new(),
                wanted: Wanted::nothing(),
                replace: None,
                stop: false,
            }),
            wake: Condvar::new(),
        });
        let worker = shared.clone();
        thread::Builder::new()
            .name("bezel-light-sensors".into())
            .spawn(move || {
                let mut next = origin;
                let mut previous = None;
                let mut sequence = 0;
                let mut targets = Vec::new();
                loop {
                    let mut state = worker
                        .state
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner);
                    if state.stop {
                        break;
                    }
                    if let Some(replace) = state.replace.take() {
                        source = replace;
                        next = Instant::now();
                        light(LightEvent::SensorRestart, 0, 0.0, "configured");
                    }
                    if state.periods.is_empty() {
                        drop(
                            worker
                                .wake
                                .wait(state)
                                .unwrap_or_else(std::sync::PoisonError::into_inner),
                        );
                        continue;
                    }
                    if targets != state.periods {
                        targets = state.periods.clone();
                        if sequence > 0 {
                            next = origin + next_after(origin.elapsed(), &targets);
                        }
                    }
                    let now = Instant::now();
                    if now < next {
                        drop(
                            worker
                                .wake
                                .wait_timeout(state, next - now)
                                .unwrap_or_else(std::sync::PoisonError::into_inner),
                        );
                        continue;
                    }
                    let wanted = state.wanted.clone();
                    drop(state);
                    let scheduled = next.duration_since(origin);
                    let at = Instant::now();
                    let interval = previous.map_or(0.0, |p: Instant| {
                        at.duration_since(p).as_secs_f64() * 1000.0
                    });
                    previous = Some(at);
                    source.want(&wanted);
                    match source.sample() {
                        Ok(snapshot) => {
                            let catalog = source.catalog().unwrap_or_default();
                            sequence += 1;
                            let mut state = worker
                                .state
                                .lock()
                                .unwrap_or_else(std::sync::PoisonError::into_inner);
                            state.latest = Arc::new(Reading {
                                millis: at.elapsed().as_secs_f64() * 1000.0,
                                at: at.duration_since(origin),
                                scheduled_at: scheduled,
                                sequence,
                                snapshot,
                                catalog,
                            });
                            drop(state);
                            wake.notify();
                            light(LightEvent::Sample, 0, interval, "interval");
                        }
                        Err(_) => light(LightEvent::SampleError, 0, interval, "reader"),
                    }
                    next = origin + next_after(origin.elapsed().max(scheduled), &targets);
                }
            })
            .map_err(|e| bezel_core::BezelError::Transport(e.to_string()))?;
        Ok(Self {
            shared,
            #[cfg(test)]
            manual: None,
        })
    }
    pub fn schedule(&self, periods: Vec<(Duration, Duration)>, wanted: Wanted) {
        let mut state = self
            .shared
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        state.periods = periods;
        state.wanted = wanted;
        drop(state);
        self.shared.wake.notify_one();
    }
    pub fn replace(&self, source: Box<dyn SensorSource>) {
        #[cfg(test)]
        if let Some(manual) = &self.manual {
            *manual.lock().unwrap() = source;
            return;
        }
        self.shared
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .replace = Some(source);
        self.shared.wake.notify_one();
    }
    pub fn latest(&self) -> Arc<Reading> {
        self.shared
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .latest
            .clone()
    }
    #[cfg(test)]
    pub fn manual(mut source: Box<dyn SensorSource>) -> Self {
        let catalog = source.catalog().unwrap();
        Self {
            shared: Arc::new(Shared {
                state: Mutex::new(State {
                    latest: Arc::new(Reading {
                        millis: 0.0,
                        at: Duration::ZERO,
                        scheduled_at: Duration::ZERO,
                        sequence: 0,
                        snapshot: Snapshot::default(),
                        catalog,
                    }),
                    periods: Vec::new(),
                    wanted: Wanted::nothing(),
                    replace: None,
                    stop: false,
                }),
                wake: Condvar::new(),
            }),
            manual: Some(Mutex::new(source)),
        }
    }
    #[cfg(test)]
    pub fn advance(&self, at: Duration) {
        let mut state = self.shared.state.lock().unwrap();
        if let Some(source) = &self.manual {
            let mut source = source.lock().unwrap();
            source.want(&state.wanted);
            let snapshot = source.sample().unwrap();
            state.latest = Arc::new(Reading {
                millis: 0.0,
                at,
                scheduled_at: at,
                sequence: state.latest.sequence + 1,
                snapshot,
                catalog: source.catalog().unwrap(),
            });
        }
    }
}
impl Drop for Sampler {
    fn drop(&mut self) {
        self.shared
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .stop = true;
        self.shared.wake.notify_one();
    }
}
pub(crate) struct Latest(pub Arc<Reading>);
impl SensorSource for Latest {
    fn catalog(&mut self) -> Result<Vec<SensorInfo>> {
        Ok(self.0.catalog.clone())
    }
    fn sample(&mut self) -> Result<Snapshot> {
        Ok(self.0.snapshot.clone())
    }
}

fn next_after(now: Duration, periods: &[(Duration, Duration)]) -> Duration {
    let deadlines: Vec<_> = periods
        .iter()
        .map(|(period, anchor)| {
            let step = period.as_nanos().max(1);
            let multiples = now.saturating_sub(*anchor).as_nanos() / step + 1;
            *anchor + Duration::from_nanos((step * multiples).min(u128::from(u64::MAX)) as u64)
        })
        .collect();
    let first = deadlines
        .iter()
        .min()
        .copied()
        .unwrap_or(now + Duration::from_secs(1));
    // Coalesce floating point refresh periods a few nanoseconds apart.
    deadlines
        .into_iter()
        .filter(|due| *due <= first + Duration::from_millis(1))
        .max()
        .unwrap_or(first)
}
