//! Game frame metrics from Intel PresentMon (Windows, on demand).
//!
//! PresentMon is a separate program Bezel starts only while a shown value
//! wants one of the game sensors AND a game window is in the foreground; it
//! stops ~60 s after the last frame and can never outlive Bezel (a Windows
//! job object kills it with us). The parser and the metrics are pure safe
//! Rust over CSV text, tested on every OS with the golden fixtures of
//! `fixtures/presentmon/`.

use std::cmp::Reverse;
use std::collections::{HashMap, VecDeque};
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use super::{FrameMetrics, FrameRateSource};
#[cfg(windows)]
use super::rtss::Rtss;
#[cfg(windows)]
use std::path::Path;

/// Processes whose frames are never a game: compositor, shell, overlays and
/// browsers -- excluded on the command line and again while parsing.
const IGNORED_PROCESSES: &[&str] = &[
    "dwm.exe",
    "explorer.exe",
    "steamwebhelper.exe",
    "discord.exe",
    "gamebar.exe",
    "chrome.exe",
    "msedge.exe",
    "firefox.exe",
    "opera.exe",
    "brave.exe",
];

/// Stop PresentMon after this long without any frames once frames were seen
/// (a pause or a loading screen should not end the session).
const STOP_AFTER: Duration = Duration::from_secs(60);
/// Stop PresentMon after this long without any frames if none were ever
/// seen: the foreground process does not present.
const NO_FRAME_WINDOW: Duration = Duration::from_secs(10);
const BACKOFF_INITIAL: Duration = Duration::from_secs(2);
const BACKOFF_MAX: Duration = Duration::from_secs(60);

/// One parsed CSV row: a frame one process presented.
#[derive(Debug, Clone)]
pub(crate) struct FrameData {
    pub timestamp: Instant,
    pub process_name: String,
    pub frametime_ms: f64,
    pub cpu_busy: Option<f64>,
    pub cpu_wait: Option<f64>,
    pub gpu_busy: Option<f64>,
    pub gpu_wait: Option<f64>,
}

/// The header of PresentMon's CSV stream, by column name (the frame-time
/// column differs between versions; the busy/wait columns are optional).
pub(crate) struct PresentMonParser {
    process_idx: usize,
    frametime_idx: usize,
    cpu_busy_idx: Option<usize>,
    cpu_wait_idx: Option<usize>,
    gpu_busy_idx: Option<usize>,
    gpu_wait_idx: Option<usize>,
}

impl PresentMonParser {
    /// `None` when `header` is not PresentMon's header.
    pub fn new(header: &str) -> Option<Self> {
        let cols: Vec<&str> = header.split(',').map(|s| s.trim()).collect();
        // PresentMon names the process column "Application" in every real
        // release (1.x, 2.x and current); "Process"/"ProcessName" cover
        // synthetic logs and older forks.
        let process_idx = cols
            .iter()
            .position(|&c| c == "Application" || c == "Process" || c == "ProcessName")?;
        let frametime_idx = cols.iter().position(|&c| {
            c == "MsBetweenPresents"
                || c == "msBetweenPresents"
                || c == "FrameTime"
                || c == "MsFrameTime"
        })?;
        let cpu_busy_idx = cols
            .iter()
            .position(|&c| c.contains("CPUBusy") || c.contains("CpuBusy"));
        let cpu_wait_idx = cols
            .iter()
            .position(|&c| c.contains("CPUWait") || c.contains("CpuWait"));
        let gpu_busy_idx = cols
            .iter()
            .position(|&c| c.contains("GPUBusy") || c.contains("GpuBusy"));
        let gpu_wait_idx = cols
            .iter()
            .position(|&c| c.contains("GPUWait") || c.contains("GpuWait"));

        Some(Self {
            process_idx,
            frametime_idx,
            cpu_busy_idx,
            cpu_wait_idx,
            gpu_busy_idx,
            gpu_wait_idx,
        })
    }

