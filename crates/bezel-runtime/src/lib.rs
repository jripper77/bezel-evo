//! The one engine hosted by Light or, after transport failure, in Studio.
//! No Tauri, serial implementation, or second renderer lives here.
#![forbid(unsafe_code)]
pub mod element_preview;
mod images;
pub mod inspector_trial;
pub mod performance;
mod presentation;
pub mod profiles;
mod sampling;
mod spectrum;
mod targets;
mod wake;
pub use wake::Wake;
pub mod wire;
use bezel_core::{
    BezelError, Result,
    app::{HOST_VIDEO_FPS, HostVideo, ThemeRuntime, VideoState},
    domain::{
        clock::{Language, LocalTime},
        discovery::Screen,
        frame::Frame,
        frame::Rgba,
        geometry::Orientation,
        idle,
        sensor::SensorInfo,
        theme::{
            Anchor, AssetRef, Background, BoxF, Element, ElementId, ElementKind, Fit, HAlign,
            Paint, Playback, TextContent, TextOverflow, TextStyle, Theme, VAlign,
        },
    },
    ports::{
        DeviceBus, FrameRenderer, MediaLocation, MediaTranscoder, RuntimeEndpoint, ScreenConnector,
        ScreenLink, SensorSource,
    },
};
use bezel_power::diagnostics::{LightEvent, light};
pub use images::Images;
use std::{
    collections::BTreeMap,
    path::PathBuf,
    sync::Arc,
    time::{Duration, Instant},
};

/// Maximum persistent scenes and bytes of each complete document.
pub const MAX_SCENES: usize = 16;
/// The shortest step between two pictures of a scene no screen paces.
const EDITOR_FRAME: Duration = Duration::from_millis(50);

/// When a scene no screen paces draws next: never sooner than
/// [`EDITOR_FRAME`] after its last picture while the editor looks at it, a
/// second when nobody does (its motion would otherwise run at every tick).
fn unpaced_due(due: Duration, rendered_at: Duration, viewed: bool) -> Duration {
    let step = if viewed {
        EDITOR_FRAME
    } else {
        Duration::from_secs(1)
    };
    due.max(rendered_at + step)
}
/// Scenes whose key starts with this belong to the connected editor alone
/// (its document outside a live preview): they are dropped when it leaves.
pub const EDITOR_SCENE: &str = "__editor__";
pub const MAX_DOCUMENT: usize = 64 * 1024 * 1024;
const MAX_ALL_ASSETS: usize = 256 * 1024 * 1024;
/// Shortest interval between two spectrum frames, under the audio worker's 50 ms step.
const SPECTRUM_FRAME: Duration = Duration::from_millis(40);
/// Step between the frames of a profile cover: up to 60 a second, as fast as
/// the screen takes them (its presenter keeps only the newest).
const COVER_FRAME: Duration = Duration::from_millis(16);
/// A complete document, with asset bytes outside the wire JSON.
#[derive(Clone, PartialEq)]
pub struct Document {
    pub theme: Theme,
    pub assets: BTreeMap<AssetRef, Vec<u8>>,
}
impl Document {
    fn bytes(&self) -> usize {
        self.assets.values().map(Vec::len).sum()
    }
    fn validate(&self) -> Result<()> {
        let size = self.theme.canvas;
        if self.assets.len() > 4096
            || self.bytes() > MAX_DOCUMENT
            || size.width == 0
            || size.height == 0
            || size.width > 2320
            || size.height > 2320
            || size.area() > 1080 * 2320
        {
            return Err(invalid("document exceeds engine limits"));
        }
        for asset in self.assets.keys() {
            bezel_themes::native::safe_asset_path(asset).map_err(|e| invalid(&e.to_string()))?;
        }
        Ok(())
    }
}
fn invalid(message: &str) -> BezelError {
    BezelError::InvalidInput(message.into())
}
struct Scene {
    performance: performance::Performance,
    last_render_millis: f64,
    overlay_at: Duration,
    id: u64,
    runtime: ThemeRuntime,
    saved: Document,
    revision: u64,
    link: Option<presentation::Link>,
    presenter: Option<presentation::Presenter>,
    sample_origin: Duration,
    screen: Option<Screen>,
    live: bool,
    frame: Option<Arc<Frame>>,
    sequence: u64,
    queued_sequence: Option<u64>,
    dirty: bool,
    orientation: Option<bezel_core::domain::geometry::Orientation>,
    source: Option<MediaLocation>,
    video_started: Duration,
    video_dirty: bool,
    rendered_at: Duration,
    error: Option<String>,
    cards: bool,
    profile_restore: Option<Document>,
    profile_id: Option<String>,
    profile_edit_at: Option<Instant>,
    profile_animation: Option<profiles::Animation>,
    /// Until when a tried profile stays on show; the profile service then
    /// covers it again and returns to what the screen should show.
    preview_until: Option<Duration>,
    /// While the monitors are off the scene shows the moving logo: what it
    /// showed before, given back when they come back on ([`Engine::idle`]).
    idle_restore: Option<Document>,
    /// The logo's place ([`idle::logo_box`]) the scene shows now.
    idle_slot: Option<u64>,
}
impl Scene {
    fn restore_profile(&mut self) {
        self.profile_animation = None;
        self.preview_until = None;
        if let Some(document) = self.profile_restore.take() {
            self.update(document);
            self.runtime.automatic_cards(self.cards);
        }
        self.profile_id = None;
    }
    /// Shows `document`; while the logo stands in for the scene, keeps it
    /// for the end of idle instead.
    fn show(&mut self, document: Document) {
        match &mut self.idle_restore {
            Some(kept) => *kept = document,
            None => self.update(document),
        }
    }
    /// Shows the logo at its `slot`-th place, keeping what the scene showed
    /// (its profile closed first) for the end of idle.
    fn show_idle_logo(&mut self, slot: u64) {
        if self.idle_restore.is_none() {
            self.restore_profile();
            self.idle_restore = Some(Document {
                theme: self.runtime.theme().clone(),
                assets: self.runtime.assets().clone(),
            });
        }
        let Some(shape) = &self.idle_restore else {
            return;
        };
        let mut document = resting_document(&shape.theme, LOGO);
        let place = idle::logo_box(shape.theme.canvas, slot);
        document.theme.elements[0].frame = BoxF::new(
            place.x as f32,
            place.y as f32,
            place.width as f32,
            place.height as f32,
        );
        self.update(document);
        self.idle_slot = Some(slot);
        self.dirty = true;
    }
    /// Ends idle: the scene shows what it showed before.
    fn leave_idle(&mut self) {
        if let Some(document) = self.idle_restore.take() {
            self.update(document);
        }
        self.idle_slot = None;
        self.queued_sequence = None;
        self.dirty = true;
    }
    fn update(&mut self, document: Document) {
        if self.runtime.theme() == &document.theme && self.runtime.assets() == &document.assets {
            return;
        }
        let asset = |theme: &Theme| match &theme.background {
            Background::Video { asset, .. } => Some(asset.clone()),
            _ => None,
        };
        let playback_before = self.runtime.video().clone();
        let before = asset(self.runtime.theme());
        let after = asset(&document.theme);
        let changed_video = before != after
            || after
                .as_ref()
                .is_some_and(|a| self.runtime.assets().get(a) != document.assets.get(a));
        if self.runtime.assets() == &document.assets {
            self.runtime.replace_theme(document.theme);
        } else {
            self.runtime.replace(document.theme, document.assets);
        }
        if self.runtime.video() != &playback_before {
            self.video_dirty = true;
        }
        if changed_video {
            self.source = None;
        }
        self.dirty = true;
    }
}
/// Sensor/media adapter replacement for explicit configuration changes.
pub type Configure = Box<
    dyn Fn(&wire::Options) -> Result<(Box<dyn SensorSource>, Box<dyn MediaTranscoder>)>
        + Send
        + Sync,
