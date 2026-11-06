//! Dimming a screen while the monitors are off and restoring it
//! (D-2026-11-06-display-idle-saver-1), through the adapters' fakes: the
//! fake's log, compared whole, shows that only the backlight changed.
#![allow(clippy::expect_used)] // helpers of a failing test panic

use bezel_core::Result;
use bezel_core::app::idle::{dim, restore};
use bezel_core::app::open_screen;
use bezel_core::domain::device::{Capabilities, DeviceModel, Family, ModelId};
use bezel_core::domain::frame::Frame;
use bezel_core::domain::geometry::{Orientation, Size};
use bezel_core::domain::idle::{IDLE_BRIGHTNESS, IdleBrightness};
use bezel_core::domain::screen::{Brightness, ScreenIdentity};
use bezel_core::ports::ScreenLink;
use bezel_devices::fake::{FakeLog, Kept};
use bezel_devices::{FakeBus, FakeConnector};

fn level(percent: u8) -> Brightness {
    Brightness::new(percent).expect("0 to 100")
}

/// The fake 8.8" (rev C) and its connector, with the log as it was once
/// the screen was open.
fn open() -> (FakeConnector, Box<dyn ScreenLink>, FakeLog) {
    let connector = FakeConnector::default();
    let link = open_screen(&FakeBus::turing_88(), &connector, None).expect("opens the fake 8.8\"");
    let opened = connector.log();
    (connector, link, opened)
}

/// `opened` with `levels` set after it, and nothing else.
fn with_levels(opened: &FakeLog, levels: &[Brightness]) -> FakeLog {
    let mut expected = opened.clone();
    expected.brightness.extend_from_slice(levels);
    expected
        .kept
        .extend(levels.iter().copied().map(Kept::Brightness));
    expected
}

#[test]
fn idle_dim_sends_only_the_idle_level() {
    let (connector, mut link, opened) = open();
    assert_eq!(dim(link.as_mut()), Ok(Some(IDLE_BRIGHTNESS)));
    assert_eq!(connector.log(), with_levels(&opened, &[level(10)]));
}

#[test]
fn idle_restore_sends_the_last_recorded_level() {
    let (connector, mut link, opened) = open();
    let mut levels = IdleBrightness::default();
    levels.record(level(65));
    dim(link.as_mut()).expect("dims");
    assert_eq!(
        restore(link.as_mut(), &levels, Brightness::MAX),
        Ok(Some(level(65)))
    );
    assert_eq!(
        connector.log(),
        with_levels(&opened, &[level(10), level(65)])
    );
}

#[test]
fn idle_restore_without_a_recorded_level_sends_the_fallback() {
    let (connector, mut link, opened) = open();
    dim(link.as_mut()).expect("dims");
    let restored = restore(link.as_mut(), &IdleBrightness::default(), level(80));
    assert_eq!(restored, Ok(Some(level(80))));
    assert_eq!(
        connector.log(),
        with_levels(&opened, &[level(10), level(80)])
    );
}

/// A screen without a backlight control.
static DARK: DeviceModel = DeviceModel {
    id: ModelId("dark"),
    name: "No backlight",
    diagonal_hundredths: 350,
    panel: Size::new(320, 480),
    native_orientation: Orientation::Portrait,
    family: Family::TuringRevA,
    capabilities: Capabilities {
        brightness: false,
        device_rotation: false,
        partial_update: false,
        backplate_led: false,
        storage: false,
        video_playback: false,
    },
    hardware_validated: false,
};

/// The fake screen behind `inner`, claiming to be [`DARK`].
struct Dark {
    inner: Box<dyn ScreenLink>,
    identity: ScreenIdentity,
}

impl ScreenLink for Dark {
    fn identity(&self) -> &ScreenIdentity {
        &self.identity
    }
    fn set_brightness(&mut self, b: Brightness) -> Result<()> {
        self.inner.set_brightness(b)
    }
    fn set_orientation(&mut self, o: Orientation) -> Result<()> {
        self.inner.set_orientation(o)
    }
    fn present(&mut self, frame: &Frame) -> Result<()> {
        self.inner.present(frame)
    }
    fn screen_off(&mut self) -> Result<()> {
        self.inner.screen_off()
    }
    fn release(&mut self) -> Result<()> {
        self.inner.release()
    }
}

#[test]
fn idle_sends_nothing_to_a_screen_without_a_backlight() {
    let (connector, inner, opened) = open();
    let mut link = Dark {
        inner,
        identity: ScreenIdentity {
            model: &DARK,
            firmware: None,
        },
    };
    let mut levels = IdleBrightness::default();
    levels.record(level(50));
    assert_eq!(dim(&mut link), Ok(None));
    assert_eq!(restore(&mut link, &levels, Brightness::MAX), Ok(None));
    assert_eq!(connector.log(), opened);
}
