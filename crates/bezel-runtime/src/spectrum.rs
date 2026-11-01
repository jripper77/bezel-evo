//! Bounded audio worker: one capture per exact source, latest results only.
use bezel_core::{
    domain::spectrum::{AudioBlock, AudioState, Beat, Spectrum, SpectrumFrame, SpectrumStyle},
    ports::{AudioCapture, AudioCaptureFactory},
};
use std::{
    collections::BTreeMap,
    sync::{Arc, Condvar, Mutex},
    thread,
    time::{Duration, Instant},
};
const N: usize = 4096;
const STEP: Duration = Duration::from_millis(50);
#[derive(Clone, PartialEq)]
pub(crate) struct Request {
    pub key: String,
    pub source: String,
    pub config: Spectrum,
    pub state: AudioState,
    pub playing: bool,
}
#[derive(Default)]
struct Shared {
    requests: Vec<Request>,
    enabled: bool,
    stop: bool,
    frames: BTreeMap<String, SpectrumFrame>,
}
pub(crate) struct Worker {
    shared: Arc<(Mutex<Shared>, Condvar)>,
}
impl Worker {
    pub fn new(factory: Arc<dyn AudioCaptureFactory>, wake: crate::Wake) -> Self {
        let shared = Arc::new((Mutex::new(Shared::default()), Condvar::new()));
        let state = shared.clone();
        let spawned = thread::Builder::new()
            .name("bezel-light-audio".into())
            .spawn(move || {
                let mut captures: BTreeMap<String, Box<dyn AudioCapture>> = BTreeMap::new();
                let mut spectra: BTreeMap<String, Analysis> = BTreeMap::new();
                let mut envelopes: BTreeMap<String, Envelope> = BTreeMap::new();
                let mut envelope_sources: BTreeMap<String, String> = BTreeMap::new();
                let mut retry: BTreeMap<String, (Instant, AudioState)> = BTreeMap::new();
                let mut previous = Instant::now();
                let demo_origin = previous;
                loop {
                    let (lock, cv) = &*state;
                    let mut s = lock
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner);
                    if s.stop {
                        break;
                    }
                    if s.requests.is_empty() {
                        captures.clear();
                        spectra.clear();
                        envelopes.clear();
                        envelope_sources.clear();
                        retry.clear();
                        s.frames.clear();
                        drop(
                            cv.wait(s)
                                .unwrap_or_else(std::sync::PoisonError::into_inner),
                        );
                        previous = Instant::now();
                        continue;
                    }
                    let requests = s.requests.clone();
                    let enabled = s.enabled;
                    let availability = factory.availability();
                    drop(s);
                    let now = Instant::now();
                    let dt = now.duration_since(previous).as_secs_f32().min(0.5);
                    previous = now;
                    captures.retain(|key, _| {
                        enabled
                            && requests.iter().any(|r| {
                                r.source == *key
                                    && r.playing
                                    && r.state == AudioState::Live
                                    && !r.config.demo
                            })
                    });
                    spectra.retain(|key, _| captures.contains_key(key));
                    retry.retain(|key, _| requests.iter().any(|r| r.source == *key));
                    envelopes.retain(|key, _| requests.iter().any(|r| r.key == *key));
                    envelope_sources.retain(|key, _| requests.iter().any(|r| r.key == *key));
                    let mut source_states = BTreeMap::new();
                    for r in &requests {
                        if !enabled
                            || availability != AudioState::Live
                            || !r.playing
                            || r.state != AudioState::Live
                            || r.config.demo
                            || source_states.contains_key(&r.source)
                        {
                            continue;
                        }
                        if !captures.contains_key(&r.source) && captures.len() < 8 {
                            if let Some((at, error)) = retry.get(&r.source)
                                && now < *at
                            {
                                source_states.insert(r.source.clone(), *error);
                                continue;
                            }
                            match factory.open(&r.source) {
                                Ok(c) => {
                                    captures.insert(r.source.clone(), c);
                                    retry.remove(&r.source);
                                }
                                Err(error) => {
                                    retry.insert(
                                        r.source.clone(),
                                        (now + Duration::from_secs(2), error),
                                    );
                                    source_states.insert(r.source.clone(), error);
                                    continue;
                                }
                            }
                        }
                        let result = captures.get_mut(&r.source).map(|c| c.read());
                        match result {
                            Some(Ok(block)) => {
                                spectra.entry(r.source.clone()).or_default().push(block);
                                source_states.insert(r.source.clone(), AudioState::Live);
                            }
                            Some(Err(error)) => {
                                captures.remove(&r.source);
                                spectra.remove(&r.source);
                                retry.insert(
                                    r.source.clone(),
                                    (now + Duration::from_secs(2), error),
                                );
                                source_states.insert(r.source.clone(), error);
                            }
                            None => {
                                source_states.insert(r.source.clone(), AudioState::Unavailable);
                            }
                        }
                    }
                    let mut frames = BTreeMap::new();
                    for r in &requests {
                        let state = if r.config.demo {
                            AudioState::Demo
                        } else if !enabled {
                            AudioState::Disabled
                        } else if availability != AudioState::Live {
                            availability
                        } else if r.state != AudioState::Live {
                            r.state
                        } else if !r.playing {
                            AudioState::Live
                        } else {
                            *source_states
                                .get(&r.source)
                                .unwrap_or(&AudioState::Unavailable)
                        };
                        let values = if state == AudioState::Demo {
                            (0..r.config.bands)
                                .map(|i| {
                                    ((demo_origin.elapsed().as_secs_f32() * 2. + i as f32 * 0.37)
                                        .sin()
                                        * 0.25
                                        + 0.45)
                                        .max(0.)
                                })
                                .collect()
                        } else if state == AudioState::Live && r.playing {
                            spectra.get(&r.source).map_or_else(
                                || vec![0.; usize::from(r.config.bands)],
                                |a| a.bands(&r.config),
                            )
                        } else {
                            vec![0.; usize::from(r.config.bands)]
                        };
                        if envelope_sources.get(&r.key) != Some(&r.source)
                            || !matches!(state, AudioState::Live | AudioState::Demo)
                        {
                            envelopes.remove(&r.key);
                        }
                        envelope_sources.insert(r.key.clone(), r.source.clone());
                        let envelope = envelopes.entry(r.key.clone()).or_default();
                        frames.insert(r.key.clone(), envelope.update(values, &r.config, dt, state));
                    }
                    let mut s = lock
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner);
                    // A changed request or revocation may arrive while activation/FFT runs.
                    // Do not publish stale audio or sleep past its notification.
                    if s.stop {
                        break;
                    }
                    if s.requests != requests || s.enabled != enabled {
                        continue;
                    }
                    if s.frames != frames {
                        s.frames = frames;
                        wake.notify();
                    }
                    let settled = !requests.iter().any(|r| {
                        r.config.demo
                            || (enabled
                                && availability == AudioState::Live
                                && r.playing
                                && r.state == AudioState::Live)
                    }) && s
                        .frames
                        .values()
                        .all(|f| f.levels.iter().chain(&f.peaks).all(|v| *v == 0.));
                    if settled {
                        drop(
                            cv.wait(s)
                                .unwrap_or_else(std::sync::PoisonError::into_inner),
                        );
                    } else {
                        drop(
                            cv.wait_timeout(s, STEP.saturating_sub(now.elapsed()))
                                .unwrap_or_else(std::sync::PoisonError::into_inner),
                        );
                    }
                }
            });
        // A failed worker leaves capture disabled; it never affects screen ownership.
        if spawned.is_err() {
            shared
                .0
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .stop = true;
        }
        Self { shared }
    }
    pub fn sync(&self, requests: Vec<Request>, enabled: bool) -> BTreeMap<String, SpectrumFrame> {
        let mut state = self
            .shared
            .0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let changed = state.requests != requests || state.enabled != enabled;
        let frames = state
            .frames
            .iter()
            .filter(|(key, _)| {
                requests.iter().any(|r| {
                    r.key == **key
                        && state
                            .requests
                            .iter()
                            .any(|old| old.key == r.key && old.source == r.source)
                })
            })
            .map(|(key, value)| (key.clone(), value.clone()))
            .collect();
        state.requests = requests;
        state.enabled = enabled;

        drop(state);
        if changed {
            self.shared.1.notify_one();
        }
        frames
    }
}
impl Drop for Worker {
    fn drop(&mut self) {
        self.shared
            .0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .stop = true;
        self.shared.1.notify_one();
    }
}

