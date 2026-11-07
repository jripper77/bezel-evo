//! The computer's monitors and sleep, as Windows reports them
//! (D-2026-11-06-display-idle-saver-2): [`watch`] registers a callback for
//! `GUID_CONSOLE_DISPLAY_STATE` (monitors off, on, dimmed) and for the
//! suspend/resume notifications, without a window, and hands every change
//! to the caller as a [`PowerEvent`]. What to do with it is the core's
//! `IdleState`.
//!
//! [`forwarder`] is the bridge both Bezel Light and Studio put between
//! Windows' thread and their own worker: every event is queued, and the
//! sleep waits for the worker to apply it, [`SUSPEND_WAIT`] at most.

use std::io;
use std::sync::mpsc;
use std::time::Duration;

use bezel_core::domain::idle::PowerEvent;

/// What Windows reports in a power notification: its kind (`PBT_*`) and,
/// for a power setting change of the console display state, the state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Notification {
    /// The `PBT_*` kind of the notification.
    pub kind: u32,
    /// The console display state (0 off, 1 on, 2 dimmed), only for a
    /// `PBT_POWERSETTINGCHANGE` of `GUID_CONSOLE_DISPLAY_STATE`.
    pub display: Option<u32>,
}

/// `PBT_APMSUSPEND`: the computer is going to sleep.
const PBT_APMSUSPEND: u32 = 4;
/// `PBT_APMRESUMESUSPEND`: the computer woke up and the user is there.
const PBT_APMRESUMESUSPEND: u32 = 7;
/// `PBT_APMRESUMEAUTOMATIC`: the computer woke up (always sent).
const PBT_APMRESUMEAUTOMATIC: u32 = 18;
/// `PBT_POWERSETTINGCHANGE`: a registered power setting changed.
const PBT_POWERSETTINGCHANGE: u32 = 0x8013;

/// The [`PowerEvent`] a notification means, if any. Both resume kinds map
/// to [`PowerEvent::Resume`]: Windows sends the automatic one always and
/// the other one too when the user is there; `IdleState` ignores the
/// second.
pub fn event_of(notification: Notification) -> Option<PowerEvent> {
    match notification.kind {
        PBT_APMSUSPEND => Some(PowerEvent::Suspend),
        PBT_APMRESUMEAUTOMATIC | PBT_APMRESUMESUSPEND => Some(PowerEvent::Resume),
        PBT_POWERSETTINGCHANGE => match notification.display? {
            0 => Some(PowerEvent::DisplayOff),
            1 => Some(PowerEvent::DisplayOn),
            2 => Some(PowerEvent::DisplayDimmed),
            _ => None,
        },
        _ => None,
    }
}

/// The registrations of [`watch`]: dropping it unregisters both and frees
/// the handler.
pub struct Watch {
    _registrations: imp::Registrations,
}

impl std::fmt::Debug for Watch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Watch").finish_non_exhaustive()
    }
}

/// Calls `handler` with every change of the monitors' state and every sleep
/// and wake of the computer, until the returned [`Watch`] is dropped.
///
/// Windows reports the current monitors' state right after the
/// registration (usually [`PowerEvent::DisplayOn`]).
///
/// The handler runs on a thread of Windows. For [`PowerEvent::Suspend`]
/// Windows waits for it to return before sleeping: the handler has a budget
/// of 2 s for it (apply the shutdown choice, stop the capture) and must not
/// block longer. The other events must not block at all: hand them to a
/// worker. A panic in the handler is caught and logged.
pub fn watch(handler: impl Fn(PowerEvent) + Send + Sync + 'static) -> io::Result<Watch> {
    let registrations = imp::register(Box::new(Box::new(handler)))?;
    Ok(Watch {
        _registrations: registrations,
    })
}

/// How long Windows waits for the handler of a [`PowerEvent::Suspend`]
/// before the computer sleeps anyway.
pub const SUSPEND_WAIT: Duration = Duration::from_secs(2);

