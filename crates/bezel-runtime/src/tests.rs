#![allow(clippy::unwrap_used)]
use super::*;
use bezel_core::{
    domain::{
        frame::Rgba,
        geometry::{Orientation, Size},
        sensor::{Reading, SensorKey, Snapshot},
        storage::{RemotePath, Repeat},
        theme::{Binding, Element, ElementKind, GraphStyle},
    },
    ports::RenderContext,
};
use bezel_devices::{
    FakeBus, FakeConnector,
    fake::{FakeStorage, StorageCall},
};
use bezel_power::owner::Owner;
use bezel_sensors::FakeSensors;
use std::sync::{
    Mutex,
    atomic::{AtomicU64, Ordering},
};
use wire::{RuntimeOp, document, request, response};
fn time() -> LocalTime {
    LocalTime {
        year: 2026,
        month: 10,
        day: 16,
        hour: 12,
        minute: 0,
        second: 0,
        weekday: 4,
        millisecond: 0,
    }
}
#[derive(Clone, Debug)]
struct Seen {
    history: Vec<Option<f64>>,
    spectra: BTreeMap<ElementId, bezel_core::domain::spectrum::AudioState>,
    clock: Duration,
    theme: String,
}
struct Renderer(Arc<Mutex<Vec<Seen>>>);
thread_local! {
    /// Each render draws a new frame, as live spectrum bars do.
    static DISTINCT: std::cell::Cell<Option<u8>> = const { std::cell::Cell::new(None) };
}
impl FrameRenderer for Renderer {
    fn render(
        &mut self,
        theme: &Theme,
        _: &BTreeMap<AssetRef, Vec<u8>>,
        context: RenderContext<'_>,
    ) -> Result<Frame> {
        let history = theme
            .graph_series()
            .first()
            .map_or(Vec::new(), |id| context.histories.points(id, 100));
        self.0.lock().unwrap().push(Seen {
            spectra: context
                .snapshot
                .spectra
                .iter()
                .map(|(id, f)| (*id, f.state))
                .collect(),
            history,
            clock: context.animation,
            theme: theme.name.clone(),
        });
        let shade = DISTINCT.with(|d| {
            let next = d.get().map(|n| n.wrapping_add(1));
            d.set(next);
            next.unwrap_or(0)
        });
        Ok(Frame::filled(theme.canvas, Rgba::opaque(shade, 0, 0)))
    }
    fn measure_element(
        &mut self,
        _: &Theme,
        _: ElementId,
        _: &BTreeMap<AssetRef, Vec<u8>>,
        _: RenderContext<'_>,
    ) -> Result<Option<BoxF>> {
        Ok(Some(BoxF::new(1.0, 2.0, 3.0, 4.0)))
    }
}
fn doc() -> Document {
    let mut theme = Theme::blank("saved", Size::new(1920, 480), Orientation::Landscape);
    theme.refresh_seconds = 1.0;
    theme.background = Background::DeviceVideo {
        path: RemotePath::parse("internal/video/earth.mp4").unwrap(),
        repeat: Repeat::Loop,
        color: Rgba::BLACK,
    };
    theme.elements.push(Element {
        id: ElementId(1),
        name: "CPU".into(),
        frame: BoxF::new(0.0, 0.0, 120.0, 60.0),
        visible: true,
        locked: false,
        opacity: 1.0,
        rotation: 0.0,
        is_group: false,
        group_parent: None,
        card_member: None,
        card: None,
        color_by_value: None,
        visualization: None,
        value_transition: None,
        sensor_motion: None,
        kind: ElementKind::Graph {
            binding: Binding {
                key: SensorKey::new("cpu.usage").unwrap(),
                min: 0.0,
                max: 100.0,
            },
            history: 100,
            interval_seconds: None,
            style: GraphStyle::Line,
            color: Rgba::WHITE,
            fill: None,
            line_width: 1.0,
            autoscale: false,
        },
    });
    Document {
        theme,
        assets: BTreeMap::new(),
    }
}
struct Fixture {
    engine: Engine,
    physical: FakeConnector,
    owner: Owner,
    seen: Arc<Mutex<Vec<Seen>>>,
    key: String,
    // Drop after the engine so media handles are closed before cleanup on Windows.
    _directory: tempfile::TempDir,
}
fn fixture() -> Fixture {
    let directory = tempfile::Builder::new()
        .prefix("bezel-test-engine-")
        .tempdir()
        .unwrap();
    let bus = Arc::new(FakeBus::turing_88());
    let key = bezel_core::app::discover_screens(bus.as_ref()).unwrap()[0]
        .address()
        .unwrap()
        .0
        .clone();
    let physical = FakeConnector::with_storage(FakeStorage::default().with_file(
        RemotePath::parse("internal/video/earth.mp4").unwrap(),
        vec![1],
    ));
    let owner = Owner::new(Arc::new(physical.clone()));
    let seen = Arc::new(Mutex::new(Vec::new()));
    let sensors = FakeSensors::new(
        Vec::new(),
        (1..=20)
            .map(|n| {
                let mut s = Snapshot::default();
                s.insert(
                    SensorKey::new("cpu.usage").unwrap(),
                    Reading::Value(f64::from(n)),
                );
                s
            })
            .collect(),
    );
    let mut engine = Engine::new(
        Box::new(Renderer(seen.clone())),
        Box::new(sensors.clone()),
        Box::new(bezel_media::FfmpegTranscoder::new(None)),
        bus,
        Arc::new(owner.client(true)),
        Language::English,
        directory.path().to_path_buf(),
        time,
    )
    .unwrap();
    engine.sensors = sampling::Sampler::manual(Box::new(sensors));
    Fixture {
        engine,
        physical,
        owner,
        seen,
        key,
        _directory: directory,
    }
}

#[test]
fn fixture_removes_its_files_after_success_and_unwinding() {
    for unwind in [false, true] {
        let mut path = None;
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let f = fixture();
            path = Some(f._directory.path().to_path_buf());
            let nested = f._directory.path().join("nested");
            std::fs::create_dir(&nested).unwrap();
            std::fs::write(nested.join("clip.mp4"), b"test media").unwrap();
            if unwind {
                std::panic::resume_unwind(Box::new("simulated test failure"));
            }
        }));
        assert_eq!(result.is_err(), unwind);
        assert!(!path.unwrap().exists());
    }
}

#[test]
fn a_profile_switch_restores_the_unsaved_editor_and_never_replaces_its_published_frame() {
    use crate::profiles::{Config, Profile, ProfileFile, Service};
    use bezel_core::ports::{ThemeLocation, ThemeStore};
    let mut f = fixture();
    let previous_pixels = DISTINCT.with(|d| d.replace(Some(0)));
    let mut saved = doc();
    saved.theme.background = Background::Color(Rgba::BLACK);
    f.engine.document(&f.key, 1, saved.clone(), true).unwrap();
    let mut draft = saved.clone();
    draft.theme.name = "unsaved editor".into();
    f.engine.document(&f.key, 2, draft.clone(), false).unwrap();
    f.engine.tick_at(Duration::ZERO, time()).unwrap();
    let published = f.engine.scenes[&f.key]
        .frame
        .as_ref()
        .unwrap()
        .as_ref()
        .clone();
    let directory = f._directory.path().join("profile-engine");
    std::fs::create_dir(&directory).unwrap();
    let path = directory.join("music.bezeltheme");
    let mut music = saved.clone();
    music.theme.name = "music profile".into();
    bezel_themes::FsThemeStore
        .save(
            &ThemeLocation(path.display().to_string()),
            &music.theme,
            &music.assets,
        )
        .unwrap();
    let file = ProfileFile(directory.join("profiles.json"));
    file.save(&Config {
        profiles: vec![Profile {
            id: "music".into(),
            name: "Music".into(),
            themes: BTreeMap::from([(f.key.clone(), path.display().to_string())]),
            rules: vec![],
            combine: Default::default(),
        }],
        automatic: false,
        manual: Some("music".into()),
        default_profile: None,
        transition: Default::default(),
    })
    .unwrap();
    f.engine.scenes.get_mut(&f.key).unwrap().live = true;
    let mut service = Service::new(file.clone());
    service.tick(&mut f.engine, &Snapshot::default(), Duration::ZERO, time());
    let status = service.status();
    assert_eq!(
        status.active,
        Some(crate::profiles::Active {
            profile: "music".into(),
            reason: crate::profiles::Reason::Manual,
        }),
        "the runtime says which profile is on and why"
    );
    assert_eq!(status.rules["music"], Vec::<bool>::new());
    assert!(
        f.engine.profile_status().is_none(),
        "this engine has no profile service"
    );
    let scene = &f.engine.scenes[&f.key];
    assert_eq!(scene.runtime.theme().name, "music profile");
    assert!(scene.saved == saved);
    assert!(scene.profile_restore.as_ref() == Some(&draft));
    f.engine.editing_profile(&f.key);
    music.theme.name = "freshly saved music".into();
    bezel_themes::FsThemeStore
        .save(
            &ThemeLocation(path.display().to_string()),
            &music.theme,
            &music.assets,
        )
        .unwrap();
    service.tick(
        &mut f.engine,
        &Snapshot::default(),
        Duration::from_millis(100),
        time(),
    );
    // Simulate the editing grace period expiring without a wall-clock sleep.
    f.engine.scenes.get_mut(&f.key).unwrap().profile_edit_at = None;
    service.tick(
        &mut f.engine,
        &Snapshot::default(),
        Duration::from_secs(1),
        time(),
    );
    assert_eq!(
        f.engine.scenes[&f.key].runtime.theme().name,
        "freshly saved music"
    );
    f.engine.tick_at(Duration::from_secs(1), time()).unwrap();
    assert!(f.engine.scenes[&f.key].frame.as_ref().unwrap().as_ref() != &published);
    // Reading the canvas returns the editor's frame, not the live profile.
    assert!(f.engine.image(&f.key, 0, 2320).unwrap().frame.as_ref() == Some(&published));
    service.tick(
        &mut f.engine,
        &Snapshot::default(),
        Duration::from_secs(1),
        time(),
    );
    let scene = &f.engine.scenes[&f.key];
    assert_eq!(scene.runtime.theme().name, "unsaved editor");
    assert!(scene.saved == saved);
    assert_eq!(scene.revision, 2);
    assert!(scene.profile_restore.is_none());
    std::fs::remove_file(&file.0).unwrap();
    std::fs::remove_file(path).unwrap();
    std::fs::remove_dir(directory).unwrap();
    DISTINCT.with(|d| d.set(previous_pixels));
}

