//! Screen-owner integration with real core ports and an in-memory channel.
#![allow(clippy::unwrap_used)]
use bezel_core::app::discover_screens;
use bezel_core::domain::{
    discovery::Screen,
    frame::{Frame, Rgba},
    geometry::{Orientation, Size},
    job::{CancelToken, Job},
    screen::{Brightness, Confirm},
    standby::{PlanB, Standby},
    storage::{Confirmed, Operation, RemotePath, Repeat, StorageLocation},
};
use bezel_core::ports::{ScreenChannel, ScreenConnector};
use bezel_core::{BezelError, Result};
use bezel_devices::fake::{FakeStorage, StorageCall};
use bezel_devices::{FakeBus, FakeConnector};
use bezel_power::{
    owner::Owner,
    owner_protocol::{self, RemoteConnector},
};
use std::io::Cursor;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

struct FakeChannel {
    owner: Owner,
    screens: Vec<Screen>,
    crash: Arc<AtomicBool>,
}
impl ScreenChannel for FakeChannel {
    fn connected(&self) -> bool {
        !self.crash.load(Ordering::SeqCst)
    }
    fn exchange(
        &mut self,
        metadata: &[u8],
        payload: &[u8],
        mut job: Option<&mut Job<'_>>,
    ) -> Result<Vec<u8>> {
        if !self.connected() {
            return Err(BezelError::Transport("owner crashed".into()));
        }
        owner_protocol::dispatch(
            &self.owner.client(true),
            &self.screens,
            metadata,
            payload,
            &mut |p| {
                if let Some(j) = job.as_mut() {
                    j.report(p);
                    !j.is_cancelled()
                } else {
                    true
                }
            },
        )
    }
}
struct Fixture {
    owner: Owner,
    physical: FakeConnector,
    screens: Vec<Screen>,
    path: RemotePath,
}
impl Fixture {
    fn new() -> Self {
        let path = RemotePath::parse("internal/video/earth.mp4").unwrap();
        let physical = FakeConnector::with_storage(
            FakeStorage::default().with_file(path.clone(), vec![1, 2, 3]),
        );
        let owner = Owner::new(Arc::new(physical.clone()));
        Self {
            owner,
            physical,
            screens: discover_screens(&FakeBus::turing_88()).unwrap(),
            path,
        }
    }
    fn remote(&self, crash: Arc<AtomicBool>) -> RemoteConnector {
        self.owner.set_editing(true);
        RemoteConnector::new(Box::new(FakeChannel {
            owner: self.owner.clone(),
            screens: self.screens.clone(),
            crash,
        }))
    }
}
#[test]
fn repeated_studio_handover_keeps_connection_and_device_video() {
    let f = Fixture::new();
    let mut saved = f.owner.client(false).connect(&f.screens[0]).unwrap();
    saved.set_orientation(Orientation::Landscape).unwrap();
    saved
        .storage()
        .unwrap()
        .resume_video(&f.path, Repeat::Loop)
        .unwrap();
    let frame = Frame::filled(Size::new(1920, 480), Rgba::BLACK);
    saved.present(&frame).unwrap();
    for _ in 0..3 {
        let remote = f.remote(Arc::default());
        let mut editor = remote.connect(&f.screens[0]).unwrap();
        editor.set_orientation(Orientation::Landscape).unwrap();
        editor
            .storage()
            .unwrap()
            .resume_video(&f.path, Repeat::Loop)
            .unwrap();
        editor.present(&frame).unwrap();
        assert!(saved.presentation_paused());
        drop(editor);
        drop(remote);
        f.owner.set_editing(false);
        assert!(f.owner.reload_pending());
        f.owner.saved_ready();
        saved.present(&frame).unwrap();
    }
    let log = f.physical.log();
    assert_eq!(log.connects, 1);
    assert_eq!(log.orientations, [Orientation::Landscape]);
    assert_eq!(log.releases, 0);
    assert_eq!(log.frames.len(), 7);
    assert_eq!(
        log.storage.calls,
        [StorageCall::PlayVideo(f.path, Repeat::Loop)]
    );
}
#[test]
fn explicit_play_and_return_are_preserved() {
    let f = Fixture::new();
    let mut link = f.remote(Arc::default()).connect(&f.screens[0]).unwrap();
    link.storage()
        .unwrap()
        .resume_video(&f.path, Repeat::Once)
        .unwrap();
    link.storage()
        .unwrap()
        .resume_video(&f.path, Repeat::Once)
        .unwrap();
    link.storage()
        .unwrap()
        .play_video(&f.path, Repeat::Once)
        .unwrap();
    link.release().unwrap();
    assert_eq!(f.physical.log().releases, 1);
    assert_eq!(f.physical.log().storage.calls.len(), 2);
}
#[test]
fn storage_brightness_progress_and_cancel_use_owner() {
    let f = Fixture::new();
    let mut link = f.remote(Arc::default()).connect(&f.screens[0]).unwrap();
    link.set_brightness(Brightness::new(42).unwrap()).unwrap();
    let storage = link.storage().unwrap();
    assert!(storage.info().unwrap().internal.total > 0);
    assert_eq!(storage.list(StorageLocation::ALL[1]).unwrap().len(), 1);
    assert_eq!(storage.size(&f.path).unwrap(), Some(3));
    let new = RemotePath::parse("internal/video/new.mp4").unwrap();
    let token = CancelToken::new();
    let mut reports = Vec::new();
    storage
        .upload(
            &new,
            &[1; 8192],
            &mut Job::new(&token, &mut |p| reports.push(p)),
        )
        .unwrap();
    assert!(!reports.is_empty());
    assert_eq!(storage.size(&new).unwrap(), Some(8192));
    let cancel = CancelToken::new();
    let result = storage.upload(
        &new,
        &[2; 8192],
        &mut Job::new(&cancel, &mut |_| cancel.cancel()),
    );
    assert!(matches!(result, Err(BezelError::Cancelled { .. })));
    storage
        .delete(
            &new,
            Confirmed::require(Confirm::Yes, &Operation::Delete(new.clone())).unwrap(),
        )
        .unwrap();
    let plan = PlanB::new(bezel_core::domain::storage::StartMode::Video, 2);
    storage
        .set_options(
            plan,
            Confirmed::require(Confirm::Yes, &Operation::Standby(Standby::Keep)).unwrap(),
        )
        .unwrap();
    assert_eq!(f.physical.log().brightness, [Brightness::new(42).unwrap()]);
    assert_eq!(f.physical.log().storage.options, Some(plan));
    assert_eq!(f.physical.log().connects, 1);
}
#[test]
fn channel_crash_never_replays_a_write() {
    let f = Fixture::new();
    let crash = Arc::new(AtomicBool::new(false));
    let remote = f.remote(Arc::clone(&crash));
    let mut editor = remote.connect(&f.screens[0]).unwrap();
    crash.store(true, Ordering::SeqCst);
    assert!(editor.set_brightness(Brightness::MAX).is_err());
    assert!(!remote.connected());
    assert!(f.physical.log().brightness.is_empty());
    assert_eq!(f.physical.log().connects, 1);
}
#[test]
fn bounded_framing_rejects_oversize_versions_truncation_before_allocation() {
    for (meta, data) in [(65537, 0), (0, 120_000_001), (u32::MAX, u32::MAX)] {
        let mut header = b"BZLO\x02\0\0\0".to_vec();
        header.extend_from_slice(&meta.to_le_bytes());
        header.extend_from_slice(&data.to_le_bytes());
        assert!(owner_protocol::read_packet(&mut Cursor::new(header)).is_err());
    }
    let mut packet = Vec::new();
    owner_protocol::write_packet(&mut packet, b"metadata", &[1, 2, 3]).unwrap();
    assert_eq!(
        owner_protocol::read_packet(&mut Cursor::new(&packet)).unwrap(),
        (b"metadata".to_vec(), vec![1, 2, 3])
    );
    packet[4] = 99;
    assert!(owner_protocol::read_packet(&mut Cursor::new(&packet)).is_err());
    assert!(owner_protocol::read_packet(&mut Cursor::new(vec![0; 4])).is_err());
}
#[test]
fn malformed_frames_and_unconfirmed_deletes_are_refused() {
    let f = Fixture::new();
    let address = f.screens[0].address().unwrap().0.clone();
    for op in [
        serde_json::json!({"command":"Present","width":1,"height":1}),
        serde_json::json!({"command":"Delete","path":f.path.to_string(),"confirmed":false}),
        serde_json::json!({"command":"Brightness","percent":101}),
    ] {
        let request = serde_json::to_vec(&serde_json::json!({"screen":address,"op":op})).unwrap();
        let reply = owner_protocol::dispatch(
            &f.owner.client(true),
            &f.screens,
            &request,
            &[],
            &mut |_| true,
        )
        .unwrap();
        let reply: serde_json::Value = serde_json::from_slice(&reply).unwrap();
        assert_eq!(reply["reply"], "Error");
    }
    assert!(f.physical.log().frames.is_empty());
    assert!(f.physical.log().brightness.is_empty());
    assert!(
        !f.physical
            .log()
            .storage
            .calls
            .iter()
            .any(|c| matches!(c, StorageCall::Delete(_)))
    );
}
#[test]
fn physical_failure_closes_broken_link_and_reconnects_only_for_recovery() {
    let physical = FakeConnector::default().breaking_after(1, BezelError::Hung("fake".into()));
    let owner = Owner::new(Arc::new(physical.clone()));
    let screen = discover_screens(&FakeBus::turing_88()).unwrap().remove(0);
    let mut link = owner.client(true).connect(&screen).unwrap();
    let frame = Frame::filled(Size::new(480, 1920), Rgba::BLACK);
    link.present(&frame).unwrap();
    assert!(link.present(&frame).is_err());
    let _recovered = owner.client(true).connect(&screen).unwrap();
    assert_eq!(physical.log().connects, 2);
}