/// The part of [`SUSPEND_WAIT`] the work done at sleep (the shutdown
/// choices, stopping the runtime) may use: the rest is the bridge's margin.
pub const SUSPEND_BUDGET: Duration = Duration::from_millis(1800);

/// The worker's answer to a [`PowerEvent::Suspend`]: one `()` once it is
/// applied.
pub type SuspendDone = mpsc::SyncSender<()>;

/// How [`forward`] handed an event to the worker.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Forwarded {
    /// Not a sleep: queued, not waited for.
    Queued,
    /// A sleep, and the worker said it is applied.
    Applied,
    /// A sleep the worker did not apply within the wait.
    TimedOut,
    /// The worker is gone (its channel closed, or it dropped the answer,
    /// as a panic does): nothing waits for it.
    Dropped,
}

/// Queues `event` for the worker behind `events` as `message` makes it;
/// for a [`PowerEvent::Suspend`] the message carries a [`SuspendDone`] and
/// this waits for it, `wait` at most.
pub fn forward<M>(
    events: &mpsc::Sender<M>,
    event: PowerEvent,
    message: impl FnOnce(PowerEvent, Option<SuspendDone>) -> M,
    wait: Duration,
) -> Forwarded {
    if event != PowerEvent::Suspend {
        return match events.send(message(event, None)) {
            Ok(()) => Forwarded::Queued,
            Err(_) => Forwarded::Dropped,
        };
    }
    let (done, applied) = mpsc::sync_channel(1);
    if events.send(message(event, Some(done))).is_err() {
        return Forwarded::Dropped;
    }
    match applied.recv_timeout(wait) {
        Ok(()) => Forwarded::Applied,
        Err(mpsc::RecvTimeoutError::Timeout) => Forwarded::TimedOut,
        Err(mpsc::RecvTimeoutError::Disconnected) => Forwarded::Dropped,
    }
}

/// The handler for [`watch`] that hands every event to the worker behind
/// `events` ([`forward`]), the sleep waited for [`SUSPEND_WAIT`] at most.
pub fn forwarder<M: Send + 'static>(
    events: mpsc::Sender<M>,
    message: impl Fn(PowerEvent, Option<SuspendDone>) -> M + Send + Sync + 'static,
) -> impl Fn(PowerEvent) + Send + Sync + 'static {
    move |event| {
        let _ = forward(&events, event, &message, SUSPEND_WAIT);
    }
}

/// The handler as [`watch`] keeps it: boxed twice so that the callback's
/// context is a thin pointer.
type Handler = Box<dyn Fn(PowerEvent) + Send + Sync>;

/// Hands `notification` to `handler`, catching a panic: it must never
/// unwind into Windows.
fn deliver(handler: &Handler, notification: Notification) {
    let Some(event) = event_of(notification) else {
        return;
    };
    let delivered = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| handler(event)));
    if delivered.is_err() {
        tracing::error!(?event, "the power event handler panicked");
    }
}

mod imp {
    use std::ffi::c_void;
    use std::io;