#[test]
fn one_gaming_profile_switches_two_different_displays_and_restores_both() {
    use crate::profiles::{Config, Profile, ProfileFile, Service};
    use bezel_core::ports::{ThemeLocation, ThemeStore};
    let mut f = fixture();
    let mut wide = doc();
    wide.theme.name = "Selected wide".into();
    let mut round = wide.clone();
    round.theme.name = "Selected round".into();
    round.theme.canvas = Size::new(480, 480);
    round.theme.orientation = Orientation::Portrait;
    f.engine.document("wide", 1, wide.clone(), true).unwrap();
    f.engine.document("round", 1, round.clone(), true).unwrap();
    for scene in f.engine.scenes.values_mut() {
        scene.live = true;
        scene.frame = Some(Arc::new(Frame::filled(
            scene.runtime.theme().canvas,
            Rgba::BLACK,
        )));
    }
    let directory = f._directory.path().join("profile-pair");
    std::fs::create_dir(&directory).unwrap();
    let mut themes = BTreeMap::new();
    for (key, base) in [("wide", &wide), ("round", &round)] {
        let mut gaming = base.clone();
        gaming.theme.name = format!("Gaming {key}");
        let path = directory.join(format!("{key}.bezeltheme"));
        bezel_themes::FsThemeStore
            .save(
                &ThemeLocation(path.display().to_string()),
                &gaming.theme,
                &gaming.assets,
            )
            .unwrap();
        themes.insert(key.to_owned(), path.display().to_string());
    }
    let file = ProfileFile(directory.join("profiles.json"));
    let config = Config {
        profiles: vec![Profile {
            id: "gaming".into(),
            name: "Gaming".into(),
            themes: themes.clone(),
            rules: vec![],
            combine: Default::default(),
        }],
        automatic: false,
        manual: Some("gaming".into()),
        default_profile: None,
        transition: crate::profiles::TransitionSettings {
            effect: "wipe".into(),
            duration_ms: 800,
            ..Default::default()
        },
    };
    file.save(&config).unwrap();
    let mut service = Service::new(file.clone());
    use crate::profiles::Phase;
    let phase = |engine: &Engine, key: &str| {
        engine.scenes[key]
            .profile_animation
            .as_ref()
            .map(|a| a.phase)
    };
    // The cover closes over the old themes first; nothing is applied yet.
    service.tick(&mut f.engine, &Snapshot::default(), Duration::ZERO, time());
    for key in ["wide", "round"] {
        assert_eq!(
            f.engine.scenes[key].runtime.theme().name,
            format!("Selected {key}")
        );
        assert_eq!(phase(&f.engine, key), Some(Phase::Closing));
        assert_eq!(
            f.engine.scenes[key]
                .profile_animation
                .as_ref()
                .unwrap()
                .target
                .as_deref(),
            Some("gaming")
        );
    }
    // Closed after half the duration: both screens switch under the cover.
    service.tick(
        &mut f.engine,
        &Snapshot::default(),
        Duration::from_millis(400),
        time(),
    );
    for key in ["wide", "round"] {
        assert_eq!(
            f.engine.scenes[key].runtime.theme().name,
            format!("Gaming {key}")
        );
        assert_eq!(f.engine.scenes[key].profile_id.as_deref(), Some("gaming"));
        assert_eq!(phase(&f.engine, key), Some(Phase::Covered));
    }
    assert!(f.engine.scenes["wide"].saved == wide);
    assert!(f.engine.scenes["round"].saved == round);
    file.select(None, false).unwrap();
    service.tick(
        &mut f.engine,
        &Snapshot::default(),
        Duration::from_secs(2),
        time(),
    );
    // Still fully covered (no frame drawn yet): the change needs no new closing.
    for key in ["wide", "round"] {
        assert_eq!(phase(&f.engine, key), Some(Phase::Covered));
        assert_eq!(f.engine.scenes[key].profile_id, None);
    }
    assert_eq!(
        f.engine.scenes["wide"].runtime.theme().name,
        "Selected wide"
    );
    assert_eq!(
        f.engine.scenes["round"].runtime.theme().name,
        "Selected round"
    );
    f.engine.editing_profile("wide");
    assert!(f.engine.scenes["wide"].profile_animation.is_none());
    for path in themes.values() {
        std::fs::remove_file(path).unwrap();
    }
    std::fs::remove_file(&file.0).unwrap();
    std::fs::remove_dir(directory).unwrap();
}
#[test]
fn profile_motion_schedules_frames_until_completion_without_waiting_for_sensor_refresh() {
    use crate::profiles::{Animation, Phase};
    use bezel_core::domain::cover::{
        Cover, CoverDirection, CoverEffect, DEFAULT_ACCENT, DEFAULT_PANEL,
    };
    let mut f = fixture();
    f.engine.document(&f.key, 1, doc(), true).unwrap();
    f.engine.tick_at(Duration::ZERO, time()).unwrap();
    let scene = f.engine.scenes.get_mut(&f.key).unwrap();
    // Covered at 0 (the theme was applied); its first frame is drawn at 50 ms.
    scene.profile_animation = Some(Animation {
        target: None,
        phase: Phase::Covered,
        started: Duration::ZERO,
        base: 1.0,
        cover: Cover {
            effect: CoverEffect::Mosaic,
            direction: CoverDirection::Left,
            panel: DEFAULT_PANEL,
            accent: DEFAULT_ACCENT,
        },
        half: Duration::from_millis(300),
        apply: true,
        show: None,
    });
    scene.dirty = true;
    let count = f.seen.lock().unwrap().len();
    let delay = f.engine.tick_at(Duration::from_millis(50), time()).unwrap();
    assert!(delay <= Duration::from_millis(50));
    assert!(f.seen.lock().unwrap().len() > count);
    let animation = f.engine.scenes[&f.key].profile_animation.as_ref().unwrap();
    // The opening starts at the first frame drawn, not when the theme changed.
    assert_eq!(
        (animation.phase, animation.started),
        (Phase::Opening, Duration::from_millis(50))
    );
    let covered = f.engine.scenes[&f.key].frame.as_ref().unwrap().clone();
    assert!(
        covered
            .as_rgba()
            .as_chunks::<4>()
            .0
            .iter()
            .all(|p| *p == [0x12, 0x0b, 0x13, 255])
    );
    f.engine
        .tick_at(Duration::from_millis(349), time())
        .unwrap();
    assert!(f.engine.scenes[&f.key].profile_animation.is_some());
    f.engine
        .tick_at(Duration::from_millis(350), time())
        .unwrap();
    assert!(f.engine.scenes[&f.key].profile_animation.is_none());
}
#[test]
fn a_cover_preview_closes_and_opens_on_the_same_theme() {
    use crate::profiles::{Phase, TransitionSettings};
    let mut f = fixture();
    f.engine.document(&f.key, 1, doc(), true).unwrap();
    f.engine.tick_at(Duration::ZERO, time()).unwrap();
    let shown = f.engine.scenes[&f.key].frame.clone().unwrap();
    let name = f.engine.scenes[&f.key].runtime.theme().name.clone();
    let curtain = TransitionSettings {
        effect: "curtain".into(),
        duration_ms: 800,
        ..Default::default()
    };
    // Nothing to try on a screen that is not live, or without a cover.
    assert_eq!(
        f.engine.preview_cover(&curtain, &BTreeMap::new()).unwrap(),
        0
    );
    assert!(f.engine.scenes[&f.key].profile_animation.is_none());
    f.engine.scenes.get_mut(&f.key).unwrap().live = true;
    assert_eq!(
        f.engine
            .preview_cover(&TransitionSettings::default(), &BTreeMap::new())
            .unwrap(),
        0
    );
    assert!(f.engine.scenes[&f.key].profile_animation.is_none());
    assert!(
        f.engine
            .preview_cover(
                &TransitionSettings {
                    effect: "spin".into(),
                    ..Default::default()
                },
                &BTreeMap::new()
            )
            .is_err()
    );
    assert_eq!(
        f.engine.preview_cover(&curtain, &BTreeMap::new()).unwrap(),
        1
    );
    let started = f.engine.scenes[&f.key]
        .profile_animation
        .as_ref()
        .unwrap()
        .started;
    let phase = |f: &Fixture| {
        f.engine.scenes[&f.key]
            .profile_animation
            .as_ref()
            .map(|a| a.phase)
    };
    advance(&mut f.engine, started + Duration::from_millis(200), time()).unwrap();
    assert_eq!(phase(&f), Some(Phase::Closing));
    assert!(f.engine.scenes[&f.key].frame.as_ref().unwrap() != &shown);
    // Closed at half the 800 ms: it opens at once on the same theme.
    advance(&mut f.engine, started + Duration::from_millis(400), time()).unwrap();
    assert_eq!(phase(&f), Some(Phase::Opening));
    let covered = f.engine.scenes[&f.key].frame.clone().unwrap();
    assert!(
        covered
            .as_rgba()
            .as_chunks::<4>()
            .0
            .iter()
            .all(|p| *p == [0x12, 0x0b, 0x13, 255])
    );
    advance(&mut f.engine, started + Duration::from_millis(800), time()).unwrap();
    assert_eq!(phase(&f), None);
    let scene = &f.engine.scenes[&f.key];
    assert_eq!(scene.runtime.theme().name, name);
    assert!(scene.profile_id.is_none() && scene.profile_restore.is_none());
}
#[test]
fn a_tried_profile_shows_under_the_cover_then_the_screen_returns() {
    use crate::profiles::{
        Config, PREVIEW_HOLD, PREVIEW_ID, Phase, ProfileFile, Service, TransitionSettings,
    };
    use bezel_core::ports::{ThemeLocation, ThemeStore};
    let mut f = fixture();
    let base = doc();
    f.engine.document(&f.key, 1, base.clone(), true).unwrap();
    f.engine.tick_at(Duration::ZERO, time()).unwrap();
    f.engine.scenes.get_mut(&f.key).unwrap().live = true;
    let temporary = tempfile::Builder::new()
        .prefix("bezel-profile-try-")
        .tempdir()
        .unwrap();
    let directory = temporary.path();
    let path = directory.join("tried.bezeltheme");
    let mut tried = base.clone();
    tried.theme.name = "tried, unsaved profile".into();
    bezel_themes::FsThemeStore
        .save(
            &ThemeLocation(path.display().to_string()),
            &tried.theme,
            &tried.assets,
        )
        .unwrap();
    let wipe = TransitionSettings {
        effect: "wipe".into(),
        duration_ms: 800,
        ..Default::default()
    };
    let file = ProfileFile(directory.join("profiles.json"));
    file.save(&Config {
        transition: wipe.clone(),
        automatic: false,
        ..Default::default()
    })
    .unwrap();
    let mut service = Service::new(file.clone());
    let themes = BTreeMap::from([
        (f.key.clone(), path.display().to_string()),
        ("absent".into(), "x".into()),
    ]);
    assert_eq!(f.engine.preview_cover(&wipe, &themes).unwrap(), 1);
    let started = f.engine.scenes[&f.key]
        .profile_animation
        .as_ref()
        .unwrap()
        .started;
    let name = |f: &Fixture| f.engine.scenes[&f.key].runtime.theme().name.clone();
    let phase = |f: &Fixture| {
        f.engine.scenes[&f.key]
            .profile_animation
            .as_ref()
            .map(|a| a.phase)
    };
    // Closing over the base theme; the service leaves the scene alone.
    service.tick(
        &mut f.engine,
        &Snapshot::default(),
        started + Duration::from_millis(200),
        time(),
    );
    advance(&mut f.engine, started + Duration::from_millis(200), time()).unwrap();
    assert_eq!(
        (name(&f), phase(&f)),
        (base.theme.name.clone(), Some(Phase::Closing))
    );
    // Closed: the tried theme is under the cover, and it opens on it.
    advance(&mut f.engine, started + Duration::from_millis(400), time()).unwrap();
    assert_eq!(
        (name(&f).as_str(), phase(&f)),
        ("tried, unsaved profile", Some(Phase::Opening))
    );
    assert_eq!(
        f.engine.scenes[&f.key].profile_id.as_deref(),
        Some(PREVIEW_ID)
    );
    let shown_at = started + Duration::from_millis(400);
    // Held on show: the service waits.
    service.tick(
        &mut f.engine,
        &Snapshot::default(),
        shown_at + PREVIEW_HOLD - Duration::from_millis(1),
        time(),
    );
    assert_eq!(name(&f), "tried, unsaved profile");
    // Then it closes again over it and returns to the base theme.
    let back = shown_at + PREVIEW_HOLD;
    service.tick(&mut f.engine, &Snapshot::default(), back, time());
    assert_eq!(phase(&f), Some(Phase::Closing));
    service.tick(
        &mut f.engine,
        &Snapshot::default(),
        back + Duration::from_millis(400),
        time(),
    );
    let scene = &f.engine.scenes[&f.key];
    assert_eq!(scene.runtime.theme().name, base.theme.name);
    assert!(
        scene.profile_id.is_none()
            && scene.profile_restore.is_none()
            && scene.preview_until.is_none()
    );
}
fn advance(engine: &mut Engine, at: Duration, time: LocalTime) -> Result<Duration> {
    if engine.scenes.values().any(|s| s.runtime.sample_due(at)) {
        engine.sensors.advance(at);
    }
    let deadline = Instant::now() + Duration::from_secs(3);
    loop {
        let next = engine.tick_at(at, time)?;
        let ready = engine.scenes.values().all(|s| {
            s.presenter
                .as_ref()
                .is_none_or(|p| s.link.is_some() && !s.video_dirty && p.idle())
        });
        if ready {
            return Ok(next);
        }
        assert!(Instant::now() < deadline, "presentation did not settle");
        std::thread::sleep(Duration::from_millis(2));
    }
}
fn send(
    engine: &mut Engine,
    key: &str,
    op: RuntimeOp,
    payload: &[u8],
) -> Result<(serde_json::Value, Vec<u8>)> {
    let (meta, pixels) = engine.request(&request(key, op)?, payload)?;
    Ok((response(&meta)?, pixels))
}
#[test]
fn editor_attach_detach_keeps_graph_clock_link_and_earth_playing() {
    let mut f = fixture();
    let saved = doc();
    f.engine.document(&f.key, 0, saved.clone(), true).unwrap();
    f.engine.live(&f.key, true).unwrap();
    for n in 0..4 {
        advance(&mut f.engine, Duration::from_secs(n), time()).unwrap();
    }
    let before = f.seen.lock().unwrap().last().unwrap().clone();
    let frames = f.physical.log().frames.len();
    for _ in 0..3 {
        f.owner.set_editing(true);
        send(
            &mut f.engine,
            &f.key,
            RuntimeOp::Draft {
                revision: 1,
                saved: false,
            },
            &document(&saved.theme, Some(&saved.assets)).unwrap(),
        )
        .unwrap();
        send(&mut f.engine, &f.key, RuntimeOp::Live { on: true }, &[]).unwrap();
        // Hold the clock: handover itself must not redraw. Advancing it now
        // legitimately draws the scrolling graph between sensor samples.
        advance(&mut f.engine, Duration::from_secs(3), time()).unwrap();
        f.engine.disconnected().unwrap();
        f.owner.set_editing(false);
        f.owner.saved_ready();
        advance(&mut f.engine, Duration::from_secs(3), time()).unwrap();
    }
    assert_eq!(
        f.physical.log().frames.len(),
        frames,
        "handover must not schedule a redraw"
    );
    advance(&mut f.engine, Duration::from_secs(4), time()).unwrap();
    let after = f.seen.lock().unwrap().last().unwrap().clone();
    assert_eq!(&after.history[..before.history.len()], before.history);
    assert_eq!(after.history.len(), 5);
    assert!(after.clock > before.clock);
    assert_eq!(f.physical.log().connects, 1);
    assert_eq!(
        f.physical
            .log()
            .storage
            .calls
            .iter()
            .filter(|c| matches!(c, StorageCall::PlayVideo(..)))
            .count(),
        1
    );
    assert!(!f.physical.log().storage.calls.contains(&StorageCall::Stop));
}