#[test]
fn editing_one_screen_does_not_pause_another_saved_screen() {
    let f = Fixture::new();
    let mut second = f.screens[0].clone();
    for endpoint in [&mut second.display, &mut second.wake]
        .into_iter()
        .flatten()
    {
        endpoint.address.0.push_str("-second");
    }
    let first_saved = f.owner.client(false).connect(&f.screens[0]).unwrap();
    let second_saved = f.owner.client(false).connect(&second).unwrap();
    f.owner.set_editing(true);
    let mut editor = f.owner.client(true).connect(&f.screens[0]).unwrap();
    assert!(
        !first_saved.presentation_paused(),
        "identification alone must keep the saved theme running"
    );
    editor.set_brightness(Brightness::MAX).unwrap();
    editor.storage().unwrap().info().unwrap();
    assert!(
        !first_saved.presentation_paused(),
        "queries and brightness do not replace the theme"
    );
    editor.set_orientation(Orientation::Landscape).unwrap();
    assert!(first_saved.presentation_paused());
    assert!(!second_saved.presentation_paused());
    assert_eq!(f.physical.log().connects, 2);
}

#[test]
fn owner_shutdown_runs_once_per_retained_screen_without_connecting() {
    let f = Fixture::new();
    let _editor = f.remote(Arc::default()).connect(&f.screens[0]).unwrap();
    let mut calls = 0;
    f.owner
        .finish_until(
            std::time::Instant::now() + std::time::Duration::from_secs(1),
            &mut |link| {
                calls += 1;
                link.turn_off_now()
            },
        )
        .unwrap();
    assert_eq!(calls, 1);
    assert_eq!(f.physical.log().connects, 1);
    assert_eq!(f.physical.log().storage.calls, [StorageCall::TurnOffNow]);
    assert!(f.owner.client(true).connect(&f.screens[0]).is_err());
}