    /// The row's data, or `None` when it is not a frame of a game process
    /// with a plausible frame time.
    pub fn parse(&self, line: &str, timestamp: Instant) -> Option<FrameData> {
        let cols: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
        let process_name = cols.get(self.process_idx)?.to_string();

        // Defense in depth: filter ignored processes even if --exclude failed.
        if IGNORED_PROCESSES
            .iter()
            .any(|&ignored| process_name.eq_ignore_ascii_case(ignored))
        {
            return None;
        }
        // PresentMon reports "<unknown>" when it cannot attribute frames to a
        // process (official docs): on an idle desktop these are all that is
        // left, and they must not pose as a game. Any name in angle brackets
        // is a placeholder, never a real process.
        if process_name.starts_with('<') && process_name.ends_with('>') {
            return None;
        }

        let frametime_ms = cols.get(self.frametime_idx)?.parse::<f64>().ok()?;
        // A frame is longer than 0 ms and shorter than 5 s.
        if !(0.0 < frametime_ms && frametime_ms < 5000.0) {
            return None;
        }

        let cpu_busy = self
            .cpu_busy_idx
            .and_then(|i| cols.get(i).and_then(|s| s.parse::<f64>().ok()));
        let cpu_wait = self
            .cpu_wait_idx
            .and_then(|i| cols.get(i).and_then(|s| s.parse::<f64>().ok()));
        let gpu_busy = self
            .gpu_busy_idx
            .and_then(|i| cols.get(i).and_then(|s| s.parse::<f64>().ok()));
        let gpu_wait = self
            .gpu_wait_idx
            .and_then(|i| cols.get(i).and_then(|s| s.parse::<f64>().ok()));

        Some(FrameData {
            timestamp,
            process_name,
            frametime_ms,
            cpu_busy,
            cpu_wait,
            gpu_busy,
            gpu_wait,
        })
    }
}

/// The lifecycle: not started, running, or waiting to retry after a failure.
enum State {
    Stopped,
    Running,
    BackingOff(Instant),
}

pub(crate) struct PresentMon {
    path: Option<PathBuf>,
    child: Option<Child>,
    #[cfg(windows)]
    job_handle: Option<isize>,
    parser: Option<PresentMonParser>,
    frames: VecDeque<FrameData>,
    last_frame_time: Option<Instant>,
    started_at: Instant,
    is_wanted: bool,
    state: State,
    backoff_duration: Duration,
    lines: Arc<Mutex<VecDeque<String>>>,
    errors: Arc<Mutex<VecDeque<String>>>,
    running: Arc<AtomicBool>,
    fallback_to_rtss: bool,
    #[cfg(windows)]
    rtss_fallback: Option<Rtss>,
}

impl PresentMon {
    /// `path` names the console executable, or `None` to search the usual
    /// places.
    pub fn new(path: Option<PathBuf>) -> Self {
        Self {
            path,
            child: None,
            #[cfg(windows)]
            job_handle: None,
            parser: None,
            frames: VecDeque::new(),
            last_frame_time: None,
            started_at: Instant::now(),
            is_wanted: false,
            state: State::Stopped,
            backoff_duration: BACKOFF_INITIAL,
            lines: Arc::new(Mutex::new(VecDeque::new())),
            errors: Arc::new(Mutex::new(VecDeque::new())),
            running: Arc::new(AtomicBool::new(true)),
            fallback_to_rtss: false,
            #[cfg(windows)]
            rtss_fallback: None,
        }
    }

    /// Where PresentMon's console executable is, in this order: the
    /// configured path, a `tools/` or `sensors/` folder next to Bezel,
    /// and Intel's install folder. In every folder the plain
    /// `PresentMon.exe` wins, and a versioned release name such as
    /// `PresentMon-2.6.0-x64.exe` is accepted as well. The `PATH` is
    /// not searched: discovery stays predictable.
    fn find_executable(&self) -> Option<PathBuf> {
        if let Some(p) = &self.path {
            if p.exists() {
                return Some(p.clone());
            }
        }
        if let Ok(exe) = std::env::current_exe() {
            // An exe without a parent folder must not skip the rest of the
            // search (Program Files) below.
            if let Some(dir) = exe.parent() {
                if let Some(found) = Self::find_in_dir(&dir.join("tools")) {
                    return Some(found);
                }
                // The portable layout keeps the sensor helper in sensors\.
                if let Some(found) = Self::find_in_dir(&dir.join("sensors")) {
                    return Some(found);
                }
            }
        }
        if let Ok(pf) = std::env::var("ProgramFiles") {
            let intel = PathBuf::from(pf).join("Intel").join("PresentMon");
            if let Some(found) = Self::find_in_dir(&intel) {
                return Some(found);
            }
        }
        None
    }