#[test]
fn graph_frames_follow_twenty_ms_clock_without_more_sensor_samples() {
    let mut f = fixture();
    let mut document = doc();
    document.theme.background = Background::Color(Rgba::BLACK);
    f.engine.document(&f.key, 1, document, true).unwrap();
    advance(&mut f.engine, Duration::ZERO, time()).unwrap();
    let delay = advance(&mut f.engine, Duration::from_secs(1), time()).unwrap();
    assert!(delay <= Duration::from_millis(20));
    let count = f.seen.lock().unwrap().len();
    advance(&mut f.engine, Duration::from_millis(1010), time()).unwrap();
    assert_eq!(f.seen.lock().unwrap().len(), count, "do not exceed 50 fps");
    advance(&mut f.engine, Duration::from_millis(1020), time()).unwrap();
    let seen = f.seen.lock().unwrap();
    assert_eq!(seen.len(), count + 1);
    assert_eq!(seen.last().unwrap().clock, Duration::from_millis(1020));
    assert_eq!(
        seen.last().unwrap().history.len(),
        2,
        "animation must not sample"
    );
}

#[test]
fn rendering_that_exhausts_the_frame_budget_does_not_add_another_wait() {
    struct Slow;
    impl FrameRenderer for Slow {
        fn render(
            &mut self,
            theme: &Theme,
            _: &BTreeMap<AssetRef, Vec<u8>>,
            _: RenderContext<'_>,
        ) -> Result<Frame> {
            std::thread::sleep(Duration::from_millis(60));
            Ok(Frame::filled(theme.canvas, Rgba::BLACK))
        }
    }
    let mut f = fixture();
    f.engine.renderer = Box::new(Slow);
    f.engine.document(&f.key, 1, doc(), true).unwrap();
    assert_eq!(f.engine.tick().unwrap(), Duration::ZERO);
}
#[test]
fn unsaved_draft_crash_restores_saved_in_same_scene_and_keeps_history() {
    let mut f = fixture();
    let saved = doc();
    f.engine.document(&f.key, 0, saved.clone(), true).unwrap();
    f.engine.live(&f.key, true).unwrap();
    advance(&mut f.engine, Duration::ZERO, time()).unwrap();
    let mut draft = saved.clone();
    draft.theme.name = "draft".into();
    draft.theme.elements[0].frame.x = 50.0;
    send(
        &mut f.engine,
        &f.key,
        RuntimeOp::Draft {
            revision: 1,
            saved: false,
        },
        &document(&draft.theme, None).unwrap(),
    )
    .unwrap();
    advance(&mut f.engine, Duration::from_secs(1), time()).unwrap();
    f.engine.disconnected().unwrap();
    advance(&mut f.engine, Duration::from_secs(2), time()).unwrap();
    let seen = f.seen.lock().unwrap();
    assert_eq!(seen.last().unwrap().theme, "saved");
    assert_eq!(
        seen.last().unwrap().history,
        [Some(1.0), Some(2.0), Some(3.0)]
    );
    assert_eq!(seen.last().unwrap().clock, Duration::from_secs(2));
    assert_eq!(f.physical.log().connects, 1);
}
#[test]
fn saved_commit_survives_client_crash_and_revisions_are_checked() {
    let mut f = fixture();
    let mut d = doc();
    f.engine.document(&f.key, 1, d.clone(), true).unwrap();
    d.theme.name = "committed".into();
    let bytes = document(&d.theme, None).unwrap();
    send(
        &mut f.engine,
        &f.key,
        RuntimeOp::Draft {
            revision: 2,
            saved: true,
        },
        &bytes,
    )
    .unwrap();
    assert!(
        send(
            &mut f.engine,
            &f.key,
            RuntimeOp::Draft {
                revision: 2,
                saved: false
            },
            &bytes
        )
        .is_err()
    );
    f.engine.detach().unwrap();
    assert_eq!(f.engine.scenes[&f.key].runtime.theme().name, "committed");
    send(
        &mut f.engine,
        &f.key,
        RuntimeOp::Draft {
            revision: 1,
            saved: false,
        },
        &bytes,
    )
    .unwrap();
}
#[test]
fn canvas_backpressure_scaling_inspector_and_measure_are_bounded() {
    let mut f = fixture();
    f.engine.document(&f.key, 0, doc(), true).unwrap();
    advance(&mut f.engine, Duration::ZERO, time()).unwrap();
    let (value, pixels) = send(
        &mut f.engine,
        &f.key,
        RuntimeOp::Image {
            after: 0,
            max: 320,
            cards: true,
        },
        &[],
    )
    .unwrap();
    assert_eq!(value["width"], 320);
    assert_eq!(pixels.len(), 320 * 80 * 4);
    let seq = value["sequence"].as_u64().unwrap();
    let (_, pixels) = send(
        &mut f.engine,
        &f.key,
        RuntimeOp::Image {
            after: seq,
            max: 320,
            cards: true,
        },
        &[],
    )
    .unwrap();
    assert!(pixels.is_empty());
    let (value, pixels) = send(
        &mut f.engine,
        &f.key,
        RuntimeOp::Inspect {
            element: 1,
            max: 64,
            motion: false,
            trial_ms: None,
        },
        &[],
    )
    .unwrap();
    assert!(value["width"].as_u64().unwrap() <= 64);
    assert!(!pixels.is_empty());
    let (value, _) = send(
        &mut f.engine,
        &f.key,
        RuntimeOp::Measure { element: 1 },
        &[],
    )
    .unwrap();
    assert_eq!(value["width"], 3.0);
    assert!(
        send(
            &mut f.engine,
            &f.key,
            RuntimeOp::Image {
                after: 0,
                max: 2321,
                cards: true
            },
            &[]
        )
        .is_err()
    );
    assert!(
        send(
            &mut f.engine,
            &f.key,
            RuntimeOp::Inspect {
                element: 999,
                max: 64,
                motion: true,
                trial_ms: None,
            },
            &[]
        )
        .is_err()
    );
    assert!(send(&mut f.engine, &f.key, RuntimeOp::Live { on: false }, &[1]).is_err());
}

