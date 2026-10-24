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
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

#[cfg(windows)]
use super::rtss::Rtss;
use super::{FrameMetrics, FrameRateSource};
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
    "bezel.exe",
    "bezel-studio.exe",
    "taskmgr.exe",
    "notepad.exe",
    "powershell.exe",
    "pwsh.exe",
    "windowsterminal.exe",
    "chatgpt.exe",
    "codex.exe",
    "code.exe",
    "steam.exe",
    "epicgameslauncher.exe",
];

/// Stop PresentMon after this long without any frames once frames were seen
/// (a pause or a loading screen should not end the session).
const STOP_AFTER: Duration = Duration::from_secs(60);
/// Stop PresentMon after this long without any frames if none were ever
/// seen: the foreground process does not present.
const NO_FRAME_WINDOW: Duration = Duration::from_secs(10);
const BACKOFF_INITIAL: Duration = Duration::from_secs(2);
const BACKOFF_MAX: Duration = Duration::from_secs(60);
const MAX_QUEUED_FRAMES: usize = 8192;
const MAX_HISTORY_FRAMES: usize = 240_000;

/// The single console owner. Unlike a thread-affine mutex, this semaphore
/// can be released after the sensor provider moves between worker threads.
/// Contenders never keep a handle: a dead owner's last handle destroys it.
struct CaptureLease(isize);

impl CaptureLease {
    #[allow(unsafe_code, reason = "Win32 capture ownership, released on drop")]
    fn acquire() -> Result<Self, String> {
        Self::named("Global\\BezelPresentMonCapture-v1")
    }

    #[allow(
        unsafe_code,
        reason = "Win32 named capture ownership, released on drop"
    )]
    fn named(value: &str) -> Result<Self, String> {
        use windows_sys::Win32::Foundation::{CloseHandle, WAIT_OBJECT_0};
        use windows_sys::Win32::System::Threading::{CreateSemaphoreW, WaitForSingleObject};
        let name: Vec<u16> = value.encode_utf16().chain(Some(0)).collect();
        // SAFETY: the NUL-terminated name remains alive, no handles are inherited.
        let handle = unsafe { CreateSemaphoreW(std::ptr::null(), 1, 1, name.as_ptr()) };
        if handle.is_null() {
            return Err("cannot create PresentMon ownership lease".into());
        }
        // SAFETY: a live semaphore handle; zero timeout never blocks sampling.
        if unsafe { WaitForSingleObject(handle, 0) } != WAIT_OBJECT_0 {
            // SAFETY: this contender has not acquired ownership and keeps no handle.
            unsafe { CloseHandle(handle) };
            return Err("PresentMon is already owned by another Bezel instance".into());
        }
        Ok(Self(handle as isize))
    }
}

impl Drop for CaptureLease {
    #[allow(unsafe_code, reason = "Win32 capture ownership, released once")]
    fn drop(&mut self) {
        use windows_sys::Win32::System::Threading::ReleaseSemaphore;
        // SAFETY: acquire consumed one semaphore count; the handle is still live.
        unsafe { ReleaseSemaphore(self.0 as _, 1, std::ptr::null_mut()) };
        close_handle(self.0);
    }
}

#[allow(unsafe_code, reason = "Read the Windows QPC clock used by PresentMon")]
fn capture_clock() -> Result<CaptureClock, String> {
    use windows_sys::Win32::System::Performance::{
        QueryPerformanceCounter, QueryPerformanceFrequency,
    };
    let mut ticks = 0;
    let mut frequency = 0;
    // SAFETY: both functions write only into the live i64 outputs.
    if unsafe { QueryPerformanceFrequency(&mut frequency) } == 0
        || frequency <= 0
        || unsafe { QueryPerformanceCounter(&mut ticks) } == 0
    {
        return Err("Windows performance clock is unavailable".into());
    }
    Ok(CaptureClock {
        qpc_ms: ticks as f64 * 1000.0 / frequency as f64,
        at: Instant::now(),
    })
}

#[derive(Clone, Copy)]
struct CaptureClock {
    qpc_ms: f64,
    at: Instant,
}