    /// True for a PresentMon console executable: the plain
    /// `PresentMon.exe`, or a versioned release name such as
    /// `PresentMon-2.6.0-x64.exe`.
    fn is_presentmon_exe(name: &str) -> bool {
        let name = name.to_ascii_lowercase();
        name == "presentmon.exe" || (name.starts_with("presentmon-") && name.ends_with(".exe"))
    }

    /// The PresentMon executable inside `dir`: the plain `PresentMon.exe`
    /// wins over versioned names; among versioned names the highest name in
    /// alphabetical order wins, so a newer release usually does.
    fn find_in_dir(dir: &std::path::Path) -> Option<PathBuf> {
        let mut exact: Option<PathBuf> = None;
        let mut versioned: Vec<PathBuf> = Vec::new();
        for entry in std::fs::read_dir(dir).ok()?.flatten() {
            let Some(name) = entry.file_name().into_string().ok() else {
                continue;
            };
            if name.eq_ignore_ascii_case("PresentMon.exe") {
                exact = Some(entry.path());
            } else if Self::is_presentmon_exe(&name) {
                versioned.push(entry.path());
            }
        }
        if exact.is_some() {
            return exact;
        }
        versioned.sort();
        versioned.pop()
    }

    /// True when the foreground process looks like a game: not a browser,
    /// compositor, overlay, or Bezel itself. The cheap gate that keeps
    /// PresentMon from ever running while the machine is idle.
    fn foreground_is_game() -> bool {
        let Some(name) = foreground_process_name() else {
            return false;
        };
        if IGNORED_PROCESSES
            .iter()
            .any(|&ignored| name.eq_ignore_ascii_case(ignored))
        {
            return false;
        }
        if is_bezel_process(&name) {
            return false;
        }
        true
    }

    /// Spawns PresentMon inside a job object, so it dies with Bezel. The job
    /// is created before the spawn: a job failure leaves nothing running.
    fn start(&mut self) -> Result<(), String> {
        let path = self.find_executable().ok_or(
            "PresentMon executable not found. Configure its path, or copy PresentMon.exe or PresentMon-<version>-x64.exe into tools\\ or sensors\\ next to bezel.exe.",
        )?;

        #[cfg(windows)]
        let job_handle =
            create_job_object().map_err(|e| format!("Failed to create job object: {}", e))?;

        let mut cmd = Command::new(&path);
        // A fixed, Bezel-specific session name: unrelated PresentMon
        // sessions stay out of reach, and clean-up only ever touches
        // Bezel's own session.
        cmd.arg("--session_name").arg("BezelPresentMon");
        // A hard kill (our stop, the job object) leaves PresentMon's ETW
        // session behind; --stop_existing_session clears it before starting.
        cmd.arg("--stop_existing_session")
            .arg("--output_stdout")
            .arg("--exclude_dropped");
        for proc in IGNORED_PROCESSES {
            cmd.arg("--exclude").arg(proc);
        }
        // stdin is closed and stderr discarded: PresentMon must never wait
        // on us, and an unread stderr pipe would block it once it fills.
        cmd.stdin(Stdio::null())
            .stdout(Stdio::piped())
            // stderr is a pipe read by a thread: PresentMon's exit reason
            // (rights, unknown flag, leaked session) becomes the sensor's
            // message instead of dying silently.
            .stderr(Stdio::piped());

        let mut child = cmd.spawn().map_err(|e| {
            #[cfg(windows)]
            close_handle(job_handle);
            if e.kind() == std::io::ErrorKind::PermissionDenied {
                "PresentMon failed: Permission denied. Ensure you have administrator rights or are in the 'Performance Log Users' group.".to_string()
            } else {
                format!("Failed to start PresentMon: {}", e)
            }
        })?;

        #[cfg(windows)]
        {
            use std::os::windows::io::AsRawHandle;
            let child_handle = child.as_raw_handle() as isize;
            if let Err(e) = assign_to_job(job_handle, child_handle) {
                let _ = child.kill();
                let _ = child.wait();
                close_handle(job_handle);
                return Err(e);
            }
            self.job_handle = Some(job_handle);
        }

        let stdout = child
            .stdout
            .take()
            .ok_or("Failed to get PresentMon's output")?;
        let stderr = child.stderr.take();
        // Revive the reader flag: a stopped reader must not kill the next
        // session's reader.
        self.running.store(true, Ordering::Relaxed);
        let lines = Arc::clone(&self.lines);
        let running = Arc::clone(&self.running);

        std::thread::spawn(move || {
            let reader = BufReader::new(stdout);
            for line in reader.lines() {
                if !running.load(Ordering::Relaxed) {
                    break;
                }
                if let Ok(line) = line {
                    let mut guard = lines
                        .lock()
                        .unwrap_or_else(|poisoned| poisoned.into_inner());
                    guard.push_back(line);
                }
            }
        });

        if let Some(stderr) = stderr {
            let errors = Arc::clone(&self.errors);
            let running = Arc::clone(&self.running);
            std::thread::spawn(move || {
                let reader = BufReader::new(stderr);
                for line in reader.lines() {
                    if !running.load(Ordering::Relaxed) {
                        break;
                    }
                    if let Ok(line) = line {
                        let mut guard = errors
                            .lock()
                            .unwrap_or_else(|poisoned| poisoned.into_inner());
                        guard.push_back(line);
                        // A few lines are enough to diagnose a death.
                        while guard.len() > 8 {
                            guard.pop_front();
                        }
                    }
                }
            });
        }

        self.child = Some(child);
        self.state = State::Running;
        self.started_at = Instant::now();
        self.fallback_to_rtss = false;
        Ok(())
    }