#[test]
fn a_wire_inspector_trial_never_changes_the_document_sample_or_live_frame() {
    let mut f = fixture();
    f.engine.document(&f.key, 0, doc(), true).unwrap();
    advance(&mut f.engine, Duration::ZERO, time()).unwrap();
    let before = f.engine.scenes[&f.key].runtime.snapshot().clone();
    let theme = f.engine.scenes[&f.key].runtime.theme().clone();
    let sequence = f.engine.image(&f.key, 0, 64).unwrap().sequence;
    let (value, _) = send(
        &mut f.engine,
        &f.key,
        RuntimeOp::Inspect {
            element: 1,
            max: 64,
            motion: true,
            trial_ms: Some(1500),
        },
        &[],
    )
    .unwrap();
    assert_eq!(value["nextMs"], 33);
    let history = &f.seen.lock().unwrap().last().unwrap().history.clone();
    assert_eq!(history.len(), 60);
    assert!(
        history
            .iter()
            .flatten()
            .all(|value| (10. ..=90.).contains(value))
    );
    assert_eq!(*f.engine.scenes[&f.key].runtime.snapshot(), before);
    assert_eq!(*f.engine.scenes[&f.key].runtime.theme(), theme);
    assert_eq!(f.engine.image(&f.key, 0, 64).unwrap().sequence, sequence);
    let (value, _) = send(
        &mut f.engine,
        &f.key,
        RuntimeOp::Inspect {
            element: 1,
            max: 64,
            motion: false,
            trial_ms: Some(1500),
        },
        &[],
    )
    .unwrap();
    assert_ne!(value["nextMs"], 33);
    assert!(f.seen.lock().unwrap().last().unwrap().history.len() < 60);
}
#[test]
fn a_wire_canvas_trial_never_changes_the_document_sample_or_live_frame() {
    let mut f = fixture();
    f.engine.document(&f.key, 0, doc(), true).unwrap();
    advance(&mut f.engine, Duration::ZERO, time()).unwrap();
    let before = f.engine.scenes[&f.key].runtime.snapshot().clone();
    let theme = f.engine.scenes[&f.key].runtime.theme().clone();
    let sequence = f.engine.image(&f.key, 0, 64).unwrap().sequence;
    let (value, pixels) = send(
        &mut f.engine,
        &f.key,
        RuntimeOp::Trial {
            max: 64,
            motion: true,
            ms: 1500,
            hold: None,
        },
        &[],
    )
    .unwrap();
    assert_eq!(value["nextMs"], 33);
    assert!(!pixels.is_empty());
    // The whole scene, its longest side reduced to `max`.
    assert_eq!(
        (value["width"].as_u64(), value["height"].as_u64()),
        (Some(64), Some(16))
    );
    let seen = f.seen.lock().unwrap().last().unwrap().history.clone();
    assert_eq!(seen.len(), 60);
    assert_eq!(*f.engine.scenes[&f.key].runtime.snapshot(), before);
    assert_eq!(*f.engine.scenes[&f.key].runtime.theme(), theme);
    assert_eq!(f.engine.image(&f.key, 0, 64).unwrap().sequence, sequence);
    send(
        &mut f.engine,
        &f.key,
        RuntimeOp::Trial {
            max: 64,
            motion: false,
            ms: 0,
            hold: Some(wire::TrialHold {
                element: 1,
                fraction: 0.5,
            }),
        },
        &[],
    )
    .unwrap();
    assert_eq!(*f.engine.scenes[&f.key].runtime.snapshot(), before);
    assert!(
        send(
            &mut f.engine,
            "missing",
            RuntimeOp::Trial {
                max: 64,
                motion: true,
                ms: 0,
                hold: None
            },
            &[]
        )
        .is_err()
    );
}
#[test]
fn untrusted_document_lengths_paths_dimensions_and_versions_are_rejected() {
    let mut f = fixture();
    let key = f.key.clone();
    for bytes in [vec![], vec![255; 4], vec![0; 4]] {
        assert!(
            send(
                &mut f.engine,
                &key,
                RuntimeOp::Draft {
                    revision: 1,
                    saved: false
                },
                &bytes
            )
            .is_err()
        );
    }
    let mut d = doc();
    d.theme.canvas = bezel_core::domain::geometry::Size::new(u32::MAX, u32::MAX);
    assert!(f.engine.document(&key, 0, d, false).is_err());
    let mut d = doc();
    d.assets.insert(AssetRef("../secret".into()), vec![1]);
    let bytes = document(&d.theme, Some(&d.assets)).unwrap();
    assert!(
        send(
            &mut f.engine,
            &key,
            RuntimeOp::Draft {
                revision: 1,
                saved: false
            },
            &bytes
        )
        .is_err()
    );
    let (meta, _) = f
        .engine
        .request(
            br#"{"runtime":99,"scene":"x","op":{"command":"Live","on":false}}"#,
            &[],
        )
        .unwrap();
    assert!(response(&meta).is_err());
    assert!(wire::frame(&serde_json::json!({"width":u32::MAX,"height":2}), vec![]).is_err());
}