impl CaptureClock {
    fn frame_time(self, qpc_ms: f64) -> Option<Instant> {
        let delta = qpc_ms - self.qpc_ms;
        let span = Duration::try_from_secs_f64(delta.abs() / 1000.0).ok()?;
        if delta >= 0.0 {
            self.at.checked_add(span)
        } else {
            self.at.checked_sub(span)
        }
    }
}

#[derive(Clone, Debug)]
struct Target {
    pid: u32,
    name: String,
}

/// One parsed CSV row: a frame one process presented.
#[derive(Debug, Clone)]
pub(crate) struct FrameData {
    pub timestamp: Instant,
    pub process_name: String,
    pub process_id: u32,
    pub swap_chain: u64,
    pub qpc_ms: Option<f64>,
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
    pid_idx: Option<usize>,
    chain_idx: Option<usize>,
    qpc_idx: Option<usize>,
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
        let frametime_idx = [
            "MsBetweenPresents",
            "msBetweenPresents",
            "FrameTime",
            "MsFrameTime",
        ]
        .iter()
        .find_map(|name| cols.iter().position(|c| c == name))?;
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
            pid_idx: cols.iter().position(|c| *c == "ProcessID"),
            chain_idx: cols.iter().position(|c| *c == "SwapChainAddress"),
            qpc_idx: cols
                .iter()
                .position(|c| matches!(*c, "CPUStartQPCTimeInMs" | "CPUStartQPCTime")),
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
            .and_then(|i| cols.get(i).and_then(|s| s.parse::<f64>().ok()))
            .filter(|v| v.is_finite() && *v >= 0.0);
        let cpu_wait = self
            .cpu_wait_idx
            .and_then(|i| cols.get(i).and_then(|s| s.parse::<f64>().ok()))
            .filter(|v| v.is_finite() && *v >= 0.0);
        let gpu_busy = self
            .gpu_busy_idx
            .and_then(|i| cols.get(i).and_then(|s| s.parse::<f64>().ok()))
            .filter(|v| v.is_finite() && *v >= 0.0);
        let gpu_wait = self
            .gpu_wait_idx
            .and_then(|i| cols.get(i).and_then(|s| s.parse::<f64>().ok()))
            .filter(|v| v.is_finite() && *v >= 0.0);