>;

type Handoff = (
    Arc<dyn bezel_core::ports::HandoffStore + Send + Sync>,
    fn() -> Duration,
);

/// Bezel's logo: Light shows it while it rests and on a screen it shows
/// without a theme of its own.
pub const LOGO: &[u8] = include_bytes!("../../../apps/bezel-studio/src-tauri/icons/icon.png");

/// The line under the logo on a screen Light shows without a theme.
pub fn unassigned_hint(language: Language) -> &'static str {
    match language {
        Language::Italian => "Usa Bezel Studio per impostare un tema",
        Language::PortugueseBr => "Use o Bezel Studio para definir um tema",
        Language::English => "Use Bezel Studio to set a theme",
    }
}

/// `shape`'s canvas, dark, with the logo and, under it, how to give the
/// screen a theme (D-2026-11-05-light-studio-split-5).
fn unassigned_document(shape: &Theme, language: Language) -> Document {
    let mut document = resting_document(shape, LOGO);
    let (width, height) = (shape.canvas.width as f32, shape.canvas.height as f32);
    let short = width.min(height);
    let edge = short * 0.42;
    let top = (height - edge - short * 0.2) / 2.0;
    document.theme.elements[0].frame = BoxF::new((width - edge) / 2.0, top, edge, edge);
    let mut hint = document.theme.elements[0].clone();
    hint.id = ElementId(2);
    hint.name = "Hint".into();
    hint.frame = BoxF::new(
        width * 0.05,
        top + edge + short * 0.04,
        width * 0.9,
        short * 0.14,
    );
    hint.kind = ElementKind::Text {
        content: TextContent::Static(unassigned_hint(language).into()),
        style: TextStyle {
            size: (short * 0.075).clamp(14.0, 40.0),
            paint: Paint::solid(Rgba::opaque(200, 204, 214)),
            align: HAlign::Center,
            valign: VAlign::Middle,
            overflow: Some(TextOverflow::Shrink),
            ..TextStyle::default()
        },
    };
    document.theme.elements.push(hint);
    document
}

/// Where the resting logo is kept among a scene's assets.
const RESTING_LOGO: &str = "assets/bezel-logo.png";

/// `shown`'s canvas, dark, with the logo `png` in the middle.
fn resting_document(shown: &Theme, png: &[u8]) -> Document {
    let mut theme = Theme::blank("Bezel", shown.canvas, shown.orientation);
    theme.canvas = shown.canvas;
    let (width, height) = (shown.canvas.width as f32, shown.canvas.height as f32);
    let edge = width.min(height) * 0.6;
    theme.elements.push(Element {
        color_by_value: None,
        visualization: None,
        value_transition: None,
        sensor_motion: None,
        card: None,
        rotation: 0.0,
        is_group: false,
        group_parent: None,
        card_member: None,
        id: ElementId(1),
        name: "Bezel".into(),
        frame: BoxF::new((width - edge) / 2.0, (height - edge) / 2.0, edge, edge),
        opacity: 1.0,
        visible: true,
        locked: false,
        kind: ElementKind::Image {
            fade: None,
            asset: AssetRef(RESTING_LOGO.into()),
            fit: Fit::Contain,
            anchor: Anchor::default(),
            playback: Playback::default(),
            radius: 0.0,
            corners: None,
        },
    });
    Document {
        theme,
        assets: BTreeMap::from([(AssetRef(RESTING_LOGO.into()), png.to_vec())]),
    }
}