#[test]
fn changing_background_stops_video_but_closing_same_document_does_not() {
    let mut f = fixture();
    let saved = doc();
    f.engine.document(&f.key, 0, saved.clone(), true).unwrap();
    f.engine.live(&f.key, true).unwrap();
    advance(&mut f.engine, Duration::ZERO, time()).unwrap();
    let mut draft = saved;
    draft.theme.background = Background::Color(Rgba::BLACK);
    f.engine.document(&f.key, 1, draft, false).unwrap();
    advance(&mut f.engine, Duration::from_secs(1), time()).unwrap();
    assert_eq!(
        f.physical
            .log()
            .storage
            .calls
            .iter()
            .filter(|c| matches!(c, StorageCall::Stop))
            .count(),
        1
    );
    f.engine.detach().unwrap();
    advance(&mut f.engine, Duration::from_secs(2), time()).unwrap();
    assert_eq!(
        f.physical
            .log()
            .storage
            .calls
            .iter()
            .filter(|c| matches!(c, StorageCall::PlayVideo(..)))
            .count(),
        2
    );
}
#[test]
fn host_decoder_clock_and_instance_survive_drafts_and_client_crash() {
    use bezel_core::{
        domain::{
            job::Job,
            media::{MediaInfo, MediaTools, StreamSpec, TranscodeTarget},
        },
        ports::VideoFrames,
    };
    struct Media {
        opened: Arc<AtomicU64>,
        times: Arc<Mutex<Vec<Duration>>>,
    }
    struct Clip {
        frame: Frame,
        times: Arc<Mutex<Vec<Duration>>>,
    }
    impl VideoFrames for Clip {
        fn frame_at(&mut self, at: Duration) -> Result<&Frame> {
            self.times.lock().unwrap().push(at);
            Ok(&self.frame)
        }
    }
    impl MediaTranscoder for Media {
        fn tools(&mut self) -> MediaTools {
            MediaTools::Ready {
                version: "fake".into(),
            }
        }
        fn probe(&mut self, _: &MediaLocation) -> Result<MediaInfo> {
            Err(invalid("no probe in this test"))
        }
        fn transcode(
            &mut self,
            _: &MediaLocation,
            _: &TranscodeTarget,
            _: &mut Job<'_>,
        ) -> Result<MediaLocation> {
            Err(invalid("no conversion"))
        }
        fn load(&mut self, _: &MediaLocation) -> Result<Vec<u8>> {
            Err(invalid("no load"))
        }
        fn stream(&mut self, _: &MediaLocation, spec: StreamSpec) -> Result<Box<dyn VideoFrames>> {
            self.opened.fetch_add(1, Ordering::SeqCst);
            Ok(Box::new(Clip {
                frame: Frame::filled(spec.size, Rgba::BLACK),
                times: self.times.clone(),
            }))
        }
    }
    let mut f = fixture();
    let opened = Arc::new(AtomicU64::new(0));
    let times = Arc::new(Mutex::new(Vec::new()));
    f.engine.media = Box::new(Media {
        opened: opened.clone(),
        times: times.clone(),
    });
    let mut d = doc();
    let asset = AssetRef("assets/clip.mp4".into());
    d.theme.background = Background::Video {
        asset: asset.clone(),
        poster: None,
        framing: None,
    };
    d.assets.insert(asset, vec![1, 2, 3]);
    f.engine.document("offline", 0, d.clone(), true).unwrap();
    advance(&mut f.engine, Duration::ZERO, time()).unwrap();
    advance(&mut f.engine, Duration::from_millis(100), time()).unwrap();
    d.theme.elements[0].frame.x = 20.0;
    f.engine.document("offline", 1, d, false).unwrap();
    advance(&mut f.engine, Duration::from_millis(200), time()).unwrap();
    f.engine.detach().unwrap();
    advance(&mut f.engine, Duration::from_millis(300), time()).unwrap();
    assert_eq!(opened.load(Ordering::SeqCst), 1);
    assert_eq!(
        *times.lock().unwrap(),
        [
            Duration::ZERO,
            Duration::from_millis(100),
            Duration::from_millis(200),
            Duration::from_millis(300)
        ]
    );
}
#[test]
fn independent_scenes_and_demo_thumbnail_leave_live_histories_untouched() {
    let mut f = fixture();
    f.engine.thumbnail_sensors = Some(Box::new(FakeSensors::demo()));
    let d = doc();
    f.engine.document("one", 0, d.clone(), true).unwrap();
    f.engine.document("two", 0, d.clone(), true).unwrap();
    advance(&mut f.engine, Duration::ZERO, time()).unwrap();
    let (meta, pixels) = send(
        &mut f.engine,
        "thumbnail",
        RuntimeOp::Thumbnail,
        &document(&d.theme, Some(&d.assets)).unwrap(),
    )
    .unwrap();
    assert_eq!(meta["width"], 360);
    assert!(!pixels.is_empty());
    advance(&mut f.engine, Duration::from_secs(1), time()).unwrap();
    let seen = f.seen.lock().unwrap();
    let last = &seen[seen.len() - 2..];
    assert_eq!(last[0].history.len(), 2);
    assert_eq!(last[1].history.len(), 2);
}

#[test]
fn sensor_options_status_and_catalog_use_engine_sources_without_losing_histories() {
    let mut f = fixture();
    let configured = Arc::new(AtomicU64::new(0));
    let count = configured.clone();
    f.engine = f.engine.with_adapters(
        Box::new(move |_| {
            count.fetch_add(1, Ordering::SeqCst);
            Ok((
                Box::new(FakeSensors::demo()),
                Box::new(bezel_media::FfmpegTranscoder::new(None)),
            ))
        }),
        Box::new(FakeSensors::demo()),
    );
    f.engine.document("editor", 0, doc(), true).unwrap();
    advance(&mut f.engine, Duration::ZERO, time()).unwrap();
    let options = wire::Options {
        audio_capture: false,
        ping_host: "127.0.0.1".into(),

        ffmpeg: None,
    };
    for _ in 0..2 {
        send(
            &mut f.engine,
            "editor",
            RuntimeOp::Options {
                options: options.clone(),
            },
            &[],
        )
        .unwrap();
    }
    assert_eq!(configured.load(Ordering::SeqCst), 1);
    // Granting audio consent leaves the sensor reader running.
    send(
        &mut f.engine,
        "editor",
        RuntimeOp::Options {
            options: wire::Options {
                audio_capture: true,
                ..options.clone()
            },
        },
        &[],
    )
    .unwrap();
    assert_eq!(configured.load(Ordering::SeqCst), 1);
    advance(&mut f.engine, Duration::from_secs(1), time()).unwrap();
    let (status, _) = send(
        &mut f.engine,
        "editor",
        RuntimeOp::Sensors {
            listed: Some(vec!["cpu.usage".into()]),
        },
        &[],
    )
    .unwrap();
    assert!(!wire::catalog(&status).is_empty());
    assert!(!wire::snapshot(&status).is_empty());
    assert_eq!(f.seen.lock().unwrap().last().unwrap().history.len(), 2);
    let mut bad = options;
    bad.ping_host = "x".repeat(254);
    assert!(
        send(
            &mut f.engine,
            "editor",
            RuntimeOp::Options { options: bad },
            &[]
        )
        .is_err()
    );
}
#[test]
fn suspend_reconnect_and_explicit_live_off_keep_the_same_document_clock() {
    let mut f = fixture();
    f.engine.document(&f.key, 0, doc(), true).unwrap();
    f.engine.live(&f.key, true).unwrap();
    advance(&mut f.engine, Duration::ZERO, time()).unwrap();
    let frames = f.physical.log().frames.len();
    f.engine.suspend(true);
    advance(&mut f.engine, Duration::from_secs(1), time()).unwrap();
    assert_eq!(f.physical.log().frames.len(), frames);
    f.engine.suspend(false);
    f.engine.disconnect_links();
    f.owner.clear().unwrap();
    advance(&mut f.engine, Duration::from_secs(2), time()).unwrap();
    assert_eq!(f.physical.log().connects, 2);
    assert_eq!(
        f.seen.lock().unwrap().last().unwrap().clock,
        Duration::from_secs(2)
    );
    f.engine.live(&f.key, false).unwrap();
    f.engine.follow_saved_live(&[]);
    advance(&mut f.engine, Duration::from_secs(3), time()).unwrap();
    assert_eq!(f.physical.log().connects, 2);
}
#[test]
fn legacy_baseline_is_seeded_after_orientation_and_before_the_first_engine_frame() {
    use bezel_core::{
        app::handoff::hand_over,
        domain::{discovery::DeviceAddress, handoff::HandedFrame},
        ports::HandoffStore,
    };
    #[derive(Default)]
    struct Store(Mutex<Option<HandedFrame>>);
    impl HandoffStore for Store {
        fn save(&self, frame: &HandedFrame) -> Result<()> {
            *self.0.lock().unwrap() = Some(frame.clone());
            Ok(())
        }
        fn take(&self, _: &DeviceAddress) -> Result<Option<HandedFrame>> {
            Ok(self.0.lock().unwrap().take())
        }
    }
    let mut f = fixture();
    let screen = bezel_core::app::discover_screens(&FakeBus::turing_88())
        .unwrap()
        .remove(0);
    let mut previous = f.physical.connect(&screen).unwrap();
    previous.set_orientation(Orientation::Landscape).unwrap();
    previous
        .present(&Frame::filled(Size::new(1920, 480), Rgba::BLACK))
        .unwrap();
    let store = Arc::new(Store::default());
    hand_over(
        previous.as_ref(),
        DeviceAddress(f.key.clone()),
        store.as_ref(),
        Duration::from_secs(99),
    )
    .unwrap();
    drop(previous);
    f.engine = f.engine.with_handoff(store, || Duration::from_secs(100));
    f.engine.document(&f.key, 0, doc(), true).unwrap();
    f.engine.live(&f.key, true).unwrap();
    let deadline = Instant::now() + Duration::from_secs(2);
    while f.physical.log().seeds.is_empty() {
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(2));
    }
    assert_eq!(f.physical.log().seeds.len(), 1);
    assert_eq!(
        f.physical.log().frames.len(),
        1,
        "only the old owner has sent a frame"
    );
}