#[test]
fn legacy_baseline_cannot_replace_a_retained_connection_baseline() {
    let f = Fixture::new();
    let mut original = f.owner.client(false).connect(&f.screens[0]).unwrap();
    let frame = Frame::filled(Size::new(480, 1920), Rgba::BLACK);
    original.present(&frame).unwrap();
    let mut next = f.owner.client(false).connect(&f.screens[0]).unwrap();
    let stale = bezel_core::domain::handoff::Baseline {
        orientation: Orientation::Portrait,
        frame: Frame::filled(Size::new(480, 1920), Rgba::WHITE),
    };
    assert!(!next.seed_baseline(&stale));
    assert!(!original.seed_baseline(&stale));
    assert!(f.physical.log().seeds.is_empty());
    assert_eq!(f.physical.log().connects, 1);
}

#[test]
fn runtime_payload_limit_is_checked_before_reading_or_allocating_it() {
    let metadata = br#"{"runtime":2}"#;
    let mut packet = Vec::new();
    owner_protocol::write_packet(&mut packet, metadata, &[]).unwrap();
    packet[12..16].copy_from_slice(&(64u32 * 1024 * 1024 + 1).to_le_bytes());
    assert_eq!(
        owner_protocol::read_packet(&mut Cursor::new(packet))
            .unwrap_err()
            .kind(),
        std::io::ErrorKind::InvalidData
    );
}