    /// Kills PresentMon and forgets the session; safe to call any time.
    fn stop(&mut self) {
        self.running.store(false, Ordering::Relaxed);
        if let Some(mut child) = self.child.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
        #[cfg(windows)]
        if let Some(job) = self.job_handle.take() {
            close_handle(job);
        }
        self.parser = None;
        self.frames.clear();
        self.last_frame_time = None;
        self.state = State::Stopped;
        self.backoff_duration = BACKOFF_INITIAL;
        // Clear the queues so a restart starts clean.
        {
            let mut guard = self
                .lines
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            guard.clear();
            let mut errors = self
                .errors
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            errors.clear();
        }
    }

    /// Moves the CSV lines the reader thread queued into `frames`.
    fn process_lines(&mut self) {
        let mut guard = self
            .lines
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        while let Some(line) = guard.pop_front() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            if self.parser.is_none() {
                self.parser = PresentMonParser::new(line);
                continue;
            }
            if let Some(parser) = &self.parser {
                if let Some(data) = parser.parse(line, Instant::now()) {
                    self.frames.push_back(data);
                    self.last_frame_time = Some(Instant::now());
                }
            }
        }
    }

    /// Drops frames older than the 1%-low window.
    fn prune_frames(&mut self) {
        let thirty_sec_ago = Instant::now() - Duration::from_secs(30);
        while let Some(front) = self.frames.front() {
            if front.timestamp < thirty_sec_ago {
                self.frames.pop_front();
            } else {
                break;
            }
        }
    }
}

/// A dropped PresentMon must never leave PresentMon.exe running: whatever
/// forgets stop(), the kill happens here too.
impl Drop for PresentMon {
    fn drop(&mut self) {
        self.stop();
    }
}

impl FrameRateSource for PresentMon {
    fn describe(&self) -> String {
        if self.fallback_to_rtss {
            "RivaTuner Statistics Server shared memory (PresentMon fallback)".to_string()
        } else {
            "Intel PresentMon (on demand)".to_string()
        }
    }

    fn want(&mut self, wanted: bool) {
        self.is_wanted = wanted;
        if wanted {
            self.fallback_to_rtss = false;
        } else {
            self.stop();
        }
    }