#[test]
fn slow_screen_does_not_delay_shared_samples_fast_screen_or_readonly_images() {
    use bezel_core::{
        domain::{device::UsbId, discovery::DeviceAddress},
        ports::{DeviceBus, TransferStats},
    };
    use std::sync::{Condvar, atomic::AtomicBool};
    struct Gate {
        entered: AtomicBool,
        released: Mutex<bool>,
        wake: Condvar,
    }
    impl Gate {
        fn release(&self) {
            *self.released.lock().unwrap() = true;
            self.wake.notify_all();
        }
    }
    struct Release(Arc<Gate>);
    impl Drop for Release {
        fn drop(&mut self) {
            self.0.release();
        }
    }
    struct Source {
        times: Arc<Mutex<Vec<Instant>>>,
    }
    impl SensorSource for Source {
        fn catalog(&mut self) -> Result<Vec<SensorInfo>> {
            Ok(Vec::new())
        }
        fn sample(&mut self) -> Result<Snapshot> {
            let mut times = self.times.lock().unwrap();
            times.push(Instant::now());
            let mut s = Snapshot::default();
            s.insert(
                SensorKey::new("cpu.usage").unwrap(),
                Reading::Value(times.len() as f64),
            );
            Ok(s)
        }
    }
    struct Connector {
        fake: FakeConnector,
        gate: Arc<Gate>,
        fast: Arc<Mutex<Vec<Instant>>>,
        slow: Arc<Mutex<Vec<Frame>>>,
    }
    struct Lag {
        inner: Box<dyn ScreenLink>,
        slow: bool,
        gate: Arc<Gate>,
        fast: Arc<Mutex<Vec<Instant>>>,
        frames: Arc<Mutex<Vec<Frame>>>,
    }
    impl ScreenConnector for Connector {
        fn connect(&self, screen: &Screen) -> Result<Box<dyn ScreenLink>> {
            Ok(Box::new(Lag {
                inner: self.fake.connect(screen)?,
                slow: screen.address().unwrap().0 == "COM35",
                gate: self.gate.clone(),
                fast: self.fast.clone(),
                frames: self.slow.clone(),
            }))
        }
    }
    impl ScreenLink for Lag {
        fn identity(&self) -> &bezel_core::domain::screen::ScreenIdentity {
            self.inner.identity()
        }
        fn set_orientation(&mut self, o: Orientation) -> Result<()> {
            self.inner.set_orientation(o)
        }
        fn set_brightness(&mut self, b: bezel_core::domain::screen::Brightness) -> Result<()> {
            self.inner.set_brightness(b)
        }
        fn present(&mut self, f: &Frame) -> Result<()> {
            if self.slow {
                self.gate.entered.store(true, Ordering::SeqCst);
                let mut released = self.gate.released.lock().unwrap();
                while !*released {
                    released = self.gate.wake.wait(released).unwrap();
                }
                self.frames.lock().unwrap().push(f.clone());
            } else {
                self.fast.lock().unwrap().push(Instant::now());
            }
            self.inner.present(f)
        }
        fn last_transfer(&self) -> Option<TransferStats> {
            Some(TransferStats {
                kind: "full",
                ..TransferStats::default()
            })
        }
        fn screen_off(&mut self) -> Result<()> {
            self.inner.screen_off()
        }
        fn release(&mut self) -> Result<()> {
            self.inner.release()
        }
    }
    struct Draw(Arc<Mutex<Vec<Seen>>>);
    impl FrameRenderer for Draw {
        fn render(
            &mut self,
            t: &Theme,
            a: &BTreeMap<AssetRef, Vec<u8>>,
            c: RenderContext<'_>,
        ) -> Result<Frame> {
            Renderer(self.0.clone()).render(t, a, c)?;
            let value = c
                .snapshot
                .get(&SensorKey::new("cpu.usage").unwrap())
                .value()
                .unwrap_or(0.0) as u8;
            Ok(Frame::filled(t.canvas, Rgba::opaque(value, 0, 0)))
        }
    }
    let mut endpoints = FakeBus::turing_88().endpoints().unwrap();
    let mut small = endpoints[1].clone();
    small.address = DeviceAddress("COM35".into());
    small.usb = UsbId::new(0x1a86, 0xfe0c);
    small.location = None;
    small.serial_number = None;
    endpoints.push(small);
    let bus = Arc::new(FakeBus::new(endpoints));
    let screens = bezel_core::app::discover_screens(bus.as_ref()).unwrap();
    let large = screens
        .iter()
        .find(|s| s.address().unwrap().0 != "COM35")
        .unwrap()
        .address()
        .unwrap()
        .0
        .clone();
    let gate = Arc::new(Gate {
        entered: AtomicBool::new(false),
        released: Mutex::new(false),
        wake: Condvar::new(),
    });
    let _release = Release(gate.clone());
    let times = Arc::new(Mutex::new(Vec::new()));
    let fast = Arc::new(Mutex::new(Vec::new()));
    let slow = Arc::new(Mutex::new(Vec::new()));
    let seen = Arc::new(Mutex::new(Vec::new()));
    let connector = Arc::new(Connector {
        fake: FakeConnector::default(),
        gate: gate.clone(),
        fast: fast.clone(),
        slow: slow.clone(),
    });
    let mut engine = Engine::new(
        Box::new(Draw(seen.clone())),
        Box::new(Source {
            times: times.clone(),
        }),
        Box::new(bezel_media::FfmpegTranscoder::new(None)),
        bus,
        connector,
        Language::English,
        PathBuf::new(),
        time,
    )
    .unwrap();
    let mut large_doc = doc();
    large_doc.theme.background = Background::Color(Rgba::BLACK);
    large_doc.theme.refresh_seconds = 0.25;
    engine.document(&large, 0, large_doc.clone(), true).unwrap();
    let mut small_doc = large_doc;
    small_doc.theme.name = "small".into();
    small_doc.theme.canvas = Size::new(320, 480);
    small_doc.theme.orientation = Orientation::Portrait;
    if let ElementKind::Graph {
        interval_seconds, ..
    } = &mut small_doc.theme.elements[0].kind
    {
        *interval_seconds = Some(1);
    }
    engine.document("COM35", 0, small_doc, true).unwrap();
    engine.live(&large, true).unwrap();
    engine.live("COM35", true).unwrap();
    let images = engine.images();
    let engine = Arc::new(Mutex::new(engine));
    let running = Arc::new(AtomicBool::new(true));
    let loop_engine = engine.clone();
    let flag = running.clone();
    let render = std::thread::spawn(move || {
        while flag.load(Ordering::SeqCst) {
            loop_engine.lock().unwrap().tick().unwrap();
            std::thread::sleep(Duration::from_millis(5));
        }
    });
    let deadline = Instant::now() + Duration::from_secs(3);
    while !gate.entered.load(Ordering::SeqCst) {
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(2));
    }
    std::thread::sleep(Duration::from_millis(1100));
    assert!(
        fast.lock().unwrap().len() >= 4,
        "fast screen stalled behind slow I/O"
    );
    let samples = times.lock().unwrap().clone();
    assert!(samples.len() >= 5);
    assert!(
        samples
            .windows(2)
            .all(|w| w[1].duration_since(w[0]) < Duration::from_millis(450)),
        "sample cadence followed serial I/O"
    );
    // Snapshot endpoint works even while somebody else owns the engine mutex.
    let guard = engine.lock().unwrap();
    let start = Instant::now();
    let image = images.latest(&large, 0, 2320).unwrap();
    assert!(image.frame.is_some());
    assert!(start.elapsed() < Duration::from_millis(200));
    drop(guard);
    assert!(
        slow.lock().unwrap().is_empty(),
        "the slow device should still be blocked"
    );
    running.store(false, Ordering::SeqCst);
    render.join().unwrap();
    let latest = engine.lock().unwrap().scenes["COM35"]
        .frame
        .as_ref()
        .unwrap()
        .pixel(0, 0);
    gate.release();
    let deadline = Instant::now() + Duration::from_secs(2);
    while slow.lock().unwrap().len() < 2 {
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(2));
    }
    let delivered = slow.lock().unwrap();
    assert_eq!(
        delivered.len(),
        2,
        "stale frames were queued instead of replaced"
    );
    assert_eq!(delivered.last().unwrap().pixel(0, 0), latest);
    let seen = seen.lock().unwrap();
    let fast_history = seen
        .iter()
        .rev()
        .find(|s| s.theme == "saved")
        .unwrap()
        .history
        .len();
    let slow_history = seen
        .iter()
        .rev()
        .find(|s| s.theme == "small")
        .unwrap()
        .history
        .len();
    assert!(fast_history >= 5);
    assert_eq!(
        slow_history, 2,
        "D-16 graph cadence changed with presentation speed"
    );
}

