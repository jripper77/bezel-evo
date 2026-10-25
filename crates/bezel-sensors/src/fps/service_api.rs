//! PresentMon 2.6 / API 3.4 frame events. Only the installed service owns ETW.
//! ABI declarations follow Intel's MIT-licensed PresentMonAPI.h (v2.6.0).
#![allow(
    unsafe_code,
    reason = "Isolated Windows service discovery and owned PresentMon C ABI"
)]
use super::presentmon::{CaptureClock, FrameData, Target, capture_clock};
use std::path::{Path, PathBuf};
use windows_sys::Win32::Foundation::FreeLibrary;
use windows_sys::Win32::System::{
    LibraryLoader::{
        GetProcAddress, LOAD_LIBRARY_SEARCH_DEFAULT_DIRS, LOAD_LIBRARY_SEARCH_DLL_LOAD_DIR,
        LoadLibraryExW,
    },
    Performance::QueryPerformanceFrequency,
    Services::{
        CloseServiceHandle, OpenSCManagerW, OpenServiceW, QUERY_SERVICE_CONFIGW,
        QueryServiceConfigW, SC_MANAGER_CONNECT, SERVICE_QUERY_CONFIG,
    },
};

const CAPACITY: usize = 512;
// PM_METRIC_{SWAP_CHAIN_ADDRESS,CPU_START_QPC,BETWEEN_PRESENTS,CPU_BUSY,
// CPU_WAIT,GPU_BUSY,GPU_WAIT} in the matching API 3.4 header.
const METRICS: [i32; 7] = [1, 7, 78, 9, 10, 14, 15];
#[repr(C)]
#[derive(Clone, Copy, Default, Debug)]
struct Element {
    metric: i32,
    stat: i32,
    device: u32,
    index: u32,
    offset: u64,
    size: u64,
}
#[repr(C)]
#[derive(Default)]
struct Version {
    major: u16,
    minor: u16,
    patch: u16,
    tag: [u8; 22],
    hash: [u8; 8],
    config: [u8; 4],
}
type Status = i32;
type GetVersion = unsafe extern "C" fn(*mut Version) -> Status;
type Open = unsafe extern "C" fn(*mut isize) -> Status;
type Close = unsafe extern "C" fn(isize) -> Status;
type Track = unsafe extern "C" fn(isize, u32) -> Status;
type Register = unsafe extern "C" fn(isize, *mut isize, *mut Element, u64, *mut u32) -> Status;
type Consume = unsafe extern "C" fn(isize, u32, *mut u8, *mut u32) -> Status;
struct Library {
    handle: isize,
    version: GetVersion,
    open: Open,
    close: Close,
    start: Track,
    stop: Track,
    flush: Track,
    register: Register,
    consume: Consume,
    free: Close,
}
impl Drop for Library {
    fn drop(&mut self) {
        // SAFETY: load transferred this module reference; clients close before it.
        unsafe { FreeLibrary(self.handle as _) };
    }
}
fn checked(code: Status, operation: &str) -> Result<(), String> {
    if code == 0 {
        Ok(())
    } else {
        Err(format!(
            "PresentMon service {operation} failed (API status {code})"
        ))
    }
}
impl Library {
    fn load(path: &Path) -> Result<Self, String> {
        use std::os::windows::ffi::OsStrExt;
        let path: Vec<u16> = path.as_os_str().encode_wide().chain(Some(0)).collect();
        // SAFETY: absolute installed-service sibling, terminated path, restricted DLL search.
        let module = unsafe {
            LoadLibraryExW(
                path.as_ptr(),
                std::ptr::null_mut(),
                LOAD_LIBRARY_SEARCH_DEFAULT_DIRS | LOAD_LIBRARY_SEARCH_DLL_LOAD_DIR,
            )
        };
        if module.is_null() {
            return Err(
                "fps.setup:PresentMon service API is not installed or could not load".into(),
            );
        }
        macro_rules! bind {
            ($name:literal, $ty:ty) => {{
                // SAFETY: exact exported names and API 3.4 C signatures from Intel's header.
                match unsafe { GetProcAddress(module, concat!($name, "\0").as_ptr()) } {
                    Some(symbol) => unsafe {
                        std::mem::transmute::<unsafe extern "system" fn() -> isize, $ty>(symbol)
                    },
                    None => {
                        unsafe { FreeLibrary(module) };
                        return Err(concat!("fps.setup:PresentMon API missing ", $name).into());
                    }
                }
            }};
        }
        let library = Self {
            handle: module as isize,
            version: bind!("pmGetApiVersion", GetVersion),
            open: bind!("pmOpenSession", Open),
            close: bind!("pmCloseSession", Close),
            start: bind!("pmStartTrackingProcess", Track),
            stop: bind!("pmStopTrackingProcess", Track),
            flush: bind!("pmSetEtwFlushPeriod", Track),
            register: bind!("pmRegisterFrameQuery", Register),
            consume: bind!("pmConsumeFrames", Consume),
            free: bind!("pmFreeFrameQuery", Close),
        };
        let mut version = Version::default();
        // SAFETY: live module, correctly sized writable C struct.
        checked(unsafe { (library.version)(&mut version) }, "version")?;
        if (version.major, version.minor) != (3, 4) {
            return Err(format!(
                "fps.setup:PresentMon API {}.{} is unsupported; expected 3.4",
                version.major, version.minor
            ));
        }
        Ok(library)
    }
}