    fn read(&mut self) -> Result<FrameMetrics, String> {
        if !self.is_wanted {
            self.stop();
            return Err("PresentMon not wanted by any theme".to_string());
        }

        // PresentMon cannot start (missing, no rights): RTSS provides fps.
        #[cfg(windows)]
        if self.fallback_to_rtss {
            if let Some(ref mut rtss) = self.rtss_fallback {
                return rtss.read();
            }
        }

        // Cheap gate before anything spawns: only run while a game is in
        // the foreground. No fallback here -- the check is transient and
        // retried on every sample.
        let started = matches!(self.state, State::Running);
        if !started && !Self::foreground_is_game() {
            return Err(
                "start a game: PresentMon only runs while a game is in the foreground"
                    .to_string(),
            );
        }

        // What the state machine wants to do, copied out so no borrow of
        // `self.state` lives across the mutating calls below.
        let retry_at = match &self.state {
            State::Stopped => Some(Instant::now()),
            State::BackingOff(next) => Some(*next),
            State::Running => None,
        };

        if let Some(at) = retry_at {
            if Instant::now() >= at {
                if let Err(e) = self.start() {
                    self.fallback_to_rtss = true;
                    #[cfg(windows)]
                    {
                        self.rtss_fallback = Some(Rtss);
                    }
                    self.state = State::BackingOff(Instant::now() + self.backoff_duration);
                    self.backoff_duration = (self.backoff_duration * 2).min(BACKOFF_MAX);
                    return Err(format!("{} -gpu.fps falls back to RTSS", e));
                }
                self.backoff_duration = BACKOFF_INITIAL;
            } else {
                return Err(format!(
                    "PresentMon backing off for {:.1} s",
                    (at - Instant::now()).as_secs_f64()
                ));
            }
        } else {
            // Running: a child that died is restarted with backoff.
            let exited = match self.child.as_mut() {
                Some(child) => matches!(child.try_wait(), Ok(Some(_))),
                None => false,
            };
            if exited {
                // stop() resets the backoff to the initial value; keep the
                // escalated one so a crash loop backs off 2 s -> 4 s -> 60 s.
                let wait = self.backoff_duration;
                let never_measured = self.last_frame_time.is_none();
                let why = {
                    let guard = self
                        .errors
                        .lock()
                        .unwrap_or_else(|poisoned| poisoned.into_inner());
                    guard
                        .iter()
                        .rev()
                        .find(|l| !l.trim().is_empty())
                        .cloned()
                };
                self.stop();
                self.state = State::BackingOff(Instant::now() + wait);
                self.backoff_duration = (wait * 2).min(BACKOFF_MAX);
                return Err(match why {
                    Some(line) if never_measured => format!(
                        "PresentMon exited before measuring: {}. Its trace session needs an administrator or the 'Performance Log Users' group.",
                        line.trim()
                    ),
                    Some(line) => format!("PresentMon exited: {}", line.trim()),
                    None => "PresentMon process died unexpectedly".to_string(),
                });
            }
        }

        self.process_lines();
        self.prune_frames();

        // The idle stop, before any early return below can hide it: after
        // the first frame PresentMon survives pauses (STOP_AFTER); without
        // any frame it stops after NO_FRAME_WINDOW. Either way it waits
        // BACKOFF_MAX before the next probe, so an idle machine never spins.
        let now = Instant::now();
        let idle_since = self.last_frame_time.unwrap_or(self.started_at);
        let limit = if self.last_frame_time.is_some() {
            STOP_AFTER
        } else {
            NO_FRAME_WINDOW
        };
        if now.duration_since(idle_since) > limit {
            self.stop();
            self.state = State::BackingOff(now + BACKOFF_MAX);
            return Err("no frames for a while: PresentMon stopped".to_string());
        }

        let one_sec_ago = now - Duration::from_secs(1);
        let thirty_sec_ago = now - Duration::from_secs(30);

        let recent_frames: Vec<_> = self
            .frames
            .iter()
            .filter(|f| f.timestamp >= one_sec_ago)
            .cloned()
            .collect();
        if recent_frames.len() < 2 {
            return Err("No game producing frames detected in the last 1s".to_string());
        }

        let mut process_counts = HashMap::new();
        for f in &recent_frames {
            *process_counts.entry(f.process_name.clone()).or_insert(0) += 1;
        }
        // The game is the busiest process; ties break by name, so this is
        // deterministic.
        let game = process_counts
            .into_iter()
            .max_by_key(|(name, count)| (*count, Reverse(name.clone())))
            .map(|(name, _)| name)
            .unwrap_or_default();

        let game_frames: Vec<_> = recent_frames
            .iter()
            .filter(|f| f.process_name == game)
            .cloned()
            .collect();
        if game_frames.len() < 2 {
            return Err(format!("Game '{}' has insufficient frames", game));
        }

        let frametime =
            game_frames.iter().map(|f| f.frametime_ms).sum::<f64>() / game_frames.len() as f64;
        let fps = 1000.0 / frametime;

        // 1% lows: the mean of the slowest 1% of the last 30 s.
        let mut game_frames_30s: Vec<_> = self
            .frames
            .iter()
            .filter(|f| f.timestamp >= thirty_sec_ago && f.process_name == game)
            .map(|f| f.frametime_ms)
            .collect();

        game_frames_30s.sort_by(|a, b| a.total_cmp(b));
        game_frames_30s.reverse();

        let one_percent_count = (game_frames_30s.len() as f64 * 0.01).ceil() as usize;
        let one_percent_count = one_percent_count.max(1);
        let slowest_frames = &game_frames_30s[..one_percent_count];
        let mean_slowest = slowest_frames.iter().sum::<f64>() / slowest_frames.len() as f64;
        let fps_low_1p = 1000.0 / mean_slowest;

        // 0.1% lows: the mean of the slowest 0.1% of the same window.
        let point_one_percent_count = (game_frames_30s.len() as f64 * 0.001).ceil() as usize;
        let point_one_percent_count = point_one_percent_count.max(1);
        let slowest_point_one = &game_frames_30s[..point_one_percent_count];
        let mean_slowest_01p =
            slowest_point_one.iter().sum::<f64>() / slowest_point_one.len() as f64;
        let fps_low_01p = 1000.0 / mean_slowest_01p;

        // The fastest and slowest single frame of the window, as FPS. The
        // vec is sorted slowest-first after the reverse above.
        let fps_min = 1000.0 / game_frames_30s[0];
        let fps_max = 1000.0 / game_frames_30s[game_frames_30s.len() - 1];

        // The limiter: which side waits for the other, judged on medians.
        let mut cpu_waits = Vec::new();
        let mut gpu_waits = Vec::new();
        let mut cpu_busys = Vec::new();
        let mut gpu_busys = Vec::new();

        for f in &game_frames {
            if let Some(w) = f.cpu_wait {
                cpu_waits.push(w);
            }
            if let Some(w) = f.gpu_wait {
                gpu_waits.push(w);
            }
            if let Some(b) = f.cpu_busy {
                cpu_busys.push(b);
            }
            if let Some(b) = f.gpu_busy {
                gpu_busys.push(b);
            }
        }

        let limiter = if !cpu_waits.is_empty() && !gpu_waits.is_empty() {
            let median_cpu_wait = median(&mut cpu_waits);
            let median_gpu_wait = median(&mut gpu_waits);
            if median_gpu_wait > median_cpu_wait {
                Some("cpu".to_string())
            } else if median_cpu_wait > median_gpu_wait {
                Some("gpu".to_string())
            } else {
                // CPUBusy/GPUBusy are milliseconds per frame, not
                // percentages: both well below the frame time is a cap.
                let median_cpu_busy = median(&mut cpu_busys);
                let median_gpu_busy = median(&mut gpu_busys);
                if median_cpu_busy < 0.9 * frametime && median_gpu_busy < 0.9 * frametime {
                    Some("cap".to_string())
                } else {
                    None
                }
            }
        } else {
            None
        };

        Ok(FrameMetrics {
            fps,
            frametime: Some(frametime),
            fps_low_1p: Some(fps_low_1p),
            fps_low_01p: Some(fps_low_01p),
            fps_min: Some(fps_min),
            fps_max: Some(fps_max),
            source: Some("PresentMon".to_string()),
            game: Some(game),
            limiter,
        })
    }
}