        Some(FrameData {
            timestamp,
            process_name,
            process_id: self
                .pid_idx
                .and_then(|i| cols.get(i)?.parse().ok())
                .unwrap_or(0),
            swap_chain: self
                .chain_idx
                .and_then(|i| u64::from_str_radix(cols.get(i)?.trim_start_matches("0x"), 16).ok())
                .unwrap_or(0),
            qpc_ms: self
                .qpc_idx
                .and_then(|i| cols.get(i)?.parse::<f64>().ok())
                .filter(|v| v.is_finite() && *v >= 0.0),
            frametime_ms,
            cpu_busy,
            cpu_wait,
            gpu_busy,
            gpu_wait,
        })
    }

    fn source_frame(&self, line: &str, clock: CaptureClock) -> Option<FrameData> {
        let mut frame = self.parse(line, clock.at)?;
        frame.timestamp = clock.frame_time(frame.qpc_ms?)?;
        (frame.process_id != 0 && frame.swap_chain != 0).then_some(frame)
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
    target: Option<Target>,
    lease: Option<CaptureLease>,
    workers: Vec<JoinHandle<()>>,
    failure: Option<String>,
    statistics_after: Option<Instant>,
    overflowed: Arc<AtomicBool>,
    frames: VecDeque<FrameData>,
    last_frame_time: Option<Instant>,
    started_at: Instant,
    is_wanted: bool,
    state: State,
    backoff_duration: Duration,
    lines: Arc<Mutex<VecDeque<FrameData>>>,
    errors: Arc<Mutex<VecDeque<String>>>,
    running: Arc<AtomicBool>,
    fallback_to_rtss: bool,
    #[cfg(windows)]
    rtss_fallback: Option<Box<dyn FrameRateSource>>,
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
            target: None,
            lease: None,
            workers: Vec::new(),
            failure: None,
            statistics_after: None,
            overflowed: Arc::new(AtomicBool::new(false)),
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
            // An explicit invalid setting must not silently pick another executable.
            return p.is_file().then(|| p.clone());
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
            // Current Intel installs keep the console separate from the GUI.
            if let Some(found) = Self::find_in_dir(&intel.join("PresentMonConsoleApplication")) {
                return Some(found);
            }
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
            if !entry.path().is_file() {
                continue;
            }
            if name.eq_ignore_ascii_case("PresentMon.exe") {
                exact = Some(entry.path());
            } else if Self::is_presentmon_exe(&name) {
                versioned.push(entry.path());
            }
        }
        if exact.is_some() {
            return exact;
        }
        versioned.sort_by_key(|path| {
            let name = path.file_name().unwrap_or_default().to_string_lossy();
            let version = name.split('-').nth(1).unwrap_or_default();
            let parts: Vec<u32> = version
                .split('.')
                .filter_map(|part| part.parse().ok())
                .collect();
            (parts, name.to_string())
        });
        versioned.pop()
    }

    /// True when the foreground process looks like a game: not a browser,
    /// compositor, overlay, or Bezel itself. The cheap gate that keeps
    /// PresentMon from ever running while the machine is idle.
    fn foreground_target() -> Option<Target> {
        let target = foreground_target()?;
        let name = &target.name;
        if IGNORED_PROCESSES
            .iter()
            .any(|&ignored| name.eq_ignore_ascii_case(ignored))
        {
            return None;
        }
        if is_bezel_process(name) {
            return None;
        }
        Some(target)
    }

    /// Spawns PresentMon inside a job object, so it dies with Bezel. The job
    /// is created before the spawn: a job failure leaves nothing running.
    fn start(&mut self, target: Target) -> Result<(), String> {
        let lease = CaptureLease::acquire()?;
        let clock = capture_clock()?;
        let path = self.find_executable().ok_or(
            "PresentMon executable not found. Configure its path, or copy PresentMon.exe or PresentMon-<version>-x64.exe into tools\\ or sensors\\ next to bezel.exe.",
        )?;

        #[cfg(windows)]
        let job_handle =
            create_job_object().map_err(|e| format!("Failed to create job object: {}", e))?;

        let mut cmd = Command::new(&path);
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            use windows_sys::Win32::System::Threading::CREATE_NO_WINDOW;
            cmd.creation_flags(CREATE_NO_WINDOW);
        }
        cmd.arg("--process_id")
            .arg(target.pid.to_string())
            .arg("--qpc_time_ms");
        // A fixed, Bezel-specific session name: unrelated PresentMon
        // sessions stay out of reach, and clean-up only ever touches
        // Bezel's own session.
        cmd.arg("--session_name").arg("BezelPresentMon");
        // A hard kill (our stop, the job object) leaves PresentMon's ETW
        // session behind; --stop_existing_session clears it before starting.
        cmd.arg("--stop_existing_session")
            .arg("--output_stdout")
            .arg("--no_console_stats");
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
        self.lines = Arc::new(Mutex::new(VecDeque::new()));
        self.errors = Arc::new(Mutex::new(VecDeque::new()));
        self.running = Arc::new(AtomicBool::new(true));
        self.overflowed = Arc::new(AtomicBool::new(false));
        let overflowed = Arc::clone(&self.overflowed);
        let lines = Arc::clone(&self.lines);
        let running = Arc::clone(&self.running);

        let target_pid = target.pid;
        self.workers.push(std::thread::spawn(move || {
            let mut parser = None;
            let reader = BufReader::new(stdout);
            for line in reader.lines() {
                if !running.load(Ordering::Relaxed) {
                    break;
                }
                if let Ok(line) = line {
                    if let Some(ref header) = parser {
                        let header: &PresentMonParser = header;
                        if let Some(frame) = header.source_frame(&line, clock)
                            && frame.process_id == target_pid
                        {
                            let mut guard = lines.lock().unwrap_or_else(|p| p.into_inner());
                            if guard.len() == MAX_QUEUED_FRAMES {
                                guard.pop_front();
                                overflowed.store(true, Ordering::Relaxed);
                            }
                            guard.push_back(frame);
                        }
                    } else {
                        parser = PresentMonParser::new(&line);
                    }
                }
            }
        }));

        if let Some(stderr) = stderr {
            let errors = Arc::clone(&self.errors);
            let running = Arc::clone(&self.running);
            self.workers.push(std::thread::spawn(move || {
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
            }));
        }

        self.lease = Some(lease);
        self.target = Some(target);
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
        for worker in self.workers.drain(..) {
            let _ = worker.join();
        }
        self.lease = None;
        self.target = None;
        self.frames.clear();
        self.statistics_after = None;
        self.last_frame_time = None;
        self.state = State::Stopped;
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
        if self.overflowed.swap(false, Ordering::Relaxed) {
            self.frames.clear();
            self.statistics_after = Some(Instant::now() + Duration::from_secs(30));
        }
        while let Some(frame) = guard.pop_front() {
            let now = Instant::now();
            if frame.timestamp > now
                || now.duration_since(frame.timestamp) > Duration::from_secs(30)
            {
                continue;
            }
            self.last_frame_time = Some(
                self.last_frame_time
                    .map_or(frame.timestamp, |last| last.max(frame.timestamp)),
            );
            if self.frames.len() == MAX_HISTORY_FRAMES {
                self.frames.pop_front();
                self.statistics_after = Some(now + Duration::from_secs(30));
            }
            self.frames.push_back(frame);
        }
    }

    fn fail(&mut self, reason: String, now: Instant, slow_retry: bool) {
        self.stop();
        self.failure = Some(reason);
        self.fallback_to_rtss = true;
        self.rtss_fallback.get_or_insert_with(|| Box::new(Rtss));
        let wait = if slow_retry {
            BACKOFF_MAX
        } else {
            self.backoff_duration
        };
        self.state = State::BackingOff(now + wait);
        self.backoff_duration = (wait * 2).min(BACKOFF_MAX);
    }

    fn fallback(&mut self) -> Result<FrameMetrics, String> {
        let why = self
            .failure
            .as_deref()
            .unwrap_or("PresentMon is waiting to retry");
        self.rtss_fallback
            .get_or_insert_with(|| Box::new(Rtss))
            .read()
            .map_err(|rtss| format!("{why}; RTSS fallback: {rtss}"))
    }

    fn read_target(
        &mut self,
        foreground: Option<Target>,
        now: Instant,
    ) -> Result<FrameMetrics, String> {
        if !self.is_wanted {
            self.stop();
            return Err("PresentMon not wanted by any theme".to_string());
        }

        if matches!(self.state, State::Running)
            && let (Some(current), Some(next)) = (&self.target, &foreground)
            && current.pid != next.pid
        {
            self.stop();
        }
        if matches!(self.state, State::Running) {
            let exited = self
                .child
                .as_mut()
                .and_then(|child| child.try_wait().ok())
                .flatten();
            if let Some(status) = exited {
                let detail = self
                    .errors
                    .lock()
                    .unwrap_or_else(|p| p.into_inner())
                    .iter()
                    .filter(|line| !line.trim().is_empty())
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(" ");
                let reason = format!("PresentMon exited with {status}: {detail}");
                self.fail(reason, now, false);
                return self.fallback();
            }
        } else {
            if let State::BackingOff(at) = self.state
                && now < at
            {
                return self.fallback();
            }
            let Some(target) = foreground else {
                return self.fallback();
            };
            if let Err(reason) = self.start(target) {
                self.fail(reason, now, true);
                return self.fallback();
            }
        }

        self.process_lines();
        self.prune_frames();

        // The idle stop, before any early return below can hide it: after
        // the first frame PresentMon survives pauses (STOP_AFTER); without
        // any frame it stops after NO_FRAME_WINDOW. Either way it waits
        // BACKOFF_MAX before the next probe, so an idle machine never spins.
        let idle_since = self.last_frame_time.unwrap_or(self.started_at);
        let limit = if self.last_frame_time.is_some() {
            STOP_AFTER
        } else {
            NO_FRAME_WINDOW
        };
        if now.duration_since(idle_since) > limit {
            self.fail(
                "no recent game frames: PresentMon stopped".into(),
                now,
                true,
            );
            return self.fallback();
        }

        let mut result = metrics(&self.frames, self.target.as_ref().map_or(0, |t| t.pid), now);
        if let Ok(m) = &mut result {
            if self.statistics_after.is_some_and(|after| now < after) {
                m.fps_low_1p = None;
                m.fps_low_01p = None;
                m.fps_min = None;
                m.fps_max = None;
            }
            self.backoff_duration = BACKOFF_INITIAL;
            self.failure = None;
        }
        result
    }

    /// Drops frames older than the 1%-low window.
    fn prune_frames(&mut self) {
        let thirty_sec_ago = Instant::now().checked_sub(Duration::from_secs(30));
        while let Some(front) = self.frames.front() {
            if thirty_sec_ago.is_some_and(|start| front.timestamp < start) {
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
        if self.is_wanted == wanted {
            return;
        }
        self.is_wanted = wanted;
        self.stop();
        self.failure = None;
        self.fallback_to_rtss = false;
        self.backoff_duration = BACKOFF_INITIAL;
    }

    fn read(&mut self) -> Result<FrameMetrics, String> {
        if !self.is_wanted {
            return Err("PresentMon not wanted by any theme".into());
        }
        self.read_target(Self::foreground_target(), Instant::now())
    }
}

fn metrics(frames: &VecDeque<FrameData>, pid: u32, now: Instant) -> Result<FrameMetrics, String> {
    let one_sec_ago = now.checked_sub(Duration::from_secs(1));
    let thirty_sec_ago = now.checked_sub(Duration::from_secs(30));

    let recent_frames: Vec<_> = frames
        .iter()
        .filter(|f| {
            one_sec_ago.is_none_or(|start| f.timestamp >= start)
                && f.timestamp <= now
                && f.process_id == pid
        })
        .cloned()
        .collect();
    if recent_frames.len() < 2 {
        return Err("fps.stale:No recent game frames".to_string());
    }

    let mut counts = HashMap::new();
    for frame in &recent_frames {
        *counts.entry(frame.swap_chain).or_insert(0usize) += 1;
    }
    let chain = counts
        .into_iter()
        .max_by_key(|(chain, count)| (*count, Reverse(*chain)))
        .map(|(chain, _)| chain)
        .unwrap_or(0);
    let game_frames: Vec<_> = recent_frames
        .iter()
        .filter(|frame| frame.swap_chain == chain)
        .cloned()
        .collect();
    let game = game_frames
        .first()
        .map(|frame| frame.process_name.clone())
        .unwrap_or_default();
    if game_frames.len() < 2 {
        return Err(format!("Game '{}' has insufficient frames", game));
    }

    let frametime =
        game_frames.iter().map(|f| f.frametime_ms).sum::<f64>() / game_frames.len() as f64;
    let fps = 1000.0 / frametime;

    // 1% lows: the mean of the slowest 1% of the last 30 s.
    let mut game_frames_30s: Vec<_> = frames
        .iter()
        .filter(|f| {
            thirty_sec_ago.is_none_or(|start| f.timestamp >= start)
                && f.timestamp <= now
                && f.process_id == pid
                && f.swap_chain == chain
        })
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
    let mean_slowest_01p = slowest_point_one.iter().sum::<f64>() / slowest_point_one.len() as f64;
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

    // Conservative estimate: a wait-time comparison alone confuses a frame
    // cap with a saturated GPU. Without clear busy-time evidence, stay unknown.
    let limiter = if cpu_busys.len() >= 3 && gpu_busys.len() >= 3 {
        let cpu = median(&mut cpu_busys);
        let gpu = median(&mut gpu_busys);
        let wait = median(&mut cpu_waits).max(median(&mut gpu_waits));
        if cpu >= 0.9 * frametime && cpu > 1.1 * gpu {
            Some("cpu".into())
        } else if gpu >= 0.9 * frametime && gpu > 1.1 * cpu {
            Some("gpu".into())
        } else if cpu.max(gpu) < 0.8 * frametime && wait >= 0.2 * frametime {
            Some("cap".into())
        } else {
            None
        }
    } else {
        None
    };

    Ok(FrameMetrics {
        fps,
        frametime: Some(frametime),
        fps_low_1p: (game_frames_30s.len() >= 100).then_some(fps_low_1p),
        fps_low_01p: (game_frames_30s.len() >= 1000).then_some(fps_low_01p),
        fps_min: Some(fps_min),
        fps_max: Some(fps_max),
        source: Some("PresentMon".to_string()),
        game: Some(game),
        limiter,
    })
}

/// The middle value, safe for even and odd lengths; empty is 0.
fn median(values: &mut [f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    values.sort_by(|a, b| a.total_cmp(b));
    let mid = values.len() / 2;
    if values.len().is_multiple_of(2) {
        (values[mid - 1] + values[mid]) / 2.0
    } else {
        values[mid]
    }
}

/// True when `name` is Bezel's own executable: Bezel is never the game.
fn is_bezel_process(name: &str) -> bool {
    if let Ok(exe) = std::env::current_exe()
        && let Some(exe_name) = exe.file_name()
    {
        return exe_name.to_string_lossy().eq_ignore_ascii_case(name);
    }
    false
}

/// The name of the executable of the foreground window, or `None` when
/// Windows does not name one.
#[cfg(windows)]
fn foreground_target() -> Option<Target> {
    let hwnd = get_foreground_window()?;
    let pid = get_window_thread_process_id(hwnd)?;
    let handle = open_process_query(pid)?;
    let path = query_full_process_image_name(handle);
    close_handle(handle);
    let path = path?;
    let name = Path::new(&path).file_name()?.to_string_lossy().to_string();
    Some(Target { pid, name })
}

/// The name of the foreground process; no game can be detected without
/// Windows, so `None` keeps PresentMon stopped there (tests only).
#[cfg(not(windows))]
fn foreground_target() -> Option<Target> {
    None
}

// ---------------------------------------------------------------------------
// Win32 foreground-process detection. Small safe wrappers in the style of
// `fps/rtss`: the only `unsafe` is the FFI call itself, each with its
// `// SAFETY:` reasoning.
// ---------------------------------------------------------------------------

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
        GetWindowThreadProcessId(hwnd as windows_sys::Win32::Foundation::HANDLE, &mut pid);
        if pid == 0 { None } else { Some(pid) }
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
        CreateJobObjectW, JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE, JOBOBJECT_EXTENDED_LIMIT_INFORMATION,
        JobObjectExtendedLimitInformation, SetInformationJobObject,
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
#[allow(
    unsafe_code,
    reason = "Win32 FFI: closes the handles the other wrappers opened"
)]
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
        match name {
            "v2_header" => include_str!("fixtures/presentmon/v2_header.csv"),
            "old_header" => include_str!("fixtures/presentmon/old_header.csv"),
            "multi_process" => include_str!("fixtures/presentmon/multi_process.csv"),
            _ => panic!("unknown fixture"),
        }
        .to_string()
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
        assert!(
            parser
                .parse(lines.next().unwrap(), Instant::now())
                .is_none()
        );
        assert!(
            parser
                .parse(lines.next().unwrap(), Instant::now())
                .is_some()
        );
    }

    #[test]
    fn validates_frame_times() {
        let csv = "Process,FrameTime\ngame.exe,0.0\ngame.exe,6000.0\ngame.exe,16.6";
        let mut lines = csv.lines();
        let header = lines.next().unwrap();
        let parser = PresentMonParser::new(header).unwrap();

        assert!(
            parser
                .parse(lines.next().unwrap(), Instant::now())
                .is_none()
        ); // 0.0
        assert!(
            parser
                .parse(lines.next().unwrap(), Instant::now())
                .is_none()
        ); // 6000.0
        assert!(
            parser
                .parse(lines.next().unwrap(), Instant::now())
                .is_some()
        ); // 16.6
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

    fn frame(now: Instant, pid: u32, chain: u64, ms: f64) -> FrameData {
        FrameData {
            timestamp: now,
            process_name: format!("game-{pid}.exe"),
            process_id: pid,
            swap_chain: chain,
            qpc_ms: Some(1000.0),
            frametime_ms: ms,
            cpu_busy: None,
            cpu_wait: None,
            gpu_busy: None,
            gpu_wait: None,
        }
    }

    #[test]
    fn source_qpc_keeps_buffered_frames_old_instead_of_refreshing_them_on_drain() {
        let now = Instant::now();
        let parser = PresentMonParser::new(
            "Application,ProcessID,SwapChainAddress,CPUStartQPCTimeInMs,MsBetweenPresents",
        )
        .unwrap();
        let clock = CaptureClock {
            at: now,
            qpc_ms: 11_000.0,
        };
        let data = parser
            .source_frame("game.exe,42,0x1,1000,16.666", clock)
            .unwrap();
        assert_eq!(now.duration_since(data.timestamp), Duration::from_secs(10));
        let queue = VecDeque::from([data.clone(), data]);
        assert!(
            metrics(&queue, 42, now)
                .unwrap_err()
                .starts_with("fps.stale:")
        );
        assert!(
            parser
                .source_frame("game.exe,42,0x1,NaN,16.666", clock)
                .is_none()
        );
    }

    #[test]
    fn qpc_is_required_for_source_frames_and_present_interval_wins_over_cpu_time() {
        let clock = CaptureClock {
            at: Instant::now(),
            qpc_ms: 1000.0,
        };
        for qpc in ["CPUStartQPCTimeInMs", "CPUStartQPCTime"] {
            let parser = PresentMonParser::new(&format!(
                "Application,ProcessID,SwapChainAddress,{qpc},FrameTime,MsBetweenPresents"
            ))
            .unwrap();
            let data = parser
                .source_frame("game.exe,42,0x1,1000,33,16", clock)
                .unwrap();
            assert_eq!(data.timestamp, clock.at);
            assert_eq!(data.frametime_ms, 16.0);
            assert!(
                parser
                    .source_frame("game.exe,0,0x1,1000,33,16", clock)
                    .is_none()
            );
        }
        let parser =
            PresentMonParser::new("Application,ProcessID,SwapChainAddress,FrameTime").unwrap();
        assert!(parser.source_frame("game.exe,42,0x1,16", clock).is_none());
    }

    #[test]
    fn other_processes_and_secondary_swap_chains_cannot_inflate_the_game_fps() {
        let now = Instant::now();
        let mut frames = VecDeque::new();
        for _ in 0..20 {
            frames.push_back(frame(now, 42, 1, 20.0));
        }
        for _ in 0..10 {
            frames.push_back(frame(now, 42, 2, 1.0));
        }
        for _ in 0..100 {
            frames.push_back(frame(now, 99, 1, 1.0));
        }
        let m = metrics(&frames, 42, now).unwrap();
        assert_eq!(m.fps, 50.0);
        assert_eq!(m.game.as_deref(), Some("game-42.exe"));
        assert_eq!(m.fps_min, Some(50.0));
        assert!(metrics(&frames, 100, now).is_err());
    }

    #[test]
    fn lows_use_only_this_streams_thirty_second_history_and_warm_up_by_sample_count() {
        let now = Instant::now();
        let mut frames = VecDeque::new();
        for i in 0..999 {
            frames.push_back(frame(now - Duration::from_millis(i * 10), 42, 1, 10.0));
        }
        let warming = metrics(&frames, 42, now).unwrap();
        assert_eq!(warming.fps_low_1p, Some(100.0));
        assert_eq!(warming.fps_low_01p, None);
        frames.push_back(frame(now - Duration::from_secs(10), 42, 1, 100.0));
        frames.push_back(frame(now - Duration::from_secs(31), 42, 1, 1000.0));
        frames.push_back(frame(now, 99, 1, 1000.0));
        let m = metrics(&frames, 42, now).unwrap();
        assert!((m.fps_low_1p.unwrap() - 1000.0 / 19.0).abs() < 0.0001);
        assert_eq!(m.fps_low_01p, Some(10.0));
        assert_eq!((m.fps_min, m.fps_max), (Some(10.0), Some(100.0)));
    }

    #[test]
    fn limiter_is_only_an_estimate_when_busy_times_give_clear_evidence() {
        let now = Instant::now();
        for (cpu, gpu, wait, expected) in [
            (9.5, 3.0, 0.5, Some("cpu")),
            (3.0, 9.5, 0.5, Some("gpu")),
            (3.0, 3.0, 6.0, Some("cap")),
            (9.5, 9.5, 0.5, None),
            (3.0, 3.0, 0.0, None),
        ] {
            let mut data = frame(now, 42, 1, 10.0);
            data.cpu_busy = Some(cpu);
            data.gpu_busy = Some(gpu);
            data.cpu_wait = Some(wait);
            data.gpu_wait = Some(0.0);
            let m = metrics(&VecDeque::from(vec![data; 3]), 42, now).unwrap();
            assert_eq!(m.limiter.as_deref(), expected);
        }
    }

    #[test]
    fn capture_ownership_is_exclusive_and_can_be_released_on_another_thread() {
        let name = format!("Global\\BezelFPS-unit-{}", std::process::id());
        let lease = CaptureLease::named(&name).unwrap();
        assert!(CaptureLease::named(&name).is_err());
        std::thread::spawn(move || drop(lease)).join().unwrap();
        drop(CaptureLease::named(&name).unwrap());
    }

    struct WorkingRtss;
    impl FrameRateSource for WorkingRtss {
        fn describe(&self) -> String {
            "RTSS test".into()
        }
        fn read(&mut self) -> Result<FrameMetrics, String> {
            Ok(FrameMetrics {
                fps: 75.0,
                frametime: None,
                fps_low_1p: None,
                fps_low_01p: None,
                fps_min: None,
                fps_max: None,
                source: Some("RTSS".into()),
                game: None,
                limiter: None,
            })
        }
    }

    #[test]
    fn a_child_that_exits_after_spawn_immediately_uses_rtss_and_keeps_retry_backoff() {
        use std::os::windows::process::CommandExt;
        use windows_sys::Win32::System::Threading::CREATE_NO_WINDOW;
        // A completed collector must fall back even if it exits successfully.
        // Reuse the test harness as an owned child; no shell/profile dependency.
        let mut child = Command::new(std::env::current_exe().unwrap())
            .arg("--list")
            .creation_flags(CREATE_NO_WINDOW)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        let status = child.wait().unwrap();
        assert!(status.success());
        let now = Instant::now();
        let mut source = PresentMon::new(None);
        source.is_wanted = true;
        source.child = Some(child);
        source.state = State::Running;
        source.rtss_fallback = Some(Box::new(WorkingRtss));
        assert_eq!(source.read_target(None, now).unwrap().fps, 75.0);
        assert!(matches!(source.state, State::BackingOff(at) if at == now + BACKOFF_INITIAL));
        source.fail("crash again".into(), now, false);
        assert!(
            matches!(source.state, State::BackingOff(at) if at == now + Duration::from_secs(4))
        );
        source.want(false);
        assert!(source.child.is_none());
        assert!(source.read_target(None, now).is_err());
        source.want(true);
        assert_eq!(source.backoff_duration, BACKOFF_INITIAL);
    }

    #[test]
    fn discovery_prefers_numeric_versions_and_ignores_directories() {
        let root = tempfile::tempdir().unwrap();
        for file in ["PresentMon-2.9.0-x64.exe", "PresentMon-2.10.0-x64.exe"] {
            std::fs::write(root.path().join(file), []).unwrap();
        }
        std::fs::create_dir(root.path().join("PresentMon.exe")).unwrap();
        assert_eq!(
            PresentMon::find_in_dir(root.path())
                .unwrap()
                .file_name()
                .unwrap(),
            "PresentMon-2.10.0-x64.exe"
        );
        let explicit_directory = PresentMon::new(Some(root.path().to_path_buf()));
        assert!(explicit_directory.find_executable().is_none());
    }

    #[test]
    fn overflow_keeps_fps_fresh_but_withholds_incomplete_long_window_statistics() {
        let now = Instant::now();
        let mut source = PresentMon::new(None);
        source.is_wanted = true;
        source.state = State::Running;
        source.target = Some(Target {
            pid: 42,
            name: "game.exe".into(),
        });
        source
            .lines
            .lock()
            .unwrap()
            .extend(vec![frame(now, 42, 1, 10.0); 1000]);
        source.overflowed.store(true, Ordering::Relaxed);
        let m = source.read_target(None, now).unwrap();
        assert_eq!(m.fps, 100.0);
        assert_eq!(
            (m.fps_low_1p, m.fps_low_01p, m.fps_min, m.fps_max),
            (None, None, None, None)
        );
        source.statistics_after = Some(now - Duration::from_secs(1));
        let resumed = source.read_target(None, now).unwrap();
        assert_eq!(resumed.fps_low_01p, Some(100.0));
        assert_eq!(resumed.fps_min, Some(100.0));
    }
}