/// The service configuration is administrator-controlled. Never load from PATH,
/// the current directory, or a path supplied by a game.
pub(super) fn installed_api() -> Result<PathBuf, String> {
    struct ServiceHandle(isize);
    impl Drop for ServiceHandle {
        fn drop(&mut self) {
            unsafe { CloseServiceHandle(self.0 as _) };
        }
    }
    // SAFETY: local SCM, query-only access; RAII closes both handles.
    let manager = unsafe { OpenSCManagerW(std::ptr::null(), std::ptr::null(), SC_MANAGER_CONNECT) };
    if manager.is_null() {
        return Err("fps.setup:Cannot query Windows service manager".into());
    }
    let manager = ServiceHandle(manager as isize);
    let name: Vec<u16> = "PresentMonSharedService"
        .encode_utf16()
        .chain(Some(0))
        .collect();
    let service = unsafe { OpenServiceW(manager.0 as _, name.as_ptr(), SERVICE_QUERY_CONFIG) };
    if service.is_null() {
        return Err("fps.setup:Install the game FPS support component (PresentMon service)".into());
    }
    let service = ServiceHandle(service as isize);
    let mut needed = 0;
    unsafe { QueryServiceConfigW(service.0 as _, std::ptr::null_mut(), 0, &mut needed) };
    if !(std::mem::size_of::<QUERY_SERVICE_CONFIGW>()..=65536).contains(&(needed as usize)) {
        return Err("fps.setup:Invalid PresentMon service configuration size".into());
    }
    let mut storage = vec![0u64; (needed as usize).div_ceil(8)];
    let config = storage.as_mut_ptr().cast::<QUERY_SERVICE_CONFIGW>();
    // SAFETY: aligned buffer large enough for the API-reported configuration.
    if unsafe { QueryServiceConfigW(service.0 as _, config, needed, &mut needed) } == 0 {
        return Err("fps.setup:Cannot read PresentMon service configuration".into());
    }
    let pointer = unsafe { (*config).lpBinaryPathName };
    if pointer.is_null() {
        return Err("fps.setup:Missing PresentMon service executable".into());
    }
    let begin = storage.as_ptr() as usize;
    let end = begin + storage.len() * 8;
    let address = pointer as usize;
    if address < begin || address >= end || !address.is_multiple_of(std::mem::align_of::<u16>()) {
        return Err("fps.setup:Invalid service path pointer".into());
    }
    let capacity = (end - address) / 2;
    // SAFETY: pointer was checked to be within the configuration buffer.
    let text = unsafe { std::slice::from_raw_parts(pointer, capacity) };
    let length = text
        .iter()
        .position(|ch| *ch == 0)
        .ok_or("fps.setup:Unterminated service path")?;
    let executable = executable_path(&String::from_utf16_lossy(&text[..length]))?;
    Ok(executable
        .parent()
        .ok_or("fps.setup:Invalid service directory")?
        .join("PresentMonAPI2.dll"))
}
fn executable_path(command: &str) -> Result<PathBuf, String> {
    let command = command.trim();
    let path = if let Some(quoted) = command.strip_prefix('"') {
        quoted.split_once('"').map(|(path, _)| path)
    } else {
        command
            .to_ascii_lowercase()
            .find(".exe")
            .map(|at| &command[..at + 4])
    }
    .ok_or("fps.setup:Invalid PresentMon service executable path")?;
    let path = PathBuf::from(path);
    if !path.is_absolute()
        || !path
            .file_name()
            .is_some_and(|name| name.eq_ignore_ascii_case("PresentMonService.exe"))
    {
        return Err("fps.setup:Unexpected PresentMon service executable".into());
    }
    Ok(path)
}
fn validate_layout(elements: &[Element], stride: usize) -> Result<(), String> {
    let mut ranges = Vec::new();
    if !(8..=4096).contains(&stride) {
        return Err("Invalid PresentMon frame-query stride".into());
    }
    for element in elements {
        let offset = usize::try_from(element.offset).map_err(|_| "Invalid frame offset")?;
        if element.size != 8
            || offset.checked_add(8).is_none_or(|end| end > stride)
            || ranges
                .iter()
                .any(|&(start, end)| offset < end && offset + 8 > start)
        {
            return Err("Invalid PresentMon frame-query layout".into());
        }
        ranges.push((offset, offset + 8));
    }
    Ok(())
}