/// The middle value, safe for even and odd lengths; empty is 0.
fn median(values: &mut [f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    values.sort_by(|a, b| a.total_cmp(b));
    let mid = values.len() / 2;
    if values.len() % 2 == 0 {
        (values[mid - 1] + values[mid]) / 2.0
    } else {
        values[mid]
    }
}

/// True when `name` is Bezel's own executable: Bezel is never the game.
fn is_bezel_process(name: &str) -> bool {
    if let Ok(exe) = std::env::current_exe() {
        if let Some(exe_name) = exe.file_name() {
            return exe_name.to_string_lossy().eq_ignore_ascii_case(name);
        }
    }
    false
}

/// The name of the executable of the foreground window, or `None` when
/// Windows does not name one.
#[cfg(windows)]
fn foreground_process_name() -> Option<String> {
    let path = foreground_image_path()?;
    Path::new(&path)
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
}

/// The name of the foreground process; no game can be detected without
/// Windows, so `None` keeps PresentMon stopped there (tests only).
#[cfg(not(windows))]
fn foreground_process_name() -> Option<String> {
    None
}

// ---------------------------------------------------------------------------
// Win32 foreground-process detection. Small safe wrappers in the style of
// `fps/rtss`: the only `unsafe` is the FFI call itself, each with its
// `// SAFETY:` reasoning.
// ---------------------------------------------------------------------------

/// The full image path of the foreground window's process.
#[cfg(windows)]
fn foreground_image_path() -> Option<String> {
    let hwnd = get_foreground_window()?;
    let pid = get_window_thread_process_id(hwnd)?;
    let handle = open_process_query(pid)?;
    let path = query_full_process_image_name(handle);
    close_handle(handle);
    path
}

#[cfg(windows)]
#[allow(
    unsafe_code,
    reason = "Win32 FFI: the foreground window says when a game runs"
)]
fn get_foreground_window() -> Option<isize> {
    use windows_sys::Win32::UI::WindowsAndMessaging::GetForegroundWindow;
    // SAFETY: GetForegroundWindow takes no arguments and only returns a
    // window handle (0 when there is none).
    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.is_null() {
            None
        } else {
            Some(hwnd as isize)
        }
    }
}