    use windows_sys::Win32::Foundation::{ERROR_SUCCESS, HANDLE};
    use windows_sys::Win32::System::Power::{
        DEVICE_NOTIFY_SUBSCRIBE_PARAMETERS, POWERBROADCAST_SETTING,
        PowerRegisterSuspendResumeNotification, PowerSettingRegisterNotification,
        PowerSettingUnregisterNotification, PowerUnregisterSuspendResumeNotification,
    };
    use windows_sys::Win32::System::SystemServices::GUID_CONSOLE_DISPLAY_STATE;
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        DEVICE_NOTIFY_CALLBACK, PBT_POWERSETTINGCHANGE,
    };
    use windows_sys::core::GUID;

    use super::{Handler, Notification, deliver};

    /// Both registrations, the parameters they point to and the handler.
    pub(super) struct Registrations {
        display: isize,
        suspend: isize,
        _params: Box<DEVICE_NOTIFY_SUBSCRIBE_PARAMETERS>,
        _handler: Box<Handler>,
    }

    // SAFETY: the raw pointers inside the parameters point to the handler,
    // which is `Send + Sync` and owned by the same value; nothing reads them
    // through `&self`, and Windows calls the handler from its own threads
    // anyway.
    #[allow(unsafe_code, reason = "the registration owns a Send + Sync handler")]
    unsafe impl Send for Registrations {}
    // SAFETY: as for `Send`: no method reads the fields through `&self`.
    #[allow(unsafe_code, reason = "the registration owns a Send + Sync handler")]
    unsafe impl Sync for Registrations {}

    #[allow(
        unsafe_code,
        reason = "Win32 FFI: the callback receives Windows' power notifications"
    )]
    unsafe extern "system" fn callback(
        context: *const c_void,
        kind: u32,
        setting: *const c_void,
    ) -> u32 {
        if context.is_null() {
            return ERROR_SUCCESS;
        }
        // SAFETY: `context` is the address of the `Handler` that
        // `Registrations` owns; the registrations end in its `Drop` before
        // the handler is freed.
        let handler = unsafe { &*context.cast::<Handler>() };
        // SAFETY: for `PBT_POWERSETTINGCHANGE`, Windows passes a valid
        // `POWERBROADCAST_SETTING` for the duration of the call.
        let display = unsafe { display_state(kind, setting) };
        deliver(handler, Notification { kind, display });
        ERROR_SUCCESS
    }

    /// The console display state in `setting`, if it is one.
    ///
    /// # Safety
    /// For `PBT_POWERSETTINGCHANGE`, `setting` is null or points to a
    /// `POWERBROADCAST_SETTING` followed by `DataLength` bytes.
    #[allow(unsafe_code, reason = "Win32 FFI: reads the setting Windows passes")]
    unsafe fn display_state(kind: u32, setting: *const c_void) -> Option<u32> {
        if kind != PBT_POWERSETTINGCHANGE || setting.is_null() {
            return None;
        }
        let setting = setting.cast::<POWERBROADCAST_SETTING>();
        // SAFETY: the caller guarantees a valid header; it may be unaligned.
        let header = unsafe { setting.read_unaligned() };
        if !is_display_state(&header.PowerSetting) || header.DataLength < 4 {
            return None;
        }
        // SAFETY: `Data` holds `DataLength` (>= 4) bytes, read unaligned.
        let data = unsafe {
            std::ptr::addr_of!((*setting).Data)
                .cast::<u32>()
                .read_unaligned()
        };
        Some(data)
    }

    /// `guid` is `GUID_CONSOLE_DISPLAY_STATE` (`GUID` has no `PartialEq`).
    fn is_display_state(guid: &GUID) -> bool {
        let wanted = GUID_CONSOLE_DISPLAY_STATE;
        (guid.data1, guid.data2, guid.data3, guid.data4)
            == (wanted.data1, wanted.data2, wanted.data3, wanted.data4)
    }

    #[allow(
        unsafe_code,
        reason = "Win32 FFI: only these calls register power notifications without a window"
    )]
    pub(super) fn register(handler: Box<Handler>) -> io::Result<Registrations> {
        let params = Box::new(DEVICE_NOTIFY_SUBSCRIBE_PARAMETERS {
            Callback: Some(callback),
            Context: std::ptr::from_ref::<Handler>(&handler).cast_mut().cast(),
        });
        let recipient: HANDLE = std::ptr::from_ref(&*params).cast_mut().cast();
        let mut display: *mut c_void = std::ptr::null_mut();
        // SAFETY: the GUID is a constant, `recipient` points to parameters
        // that live as long as the registration, `display` is a valid out
        // pointer.
        let status = unsafe {
            PowerSettingRegisterNotification(
                &GUID_CONSOLE_DISPLAY_STATE,
                DEVICE_NOTIFY_CALLBACK,
                recipient,
                &mut display,
            )
        };
        if status != ERROR_SUCCESS {
            return Err(io::Error::from_raw_os_error(status as i32));
        }
        let mut suspend: *mut c_void = std::ptr::null_mut();
        // SAFETY: as above.
        let status = unsafe {
            PowerRegisterSuspendResumeNotification(DEVICE_NOTIFY_CALLBACK, recipient, &mut suspend)
        };
        if status != ERROR_SUCCESS {
            // SAFETY: `display` was registered just above and not yet freed.
            unsafe { PowerSettingUnregisterNotification(display as isize) };
            return Err(io::Error::from_raw_os_error(status as i32));
        }
        Ok(Registrations {
            display: display as isize,
            suspend: suspend as isize,
            _params: params,
            _handler: handler,
        })
    }

    impl Drop for Registrations {
        #[allow(
            unsafe_code,
            reason = "Win32 FFI: ends the registrations of `register`"
        )]
        fn drop(&mut self) {
            // SAFETY: both handles come from a successful registration and
            // are freed once, here, before the parameters and the handler.
            let display_status = unsafe { PowerSettingUnregisterNotification(self.display) };
            // SAFETY: as above.
            let suspend_status = unsafe { PowerUnregisterSuspendResumeNotification(self.suspend) };
            if display_status != ERROR_SUCCESS || suspend_status != ERROR_SUCCESS {
                tracing::warn!(
                    display_status,
                    suspend_status,
                    "unregistering the power notifications failed"
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    use super::*;

    fn display(state: u32) -> Notification {
        Notification {
            kind: PBT_POWERSETTINGCHANGE,
            display: Some(state),
        }
    }

    fn kind(kind: u32) -> Notification {
        Notification {
            kind,
            display: None,
        }
    }

    #[test]
    fn idle_event_maps_display_states() {
        assert_eq!(event_of(display(0)), Some(PowerEvent::DisplayOff));
        assert_eq!(event_of(display(1)), Some(PowerEvent::DisplayOn));
        assert_eq!(event_of(display(2)), Some(PowerEvent::DisplayDimmed));
        assert_eq!(event_of(display(3)), None);
        assert_eq!(event_of(kind(PBT_POWERSETTINGCHANGE)), None);
    }

    #[test]
    fn idle_event_maps_suspend_and_resume() {
        assert_eq!(event_of(kind(PBT_APMSUSPEND)), Some(PowerEvent::Suspend));
        assert_eq!(
            event_of(kind(PBT_APMRESUMEAUTOMATIC)),
            Some(PowerEvent::Resume)
        );
        assert_eq!(
            event_of(kind(PBT_APMRESUMESUSPEND)),
            Some(PowerEvent::Resume)
        );
        assert_eq!(event_of(kind(10)), None);
    }

    #[test]
    fn idle_constants_match_windows() {
        use windows_sys::Win32::UI::WindowsAndMessaging as w;
        assert_eq!(PBT_APMSUSPEND, w::PBT_APMSUSPEND);
        assert_eq!(PBT_APMRESUMESUSPEND, w::PBT_APMRESUMESUSPEND);
        assert_eq!(PBT_APMRESUMEAUTOMATIC, w::PBT_APMRESUMEAUTOMATIC);
        assert_eq!(PBT_POWERSETTINGCHANGE, w::PBT_POWERSETTINGCHANGE);
    }

    #[test]
    fn idle_handler_panic_is_caught() {
        let calls = Arc::new(AtomicUsize::new(0));
        let seen = Arc::clone(&calls);
        let handler: Handler = Box::new(move |_| {
            seen.fetch_add(1, Ordering::SeqCst);
            panic!("handler failure");
        });
        deliver(&handler, kind(PBT_APMSUSPEND));
        deliver(&handler, kind(10));
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }

    type Message = (PowerEvent, Option<SuspendDone>);

    fn message(event: PowerEvent, done: Option<SuspendDone>) -> Message {
        (event, done)
    }

    /// A worker that answers each sleep as `answer` says, on a thread of
    /// its own; it ends when the sender is dropped.
    fn worker(answer: fn(SuspendDone)) -> mpsc::Sender<Message> {
        let (events, received) = mpsc::channel::<Message>();
        std::thread::spawn(move || {
            for (_, done) in received {
                if let Some(done) = done {
                    answer(done);
                }
            }
        });
        events
    }

    #[test]
    fn idle_forward_queues_other_events_without_waiting() {
        let (events, received) = mpsc::channel::<Message>();
        for event in [
            PowerEvent::DisplayOff,
            PowerEvent::DisplayOn,
            PowerEvent::DisplayDimmed,
            PowerEvent::Resume,
        ] {
            let forwarded = forward(&events, event, message, SUSPEND_WAIT);
            assert_eq!(forwarded, Forwarded::Queued);
            let (got, done) = received.try_recv().unwrap();
            assert_eq!(got, event);
            assert!(done.is_none(), "only the sleep is waited for");
        }
        drop(received);
        let forwarded = forward(&events, PowerEvent::DisplayOff, message, SUSPEND_WAIT);
        assert_eq!(forwarded, Forwarded::Dropped);
    }

    #[test]
    fn idle_forward_waits_for_the_sleep_to_be_applied() {
        let events = worker(|done| {
            std::thread::sleep(Duration::from_millis(50));
            let _ = done.send(());
        });
        let started = std::time::Instant::now();
        let forwarded = forward(&events, PowerEvent::Suspend, message, SUSPEND_WAIT);
        assert_eq!(forwarded, Forwarded::Applied);
        assert!(started.elapsed() >= Duration::from_millis(50));
    }

    #[test]
    fn idle_forward_gives_up_on_a_slow_worker() {
        let events = worker(|done| {
            std::thread::sleep(Duration::from_millis(500));
            let _ = done.send(());
        });
        let started = std::time::Instant::now();
        let wait = Duration::from_millis(30);
        let forwarded = forward(&events, PowerEvent::Suspend, message, wait);
        assert_eq!(forwarded, Forwarded::TimedOut);
        assert!(started.elapsed() < Duration::from_millis(400));
    }

    #[test]
    fn idle_forward_does_not_wait_for_a_worker_that_is_gone() {
        // A worker that panics drops the answer without sending it.
        let events = worker(|done| {
            let _ = std::panic::catch_unwind(move || {
                let _held = done;
                panic!("worker failure");
            });
        });
        let started = std::time::Instant::now();
        let forwarded = forward(&events, PowerEvent::Suspend, message, SUSPEND_WAIT);
        assert_eq!(forwarded, Forwarded::Dropped);
        assert!(started.elapsed() < SUSPEND_WAIT);
        let (closed, received) = mpsc::channel::<Message>();
        drop(received);
        let forwarded = forward(&closed, PowerEvent::Suspend, message, SUSPEND_WAIT);
        assert_eq!(forwarded, Forwarded::Dropped);
    }

    #[test]
    fn idle_forwarder_hands_events_to_the_worker() {
        let (events, received) = mpsc::channel::<Message>();
        let answered = std::thread::spawn(move || {
            let mut seen = Vec::new();
            for (event, done) in received {
                seen.push(event);
                if let Some(done) = done {
                    let _ = done.send(());
                }
            }
            seen
        });
        let handler = forwarder(events, message);
        handler(PowerEvent::DisplayOff);
        handler(PowerEvent::Suspend);
        handler(PowerEvent::Resume);
        drop(handler);
        let seen = answered.join().unwrap();
        assert_eq!(
            seen,
            [
                PowerEvent::DisplayOff,
                PowerEvent::Suspend,
                PowerEvent::Resume
            ]
        );
    }

    #[test]
    fn idle_budget_fits_in_the_wait() {
        assert!(SUSPEND_BUDGET < SUSPEND_WAIT);
    }

    #[test]
    fn idle_watch_registers_and_unregisters() {
        let events = Arc::new(AtomicUsize::new(0));
        let seen = Arc::clone(&events);
        let watch = watch(move |event| {
            assert_ne!(event, PowerEvent::Suspend);
            seen.fetch_add(1, Ordering::SeqCst);
        });
        let watch = match watch {
            Ok(watch) => watch,
            Err(error) => panic!("registration failed: {error}"),
        };
        drop(watch);
        let again = super::watch(|_| {});
        assert!(again.is_ok());
    }
}