pub(super) struct Client {
    #[cfg(test)]
    raw_frames: usize,
    #[cfg(test)]
    first_values: Option<[[u8; 8]; 7]>,
    library: Library,
    session: isize,
    query: isize,
    tracked: bool,
    target: Target,
    elements: [Element; 7],
    stride: usize,
    blob: Vec<u8>,
    clock: CaptureClock,
    frequency: f64,
}
pub(super) struct Batch {
    pub frames: Vec<FrameData>,
    pub saturated: bool,
}
impl Client {
    pub fn open(target: Target) -> Result<Self, String> {
        let library = Library::load(&installed_api()?)?;
        let mut frequency = 0i64;
        if unsafe { QueryPerformanceFrequency(&mut frequency) } == 0 || frequency <= 0 {
            return Err("Cannot read the frame source clock".into());
        }
        let mut client = Self {
            #[cfg(test)]
            raw_frames: 0,
            #[cfg(test)]
            first_values: None,
            library,
            session: 0,
            query: 0,
            tracked: false,
            target,
            elements: METRICS.map(|metric| Element {
                metric,
                ..Default::default()
            }),
            stride: 0,
            blob: Vec::new(),
            clock: capture_clock()?,
            frequency: frequency as f64,
        };
        // SAFETY: all handles and query elements are initialized and owned by this client.
        unsafe {
            checked((client.library.open)(&mut client.session), "connection")?;
            checked((client.library.flush)(client.session, 100), "flush period")?;
            let mut size = 0;
            checked(
                (client.library.register)(
                    client.session,
                    &mut client.query,
                    client.elements.as_mut_ptr(),
                    7,
                    &mut size,
                ),
                "frame query",
            )?;
            client.stride = size as usize;
            checked(
                (client.library.start)(client.session, client.target.pid),
                "tracking",
            )?;
            client.tracked = true;
        }
        validate_layout(&client.elements, client.stride)?;
        client.blob.resize(client.stride * CAPACITY, 0);
        Ok(client)
    }
    pub fn consume(&mut self) -> Result<Batch, String> {
        let mut batch = Batch {
            frames: Vec::new(),
            saturated: false,
        };
        for round in 0..16 {
            let mut count = CAPACITY as u32;
            // SAFETY: validated stride, sufficient blob capacity, correct live query/PID.
            checked(
                unsafe {
                    (self.library.consume)(
                        self.query,
                        self.target.pid,
                        self.blob.as_mut_ptr(),
                        &mut count,
                    )
                },
                "frame consumption",
            )?;
            if count as usize > CAPACITY {
                return Err("PresentMon exceeded frame-query capacity".into());
            }
            #[cfg(test)]
            {
                self.raw_frames += count as usize;
            }
            for row in self.blob[..count as usize * self.stride].chunks_exact(self.stride) {
                let values: [[u8; 8]; 7] = std::array::from_fn(|i| {
                    let offset = self.elements[i].offset as usize;
                    // Layout validation above guarantees these eight bytes.
                    let mut value = [0; 8];
                    value.copy_from_slice(&row[offset..offset + 8]);
                    value
                });
                let chain = u64::from_ne_bytes(values[0]);
                #[cfg(test)]
                {
                    if self.first_values.is_none() {
                        self.first_values = Some(values);
                    }
                }
                let qpc = u64::from_ne_bytes(values[1]);
                let interval = f64::from_ne_bytes(values[2]);
                let qpc_ms = qpc as f64 * 1000.0 / self.frequency;
                if chain == 0
                    || qpc == 0
                    || !interval.is_finite()
                    || interval <= 0.0
                    || interval >= 5000.0
                {
                    continue;
                }
                let Some(timestamp) = self.clock.frame_time(qpc_ms) else {
                    continue;
                };
                let optional = |index| {
                    let value = f64::from_ne_bytes(values[index]);
                    (value.is_finite() && value >= 0.0).then_some(value)
                };
                batch.frames.push(FrameData {
                    timestamp,
                    process_name: self.target.name.clone(),
                    process_id: self.target.pid,
                    swap_chain: chain,
                    qpc_ms: Some(qpc_ms),
                    frametime_ms: interval,
                    cpu_busy: optional(3),
                    cpu_wait: optional(4),
                    gpu_busy: optional(5),
                    gpu_wait: optional(6),
                });
            }
            if count < CAPACITY as u32 {
                break;
            }
            batch.saturated = round == 15;
        }
        Ok(batch)
    }
}
impl Drop for Client {
    fn drop(&mut self) {
        // SAFETY: handles belong only to this client; the module outlives this body.
        unsafe {
            if self.query != 0 {
                (self.library.free)(self.query);
            }
            if self.tracked {
                (self.library.stop)(self.session, self.target.pid);
            }
            if self.session != 0 {
                (self.library.close)(self.session);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    #[ignore = "requires installed FPS service and an explicitly supplied owned graphics workload PID"]
    fn installed_service_streams_fresh_frames_without_elevation() {
        use std::{
            collections::VecDeque,
            time::{Duration, Instant},
        };
        let pid: u32 = std::env::var("BEZEL_FPS_TEST_PID")
            .unwrap()
            .parse()
            .unwrap();
        let mut client = Client::open(Target {
            pid,
            name: "owned-fps-workload.exe".into(),
        })
        .unwrap();
        let deadline = Instant::now() + Duration::from_secs(18);
        let mut frames = VecDeque::new();
        let mut rates = Vec::new();
        let mut stale_after_frames = false;
        let mut first_age_ms = None;
        while Instant::now() < deadline {
            match client.consume() {
                Ok(batch) => {
                    for frame in batch.frames {
                        if first_age_ms.is_none() {
                            first_age_ms = Some(
                                Instant::now()
                                    .saturating_duration_since(frame.timestamp)
                                    .as_secs_f64()
                                    * 1000.0,
                            );
                        }
                        assert_eq!(frame.process_id, pid);
                        assert!(frame.swap_chain != 0 && frame.frametime_ms.is_finite());
                        frames.push_back(frame);
                    }
                }
                Err(error) => {
                    assert!(
                        !frames.is_empty(),
                        "service failed before producing frames: {error}"
                    );
                }
            }
            match super::super::presentmon::metrics(&frames, pid, Instant::now()) {
                Ok(value) => rates.push(value.fps),
                Err(_)
                    if frames
                        .iter()
                        .map(|frame| frame.timestamp)
                        .max()
                        .is_some_and(|last| {
                            Instant::now().saturating_duration_since(last) > super::super::MAX_AGE
                        }) =>
                {
                    stale_after_frames = true;
                }
                Err(_) => {}
            }
            std::thread::sleep(Duration::from_millis(100));
        }
        println!(
            "frame ABI evidence: raw={} decoded={} layout={:?} first_u64={:?} first_f64={:?}",
            client.raw_frames,
            frames.len(),
            client.elements,
            client.first_values.map(|row| row.map(u64::from_ne_bytes)),
            client.first_values.map(|row| row.map(f64::from_ne_bytes))
        );
        assert!(frames.len() >= 100, "insufficient real frame events");
        assert!(
            rates.iter().any(|fps| *fps < 45.0) && rates.iter().any(|fps| *fps > 50.0),
            "60/30/60 workload rate changes missing"
        );
        assert!(
            stale_after_frames,
            "ended workload must expire instead of holding its FPS"
        );
        println!(
            "service frame-query proof: frames={} first_age_ms={first_age_ms:?} rate_min={:?} rate_max={:?} expired={stale_after_frames}",
            frames.len(),
            rates.iter().copied().reduce(f64::min),
            rates.iter().copied().reduce(f64::max)
        );
    }
    #[test]
    fn c_layout_and_blob_bounds_match_the_supported_sdk() {
        assert_eq!(std::mem::size_of::<Element>(), 32);
        assert_eq!(std::mem::size_of::<Version>(), 40);
        let mut elements: Vec<_> = (0..7)
            .map(|index| Element {
                offset: index * 8,
                size: 8,
                ..Default::default()
            })
            .collect();
        assert!(validate_layout(&elements, 56).is_ok());
        assert!(validate_layout(&elements, 55).is_err());
        elements[1].offset = 0;
        assert!(validate_layout(&elements, 56).is_err());
        elements[1].offset = u64::MAX;
        assert!(validate_layout(&elements, 56).is_err());
    }
    #[test]
    fn service_configuration_accepts_only_an_absolute_presentmon_executable() {
        assert_eq!(executable_path(r#""C:\Program Files\Intel\PresentMonSharedService\PresentMonService.exe" --etw-session-name test"#).unwrap(), PathBuf::from(r"C:\Program Files\Intel\PresentMonSharedService\PresentMonService.exe"));
        assert!(executable_path(r"cmd.exe /c anything").is_err());
        assert!(executable_path(r"PresentMonService.exe").is_err());
    }
}