/// Persistent service. Scene lifetimes do not depend on an editor connection.
pub struct Engine {
    debug_overlay: Option<performance::Corner>,
    targets: Arc<targets::Targets>,
    profiles: Option<profiles::Service>,
    theme_trial: Option<(Duration, profiles::PreviewStatus)>,
    scenes: BTreeMap<String, Scene>,
    renderer: Box<dyn FrameRenderer>,
    sensors: sampling::Sampler,
    wake: Wake,
    catalog_sequence: u64,
    images: Arc<Images>,
    media: Box<dyn MediaTranscoder>,
    bus: Arc<dyn DeviceBus + Send + Sync>,
    connector: Arc<dyn ScreenConnector + Send + Sync>,
    origin: Instant,
    clock: fn() -> LocalTime,
    language: Language,
    cache: PathBuf,
    handoff: Option<Handoff>,
    thumbnail_sensors: Option<Box<dyn SensorSource>>,
    configure: Option<Configure>,
    options: Option<wire::Options>,
    suspended: bool,
    catalog: Vec<SensorInfo>,
    next_id: u64,
    audio: Option<spectrum::Worker>,
    audio_enabled: bool,
    /// Last state each spectrum widget was drawn with, for Light diagnostics.
    spectrum_states: BTreeMap<String, bezel_core::domain::spectrum::AudioState>,
    /// The logo every screen shows while Light is switched off ([`Engine::rest`]).
    resting: Option<Vec<u8>>,
    /// The user's option: the screens idle while the monitors are off
    /// (on by default, D-2026-11-06-display-idle-saver-4).
    idle_saver: bool,
    /// The monitors are off and the screens show the moving logo.
    idle: bool,
    /// When the idle logo took its first place, at the first idle tick.
    idle_since: Option<Duration>,
}
impl Engine {
    /// All adapters are injected; this exact constructor serves real Light,
    /// in-process recovery, and fake-device regression tests.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        renderer: Box<dyn FrameRenderer>,
        sensors: Box<dyn SensorSource>,
        media: Box<dyn MediaTranscoder>,
        bus: Arc<dyn DeviceBus + Send + Sync>,
        connector: Arc<dyn ScreenConnector + Send + Sync>,
        language: Language,
        cache: PathBuf,
        clock: fn() -> LocalTime,
    ) -> Result<Self> {
        let origin = Instant::now();
        let targets = Arc::new(targets::Targets::load(if cache.as_os_str().is_empty() {
            PathBuf::new()
        } else {
            cache.join("screen-addresses.json")
        }));
        targets.remember(&bezel_core::app::discover_screens(bus.as_ref()).unwrap_or_default());
        let wake = Wake::default();
        let sensors = sampling::Sampler::new(sensors, origin, wake.clone())?;
        let catalog = sensors.latest().catalog.clone();
        Ok(Self {
            debug_overlay: None,
            targets,
            profiles: None,
            theme_trial: None,
            scenes: BTreeMap::new(),
            renderer,
            sensors,
            wake,
            catalog_sequence: 0,
            images: Arc::default(),
            media,
            bus,
            connector,
            origin,
            clock,
            language,
            cache,
            handoff: None,
            thumbnail_sensors: None,
            configure: None,
            options: None,
            suspended: false,
            catalog,
            next_id: 1,
            audio: None,
            audio_enabled: false,
            spectrum_states: BTreeMap::new(),
            resting: None,
            idle_saver: true,
            idle: false,
            idle_since: None,
        })
    }
    /// Inject process capture; consent remains disabled until locally configured.
    pub fn with_audio(mut self, factory: Arc<dyn bezel_core::ports::AudioCaptureFactory>) -> Self {
        self.audio = Some(spectrum::Worker::new(factory, self.wake.clone()));
        self
    }
    /// Local profiles are deliberately outside portable theme documents.
    pub fn with_profiles(mut self, file: profiles::ProfileFile) -> Self {
        self.profiles = Some(profiles::Service::new(file));
        self
    }
    /// Apply a locally stored choice; theme JSON can never grant capture permission.
    pub fn audio_consent(&mut self, enabled: bool) {
        self.set_audio_enabled(enabled);
    }
    fn set_audio_enabled(&mut self, enabled: bool) {
        if self.audio_enabled != enabled {
            light(
                LightEvent::AudioConsent,
                0,
                0.0,
                if enabled { "on" } else { "off" },
            );
        }
        self.audio_enabled = enabled;
    }
    /// Compatibility baseline used only when a physical connection is actually opened.
    pub fn with_handoff(
        mut self,
        store: Arc<dyn bezel_core::ports::HandoffStore + Send + Sync>,
        wall: fn() -> Duration,
    ) -> Self {
        self.handoff = Some((store, wall));
        self
    }
    /// Configure adapter factories; thumbnails use an isolated demo source.
    pub fn with_adapters(
        mut self,
        configure: Configure,
        thumbnails: Box<dyn SensorSource>,
    ) -> Self {
        self.configure = Some(configure);
        self.thumbnail_sensors = Some(thumbnails);
        self
    }
    /// Change sources in place without replacing scene runtimes or histories.
    pub fn options(&mut self, options: wire::Options) -> Result<()> {
        if self.options.as_ref() == Some(&options) {
            return Ok(());
        }
        if options.ping_host.len() > 253
            || options.ffmpeg.as_ref().is_some_and(|p| p.len() > 32768)
            || options
                .presentmon_path
                .as_ref()
                .is_some_and(|p| p.len() > 32768)
        {
            return Err(invalid("option bounds"));
        }
        // Audio consent alone never restarts the sensor reader.
        let sources_changed = self.options.as_ref().is_none_or(|old| {
            old.ping_host != options.ping_host
                || old.ffmpeg != options.ffmpeg
                || old.presentmon_path != options.presentmon_path
        });
        if sources_changed && let Some(configure) = &self.configure {
            let (sensors, media) = configure(&options)?;
            self.sensors.replace(sensors);
            self.media = media;
            for scene in self.scenes.values_mut() {
                scene.runtime.use_catalog(&self.catalog);
                scene.dirty = true;
            }
        }
        self.set_audio_enabled(options.audio_capture);
        self.set_idle_saver(options.idle_saver);
        self.diagnostic_overlay(options.debug_overlay);
        self.options = Some(options);
        Ok(())
    }

    /// Transient diagnostics belong only to physical output, never to themes.
    pub fn diagnostic_overlay(&mut self, corner: Option<performance::Corner>) {
        if self.debug_overlay == corner {
            return;
        }
        self.debug_overlay = corner;
        for scene in self.scenes.values_mut() {
            scene.dirty = true;
            scene.queued_sequence = None;
            scene.overlay_at = Duration::ZERO;
        }
        self.wake.notify();
    }
    /// Register/update a draft, or commit the document only after Studio saved it.
    /// Revisions are scoped to the connected editor and reset on disconnect.
    pub fn document(
        &mut self,
        key: &str,
        revision: u64,
        document: Document,
        saved: bool,
    ) -> Result<()> {
        document.validate()?;
        if key.is_empty() || key.len() > 256 {
            return Err(invalid("invalid scene key"));
        }
        if let Some(scene) = self.scenes.get(key) {
            if revision != 0 && revision <= scene.revision {
                return Err(invalid("obsolete draft revision"));
            }
        } else if self.scenes.len() >= MAX_SCENES {
            return Err(invalid("too many scenes"));
        }
        let other_bytes: usize = self
            .scenes
            .iter()
            .filter(|(k, _)| k.as_str() != key)
            .map(|(_, s)| {
                s.runtime.assets().values().map(Vec::len).sum::<usize>()
                    + s.saved.bytes()
                    + s.profile_restore.as_ref().map_or(0, Document::bytes)
            })
            .sum();
        if other_bytes.saturating_add(document.bytes().saturating_mul(2)) > MAX_ALL_ASSETS {
            return Err(invalid("engine asset budget exceeded"));
        }
        if let Some(scene) = self.scenes.get_mut(key) {
            scene.restore_profile();
            if scene.runtime.theme() != &document.theme
                || scene.runtime.assets() != &document.assets
            {
                self.images.forget(key);
            }
            if saved {
                scene.saved = document.clone();
            }
            scene.show(document);
            scene.revision = revision;
        } else {
            let mut runtime = ThemeRuntime::new(
                document.theme.clone(),
                document.assets.clone(),
                self.language,
            );
            runtime.use_catalog(&self.catalog);
            runtime.reuse_recent_frames(true);
            self.scenes.insert(
                key.into(),
                Scene {
                    performance: performance::Performance::default(),
                    last_render_millis: 0.0,
                    overlay_at: Duration::ZERO,
                    id: self.next_id,
                    runtime,
                    saved: document,
                    revision,
                    link: None,
                    presenter: None,
                    sample_origin: self.sensors.latest().scheduled_at,
                    screen: None,
                    live: false,
                    frame: None,
                    sequence: 0,
                    queued_sequence: None,
                    dirty: true,
                    orientation: None,
                    source: None,
                    video_started: Duration::ZERO,
                    video_dirty: true,
                    rendered_at: Duration::ZERO,
                    error: None,
                    cards: true,
                    profile_restore: None,
                    profile_id: None,
                    profile_edit_at: None,
                    profile_animation: None,
                    preview_until: None,
                    idle_restore: None,
                    idle_slot: None,
                },
            );
            self.next_id += 1;
        }
        self.schedule_sensors();
        self.wake.notify();
        light(
            if saved {
                LightEvent::Saved
            } else {
                LightEvent::Draft
            },
            0,
            0.0,
            "document",
        );
        Ok(())
    }
    fn schedule_sensors(&self) {
        let (periods, wanted) = self.sensor_schedule();
        self.sensors.schedule(periods, wanted);
    }
    /// What the sensor reader samples and how often: nothing while the
    /// computer sleeps or the monitors are off.
    fn sensor_schedule(
        &self,
    ) -> (
        Vec<(Duration, Duration)>,
        bezel_core::domain::sensor::Wanted,
    ) {
        use bezel_core::domain::{history::Cadence, sensor::Wanted};
        let mut periods = Vec::new();
        let mut wanted = Wanted::nothing();
        if !self.suspended && !self.idle {
            if let Some(profiles) = &self.profiles {
                let activity = profiles.wanted();
                if activity != Wanted::nothing() {
                    wanted = wanted.union(&activity);
                    periods.push((Duration::from_millis(500), Duration::ZERO));
                }
            }
            for scene in self.scenes.values() {
                wanted = wanted.union(scene.runtime.wanted());
                periods.push((scene.runtime.refresh(), scene.sample_origin));
                for series in scene.runtime.theme().graph_series() {
                    if let Cadence::Every(period) = series.cadence {
                        periods.push((period, scene.sample_origin));
                    }
                }
            }
        }
        periods.sort();
        periods.dedup();
        (periods, wanted)
    }
    /// Reconcile only explicit persisted live choices after legacy direct recovery.
    pub fn follow_saved_live(&mut self, keys: &[String]) {
        for (key, scene) in &mut self.scenes {
            if !keys.contains(key) {
                scene.live = false;
                scene.link = None;
                scene.presenter = None;
            }
        }
    }
    /// What the profiles do now (the active profile and why, every rule's
    /// truth); `None` without the profile service.
    pub fn profile_status(&self) -> Option<profiles::Status> {
        let mut status = self.profiles.as_ref().map(profiles::Service::status)?;
        if let Some((until, preview)) = &self.theme_trial {
            let mut preview = preview.clone();
            preview.remaining_seconds = until
                .saturating_sub(self.origin.elapsed())
                .as_secs_f64()
                .ceil() as u32;
            if preview.remaining_seconds > 0 {
                status.active = preview.profile.clone().map(|profile| profiles::Active {
                    profile,
                    reason: profiles::Reason::Manual,
                });
                status.preview = Some(preview);
            }
        }
        Some(status)
    }
    /// Attach a physical display once, preserving the runtime and its clocks.
    /// Tries a profile cover on every live screen, whoever opened its scene
    /// (Light's saved themes or Studio's drafts). It closes over what the
    /// screen shows, opens on the theme `themes` names for that screen (the
    /// profile tried, saved or not; the same theme for a screen it does not
    /// name or whose theme cannot be shown there), and after
    /// [`profiles::PREVIEW_HOLD`] the profile service covers it again and
    /// returns. Screens that show nothing yet are left alone, and nothing
    /// moves with no cover chosen. Answers how many screens it covers.
    pub fn preview_cover(
        &mut self,
        transition: &profiles::TransitionSettings,
        themes: &BTreeMap<String, String>,
    ) -> Result<usize> {
        let Some(cover) = transition.cover()? else {
            return Ok(0);
        };
        if self.idle {
            return Ok(0);
        }
        let now = self.origin.elapsed();
        let mut covered = 0;
        for (key, scene) in self
            .scenes
            .iter_mut()
            .filter(|(_, s)| s.live && s.frame.is_some())
        {
            let show = themes
                .get(key)
                .and_then(|path| profiles::load_document(path))
                .filter(|document| profiles::fits(scene, document));
            let base = scene
                .profile_animation
                .as_ref()
                .map_or(0.0, |a| a.coverage(now));
            scene.profile_animation = Some(profiles::Animation::preview(
                scene.profile_id.clone(),
                cover,
                transition.duration_ms,
                now,
                base,
                show,
            ));
            scene.dirty = true;
            covered += 1;
        }
        if covered > 0 {
            self.wake.notify();
        }
        Ok(covered)
    }
    /// Show the requested themes for thirty seconds, preserving the editor and config.
    pub fn try_themes(
        &mut self,
        themes: &BTreeMap<String, String>,
        profile: Option<String>,
    ) -> Result<usize> {
        // The service owns expiry and restoration. Without it a trial cannot run.
        if self.profiles.is_none() || self.idle {
            return Ok(0);
        }
        let until = self.origin.elapsed() + Duration::from_secs(30);
        let documents: Vec<_> = themes
            .iter()
            .filter_map(|(key, path)| {
                let scene = self.scenes.get(key)?;
                if !scene.live || scene.frame.is_none() {
                    return None;
                }
                let document =
                    profiles::load_document(path).filter(|d| profiles::fits(scene, d))?;
                Some((key.clone(), path.clone(), document))
            })
            .collect();
        if documents.is_empty() {
            return Ok(0);
        }
        // A new trial replaces the previous one, including screens no longer targeted.
        if self.theme_trial.take().is_some() {
            for scene in self
                .scenes
                .values_mut()
                .filter(|s| s.profile_id.as_deref() == Some(profiles::PREVIEW_ID))
            {
                scene.restore_profile();
                scene.dirty = true;
            }
        }
        let mut budget: usize = self.scenes.values().map(profiles::scene_bytes).sum();
        let mut shown = BTreeMap::new();
        for (key, path, document) in documents {
            let Some(scene) = self.scenes.get_mut(&key) else {
                continue;
            };
            let before = profiles::scene_bytes(scene);
            let retained = scene.profile_restore.as_ref().map_or_else(
                || scene.runtime.assets().values().map(Vec::len).sum(),
                Document::bytes,
            );
            let after = scene.saved.bytes() + retained + document.bytes();
            let next = budget.saturating_sub(before).saturating_add(after);
            if next > MAX_ALL_ASSETS {
                continue;
            }
            budget = next;
            if scene.profile_restore.is_none() {
                scene.profile_restore = Some(Document {
                    theme: scene.runtime.theme().clone(),
                    assets: scene.runtime.assets().clone(),
                });
            }
            scene.profile_animation = None;
            scene.profile_id = Some(profiles::PREVIEW_ID.into());
            scene.update(document);
            scene.runtime.automatic_cards(true);
            scene.preview_until = Some(until);
            scene.dirty = true;
            shown.insert(key, path);
        }
        let count = shown.len();
        if count > 0 {
            self.theme_trial = Some((
                until,
                profiles::PreviewStatus {
                    themes: shown,
                    profile,
                    remaining_seconds: 30,
                },
            ));
            self.wake.notify();
        }
        Ok(count)
    }

    /// Light shows the screen `key` without a theme of its own: the logo and
    /// how to set one, in Light's language, as its saved document. The shape
    /// is the scene's, else the connected screen's in `orientation` (its
    /// usual one when `None`); a screen not connected is left alone.
    pub fn unassigned(&mut self, key: &str, orientation: Option<Orientation>) -> Result<()> {
        let shape = match self.scenes.get(key) {
            Some(scene) => scene.runtime.theme().clone(),
            None => {
                let screens = bezel_core::app::discover_screens(self.bus.as_ref())?;
                let Some(model) = screens
                    .iter()
                    .find(|screen| screen.answers_to(key))
                    .and_then(Screen::model)
                else {
                    return Ok(());
                };
                let panel = model.panel.portrait();
                let usual = if u64::from(panel.height) >= 2 * u64::from(panel.width) {
                    Orientation::Landscape
                } else {
                    model.native_orientation
                };
                Theme::blank("Bezel", model.panel, orientation.unwrap_or(usual))
            }
        };
        self.document(key, 0, unassigned_document(&shape, self.language), true)?;
        self.revert(key);
        self.live(key, true)
    }
    /// The scene shows its saved document again and its profile may cover
    /// it at once: an editor's draft there is dropped. An unknown scene is
    /// left alone.
    pub fn revert(&mut self, key: &str) {
        let Some(scene) = self.scenes.get_mut(key) else {
            return;
        };
        scene.restore_profile();
        scene.profile_edit_at = None;
        self.images.unview(key);
        if scene.idle_restore.is_some() {
            scene.show(scene.saved.clone());
        } else if scene.runtime.theme() != &scene.saved.theme
            || scene.runtime.assets() != &scene.saved.assets
        {
            self.images.forget(key);
            scene.update(scene.saved.clone());
        }
        scene.revision = 0;
        self.schedule_sensors();
        self.wake.notify();
    }
    pub fn live(&mut self, key: &str, on: bool) -> Result<()> {
        let scene = self
            .scenes
            .get_mut(key)
            .ok_or_else(|| invalid("unknown scene"))?;
        if scene.live == on && (!on || scene.presenter.is_some()) {
            return Ok(());
        }
        // No connection opens while the monitors are off: the end of idle
        // opens it.
        if on && self.idle && scene.presenter.is_none() {
            scene.live = true;
            return Ok(());
        }
        if on && let Some(presenter) = &scene.presenter {
            presenter.pause(false);
            scene.dirty = true;
            scene.orientation = None;
            if matches!(scene.runtime.video(), VideoState::OnDevice(_)) {
                scene.video_dirty = true;
            }
        } else if on {
            scene.presenter = Some(presentation::Presenter::start(
                scene.id,
                targets::Target {
                    key: key.into(),
                    remembered: self.targets.clone(),
                },
                self.bus.clone(),
                self.connector.clone(),
                self.handoff.clone(),
                scene.runtime.theme().clone(),
                self.wake.clone(),
            )?);
            scene.dirty = true;
        } else if let Some(presenter) = &scene.presenter {
            presenter.pause(true);
        }
        scene.live = on;
        self.wake.notify();
        Ok(())
    }
    fn source(
        scene: &mut Scene,
        media: &mut dyn MediaTranscoder,
        cache: &std::path::Path,
    ) -> Result<()> {
        if scene.source.is_some() {
            return Ok(());
        }
        let Background::Video { asset, .. } = &scene.runtime.theme().background else {
            return Ok(());
        };
        if let Some(bytes) = scene.runtime.assets().get(asset) {
            std::fs::create_dir_all(cache).map_err(|e| BezelError::Transport(e.to_string()))?;
            let name = std::path::Path::new(&asset.0)
                .file_name()
                .ok_or_else(|| invalid("invalid video asset"))?
                .to_string_lossy();
            let path = cache.join(format!("scene-{}-{}-{name}", std::process::id(), scene.id));
            std::fs::write(&path, bytes).map_err(|e| BezelError::Transport(e.to_string()))?;
            let location = MediaLocation(path.to_string_lossy().into_owned());
            scene.runtime.set_video_info(media.probe(&location).ok());
            scene.source = Some(location);
        }
        Ok(())
    }
    /// Readies the scene's screen for its next frame: the theme fits it, its
    /// orientation, and its video started or stopped. `false` while the
    /// screen still answers a query (the frame waits).
    fn playback(
        scene: &mut Scene,
        media: &mut dyn MediaTranscoder,
        cache: &std::path::Path,
        now: Duration,
    ) -> Result<bool> {
        Self::source(scene, media, cache)?;
        if let Some(link) = &mut scene.link {
            if scene
                .runtime
                .theme()
                .misfit(link.identity().model.panel)
                .is_some()
            {
                return Err(invalid("theme does not fit screen"));
            }
            let orientation = scene.runtime.theme().orientation;
            if scene.orientation != Some(orientation) {
                link.set_orientation(orientation)?;
                scene.orientation = Some(orientation);
            }
            if scene.video_dirty || matches!(scene.runtime.video(), VideoState::NotStarted) {
                link.pending = false;
                let host = scene.source.as_ref().map(|source| HostVideo {
                    media: &mut *media,
                    source: source.clone(),
                    fps: HOST_VIDEO_FPS,
                });
                let result = scene.runtime.start_video(link, host);
                if link.pending {
                    return Ok(false);
                }
                result?;
                scene.video_started = now;
                scene.video_dirty = false;
            }
        }
        if scene.link.is_none()
            && !scene.live
            && matches!(scene.runtime.video(), VideoState::NotStarted)
            && let Some(source) = &scene.source
        {
            scene.runtime.start_host_video(HostVideo {
                media,
                source: source.clone(),
                fps: HOST_VIDEO_FPS,
            })?;
            scene.video_started = now;
        }
        Ok(true)
    }
    /// The monitors went off (`on`) or came back. While they are off every
    /// screen shown with an open link dims (the caller's, through the
    /// owner) and shows the logo, moved every [`idle::interval`]; nothing
    /// samples the sensors, no spectrum listens, no theme frame is drawn,
    /// and no connection opens. When they come back each scene shows what
    /// it showed before: its theme, or the resting logo while Light is off
    /// ([`Engine::rest`], also when switched off meanwhile). With the
    /// option off ([`Engine::set_idle_saver`]) the screens never idle.
    pub fn idle(&mut self, on: bool) {
        let on = on && self.idle_saver;
        if self.idle == on {
            return;
        }
        self.idle = on;
        self.idle_since = None;
        if !on {
            for scene in self.scenes.values_mut() {
                scene.leave_idle();
            }
        }
        tracing::info!(target: "bezel_light", idle = on, "monitors idle");
        self.schedule_sensors();
        self.wake.notify();
    }
    /// The screens show the idle logo ([`Engine::idle`]).
    pub fn is_idle(&self) -> bool {
        self.idle
    }
    /// The user's option: the screens idle while the monitors are off.
    pub fn idle_saver(&self) -> bool {
        self.idle_saver
    }
    /// Changes the option; turning it off while idle leaves idle at once.
    pub fn set_idle_saver(&mut self, on: bool) {
        self.idle_saver = on;
        if !on {
            self.idle(false);
        }
    }
    /// A tick while the monitors are off: each live scene with an open link
    /// shows the logo at its place for `now` and draws only when the place
    /// changes; a presenter without a link stops (no reconnection). When the
    /// next move is due.
    fn idle_tick(&mut self, now: Duration, time: LocalTime) -> Duration {
        let since = *self.idle_since.get_or_insert(now);
        let mut next = Duration::from_secs(60);
        for scene in self.scenes.values_mut() {
            Self::poll_presenter(scene);
            if scene.link.is_none() {
                scene.presenter = None;
                continue;
            }
            let Some(family) = scene
                .link
                .as_ref()
                .filter(|_| scene.live)
                .map(|link| link.identity().model.family)
            else {
                continue;
            };
            let every = idle::interval(family);
            let slot = (now.saturating_sub(since).as_nanos() / every.as_nanos().max(1)) as u64;
            if scene.idle_slot != Some(slot) {
                scene.show_idle_logo(slot);
            }
            let moves = u32::try_from(slot.saturating_add(1)).unwrap_or(u32::MAX);
            let mut wait = (since + every.saturating_mul(moves)).saturating_sub(now);
            if scene.dirty {
                let (renderer, media) = (self.renderer.as_mut(), self.media.as_mut());
                match Self::present_idle(scene, renderer, media, &self.cache, now, time) {
                    Ok(true) => {}
                    Ok(false) => wait = Duration::from_millis(5),
                    Err(error) => {
                        scene.error = Some(error.to_string());
                        light(LightEvent::ScreenError, scene.id, 0.0, "scene");
                    }
                }
            }
            next = next.min(wait);
        }
        next.max(Duration::from_millis(5))
    }
    /// Takes what the scene's presenter reports: a link opened or lost.
    fn poll_presenter(scene: &mut Scene) {
        let Some(presenter) = &scene.presenter else {
            return;
        };
        let update = presenter.poll();
        if update.error {
            scene.error = Some("screen I/O failed".into());
            scene.link = None;
            scene.runtime.forget_screen();
            scene.dirty = true;
        }
        if let Some((screen, link)) = update.opened {
            scene.screen = Some(screen);
            scene.link = Some(link);
            scene.queued_sequence = None;
            scene.orientation = None;
            scene.runtime.forget_screen();
            scene.dirty = true;
            scene.error = None;
        }
    }
    /// Draws the idle logo and hands it to the screen; `false` while the
    /// screen still answers a query (the frame waits).
    fn present_idle(
        scene: &mut Scene,
        renderer: &mut dyn FrameRenderer,
        media: &mut dyn MediaTranscoder,
        cache: &std::path::Path,
        now: Duration,
        time: LocalTime,
    ) -> Result<bool> {
        // The logo plays no video: this stops the one the screen played.
        let ready = Self::playback(scene, media, cache, now)?;
        if !ready {
            return Ok(false);
        }
        renderer.set_scene(scene.id);
        renderer.set_motion(false);
        let frame = scene
            .runtime
            .render_at(renderer, time, now, Duration::ZERO)?;
        if scene
            .frame
            .as_ref()
            .is_none_or(|previous| previous.as_ref() != &frame)
        {
            scene.sequence = scene.sequence.saturating_add(1);
            scene.frame = Some(Arc::new(frame));
        }
        if scene.queued_sequence != Some(scene.sequence)
            && let (Some(presenter), Some(frame)) = (&scene.presenter, &scene.frame)
        {
            presenter.frame(frame.clone());
            scene.queued_sequence = Some(scene.sequence);
        }
        scene.rendered_at = now;
        scene.dirty = false;
        Ok(true)
    }
    /// Light's off switch: with `logo` (a PNG) every screen shows it on a
    /// dark background instead of its theme, and profiles wait; `None` puts
    /// the saved themes back and lets profiles choose again.
    pub fn rest(&mut self, logo: Option<Vec<u8>>) {
        if logo.is_none() && self.resting.is_none() {
            return;
        }
        for (key, scene) in &mut self.scenes {
            scene.restore_profile();
            let document = match &logo {
                Some(png) => resting_document(scene.runtime.theme(), png),
                None => scene.saved.clone(),
            };
            scene.show(document);
            scene.dirty = true;
            self.images.forget(key);
        }
        if logo.is_none()
            && let Some(profiles) = &mut self.profiles
        {
            *profiles = profiles::Service::new(profiles.file());
        }
        self.resting = logo;
        self.schedule_sensors();
        self.wake.notify();
    }
    /// What each live screen shows, for Light's menu: its model's name once
    /// connected, and the theme's name (the saved one while resting).
    pub fn shown(&self) -> Vec<(Option<&'static str>, String)> {
        self.scenes
            .values()
            .filter(|scene| scene.live)
            .map(|scene| {
                let theme = if self.resting.is_some() {
                    &scene.saved.theme
                } else if let Some(kept) = &scene.idle_restore {
                    &kept.theme
                } else {
                    scene.runtime.theme()
                };
                (
                    scene
                        .screen
                        .as_ref()
                        .and_then(Screen::model)
                        .map(|m| m.name),
                    theme.name.clone(),
                )
            })
            .collect()
    }
    /// Whether Light is switched off ([`Engine::rest`]).
    pub fn resting(&self) -> bool {
        self.resting.is_some()
    }
    /// Stop scheduling writes during final shutdown; links remain available to finish.
    pub fn suspend(&mut self, on: bool) {
        let resuming = self.suspended && !on;
        self.suspended = on;
        for scene in self.scenes.values_mut() {
            if let Some(presenter) = &scene.presenter {
                presenter.pause(on);
            }
            if !on {
                scene.dirty = true;
                if resuming && matches!(scene.runtime.video(), VideoState::OnDevice(_)) {
                    scene.video_dirty = true;
                }
            }
        }
        self.schedule_sensors();
    }
    /// Physical link loss/legacy takeover preserves document state, not handles.
    pub fn disconnect_links(&mut self) {
        for scene in self.scenes.values_mut() {
            scene.link = None;
            scene.presenter = None;
            scene.runtime.forget_screen();
            scene.orientation = None;
            scene.dirty = true;
        }
    }
    /// Run at the engine's own monotonic time, independent of canvas requests.
    pub fn tick(&mut self) -> Result<Duration> {
        let began = Instant::now();
        let delay = self.tick_at(self.origin.elapsed(), (self.clock)())?;
        // Deadlines are relative to the beginning of the tick. Rendering is
        // part of the frame budget, not extra sleep after every frame.
        Ok(delay.saturating_sub(began.elapsed()))
    }
    /// Deterministic clock entry for tests and the same loop's scheduling.
    pub fn tick_at(&mut self, now: Duration, time: LocalTime) -> Result<Duration> {
        if self
            .theme_trial
            .as_ref()
            .is_some_and(|(until, _)| now >= *until)
        {
            self.theme_trial = None;
        }
        if self.suspended {
            if let Some(audio) = &self.audio {
                audio.sync(Vec::new(), false);
            }
            return Ok(Duration::from_millis(50));
        }
        if self.idle {
            if let Some(audio) = &self.audio {
                audio.sync(Vec::new(), false);
            }
            return Ok(self.idle_tick(now, time));
        }
        let reading = self.sensors.latest();
        if self.resting.is_none()
            && let Some(mut profiles) = self.profiles.take()
        {
            let changed = profiles.tick(self, &reading.snapshot, now, time);
            self.profiles = Some(profiles);
            if changed {
                self.schedule_sensors();
            }
        }
        if reading.sequence != self.catalog_sequence && !reading.catalog.is_empty() {
            self.catalog = reading.catalog.clone();
            self.catalog_sequence = reading.sequence;
        }
        let mut requests = Vec::new();
        for (key, scene) in &self.scenes {
            let screen_consumes = scene.live
                && scene
                    .link
                    .as_ref()
                    .is_some_and(|l| !l.presentation_paused());
            if !screen_consumes && !self.images.viewed(key) {
                continue;
            }
            // Every Card face, shown or not: a face can come into view at any frame
            // (playback triggers, timers, transitions) and must find its spectrum live.
            let theme = scene.runtime.theme();
            for element in theme
                .elements
                .iter()
                .filter(|e| theme.visible_without_face(e))
            {
                if let bezel_core::domain::theme::ElementKind::Text {
                    content: bezel_core::domain::theme::TextContent::Spectrum(config),
                    ..
                } = &element.kind
                {
                    if requests.len() >= 256 {
                        continue;
                    }
                    let snapshot = if reading.sequence > 0 {
                        scene
                            .runtime
                            .playback_snapshot(&reading.snapshot, reading.scheduled_at)
                    } else {
                        scene.runtime.snapshot()
                    };
                    let selected =
                        if config.audio == bezel_core::domain::spectrum::SpectrumAudio::System {
                            // The PC's output needs no Player: it is always "playing".
                            Ok(None)
                        } else {
                            config.session(scene.runtime.theme(), snapshot)
                        };
                    let (source, playing, state) = match selected {
                        Ok(None)
                            if config.audio
                                == bezel_core::domain::spectrum::SpectrumAudio::System =>
                        {
                            (
                                bezel_core::domain::spectrum::SYSTEM_SOURCE.to_owned(),
                                true,
                                bezel_core::domain::spectrum::AudioState::Live,
                            )
                        }
                        Ok(Some(s)) => (
                            s.source.clone(),
                            s.playing,
                            bezel_core::domain::spectrum::AudioState::Live,
                        ),
                        // No media session is silence: zero bars, no message.
                        Ok(None) => (
                            String::new(),
                            false,
                            bezel_core::domain::spectrum::AudioState::Live,
                        ),
                        Err(state) => (String::new(), false, state),
                    };
                    requests.push(spectrum::Request {
                        key: format!("{}/{}", scene.id, element.id.0),
                        source,
                        playing,
                        state,
                        config: config.clone(),
                    });
                }
            }
        }
        let audio_frames = self
            .audio
            .as_ref()
            .map(|a| a.sync(requests.clone(), self.audio_enabled))
            .unwrap_or_default();
        let mut next = Duration::from_millis(50);
        for (key, scene) in &mut self.scenes {
            if scene.live && scene.presenter.is_none() {
                scene.presenter = Some(presentation::Presenter::start(
                    scene.id,
                    targets::Target {
                        key: key.clone(),
                        remembered: self.targets.clone(),
                    },
                    self.bus.clone(),
                    self.connector.clone(),
                    None,
                    scene.runtime.theme().clone(),
                    self.wake.clone(),
                )?);
            }
            if let Some(presenter) = &scene.presenter {
                let update = presenter.poll();
                if self.debug_overlay.is_some() {
                    for (at, transfer, present_ms) in update.deliveries {
                        let gap = scene.performance.count(true, transfer, at);
                        scene.performance.completed(
                            performance::Timings {
                                render_ms: scene.last_render_millis,
                                render_reused: false,
                                render_stats: Default::default(),
                                present_ms,
                                handoff_ms: 0.0,
                                sensors_ms: reading.millis,
                                refresh_ms: scene.runtime.refresh().as_secs_f64() * 1000.0,
                                animating: false,
                            },
                            transfer,
                            key,
                            true,
                            gap,
                            at,
                        );
                    }
                }
                if update.error {
                    scene.error = Some("screen I/O failed".into());
                    scene.link = None;
                    scene.runtime.forget_screen();
                    scene.dirty = true;
                }
                if let Some((screen, link)) = update.opened {
                    scene.screen = Some(screen);
                    scene.link = Some(link);
                    scene.queued_sequence = None;
                    scene.orientation = None;
                    scene.runtime.forget_screen();
                    scene.dirty = true;
                    scene.error = None;
                }
            }
            let result = (|| -> Result<Duration> {
                // A tried cover, closed: it shows the profile tried (if any) under
                // it, before the video and the frame of this tick, and opens on it.
                let tried = scene
                    .profile_animation
                    .as_mut()
                    .filter(|a| !a.apply && a.closed(now))
                    .map(|a| {
                        a.phase = profiles::Phase::Covered;
                        a.show.take()
                    });
                if let Some(Some(document)) = tried {
                    if scene.profile_restore.is_none() {
                        scene.profile_restore = Some(Document {
                            theme: scene.runtime.theme().clone(),
                            assets: scene.runtime.assets().clone(),
                        });
                    }
                    scene.profile_id = Some(profiles::PREVIEW_ID.into());
                    scene.update(document);
                    scene.runtime.automatic_cards(true);
                    scene.preview_until = Some(now + profiles::PREVIEW_HOLD);
                }
                if reading.sequence > 0
                    && scene.runtime.sample_on_time(
                        &mut sampling::Latest(reading.clone()),
                        reading.scheduled_at,
                    )?
                {
                    scene.dirty = true;
                }
                let audible = requests
                    .iter()
                    .any(|r| r.key.starts_with(&format!("{}/", scene.id)));
                // Audio and profile motion follow the physical presenter's pace.
                // Its slot and the audio worker both retain only their newest result.
                if scene.live
                    && (audible
                        || scene.profile_animation.is_some()
                        || (!scene.dirty
                            && (scene.runtime.graph_animating(now)
                                || scene.runtime.theme().elements.iter().any(|e| {
                                    e.sensor_motion.is_some() && scene.runtime.theme().is_visible(e)
                                }))))
                    && scene.presenter.as_ref().is_some_and(|p| !p.idle())
                {
                    return Ok(Duration::from_millis(50));
                }
                // Spectra draw at most every SPECTRUM_FRAME, a little under the audio
                // worker's step, so two equal 50 ms rhythms never beat and drop a result.
                // A wake inside that window (a worker result) comes back when it ends, not a
                // whole interval later. A render the window cannot hold off (a sensor sample,
                // a due timer) takes the newest spectra instead of drawing stale bars.
                let mut audio_wait = None;
                if now.saturating_sub(scene.rendered_at) < SPECTRUM_FRAME
                    && scene.frame.is_some()
                    && !scene.dirty
                    && now < scene.runtime.next_due()
                {
                    if audible {
                        audio_wait = Some(scene.rendered_at + SPECTRUM_FRAME - now);
                    }
                } else {
                    let frames = requests
                        .iter()
                        .filter(|r| r.key.starts_with(&format!("{}/", scene.id)))
                        .filter_map(|r| {
                            let id = r.key.rsplit('/').next()?.parse::<u32>().ok()?;
                            let state = if r.config.demo {
                                bezel_core::domain::spectrum::AudioState::Demo
                            } else if !self.audio_enabled {
                                bezel_core::domain::spectrum::AudioState::Disabled
                            } else {
                                // Without a capture worker (Studio's fallback engine) a
                                // live spectrum is silent bars, never a message.
                                r.state
                            };
                            let frame = if matches!(
                                state,
                                bezel_core::domain::spectrum::AudioState::Live
                                    | bezel_core::domain::spectrum::AudioState::Demo
                            ) {
                                audio_frames.get(&r.key).cloned().unwrap_or(
                                    bezel_core::domain::spectrum::SpectrumFrame {
                                        state,
                                        ..Default::default()
                                    },
                                )
                            } else {
                                bezel_core::domain::spectrum::SpectrumFrame {
                                    state,
                                    ..Default::default()
                                }
                            };
                            if self.spectrum_states.insert(r.key.clone(), state) != Some(state) {
                                light(LightEvent::Spectrum, scene.id, 0.0, spectrum::kind(state));
                            }
                            Some((ElementId(id), frame))
                        })
                        .collect();
                    if scene.runtime.set_spectra(frames) {
                        scene.dirty = true;
                    }
                }
                let mut due = scene.runtime.next_due();
                if self.debug_overlay.is_some() && scene.live {
                    due = due.min(scene.overlay_at + Duration::from_millis(250));
                }
                if scene.profile_animation.is_some() {
                    due = due.min(scene.rendered_at + COVER_FRAME);
                }
                if matches!(scene.runtime.video(), VideoState::Host) {
                    due = due.min(scene.rendered_at + Duration::from_secs(1) / HOST_VIDEO_FPS);
                }
                // No screen paces the editor's own scene: its motion is drawn
                // at most every EDITOR_FRAME while the editor looks at it, and
                // once a second when it does not. An edit (`dirty`) is still
                // drawn at once.
                if !scene.live && scene.frame.is_some() && key.starts_with(EDITOR_SCENE) {
                    due = unpaced_due(due, scene.rendered_at, self.images.viewed(key));
                }
                if !scene.dirty && now < due {
                    return Ok(audio_wait.map_or(due - now, |wait| wait.min(due - now)));
                }
                // A video/device failure must not keep the preceding document's
                // picture on the editor canvas: its poster and objects still render.
                let playback = Self::playback(scene, self.media.as_mut(), &self.cache, now);
                let playback_ready = match playback {
                    Ok(ready) => ready,
                    Err(error) => {
                        scene.error = Some(error.to_string());
                        light(LightEvent::ScreenError, scene.id, 0.0, "video");
                        false
                    }
                };
                self.renderer.set_scene(scene.id);
                self.renderer.set_motion(true);
                let render_started = Instant::now();
                let mut frame = scene.runtime.render_at(
                    self.renderer.as_mut(),
                    time,
                    now,
                    now.saturating_sub(scene.video_started),
                )?;
                let animating = scene.profile_animation.is_some();
                // The cover is painted over the frame, also over a video the
                // screen plays itself. The opening starts at the new theme's first
                // frame, so loading it never shortens the motion.
                let finished = scene.profile_animation.as_mut().is_some_and(|animation| {
                    animation.open(now);
                    let finished = animation.finished(now);
                    if !finished {
                        animation.cover.paint(
                            &mut frame,
                            animation.stage(),
                            animation.coverage(now),
                        );
                    }
                    finished
                });
                if finished {
                    scene.profile_animation = None;
                }
                let render_millis = render_started.elapsed().as_secs_f64() * 1000.0;
                scene.last_render_millis = render_millis;
                let frame_changed = scene
                    .frame
                    .as_ref()
                    .is_none_or(|previous| previous.as_ref() != &frame);
                if frame_changed {
                    scene.sequence = scene.sequence.saturating_add(1);
                    scene.frame = Some(Arc::new(frame));
                }
                let frame = scene
                    .frame
                    .as_ref()
                    .ok_or_else(|| invalid("missing rendered frame"))?
                    .clone();
                let overlay_due = self.debug_overlay.is_some()
                    && now >= scene.overlay_at + Duration::from_millis(250);
                if (overlay_due
                    || scene.queued_sequence != Some(scene.sequence)
                    || !requests
                        .iter()
                        .any(|r| r.key.starts_with(&format!("{}/", scene.id))))
                    && scene.live
                    && playback_ready
                    && scene.link.is_some()
                    && let Some(presenter) = &scene.presenter
                {
                    let outgoing = if let Some(corner) = self.debug_overlay {
                        let mut output = frame.as_ref().clone();
                        scene.performance.paint(&mut output, corner);
                        scene.overlay_at = now;
                        Arc::new(output)
                    } else {
                        frame.clone()
                    };
                    presenter.frame(outgoing);
                    scene.queued_sequence = Some(scene.sequence);
                }
                // An identical frame never reached the screen: it must not restart the
                // spectrum window and hold the next audio result back a whole frame.
                if frame_changed || !audible {
                    scene.rendered_at = now;
                }
                scene.dirty = !playback_ready;
                let mut delay = scene.runtime.next_due().saturating_sub(now);
                if self.debug_overlay.is_some() && scene.live {
                    delay = delay
                        .min((scene.overlay_at + Duration::from_millis(250)).saturating_sub(now));
                }
                if scene.profile_animation.is_some() {
                    delay = delay.min(COVER_FRAME);
                }
                if matches!(scene.runtime.video(), VideoState::Host) {
                    delay = delay.min(Duration::from_secs(1) / HOST_VIDEO_FPS);
                }
                if scene.profile_restore.is_none() && !animating {
                    self.images.publish(
                        key,
                        images::Snapshot {
                            render_millis,
                            frame,
                            sequence: scene.sequence,
                            next_ms: delay.as_millis().clamp(5, 1000) as u32,
                        },
                    );
                }
                Ok(if playback_ready {
                    delay
                } else {
                    delay.min(Duration::from_millis(5))
                })
            })();
            match result {
                Ok(delay) => next = next.min(delay),
                Err(error) => {
                    scene.error = Some(error.to_string());
                    light(LightEvent::ScreenError, scene.id, 0.0, "scene");
                }
            }
        }
        Ok(next.max(Duration::from_millis(5)))
    }
    /// Completion barrier inspected by shutdown callers outside the engine lock.
    pub fn quiescent(&self) -> bool {
        self.scenes.values().all(|s| {
            s.presenter
                .as_ref()
                .is_none_or(presentation::Presenter::idle)
        })
    }
    /// Readonly frames: no renderer, sensor source or device is touched.
    pub fn wake(&self) -> Wake {
        self.wake.clone()
    }
    pub fn images(&self) -> Arc<Images> {
        self.images.clone()
    }
    pub fn image(&self, key: &str, after: u64, max: u32) -> Result<wire::Image> {
        self.images.latest(key, after, max)
    }
    fn editing_profile(&mut self, key: &str) {
        let mut restored = false;
        if let Some(scene) = self.scenes.get_mut(key) {
            restored = scene.profile_restore.is_some();
            scene.restore_profile();
            scene.profile_edit_at = Some(Instant::now());
        }
        if restored {
            self.schedule_sensors();
        }
    }
    /// Isolated inspector scene, with the runtime's current readings/histories.
    pub fn inspect(&mut self, key: &str, id: ElementId, max: u32, motion: bool) -> Result<Frame> {
        self.inspect_trial(key, id, max, motion, None)
            .map(|(frame, _)| frame)
    }

    /// Like inspect, with optional temporary inputs for the inspector's Try action.
    pub fn inspect_trial(
        &mut self,
        key: &str,
        id: ElementId,
        max: u32,
        motion: bool,
        trial_ms: Option<u32>,
    ) -> Result<(Frame, Option<Duration>)> {
        let scene = self
            .scenes
            .get_mut(key)
            .ok_or_else(|| invalid("unknown scene"))?;
        let theme = element_preview::isolate(scene.runtime.theme(), id)
            .ok_or_else(|| invalid("unknown element"))?;
        self.renderer.set_scene(scene.id | (1 << 63));
        self.renderer.set_motion(motion);
        let trial = inspector_trial::Trial::new(
            scene.runtime.theme(),
            id,
            scene.runtime.snapshot(),
            (self.clock)(),
            trial_ms,
            motion,
        );
        if trial.is_some() {
            self.renderer.set_scene(scene.id | (1 << 62));
        }
        let (theme, time, animation, sample) = match &trial {
            Some(trial) => (
                &trial.theme,
                trial.time,
                trial.animation,
                Some((&trial.snapshot, &trial.histories)),
            ),
            None => (&theme, (self.clock)(), self.origin.elapsed(), None),
        };
        let (frame, change) = scene.runtime.preview_detached_sample(
            self.renderer.as_mut(),
            theme,
            time,
            animation,
            sample,
        )?;
        let next = if trial.is_some() {
            Some(Duration::from_millis(inspector_trial::FRAME_MS))
        } else {
            change.map(|at| at.saturating_sub(animation))
        };
        Ok((
            element_preview::fit_within(frame, element_preview::side_of(Some(max))),
            next,
        ))
    }
    /// The whole scene with trial inputs for the canvas
    /// ([`inspector_trial::Trial::scene`]), its longest side at most `max`.
    /// The live scene, its screens and the document are untouched.
    pub fn trial_canvas(
        &mut self,
        key: &str,
        max: u32,
        motion: bool,
        ms: u32,
        hold: Option<inspector_trial::Hold>,
    ) -> Result<Frame> {
        let scene = self
            .scenes
            .get_mut(key)
            .ok_or_else(|| invalid("unknown scene"))?;
        self.renderer.set_scene(scene.id | (1 << 61));
        self.renderer.set_motion(motion);
        let trial = inspector_trial::Trial::scene(
            scene.runtime.theme(),
            scene.runtime.snapshot(),
            scene.runtime.histories(),
            (self.clock)(),
            ms,
            hold,
        );
        let (frame, _) = scene.runtime.preview_detached_sample(
            self.renderer.as_mut(),
            &trial.theme,
            trial.time,
            trial.animation,
            Some((&trial.snapshot, &trial.histories)),
        )?;
        Ok(element_preview::fit_within(frame, max.clamp(16, 2320)))
    }
    /// Measure through the same renderer without advancing the live scene.
    pub fn measure(&mut self, key: &str, id: ElementId) -> Result<Option<BoxF>> {
        let scene = self
            .scenes
            .get_mut(key)
            .ok_or_else(|| invalid("unknown scene"))?;
        self.renderer.set_scene(scene.id | (1 << 63));
        scene.runtime.measure_element(
            self.renderer.as_mut(),
            id,
            (self.clock)(),
            self.origin.elapsed(),
        )
    }
    /// EOF restores only documents; the scene objects, clocks and links survive.
    pub fn detach(&mut self) -> Result<()> {
        for scene in self.scenes.values_mut() {
            scene.restore_profile();
            scene.profile_edit_at = None;
        }
        let unsaved = self
            .scenes
            .values()
            .any(|s| s.runtime.theme() != &s.saved.theme || s.runtime.assets() != &s.saved.assets);
        light(
            if unsaved {
                LightEvent::ClosedUnsaved
            } else {
                LightEvent::ClosedSaved
            },
            0,
            0.0,
            "session",
        );
        self.scenes
            .retain(|key, scene| scene.live || !key.starts_with(EDITOR_SCENE));
        for scene in self.scenes.values_mut() {
            scene.restore_profile();
            scene.show(scene.saved.clone());
            scene.revision = 0;
            if !scene.cards {
                scene.cards = true;
                scene.runtime.automatic_cards(true);
                scene.dirty = true;
            }
        }
        self.schedule_sensors();
        Ok(())
    }
}
impl RuntimeEndpoint for Engine {
    fn images(&self) -> Option<Arc<dyn bezel_core::ports::RuntimeImages>> {
        Some(self.images.clone())
    }
    fn request(&mut self, metadata: &[u8], payload: &[u8]) -> Result<(Vec<u8>, Vec<u8>)> {
        wire::dispatch(self, metadata, payload)
    }
    fn disconnected(&mut self) -> Result<()> {
        self.detach()
    }
}

#[cfg(test)]
mod tests;