#[test]
fn large_sensor_catalog_uses_bounded_binary_json_instead_of_breaking_the_pipe() {
    let mut engine = fixture().engine;
    let info = bezel_core::domain::sensor::SensorInfo {
        key: SensorKey::new("cpu.usage").unwrap(),
        category: bezel_core::domain::sensor::Category::Cpu,
        quantity: bezel_core::domain::sensor::Quantity::Percent,
        label: "CPU utilization".into(),
        source: "test".into(),
    };
    engine.catalog = vec![info; 1000];
    let metadata = request("__sensors__", RuntimeOp::Sensors { listed: None }).unwrap();
    let reply = engine.request(&metadata, &[]).unwrap();
    assert!(reply.0.len() <= wire::MAX_METADATA);
    assert!(reply.1.len() > wire::MAX_METADATA);
    assert!(reply.1.len() <= wire::MAX_JSON);
    let reply = wire::unpack_reply(reply).unwrap();
    assert_eq!(
        response(&reply.0).unwrap()["catalog"]
            .as_array()
            .unwrap()
            .len(),
        1000
    );
    assert!(reply.1.is_empty());
    assert!(
        wire::unpack_reply((
            br#"{"encoding":"json"}"#.to_vec(),
            vec![0; wire::MAX_JSON + 1]
        ))
        .is_err()
    );
}

#[test]
fn large_catalog_survives_the_first_request_over_the_windows_pipe() {
    use bezel_core::ports::ScreenChannel;
    use bezel_power::owner_pipe::{PipeChannel, Server};
    use std::sync::atomic::AtomicBool;
    let mut engine = fixture().engine;
    // Deterministic catalog size; no real sensor or device reads in this test.
    let info = bezel_core::domain::sensor::SensorInfo {
        key: SensorKey::new("cpu.usage").unwrap(),
        category: bezel_core::domain::sensor::Category::Cpu,
        quantity: bezel_core::domain::sensor::Quantity::Percent,
        label: "CPU utilization".into(),
        source: "test".into(),
    };
    engine.catalog = vec![info; 1000];
    let directory = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../../target/catalog-pipe-{}", std::process::id()));
    let server = Server::bind(&directory).unwrap();
    let stop = Arc::new(AtomicBool::new(false));
    let flag = stop.clone();
    let thread = std::thread::spawn(move || {
        server.serve_runtime(
            Owner::new(Arc::new(FakeConnector::default())),
            Arc::new(FakeBus::turing_88()),
            flag,
            Some(Arc::new(Mutex::new(engine))),
        )
    });
    let mut client = PipeChannel::connect(&directory, Duration::from_secs(1)).unwrap();
    for _ in 0..2 {
        let reply = client
            .exchange_binary(
                &request("__sensors__", RuntimeOp::Sensors { listed: None }).unwrap(),
                &[],
            )
            .unwrap();
        let reply = wire::unpack_reply(reply).unwrap();
        assert_eq!(
            response(&reply.0).unwrap()["catalog"]
                .as_array()
                .unwrap()
                .len(),
            1000
        );
        assert!(client.connected());
    }
    stop.store(true, Ordering::SeqCst);
    drop(client);
    thread.join().unwrap().unwrap();
}

#[test]
fn silent_spectrum_does_not_resend_identical_frames_on_sensor_ticks() {
    use bezel_core::domain::{
        spectrum::Spectrum,
        theme::{TextContent, TextStyle},
    };
    let mut f = fixture();
    let mut document = doc();
    let element = &mut document.theme.elements[0];
    element.kind = ElementKind::Text {
        content: TextContent::Spectrum(Spectrum::default()),
        style: TextStyle::default(),
    };
    f.engine.document(&f.key, 0, document, true).unwrap();
    f.engine.live(&f.key, true).unwrap();
    advance(&mut f.engine, Duration::ZERO, time()).unwrap();
    advance(&mut f.engine, Duration::from_millis(100), time()).unwrap();
    let count = f.physical.log().frames.len();
    assert!(count > 0);
    for n in 1..5 {
        advance(&mut f.engine, Duration::from_secs(n), time()).unwrap();
    }
    assert_eq!(
        f.physical.log().frames.len(),
        count,
        "settled audio must not submit unchanged frames"
    );
    f.engine.suspend(true);
    assert_eq!(
        f.engine.tick_at(Duration::from_secs(6), time()).unwrap(),
        Duration::from_millis(50)
    );
}

#[test]
fn rejected_options_do_not_grant_capture_permission() {
    let mut f = fixture();
    let invalid = wire::Options {
        audio_capture: true,
        ping_host: "x".repeat(254),
        ffmpeg: None,
    };
    assert!(f.engine.options(invalid).is_err());
    assert!(!f.engine.audio_enabled);
}

#[test]
fn scene_names_with_shared_prefixes_cannot_exchange_audio_results() {
    use bezel_core::domain::{
        spectrum::{AudioState, Spectrum},
        theme::{TextContent, TextStyle},
    };
    let mut f = fixture();
    let mut plain = doc();
    plain.theme.name = "plain-spectrum".into();
    plain.theme.elements[0].kind = ElementKind::Text {
        content: TextContent::Spectrum(Spectrum::default()),
        style: TextStyle::default(),
    };
    let mut demo = plain.clone();
    demo.theme.name = "demo-spectrum".into();
    if let ElementKind::Text {
        content: TextContent::Spectrum(s),
        ..
    } = &mut demo.theme.elements[0].kind
    {
        s.demo = true;
    }
    f.engine.document("a", 0, plain, false).unwrap();
    f.engine.document("a/b", 0, demo, false).unwrap();
    f.engine.audio_consent(true);
    f.engine.tick_at(Duration::ZERO, time()).unwrap();
    f.engine.images.latest("a", 0, 64).unwrap();
    f.engine.images.latest("a/b", 0, 64).unwrap();
    f.engine.tick_at(Duration::from_millis(50), time()).unwrap();
    let seen = f.seen.lock().unwrap();
    let plain = seen
        .iter()
        .rev()
        .find(|s| s.theme == "plain-spectrum")
        .unwrap();
    let demo = seen
        .iter()
        .rev()
        .find(|s| s.theme == "demo-spectrum")
        .unwrap();
    assert_eq!(
        plain.spectra.values().copied().collect::<Vec<_>>(),
        vec![AudioState::PlayerMissing]
    );
    assert_eq!(
        demo.spectra.values().copied().collect::<Vec<_>>(),
        vec![AudioState::Demo]
    );
}

#[test]
fn spectrum_switches_source_on_the_same_refresh_that_changes_the_player() {
    use bezel_core::{
        domain::{
            playback::{MediaSession, Player},
            spectrum::{AudioBlock, AudioState, Spectrum},
            theme::{TextContent, TextStyle},
        },
        ports::{AudioCapture, AudioCaptureFactory},
    };
    struct Factory(Arc<Mutex<Vec<String>>>);
    struct Capture;
    impl AudioCapture for Capture {
        fn read(&mut self) -> std::result::Result<AudioBlock, AudioState> {
            Ok(AudioBlock {
                samples: vec![],
                rate: 48000,
                channels: 2,
            })
        }
    }
    impl AudioCaptureFactory for Factory {
        fn open(&self, source: &str) -> std::result::Result<Box<dyn AudioCapture>, AudioState> {
            self.0.lock().unwrap().push(source.into());
            Ok(Box::new(Capture))
        }
    }
    let mut f = fixture();
    let mut document = doc();
    document.theme.refresh_seconds = 1.;
    document.theme.elements[0].kind = ElementKind::Text {
        content: TextContent::Spectrum(Spectrum::default()),
        style: TextStyle::default(),
    };
    let mut player = document.theme.elements[0].clone();
    player.id = ElementId(2);
    player.kind = ElementKind::Text {
        content: TextContent::Player(Player::default()),
        style: TextStyle::default(),
    };
    document.theme.elements.push(player);
    let samples = ["Spotify.exe", "Chrome.exe"].map(|source| {
        let mut s = Snapshot::default();
        s.media.push(MediaSession {
            source: source.into(),
            playing: true,
            ..Default::default()
        });
        s
    });
    f.engine.sensors =
        sampling::Sampler::manual(Box::new(FakeSensors::new(Vec::new(), samples.into())));
    let opened = Arc::new(Mutex::new(Vec::new()));
    f.engine = f.engine.with_audio(Arc::new(Factory(opened.clone())));
    f.engine.audio_consent(true);
    f.engine.document("preview", 0, document, false).unwrap();
    f.engine.sensors.advance(Duration::ZERO);
    f.engine.tick_at(Duration::ZERO, time()).unwrap();
    f.engine.images.latest("preview", 0, 64).unwrap();
    f.engine.tick_at(Duration::from_millis(50), time()).unwrap();
    let wait = |source: &str| {
        let until = Instant::now() + Duration::from_secs(1);
        while !opened.lock().unwrap().iter().any(|s| s == source) {
            assert!(
                Instant::now() < until,
                "source was not requested in the Player refresh"
            );
            std::thread::sleep(Duration::from_millis(5));
        }
    };
    wait("Spotify.exe");
    f.engine.sensors.advance(Duration::from_secs(1));
    f.engine.tick_at(Duration::from_secs(1), time()).unwrap();
    wait("Chrome.exe");
}