#[cfg(windows)]
#[allow(
    unsafe_code,
    reason = "Win32 FFI: the foreground window's process owns the game's frames"
)]
fn get_window_thread_process_id(hwnd: isize) -> Option<u32> {
    use windows_sys::Win32::UI::WindowsAndMessaging::GetWindowThreadProcessId;
    // SAFETY: GetWindowThreadProcessId writes the owning process ID to
    // `pid`, which outlives the call.
    unsafe {
        let mut pid = 0u32;
        GetWindowThreadProcessId(
            hwnd as windows_sys::Win32::Foundation::HANDLE,
            &mut pid,
        );
        if pid == 0 {
            None
        } else {
            Some(pid)
        }
    }
}

#[cfg(windows)]
#[allow(
    unsafe_code,
    reason = "Win32 FFI: opening the foreground process to name it"
)]
fn open_process_query(pid: u32) -> Option<isize> {
    use windows_sys::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION};
    // SAFETY: OpenProcess with PROCESS_QUERY_LIMITED_INFORMATION only opens
    // the process for querying; the handle is closed by the caller.
    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
        if handle.is_null() {
            None
        } else {
            Some(handle as isize)
        }
    }
}

#[cfg(windows)]
#[allow(
    unsafe_code,
    reason = "Win32 FFI: the foreground process's name gates starting PresentMon"
)]
fn query_full_process_image_name(handle: isize) -> Option<String> {
    use windows_sys::Win32::System::Threading::QueryFullProcessImageNameW;
    // SAFETY: QueryFullProcessImageNameW writes the image path into
    // `buffer` and its length to `size`, both of which outlive the call.
    unsafe {
        let mut buffer = [0u16; 260];
        let mut size = 260u32;
        let result = QueryFullProcessImageNameW(
            handle as windows_sys::Win32::Foundation::HANDLE,
            0,
            buffer.as_mut_ptr(),
            &mut size,
        );
        if result == 0 {
            None
        } else {
            Some(String::from_utf16_lossy(&buffer[..size as usize]))
        }
    }
}

// ---------------------------------------------------------------------------
// Win32 job object: PresentMon dies with Bezel however Bezel ends.
// ---------------------------------------------------------------------------

#[cfg(windows)]
#[allow(
    unsafe_code,
    reason = "Win32 FFI: the job object keeps PresentMon from outliving Bezel"
)]
fn create_job_object() -> Result<isize, String> {
    use windows_sys::Win32::Foundation::CloseHandle;
    use windows_sys::Win32::System::JobObjects::{
        CreateJobObjectW, JobObjectExtendedLimitInformation, JOBOBJECT_EXTENDED_LIMIT_INFORMATION,
        JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE, SetInformationJobObject,
    };

    // SAFETY: CreateJobObjectW creates a new, unnamed job object.
    let job = unsafe { CreateJobObjectW(std::ptr::null(), std::ptr::null()) };
    if job.is_null() {
        return Err("Failed to create job object".to_string());
    }

    let mut info: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = unsafe { std::mem::zeroed() };
    info.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;

    // SAFETY: SetInformationJobObject configures the job to kill its
    // children when the last handle closes (Bezel's exit).
    let result = unsafe {
        SetInformationJobObject(
            job,
            JobObjectExtendedLimitInformation,
            &mut info as *mut _ as *mut _,
            std::mem::size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
        )
    };

    if result == 0 {
        // SAFETY: CloseHandle releases the job we created but did not use.
        unsafe { CloseHandle(job) };
        return Err("Failed to set job object information".to_string());
    }

    Ok(job as isize)
}