fn level(percent: u8) -> Brightness {
    Brightness::new(percent).unwrap()
}

fn second_screen(f: &Fixture) -> Screen {
    let mut second = f.screens[0].clone();
    for endpoint in [&mut second.display, &mut second.wake]
        .into_iter()
        .flatten()
    {
        endpoint.address.0.push_str("-second");
    }
    second
}

#[test]
fn idle_dims_every_kept_screen_and_restores_the_last_level() {
    let f = Fixture::new();
    let mut light = f.owner.client(false).connect(&f.screens[0]).unwrap();
    let _second = f.owner.client(false).connect(&second_screen(&f)).unwrap();
    light.set_brightness(level(70)).unwrap();
    let fallback = |_: &bezel_core::domain::screen::ScreenIdentity| level(40);
    f.owner.idle(true, &fallback).unwrap();
    f.owner.idle(true, &fallback).unwrap(); // repeated: nothing more sent
    assert!(f.owner.is_idle());
    f.owner.idle(false, &fallback).unwrap();
    f.owner.idle(false, &fallback).unwrap();
    assert!(!f.owner.is_idle());
    let dim = bezel_core::domain::idle::IDLE_BRIGHTNESS;
    assert_eq!(
        f.physical.log().brightness,
        [level(70), dim, dim, level(70), level(40)]
    );
    assert_eq!(f.physical.log().connects, 2);
}

#[test]
fn idle_never_opens_a_screen() {
    let f = Fixture::new();
    let fallback = |_: &bezel_core::domain::screen::ScreenIdentity| Brightness::MAX;
    f.owner.idle(true, &fallback).unwrap();
    f.owner.idle(false, &fallback).unwrap();
    let mut asleep = f.owner.client(false).connect(&f.screens[0]).unwrap();
    asleep.screen_off().unwrap();
    f.owner.idle(true, &fallback).unwrap();
    f.owner.idle(false, &fallback).unwrap();
    let log = f.physical.log();
    assert_eq!(log.connects, 1);
    assert_eq!(log.offs, 1);
    assert!(log.brightness.is_empty());
}

#[test]
fn idle_keeps_a_level_set_while_idle_for_the_restore() {
    let f = Fixture::new();
    let mut light = f.owner.client(false).connect(&f.screens[0]).unwrap();
    let mut studio = f.remote(Arc::default()).connect(&f.screens[0]).unwrap();
    light.set_brightness(level(80)).unwrap();
    let fallback = |_: &bezel_core::domain::screen::ScreenIdentity| Brightness::MAX;
    f.owner.idle(true, &fallback).unwrap();
    studio.set_brightness(level(30)).unwrap();
    assert_eq!(
        f.physical.log().brightness,
        [level(80), bezel_core::domain::idle::IDLE_BRIGHTNESS],
        "a level set while idle waits for the restore"
    );
    f.owner.idle(false, &fallback).unwrap();
    assert_eq!(f.physical.log().brightness.last(), Some(&level(30)));
    assert_eq!(f.physical.log().connects, 1);
}