#[derive(Default)]
struct Analysis {
    channels: Vec<Vec<f32>>,
    cursor: usize,
    filled: usize,
    rate: u32,
    power: Vec<f32>,
}
impl Analysis {
    fn push(&mut self, block: AudioBlock) {
        let channels = usize::from(block.channels);
        if !(1..=8).contains(&channels)
            || !(8000..=192000).contains(&block.rate)
            || block.samples.len() > 192000 * 8
        {
            self.power.clear();
            return;
        }
        if self.channels.len() != channels || self.rate != block.rate {
            self.channels = vec![vec![0.; N]; channels];
            self.cursor = 0;
            self.filled = 0;
            self.rate = block.rate;
        }
        for frame in block.samples.chunks_exact(channels) {
            for (c, value) in frame.iter().enumerate() {
                self.channels[c][self.cursor] = if value.is_finite() {
                    value.clamp(-1., 1.)
                } else {
                    0.
                };
            }
            self.cursor = (self.cursor + 1) % N;
            self.filled = (self.filled + 1).min(N);
        }
        // An empty WASAPI packet stream represents silence; clear old energy.
        if block.samples.iter().all(|v| *v == 0.) {
            self.channels.iter_mut().for_each(|c| c.fill(0.));
            self.power = vec![0.; N / 2 + 1];
            return;
        }
        self.power = vec![0.; N / 2 + 1];
        for channel in &self.channels {
            let mut real = vec![0.; N];
            let mut imag = vec![0.; N];
            for (i, v) in real.iter_mut().enumerate() {
                static WINDOW: std::sync::OnceLock<Vec<f32>> = std::sync::OnceLock::new();
                let window = WINDOW.get_or_init(|| {
                    (0..N)
                        .map(|j| {
                            0.5 - 0.5 * (std::f32::consts::TAU * j as f32 / (N - 1) as f32).cos()
                        })
                        .collect()
                });
                *v = channel[(self.cursor + i) % N] * window[i];
            }
            fft(&mut real, &mut imag);
            for (i, p) in self.power.iter_mut().enumerate() {
                *p += (real[i] * real[i] + imag[i] * imag[i])
                    / (channels as f32 * (N as f32 * 0.25).powi(2));
            }
        }
    }
    fn bands(&self, s: &Spectrum) -> Vec<f32> {
        (0..s.bands)
            .map(|i| {
                let edge = |index: u16| {
                    s.low * (s.high / s.low).powf(f32::from(index) / f32::from(s.bands))
                };
                let lo = (edge(i) * N as f32 / self.rate.max(1) as f32).floor() as usize;
                let hi = (edge(i + 1) * N as f32 / self.rate.max(1) as f32).ceil() as usize;
                let power = (lo.max(1)..hi.max(lo + 1).min(self.power.len()))
                    .filter_map(|k| self.power.get(k))
                    .copied()
                    .fold(0., f32::max);
                if power <= 10f32.powf(-7.2) {
                    0.
                } else {
                    ((10. * power.log10() + 72.) / 72.).clamp(0., 1.)
                }
            })
            .collect()
    }
}
fn fft(real: &mut [f32], imag: &mut [f32]) {
    let mut j = 0;
    for i in 1..N {
        let mut bit = N >> 1;
        while j & bit != 0 {
            j ^= bit;
            bit >>= 1;
        }
        j ^= bit;
        if i < j {
            real.swap(i, j);
            imag.swap(i, j);
        }
    }
    let mut len = 2;
    while len <= N {
        for start in (0..N).step_by(len) {
            let angle = -std::f32::consts::TAU / len as f32;
            let (step_sin, step_cos) = angle.sin_cos();
            let (mut sin, mut cos) = (0., 1.);
            for k in 0..len / 2 {
                let a = start + k;
                let b = a + len / 2;
                let tr = cos * real[b] - sin * imag[b];
                let ti = sin * real[b] + cos * imag[b];
                real[b] = real[a] - tr;
                imag[b] = imag[a] - ti;
                real[a] += tr;
                imag[a] += ti;
                (sin, cos) = (
                    sin * step_cos + cos * step_sin,
                    cos * step_cos - sin * step_sin,
                );
            }
        }
        len *= 2;
    }
}
#[derive(Default)]
struct Envelope {
    levels: Vec<f32>,
    peaks: Vec<f32>,
    held: Vec<f32>,
    pulse: Pulse,
}
/// Highest frequency counted as bass for the beat, Hz.
const BASS_TOP: f32 = 200.;
/// How long the bass's average remembers, seconds.
const BASS_MEMORY: f32 = 0.8;
/// Shortest time between two beats, seconds (about 220 beats a minute).
const BEAT_GAP: f32 = 0.27;
/// How much the bass rises within one update, and over its average, for a beat.
const BEAT_RISE: f32 = 0.03;
const BEAT_OVER: f32 = 0.04;
/// Quietest bass that beats.
const BEAT_FLOOR: f32 = 0.2;
/// A beat this many times the usual one is as strong as it gets.
const BEAT_FULL: f32 = 1.5;
/// Share of each new beat in the usual one.
const BEAT_USUAL: f32 = 0.25;
/// Steps of `Beat::since`, seconds: frames change at most this often for it.
const BEAT_STEP: f32 = 0.05;