#[test]
fn spectrum_on_any_card_face_keeps_its_state_before_and_after_the_face_shows() {
    use bezel_core::domain::{
        playback::{CardTrigger, TriggerSource},
        spectrum::{AudioState, Spectrum},
        theme::{Card, CardMember, TextContent, TextStyle},
    };
    let mut f = fixture();
    let mut document = doc();
    document.theme.name = "card-spectrum".into();
    let mut spectrum = document.theme.elements[0].clone();
    spectrum.id = ElementId(2);
    spectrum.kind = ElementKind::Text {
        content: TextContent::Spectrum(Spectrum::default()),
        style: TextStyle::default(),
    };
    // Shown only on face 1, which a timer reaches while the saved face stays 0.
    spectrum.card_member = Some(CardMember {
        parent: ElementId(1),
        face: Some(1),
    });
    document.theme.elements[0].card = Some(Card {
        faces: vec!["sensors".into(), "music".into()],
        active_face: 0,
        rotation_seconds: None,
        transition: None,
        triggers: vec![CardTrigger {
            source: TriggerSource::Timer,
            interval_seconds: Some(5),
            next_face: true,
            app: String::new(),
            face: 0,
            priority: 0,
            return_seconds: 0,
            return_face: None,
        }],
    });
    document.theme.elements.push(spectrum);
    f.engine.document("a", 0, document, false).unwrap();
    send(&mut f.engine, "a", RuntimeOp::Cards { on: true }, &[]).unwrap();
    f.engine.audio_consent(true);
    let state = |f: &Fixture| {
        let seen = f.seen.lock().unwrap();
        let last = seen
            .iter()
            .rev()
            .find(|s| s.theme == "card-spectrum")
            .unwrap();
        last.spectra.values().copied().collect::<Vec<_>>()
    };
    for ms in [0, 50, 100] {
        f.engine.tick_at(Duration::from_millis(ms), time()).unwrap();
        f.engine.images.latest("a", 0, 64).unwrap();
    }
    // Face 1 is not shown yet, but its spectrum is already live: no default frame
    // ("audio capture disabled") when the face comes into view.
    assert_eq!(state(&f), vec![AudioState::PlayerMissing]);
    for ms in [5_000, 5_050, 5_100] {
        f.engine.tick_at(Duration::from_millis(ms), time()).unwrap();
        f.engine.images.latest("a", 0, 64).unwrap();
    }
    assert_eq!(state(&f), vec![AudioState::PlayerMissing]);
}

#[test]
fn a_wake_inside_the_spectrum_window_returns_when_the_window_ends() {
    use bezel_core::domain::{
        spectrum::Spectrum,
        theme::{TextContent, TextStyle},
    };
    let mut f = fixture();
    let mut document = doc();
    document.theme.background = Background::Color(Rgba::BLACK);
    document.theme.elements[0].kind = ElementKind::Text {
        content: TextContent::Spectrum(Spectrum::default()),
        style: TextStyle::default(),
    };
    DISTINCT.with(|d| d.set(Some(0)));
    f.engine.document("a", 0, document, false).unwrap();
    f.engine.audio_consent(true);
    advance(&mut f.engine, Duration::ZERO, time()).unwrap();
    f.engine.images.latest("a", 0, 64).unwrap();
    advance(&mut f.engine, Duration::from_millis(100), time()).unwrap();
    // Another wake 20 ms after that frame (a worker result): the bars are due
    // again in 30 ms, not after a whole sleep.
    let wait = f
        .engine
        .tick_at(Duration::from_millis(120), time())
        .unwrap();
    assert!(wait <= Duration::from_millis(30), "{wait:?}");
}

#[test]
fn an_identical_frame_does_not_hold_the_next_spectrum_back() {
    use bezel_core::domain::{
        spectrum::Spectrum,
        theme::{TextContent, TextStyle},
    };
    let mut f = fixture();
    let mut document = doc();
    document.theme.background = Background::Color(Rgba::BLACK);
    document.theme.elements[0].kind = ElementKind::Text {
        content: TextContent::Spectrum(Spectrum::default()),
        style: TextStyle::default(),
    };
    f.engine.document("a", 0, document, false).unwrap();
    f.engine.audio_consent(true);
    advance(&mut f.engine, Duration::ZERO, time()).unwrap();
    f.engine.images.latest("a", 0, 64).unwrap();
    let rendered = f.engine.scenes["a"].rendered_at;
    // A sensor sample redraws nothing visible: the window keeps the last frame
    // that changed, so the next worker result is drawn as soon as it arrives.
    advance(&mut f.engine, Duration::from_millis(100), time()).unwrap();
    assert_eq!(f.engine.scenes["a"].rendered_at, rendered);
}

#[test]
fn a_spectrum_on_the_pcs_output_listens_without_any_player() {
    use bezel_core::{
        domain::{
            spectrum::{AudioBlock, AudioState, SYSTEM_SOURCE, Spectrum, SpectrumAudio},
            theme::{TextContent, TextStyle},
        },
        ports::{AudioCapture, AudioCaptureFactory},
    };
    struct Factory(Arc<Mutex<Vec<String>>>);
    struct Capture;
    impl AudioCapture for Capture {
        fn read(&mut self) -> std::result::Result<AudioBlock, AudioState> {
            Ok(AudioBlock {
                samples: vec![],
                rate: 48000,
                channels: 2,
            })
        }
    }
    impl AudioCaptureFactory for Factory {
        fn open(&self, source: &str) -> std::result::Result<Box<dyn AudioCapture>, AudioState> {
            self.0.lock().unwrap().push(source.into());
            Ok(Box::new(Capture))
        }
    }
    let mut f = fixture();
    let mut document = doc();
    // No Player in the theme: the PC's output does not need one.
    document.theme.elements[0].kind = ElementKind::Text {
        content: TextContent::Spectrum(Spectrum {
            audio: SpectrumAudio::System,
            ..Spectrum::default()
        }),
        style: TextStyle::default(),
    };
    let opened = Arc::new(Mutex::new(Vec::new()));
    f.engine = f.engine.with_audio(Arc::new(Factory(opened.clone())));
    f.engine.audio_consent(true);
    f.engine.document("preview", 0, document, false).unwrap();
    f.engine.tick_at(Duration::ZERO, time()).unwrap();
    f.engine.images.latest("preview", 0, 64).unwrap();
    f.engine.tick_at(Duration::from_millis(50), time()).unwrap();
    let until = Instant::now() + Duration::from_secs(1);
    while !opened.lock().unwrap().iter().any(|s| s == SYSTEM_SOURCE) {
        assert!(Instant::now() < until, "the PC's output was not opened");
        std::thread::sleep(Duration::from_millis(5));
    }
    assert!(opened.lock().unwrap().iter().all(|s| s == SYSTEM_SOURCE));
}

#[test]
fn a_thirty_second_theme_trial_restores_without_changing_saved_documents() {
    use crate::profiles::{Config, ProfileFile, Service};
    use bezel_core::ports::{ThemeLocation, ThemeStore};
    let temporary = tempfile::Builder::new()
        .prefix("bezel-profile-thirty-")
        .tempdir()
        .unwrap();
    let mut f = fixture();
    let base = doc();
    f.engine.document(&f.key, 1, base.clone(), true).unwrap();
    f.engine.tick_at(Duration::ZERO, time()).unwrap();
    let file = ProfileFile(temporary.path().join("profiles.json"));
    let config = Config {
        automatic: false,
        ..Default::default()
    };
    file.save(&config).unwrap();
    f.engine.profiles = Some(Service::new(file.clone()));
    let mut tried = base.clone();
    tried.theme.name = "thirty seconds".into();
    let location = ThemeLocation(
        temporary
            .path()
            .join("trial.bezeltheme")
            .display()
            .to_string(),
    );
    bezel_themes::FsThemeStore
        .save(&location, &tried.theme, &tried.assets)
        .unwrap();
    let themes = BTreeMap::from([(f.key.clone(), location.0.clone())]);
    assert_eq!(
        f.engine.try_themes(&themes, None).unwrap(),
        0,
        "no live screen"
    );
    f.engine.scenes.get_mut(&f.key).unwrap().live = true;
    assert_eq!(
        f.engine
            .try_themes(&BTreeMap::from([(f.key.clone(), "missing".into())]), None)
            .unwrap(),
        0
    );
    assert_eq!(
        f.engine
            .try_themes(&themes, Some("profile-1".into()))
            .unwrap(),
        1
    );
    let scene = &f.engine.scenes[&f.key];
    assert_eq!(scene.runtime.theme().name, tried.theme.name);
    assert_eq!(scene.saved.theme.name, base.theme.name);
    let until = scene.preview_until.unwrap();
    let status = f.engine.profile_status().unwrap();
    assert_eq!(status.active.unwrap().profile, "profile-1");
    assert_eq!(status.preview.unwrap().themes, themes);
    f.engine
        .tick_at(until - Duration::from_millis(1), time())
        .unwrap();
    assert_eq!(
        f.engine.scenes[&f.key].runtime.theme().name,
        tried.theme.name
    );
    f.engine.tick_at(until, time()).unwrap();
    assert_eq!(
        f.engine.scenes[&f.key].runtime.theme().name,
        base.theme.name
    );
    assert!(f.engine.profile_status().unwrap().preview.is_none());
    assert_eq!(file.load().unwrap(), config);
}