#[cfg(windows)]
#[allow(
    unsafe_code,
    reason = "Win32 FFI: PresentMon joins the job before it can run unsupervised"
)]
fn assign_to_job(job: isize, child: isize) -> Result<(), String> {
    use windows_sys::Win32::System::JobObjects::AssignProcessToJobObject;

    // SAFETY: AssignProcessToJobObject adds the just-spawned child to the
    // job; the handles are valid and closed by the caller.
    let result = unsafe {
        AssignProcessToJobObject(
            job as windows_sys::Win32::Foundation::HANDLE,
            child as windows_sys::Win32::Foundation::HANDLE,
        )
    };

    if result == 0 {
        return Err("Failed to assign process to job object".to_string());
    }

    Ok(())
}

#[cfg(windows)]
#[allow(unsafe_code, reason = "Win32 FFI: closes the handles the other wrappers opened")]
fn close_handle(handle: isize) {
    use windows_sys::Win32::Foundation::CloseHandle;
    // SAFETY: CloseHandle releases a handle one of the wrappers above
    // opened; it is not used again afterwards.
    unsafe {
        CloseHandle(handle as windows_sys::Win32::Foundation::HANDLE);
    }
}

// ---------------------------------------------------------------------------
// Tests: the parser and metrics are pure functions over CSV text, so they
// run on every OS with the golden fixtures of `fixtures/presentmon/`.
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn load_fixture(name: &str) -> String {
        include_str!(concat!("fixtures/presentmon/", name, ".csv")).to_string()
    }

    #[test]
    fn parses_v2_header() {
        let csv = load_fixture("v2_header");
        let mut lines = csv.lines();
        let header = lines.next().unwrap();
        let parser = PresentMonParser::new(header).unwrap();

        let data = parser.parse(lines.next().unwrap(), Instant::now()).unwrap();
        assert_eq!(data.process_name, "game.exe");
        assert!((data.frametime_ms - 16.666).abs() < 0.001);
    }

    #[test]
    fn parses_old_header() {
        let csv = load_fixture("old_header");
        let mut lines = csv.lines();
        let header = lines.next().unwrap();
        let parser = PresentMonParser::new(header).unwrap();

        let data = parser.parse(lines.next().unwrap(), Instant::now()).unwrap();
        assert_eq!(data.process_name, "game.exe");
        assert!((data.frametime_ms - 33.333).abs() < 0.001);
    }

    #[test]
    fn filters_ignored_processes() {
        let csv = load_fixture("multi_process");
        let mut lines = csv.lines();
        let header = lines.next().unwrap();
        let parser = PresentMonParser::new(header).unwrap();

        // dwm.exe is filtered out, game.exe is kept.
        assert!(parser.parse(lines.next().unwrap(), Instant::now()).is_none());
        assert!(parser.parse(lines.next().unwrap(), Instant::now()).is_some());
    }

    #[test]
    fn validates_frame_times() {
        let csv = "Process,FrameTime\ngame.exe,0.0\ngame.exe,6000.0\ngame.exe,16.6";
        let mut lines = csv.lines();
        let header = lines.next().unwrap();
        let parser = PresentMonParser::new(header).unwrap();

        assert!(parser.parse(lines.next().unwrap(), Instant::now()).is_none()); // 0.0
        assert!(parser.parse(lines.next().unwrap(), Instant::now()).is_none()); // 6000.0
        assert!(parser.parse(lines.next().unwrap(), Instant::now()).is_some()); // 16.6
    }

    #[test]
    fn parses_wait_and_busy_columns() {
        let csv = "Application,FrameTime,MsCPUBusy,MsCPUWait,MsGPUBusy,MsGPUWait\ngame.exe,16.6,8.0,5.0,9.0,15.0";
        let mut lines = csv.lines();
        let header = lines.next().unwrap();
        let parser = PresentMonParser::new(header).unwrap();

        let data = parser.parse(lines.next().unwrap(), Instant::now()).unwrap();
        assert_eq!(data.cpu_busy, Some(8.0));
        assert_eq!(data.cpu_wait, Some(5.0));
        assert_eq!(data.gpu_busy, Some(9.0));
        assert_eq!(data.gpu_wait, Some(15.0));
    }
}