/// Beats in the bass: a sudden rise above its recent average.
#[derive(Default)]
struct Pulse {
    average: f32,
    previous: f32,
    /// How far a beat usually rises over the average.
    usual: f32,
    last: Option<Beat>,
}
impl Pulse {
    /// The beat after `bands` (the raw levels), `dt` seconds after the last.
    fn update(&mut self, bands: &[f32], s: &Spectrum, dt: f32) -> Option<Beat> {
        let edge = |i: u16| s.low * (s.high / s.low).powf(f32::from(i) / f32::from(s.bands));
        let count = (1..=s.bands)
            .take_while(|i| edge(*i) <= BASS_TOP)
            .count()
            .max(1);
        let bass = bands.iter().take(count).sum::<f32>() / count as f32;
        let rise = bass - self.previous;
        self.previous = bass;
        if let Some(beat) = &mut self.last {
            beat.since += dt;
        }
        let ready = self.last.is_none_or(|b| b.since >= BEAT_GAP);
        let over = bass - self.average;
        if ready && rise > BEAT_RISE && over > BEAT_OVER && bass > BEAT_FLOOR {
            self.usual = if self.last.is_some() {
                self.usual + (over - self.usual) * BEAT_USUAL
            } else {
                over
            };
            self.last = Some(Beat {
                count: self.last.map_or(0, |b| b.count.wrapping_add(1)),
                since: 0.,
                strength: (over / self.usual / BEAT_FULL).clamp(0., 1.),
            });
        }
        self.average += (bass - self.average) * (1. - (-dt / BASS_MEMORY).exp());
        self.last.map(|b| Beat {
            since: ((b.since.min(Beat::LONGEST) / BEAT_STEP).floor() * BEAT_STEP)
                .min(Beat::LONGEST),
            ..b
        })
    }
}
impl Envelope {
    fn update(
        &mut self,
        target: Vec<f32>,
        s: &Spectrum,
        dt: f32,
        state: AudioState,
    ) -> SpectrumFrame {
        if self.levels.len() != target.len() {
            self.levels = vec![0.; target.len()];
            self.peaks = self.levels.clone();
            self.held = self.levels.clone();
        }
        let beat = (s.style == SpectrumStyle::Dancer)
            .then(|| self.pulse.update(&target, s, dt))
            .flatten();
        let (attack, fall) = s.motion.times();
        for (i, target) in target.into_iter().enumerate() {
            let tau = if target > self.levels[i] {
                attack
            } else {
                fall
            };
            self.levels[i] += (target - self.levels[i]) * (1. - (-dt / tau).exp());
            if self.levels[i] < 1. / 512. {
                self.levels[i] = 0.;
            }
            if !s.peaks {
                self.peaks[i] = 0.;
                self.held[i] = 0.;
                continue;
            }
            if self.levels[i] >= self.peaks[i] {
                self.peaks[i] = self.levels[i];
                self.held[i] = s.peak_hold;
            } else {
                self.held[i] = (self.held[i] - dt).max(0.);
                if self.held[i] == 0. {
                    self.peaks[i] = (self.peaks[i] - dt / fall).max(self.levels[i]);
                }
            }
            if self.peaks[i] < 1. / 512. {
                self.peaks[i] = 0.;
            }
        }
        SpectrumFrame {
            state,
            levels: self
                .levels
                .iter()
                .map(|v| (v * 512.).round() / 512.)
                .collect(),
            peaks: self
                .peaks
                .iter()
                .map(|v| (v * 512.).round() / 512.)
                .collect(),
            beat,
        }
    }
}
#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::panic)]
mod tests {
    use super::*;
    use bezel_core::domain::spectrum::Motion;
    #[test]
    fn hann_log_bands_combine_opposite_phase_channels_without_cancellation() {
        let mut a = Analysis::default();
        let rate = 48000;
        let samples = (0..N)
            .flat_map(|i| {
                let v = (std::f32::consts::TAU * 1000. * i as f32 / rate as f32).sin();
                [v, -v]
            })
            .collect();
        a.push(AudioBlock {
            samples,
            rate,
            channels: 2,
        });
        let s = Spectrum::default();
        let bands = a.bands(&s);
        let index = bands
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.total_cmp(b.1))
            .unwrap()
            .0;
        let low = s.low * (s.high / s.low).powf(index as f32 / s.bands as f32);
        let high = s.low * (s.high / s.low).powf((index + 1) as f32 / s.bands as f32);
        assert!(low < 1000. && high > 1000.);
        assert!(bands[index] > 0.95);
        assert!(
            bands
                .iter()
                .all(|v| v.is_finite() && (0. ..=1.).contains(v))
        );
    }
    #[test]
    fn beats_are_found_in_the_bass_and_a_harder_kick_is_stronger() {
        let s = Spectrum {
            style: SpectrumStyle::Dancer,
            ..Spectrum::default()
        };
        let mut e = Envelope::default();
        let mut beats = vec![];
        // 120 beats a minute, updates every 50 ms: a kick on every tenth, the
        // fourth of each bar harder; mids and highs steady throughout.
        for step in 0..80 {
            let kick = match (step % 10, (step / 10) % 4) {
                (0, 3) => 0.95,
                (0, _) => 0.6,
                (1, _) => 0.45,
                _ => 0.3,
            };
            let mut bands = vec![0.5; 32];
            bands[..6].fill(kick);
            let frame = e.update(bands, &s, 0.05, AudioState::Live);
            if let Some(beat) = frame.beat.filter(|b| b.since == 0.) {
                beats.push((step, beat.count, beat.strength));
            }
        }
        assert_eq!(
            beats.iter().map(|b| b.0).collect::<Vec<_>>(),
            (0..8).map(|i| i * 10).collect::<Vec<_>>()
        );
        assert!(beats.iter().enumerate().all(|(i, b)| b.1 == i as u32));
        let (soft, hard) = (beats[4].2, beats[3].2);
        assert!(hard >= 0.8 && soft < 0.8, "soft {soft}, hard {hard}");
        let mut other = Envelope::default();
        let plain = other.update(vec![0.9; 32], &Spectrum::default(), 0.05, AudioState::Live);
        assert_eq!(plain.beat, None, "only a dancer listens for beats");
    }

    #[test]
    fn a_steady_or_quiet_bass_does_not_beat_and_an_old_beat_stops_changing() {
        let s = Spectrum {
            style: SpectrumStyle::Dancer,
            ..Spectrum::default()
        };
        let mut e = Envelope::default();
        for _ in 0..40 {
            assert_eq!(
                e.update(vec![0.7; 32], &s, 0.05, AudioState::Live)
                    .beat
                    .map(|b| b.count),
                e.pulse.last.map(|b| b.count)
            );
        }
        let steady = e.pulse.last.map_or(0, |b| b.count);
        assert!(steady <= 1, "one rise from silence at most, not {steady}");
        let mut quiet = Envelope::default();
        for step in 0..40 {
            let mut bands = vec![0.; 32];
            bands[..6].fill(if step % 10 == 0 { 0.15 } else { 0. });
            assert_eq!(quiet.update(bands, &s, 0.05, AudioState::Live).beat, None);
        }
        let first = e.update(vec![0.7; 32], &s, 0.05, AudioState::Live);
        let again = e.update(vec![0.7; 32], &s, 0.05, AudioState::Live);
        assert_eq!(first.beat, again.beat, "past a second the frame stays");
    }

    #[test]
    fn silence_settles_to_exact_zero_and_stops_changes() {
        for motion in [Motion::Smooth, Motion::Balanced, Motion::Snappy] {
            let s = Spectrum {
                motion,
                ..Spectrum::default()
            };
            let mut e = Envelope::default();
            e.update(vec![1.; 32], &s, 0.05, AudioState::Live);
            let mut last = SpectrumFrame::default();
            for _ in 0..300 {
                last = e.update(vec![0.; 32], &s, 0.05, AudioState::Live);
            }
            assert!(last.levels.iter().chain(&last.peaks).all(|v| *v == 0.));
            assert_eq!(last, e.update(vec![0.; 32], &s, 0.05, AudioState::Live));
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod worker_tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    struct Factory {
        opened: Arc<AtomicUsize>,
        closed: Arc<AtomicUsize>,
    }
    struct Capture(Arc<AtomicUsize>);
    impl Drop for Capture {
        fn drop(&mut self) {
            self.0.fetch_add(1, Ordering::SeqCst);
        }
    }
    impl AudioCapture for Capture {
        fn read(&mut self) -> Result<AudioBlock, AudioState> {
            Ok(AudioBlock {
                samples: vec![],
                rate: 48000,
                channels: 2,
            })
        }
    }
    impl AudioCaptureFactory for Factory {
        fn open(&self, _: &str) -> Result<Box<dyn AudioCapture>, AudioState> {
            self.opened.fetch_add(1, Ordering::SeqCst);
            Ok(Box::new(Capture(self.closed.clone())))
        }
    }
    fn until(mut condition: impl FnMut() -> bool) {
        let limit = Instant::now() + Duration::from_secs(2);
        while !condition() {
            assert!(Instant::now() < limit, "worker did not settle");
            thread::sleep(Duration::from_millis(5));
        }
    }
    #[test]
    fn consumers_share_one_capture_and_release_it_on_pause_disable_or_removal() {
        let opened = Arc::new(AtomicUsize::new(0));
        let closed = Arc::new(AtomicUsize::new(0));
        let worker = Worker::new(
            Arc::new(Factory {
                opened: opened.clone(),
                closed: closed.clone(),
            }),
            crate::Wake::default(),
        );
        let a = Request {
            key: "scene/1".into(),
            source: "Spotify.exe".into(),
            config: Spectrum::default(),
            state: AudioState::Live,
            playing: true,
        };
        let b = Request {
            key: "other/2".into(),
            ..a.clone()
        };
        let requests = vec![a.clone(), b];
        worker.sync(requests.clone(), false);
        thread::sleep(Duration::from_millis(60));
        assert_eq!(opened.load(Ordering::SeqCst), 0);
        worker.sync(requests.clone(), true);
        until(|| {
            let frames = worker.sync(requests.clone(), true);
            frames.len() == 2 && frames.values().all(|f| f.state == AudioState::Live)
        });
        assert_eq!(opened.load(Ordering::SeqCst), 1);
        let mut paused = requests.clone();
        for r in &mut paused {
            r.playing = false;
        }
        worker.sync(paused, true);
        until(|| closed.load(Ordering::SeqCst) == 1);
        worker.sync(vec![a.clone()], true);
        until(|| opened.load(Ordering::SeqCst) == 2);
        worker.sync(Vec::new(), true);
        until(|| closed.load(Ordering::SeqCst) == 2);
        worker.sync(vec![a.clone()], true);
        until(|| opened.load(Ordering::SeqCst) == 3);
        worker.sync(vec![a], false);
        until(|| closed.load(Ordering::SeqCst) == 3);
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod failure_tests {
    use super::*;
    struct Unsupported;
    impl AudioCaptureFactory for Unsupported {
        fn open(&self, _: &str) -> Result<Box<dyn AudioCapture>, AudioState> {
            Err(AudioState::Unsupported)
        }
    }
    #[test]
    fn unsupported_is_not_silence_and_has_no_fallback_capture() {
        let worker = Worker::new(Arc::new(Unsupported), crate::Wake::default());
        let request = Request {
            key: "a/1".into(),
            source: "App.exe".into(),
            playing: true,
            state: AudioState::Live,
            config: Spectrum::default(),
        };
        let until = Instant::now() + Duration::from_secs(2);
        loop {
            let frames = worker.sync(vec![request.clone()], true);
            if let Some(frame) = frames.get("a/1") {
                assert_eq!(frame.state, AudioState::Unsupported);
                assert!(frame.levels.iter().all(|v| *v == 0.));
                break;
            }
            assert!(Instant::now() < until);
            thread::sleep(Duration::from_millis(5));
        }
    }
}

/// The fixed diagnostic label of a spectrum state.
pub(crate) fn kind(state: AudioState) -> &'static str {
    match state {
        AudioState::Live => "live",
        AudioState::Disabled => "disabled",
        AudioState::PlayerMissing => "player-missing",
        AudioState::Unavailable => "unavailable",
        AudioState::Unsupported => "unsupported",
        AudioState::Demo => "demo",
    }
}
