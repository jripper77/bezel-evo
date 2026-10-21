//! Running a theme: sample the sensors, keep graph histories, render a frame
//! and show it.
//!
//! The caller owns the clock and the waiting; the runtime says when the next
//! frame is due ([`ThemeRuntime::next_due`]): the sensors are sampled once
//! per `refresh_seconds`, and a visible animated image (GIF) adds frames of
//! its own at its frame times, at most `MAX_ANIMATION_FPS` a second, each
//! from the last sample (T-7.11). Every frame shows the images as they are
//! when it is drawn, so a screen slower than an animation skips frames
//! instead of falling behind; the screen's adapter sends only what changed.
//!
//! A video background (D-2026-09-30-storage-video-4) reaches a screen in one
//! of three ways, chosen by [`ThemeRuntime::start_video`] once the screen is
//! open: the screen loops the stored file and every frame is an overlay whose
//! alpha lets it show through; the host decodes the video and every frame
//! draws one of its pictures under the elements (screens that cannot play
//! stored videos); or the poster. The runtime queries and plays stored files
//! but never sends one: putting the video on the screen is an explicit user
//! action ([`MissingVideo::upload_request`]).
//!
//! The theme frames its video (D-2026-10-01-video-background-framing-2 to
//! -4) from the video's probed size, which the caller hands over
//! ([`ThemeRuntime::set_video_info`]; without it Auto turns nothing): the
//! screen looks for the copy framed that way ([`device_video_name`]), a copy
//! that is the asset as it is only at the asset's size, and the pictures
//! decoded on the host (or for an editor's preview,
//! [`ThemeRuntime::preview_video`]) are framed here, so a framing edit never
//! restarts the decoder.
//!
//! An editor drives the same runtime: it swaps every edit in place
//! ([`ThemeRuntime::replace_theme`], histories kept), shows the readings of
//! the last sample ([`ThemeRuntime::snapshot`]) and previews frames over the
//! poster ([`ThemeRuntime::render_with`]) while the screen gets what
//! [`ThemeRuntime::render`] draws for it.
//!
//! Every sample first tells the sensors what is shown
//! ([`ThemeRuntime::wanted`]): the theme's visible elements, plus what the
//! caller shows beside it ([`ThemeRuntime::want_also`], an editor's sensor
//! list). A sensor whose measuring reaches outside the machine (`net.ping`)
//! is measured only while wanted (D-2026-09-30-release-polish-11).
//!
//! Elements with a smooth value transition
//! (D-2026-10-15-studio-redesign-fidelity-18) ease to each new sample
//! ([`crate::domain::transition`]): while one moves, the screen and the
//! previews get a frame every `MIN_TRANSITION_STEP` at most, and none once
//! it settles. The screen's pace ([`ThemeRuntime::screen_pace`]) spaces
//! its frames and, when it cannot draw two within a transition, shows the
//! new value at once; previews snap when motion is reduced
//! ([`ThemeRuntime::still_previews`]).

use std::borrow::Cow;
use std::collections::BTreeMap;
use std::fmt;
use std::time::Duration;

use crate::Result;
use crate::app::storage::{Presence, UploadRequest, presence};
use crate::domain::animation::{MIN_FRAME_STEP, Timeline};
use crate::domain::clock::{Language, LocalTime};
use crate::domain::device::DeviceModel;
use crate::domain::frame::Frame;
use crate::domain::framing::{
    FramingGeometry, PanelLayout, ResolvedFraming, VideoFraming, frame_picture,
};
use crate::domain::geometry::Orientation;
use crate::domain::history::{GRAPH_FRAME_STEP, Histories, SeriesId, Tick};
use crate::domain::media::{
    ConvertOptions, MediaInfo, MediaKind, MediaTools, StreamSpec, UploadProfile, device_video_name,
    framed_options,
};
use crate::domain::sensor::{Quantities, SensorInfo, Snapshot, Wanted};
use crate::domain::storage::{FileName, Medium, RemotePath, Repeat, StorageLocation};
use crate::domain::theme::{AssetRef, Background, BoxF, ElementId, Theme, refresh_interval};
use crate::domain::transition::{MIN_TRANSITION_STEP, ShownValues, Transitions, View};
use crate::ports::{
    Backdrop, FrameRenderer, MediaLocation, MediaTranscoder, RenderContext, ScreenLink,
    ScreenStorage, SensorSource, VideoFrames,
};

/// How the theme's video background is shown.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VideoState {
    /// The theme has no video background.
    NoVideo,
    /// The exact device file selected by the theme is missing.
    StoredMissing(RemotePath),
    /// Not started on a screen ([`ThemeRuntime::start_video`]): frames show
    /// the poster. Previews stay here.
    NotStarted,
    /// The screen loops this stored file; frames are overlays on a
    /// transparent base (A = 0 shows the video).
    OnDevice(RemotePath),
    /// The screen can play the video but does not store it (framed as the
    /// theme says, or, for a copy that is the asset as it is, at the
    /// asset's size): frames show the poster until the user sends it ("Send
    /// to screen") and the video is started again.
    VideoMissing(MissingVideo),
    /// The screen cannot play stored videos: the host decodes the video and
    /// frames draw its pictures, framed as the theme says, under the
    /// elements.
    Host,
    /// The screen cannot play stored videos and the host's converter is
    /// missing: frames show the poster.
    NoConverter {
        /// How to install the converter, most likely first.
        install_hints: Vec<String>,
    },
    /// The screen cannot play stored videos and no host decoding was offered
    /// (the video file is not on this computer): frames show the poster.
    NoPlayback,
}

/// A theme video the screen could play but does not store, and what sending
/// it takes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MissingVideo {
    /// The theme asset holding the video.
    pub asset: AssetRef,
    /// Where it belongs: [`device_video_name`] in the card's video folder
    /// when a card is present, else in the internal one (the vendor's
    /// choice). The runtime looks for that name in both. A stored file
    /// there that is not the asset (a copy sent as it is, of another size)
    /// keeps its path: sending replaces it.
    pub path: RemotePath,
    /// The conversion that puts the video on the panel as the theme frames
    /// it ([`framed_options`]: the framing turned to the panel, cropped,
    /// scaled and padded onto its picture; only the turns while the
    /// video's size is unknown). The identity for a video already in the
    /// screen's profile: it is sent as it is.
    pub options: ConvertOptions,
}

impl MissingVideo {
    /// The upload that puts the video where the runtime looks for it, from
    /// `source` (the asset's file on the host). Still an explicit action:
    /// the caller runs it through `app::storage::prepare_upload` and
    /// `app::storage::upload`, then starts the video again.
    pub fn upload_request(&self, source: MediaLocation) -> UploadRequest {
        UploadRequest {
            source,
            name: self.path.name.to_string(),
            location: self.path.location,
            options: self.options,
        }
    }
}

/// Pictures per second of a video background decoded on the host (for
/// screens that cannot play videos themselves); a slow link shows fewer.
pub const HOST_VIDEO_FPS: u32 = 10;

/// Host decoding offered for screens that cannot play stored videos.
pub struct HostVideo<'a> {
    /// Decodes the video.
    pub media: &'a mut dyn MediaTranscoder,
    /// The theme's video file on the host.
    pub source: MediaLocation,
    /// Pictures per second to decode (the caller caps it to the link's rate).
    pub fps: u32,
}

impl fmt::Debug for HostVideo<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("HostVideo")
            .field("source", &self.source)
            .field("fps", &self.fps)
            .finish_non_exhaustive()
    }
}

/// What a frame is drawn from: the theme with its assets and the readings.
struct Scene {
    revision: u64,
    assets_revision: u64,
    theme: Theme,
    assets: BTreeMap<AssetRef, Vec<u8>>,
    histories: Histories,
    quantities: Quantities,
    snapshot: Snapshot,
    language: Language,
}

fn content_revision() -> u64 {
    static REVISION: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
    REVISION.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
}

impl Scene {
    /// Renders the theme from the last sample over `backdrop`, its animated
    /// images as they are `animation` after the theme started and its
    /// moving values as `shown`.
    fn draw(
        &self,
        renderer: &mut dyn FrameRenderer,
        theme: &Theme,
        moment: Moment,
        backdrop: Backdrop<'_>,
        shown: &ShownValues,
    ) -> Result<Frame> {
        renderer.set_content_revision(self.revision);
        let context = self.context(moment, backdrop, shown);
        renderer.render(theme, &self.assets, context)
    }

    /// What a frame drawn now depends on besides its theme.
    fn context<'a>(
        &'a self,
        moment: Moment,
        backdrop: Backdrop<'a>,
        shown: &'a ShownValues,
    ) -> RenderContext<'a> {
        RenderContext {
            snapshot: &self.snapshot,
            histories: &self.histories,
            quantities: &self.quantities,
            time: moment.time,
            animation: moment.animation,
            language: self.language,
            backdrop,
            shown,
        }
    }
}

/// The animated images `theme` shows whose frame times `timelines` does not
/// hold yet.
fn unknown_images(
    theme: &Theme,
    timelines: &BTreeMap<AssetRef, Option<Timeline>>,
) -> Vec<AssetRef> {
    theme
        .shown_images()
        .filter(|asset| !timelines.contains_key(*asset))
        .cloned()
        .collect()
}

/// When the images `theme` shows next change after `at`, by `timelines`.
fn image_change(
    theme: &Theme,
    timelines: &BTreeMap<AssetRef, Option<Timeline>>,
    at: Duration,
) -> Option<Duration> {
    theme
        .shown_pictures()
        .filter_map(|(asset, playback)| {
            let timeline = timelines.get(asset)?.as_ref()?;
            playback.next_change(at, timeline.total(), |local| timeline.next_change(local))
        })
        .min()
}

/// Slowest refresh, seconds, until the caller sets its own
/// ([`ThemeRuntime::limit_refresh`]).
pub const DEFAULT_SLOWEST_REFRESH: f32 = 60.0;

/// When a live loop samples and draws, on the caller's clock.
#[derive(Debug, Clone, Copy, Default)]
struct Cadence {
    /// The first refresh on this clock, shared with the initial graph points.
    origin: Option<Duration>,
    /// The scheduled time of the last refresh, independent of later edits.
    refreshed: Duration,
    /// When the next sample is due (`None`: at once).
    sample: Option<Duration>,
    /// When the screen's last frame was drawn (`None`: none yet).
    drawn: Option<Duration>,
}

/// When a frame is drawn: the wall clock its clocks show and the time its
/// animated images show.
#[derive(Debug, Clone, Copy)]
struct Moment {
    time: LocalTime,
    animation: Duration,
}

/// One unpainted frame shared by preview and live within a frame interval.
struct RecentFrame {
    frame: Frame,
    shown: ShownValues,
    time: LocalTime,
    at: Duration,
    until: Duration,
}

/// A theme being shown.
pub struct ThemeRuntime {
    scene: Scene,
    /// Latest measurement for the sensor panel, including graph-only samples.
    latest: Snapshot,
    /// The frame times of the theme's images, learned from the renderer
    /// (`None`: a still image).
    timelines: BTreeMap<AssetRef, Option<Timeline>>,
    /// The caller's clock at the last frame drawn: animated images show
    /// their frame of it.
    clock: Duration,
    renderer_change: Option<Duration>,
    card_timers: super::card_timer::CardTimers,
    /// The elements easing to their readings.
    transitions: Transitions,
    card_rotation: bool,
    reuse_frames: bool,
    recent_frame: Option<RecentFrame>,
    render_reused: bool,
    cadence: Cadence,
    /// Slowest refresh the caller allows, seconds.
    slowest: f32,
    /// What the caller shows besides the theme ([`ThemeRuntime::want_also`]).
    also: Wanted,
    /// The theme's sensors and `also`, declared at every sample.
    wanted: Wanted,
    video: VideoState,
    /// The theme's video as probed ([`ThemeRuntime::set_video_info`]).
    info: Option<MediaInfo>,
    /// The model of the screen the video was started on (its panel decides
    /// Auto and the stored copy's name).
    screen: Option<&'static DeviceModel>,
    /// Pictures of the host-decoded video ([`VideoState::Host`]).
    host: Option<Box<dyn VideoFrames>>,
    /// The stored video this runtime told the screen to loop.
    playing: Option<RemotePath>,
}

impl fmt::Debug for ThemeRuntime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ThemeRuntime")
            .field("theme", &self.scene.theme.name)
            .field("video", &self.video)
            .finish_non_exhaustive()
    }
}

/// Exact device selection; the outside color does not restart playback.
fn stored_video(theme: &Theme) -> Option<(&RemotePath, &Repeat)> {
    match &theme.background {
        Background::DeviceVideo { path, repeat, .. } => Some((path, repeat)),
        _ => None,
    }
}

/// The theme's video asset and orientation, when its background is a video.
fn video_of(theme: &Theme) -> Option<(&AssetRef, Orientation)> {
    match &theme.background {
        Background::Video { asset, .. } => Some((asset, theme.orientation)),
        _ => None,
    }
}

/// The theme's video asset and its bytes among `assets`.
fn video_bytes<'a>(
    theme: &'a Theme,
    assets: &'a BTreeMap<AssetRef, Vec<u8>>,
) -> Option<(&'a AssetRef, Option<&'a Vec<u8>>)> {
    video_of(theme).map(|(asset, _)| (asset, assets.get(asset)))
}

/// The framing of the theme's video background (the default when the theme
/// sets none, or has no video).
fn framing_of(theme: &Theme) -> VideoFraming {
    match &theme.background {
        Background::Video { framing, .. } => framing.unwrap_or_default(),
        _ => VideoFraming::default(),
    }
}

/// How a theme's video is shown before it is started on a screen.
fn unstarted(theme: &Theme) -> VideoState {
    if matches!(theme.background, Background::DeviceVideo { .. }) {
        return VideoState::NotStarted;
    }
    match video_of(theme) {
        Some(_) => VideoState::NotStarted,
        None => VideoState::NoVideo,
    }
}

/// Which stored file under the theme video's name is that video.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Expect {
    /// Any: a conversion's size is not known before it runs.
    Any,
    /// One of exactly these bytes: the asset sent as it is
    /// (D-2026-10-01-video-background-framing-4).
    Bytes(u64),
}

impl Expect {
    /// Whether a stored file found as `presence` is the video. A file whose
    /// size the screen cannot report is.
    fn met_by(self, presence: Presence) -> bool {
        match (self, presence) {
            (_, Presence::Absent) => false,
            (Expect::Bytes(bytes), Presence::Stored(Some(size))) => size == bytes,
            _ => true,
        }
    }
}

/// The theme's video on a screen that stores and plays videos.
#[derive(Debug, Clone, PartialEq, Eq)]
struct DeviceVideo {
    /// The name its copy has ([`device_video_name`]).
    name: FileName,
    /// The conversion that makes that copy (the identity: the asset as it
    /// is).
    options: ConvertOptions,
    /// Which stored file under that name is the copy.
    expect: Expect,
}

/// Where a screen stores the theme's video.
enum Lookup {
    /// Stored there.
    Stored(RemotePath),
    /// Not stored; it belongs there.
    Absent(RemotePath),
}

/// Looks for `name` in the internal and (with a card) the card video folder
/// with size queries only: listing a folder creates it. The first file that
/// meets `expect` is the video; without one, a file of another size keeps
/// the place (sending replaces it), else the card's folder when there is a
/// card, the internal one otherwise.
fn find_video(storage: &mut dyn ScreenStorage, name: FileName, expect: Expect) -> Result<Lookup> {
    let card = storage.info()?.card.is_some();
    let media: &[Medium] = if card {
        &[Medium::Internal, Medium::Card]
    } else {
        &[Medium::Internal]
    };
    let at = |medium| StorageLocation::new(medium, MediaKind::Video);
    let mut taken = None;
    for medium in media {
        let path = RemotePath::new(at(*medium), name.clone());
        let found = presence(storage, &path)?;
        if expect.met_by(found) {
            return Ok(Lookup::Stored(path));
        }
        if found != Presence::Absent && taken.is_none() {
            taken = Some(path);
        }
    }
    let target = if card { Medium::Card } else { Medium::Internal };
    Ok(Lookup::Absent(
        taken.unwrap_or_else(|| RemotePath::new(at(target), name)),
    ))
}

impl ThemeRuntime {
    /// Starts running `theme` with its asset bytes. A video background shows
    /// its poster until [`Self::start_video`].
    pub fn new(theme: Theme, assets: BTreeMap<AssetRef, Vec<u8>>, language: Language) -> Self {
        let histories = Histories::new(&theme.graph_series());
        let video = unstarted(&theme);
        let wanted = Wanted::Keys(theme.sensor_keys());
        Self {
            also: Wanted::nothing(),
            wanted,
            scene: Scene {
                revision: content_revision(),
                assets_revision: content_revision(),
                theme,
                assets,
                histories,
                quantities: Quantities::default(),
                snapshot: Snapshot::default(),
                language,
            },
            latest: Snapshot::default(),
            video,
            info: None,
            screen: None,
            host: None,
            playing: None,
            timelines: BTreeMap::new(),
            clock: Duration::ZERO,
            renderer_change: None,
            card_timers: super::card_timer::CardTimers::default(),
            transitions: Transitions::default(),
            card_rotation: true,
            reuse_frames: false,
            recent_frame: None,
            render_reused: false,
            cadence: Cadence::default(),
            slowest: DEFAULT_SLOWEST_REFRESH,
        }
    }

    /// Opt in to sharing recent frames between preview and live. The caller
    /// must use the same renderer and invalidate reuse when motion policy changes.
    /// Video backgrounds are excluded because their preview and live differ.
    pub fn reuse_recent_frames(&mut self, enabled: bool) {
        if self.reuse_frames != enabled {
            self.recent_frame = None;
        }
        self.reuse_frames = enabled;
    }

    /// Pause automatic face playback while editing. Saved faces remain unchanged.
    pub fn automatic_cards(&mut self, enabled: bool) {
        if self.card_rotation != enabled {
            self.card_rotation = enabled;
            self.recent_frame = None;
            self.card_timers = super::card_timer::CardTimers::default();
        }
    }

    fn advance_cards(&mut self, now: Duration) {
        if self.card_timers.update(
            &self.scene.theme,
            &self.scene.snapshot,
            now,
            self.card_rotation,
        ) {
            self.recent_frame = None;
        }
    }

    /// Whether the last render reused a recent frame instead of drawing again.
    pub fn render_reused(&self) -> bool {
        self.render_reused
    }

    fn draw_frame(
        &mut self,
        renderer: &mut dyn FrameRenderer,
        time: LocalTime,
        now: Duration,
        backdrop: Backdrop<'_>,
        view: View,
    ) -> Result<Frame> {
        self.render_reused = false;
        let shown = self.shown(now, view);
        let eligible = self.reuse_frames
            && matches!(self.video, VideoState::NoVideo)
            && matches!(backdrop, Backdrop::Poster);
        if eligible
            && let Some(recent) = &self.recent_frame
            && recent.time.whole_second() == time.whole_second()
            && recent.shown == shown
            && now >= recent.at
            && now < recent.until
        {
            self.render_reused = true;
            return Ok(recent.frame.clone());
        }
        self.recent_frame = None;
        let moment = Moment {
            time,
            animation: now,
        };
        let frame = self.scene.draw(
            renderer,
            &self.card_timers.theme(&self.scene.theme),
            moment,
            backdrop,
            &shown,
        )?;
        self.renderer_change = renderer.next_change();
        if eligible {
            // Never cross an image/transition boundary or hold a frame longer
            // than one animation interval. Reuse does not extend its lifetime.
            let until = self
                .change_after(now, view)
                .map_or(now + MIN_FRAME_STEP, |due| due.min(now + MIN_FRAME_STEP));
            self.recent_frame = Some(RecentFrame {
                frame: frame.clone(),
                shown,
                time,
                at: now,
                until,
            });
        }
        Ok(frame)
    }

    /// The values `view` draws at `at` for the elements still easing.
    fn shown(&self, at: Duration, view: View) -> ShownValues {
        let theme = self.card_timers.theme(&self.scene.theme);
        self.transitions.shown(&theme, at, view)
    }

    /// Whether an element still eases to its reading at `at` on the screen.
    pub fn transitioning(&self, at: Duration) -> bool {
        let theme = self.card_timers.theme(&self.scene.theme);
        self.transitions
            .next_change(&theme, at, View::Screen)
            .is_some()
    }

    /// The screen showed a frame that took `cost` from drawing to shown:
    /// value transitions take that pace, and snap when the screen cannot
    /// show two frames within one ([`crate::domain::transition::Pace`]).
    pub fn screen_pace(&mut self, cost: Duration) {
        self.transitions.screen_pace(cost);
    }

    /// Whether previews show new readings at once instead of easing to
    /// them (reduced motion). The screen still eases.
    pub fn still_previews(&mut self, still: bool) {
        self.transitions.still_previews(still);
    }

    /// Revision of the document/assets/readings; adapters can avoid resending
    /// an unchanged editor document when its local model is not sampling.
    pub fn content_revision(&self) -> u64 {
        self.scene.revision
    }

    /// The theme being shown.
    pub fn theme(&self) -> &Theme {
        &self.scene.theme
    }

    /// Revision of asset contents.
    pub fn assets_revision(&self) -> u64 {
        self.scene.assets_revision
    }

    /// The theme's asset bytes.
    pub fn assets(&self) -> &BTreeMap<AssetRef, Vec<u8>> {
        &self.scene.assets
    }

    /// Adds `asset` for the theme to use (new bytes for one it has replace
    /// them); the other assets stay. New bytes for the theme's video drop
    /// what [`Self::set_video_info`] said of it.
    pub fn add_asset(&mut self, asset: AssetRef, bytes: Vec<u8>) {
        self.scene.assets_revision = content_revision();
        self.scene.revision = content_revision();
        self.recent_frame = None;
        self.timelines.remove(&asset);
        if self.is_video(&asset) && self.scene.assets.get(&asset) != Some(&bytes) {
            self.set_video_info(None);
        }
        self.scene.assets.insert(asset, bytes);
    }

    /// Whether `asset` is the theme's video.
    fn is_video(&self, asset: &AssetRef) -> bool {
        video_of(&self.scene.theme).is_some_and(|(video, _)| video == asset)
    }

    /// How the theme's video background is shown.
    pub fn video(&self) -> &VideoState {
        &self.video
    }

    /// The sensors each [`Self::sample`] asks the source to measure: those
    /// the theme's visible elements show and those of [`Self::want_also`].
    pub fn wanted(&self) -> &Wanted {
        &self.wanted
    }

    /// Sensors the caller shows besides the theme (an editor's sensor list),
    /// replacing what it said before: wanted from the next sample on, with
    /// the theme's.
    pub fn want_also(&mut self, also: Wanted) {
        self.also = also;
        self.update_wanted();
    }

    /// The theme's sensors and the caller's.
    fn update_wanted(&mut self) {
        self.wanted = Wanted::Keys(self.scene.theme.sensor_keys()).union(&self.also);
    }

    /// The readings of the last [`Self::sample`] (empty before the first).
    /// Snapshot the Player will display after accepting an incoming sample at `at`.
    /// Independent graph points do not change the Player before the theme refresh.
    pub fn playback_snapshot<'a>(&'a self, incoming: &'a Snapshot, at: Duration) -> &'a Snapshot {
        if self.refresh_due(at) {
            incoming
        } else {
            &self.scene.snapshot
        }
    }

    /// Update independent audio results without sampling or resetting histories.
    pub fn set_spectra(
        &mut self,
        frames: BTreeMap<ElementId, crate::domain::spectrum::SpectrumFrame>,
    ) -> bool {
        if self.scene.snapshot.spectra == frames {
            return false;
        }
        self.scene.snapshot.spectra = frames;
        self.scene.revision = content_revision();
        self.recent_frame = None;
        true
    }

    /// The latest sensor readings used by this Player and its followers.
    pub fn snapshot(&self) -> &Snapshot {
        &self.latest
    }

    /// The graph histories the scene draws from.
    pub fn histories(&self) -> &Histories {
        &self.scene.histories
    }

    /// What each sensor measures (the units of sensor text): from the last
    /// [`Self::use_catalog`], else from the catalog read at the first sample.
    pub fn quantities(&self) -> &Quantities {
        &self.scene.quantities
    }

    /// Takes what each sensor measures from `catalog`, read again when
    /// sensors come and go.
    pub fn use_catalog(&mut self, catalog: &[SensorInfo]) {
        self.scene.revision = content_revision();
        self.recent_frame = None;
        self.scene.quantities = Quantities::from_catalog(catalog);
    }

    /// Swaps in an edited theme, keeping the history of sensors still graphed.
    /// Another video (a different file or orientation) goes back to
    /// [`VideoState::NotStarted`] (or [`VideoState::NoVideo`]): start it
    /// again with [`Self::start_video`]. New bytes for the video drop what
    /// [`Self::set_video_info`] said of it.
    pub fn replace(&mut self, theme: Theme, assets: BTreeMap<AssetRef, Vec<u8>>) {
        self.scene.assets_revision = content_revision();
        self.scene.revision = content_revision();
        self.recent_frame = None;
        let before = self.device_video();
        if video_bytes(&theme, &assets) != video_bytes(&self.scene.theme, &self.scene.assets) {
            self.info = None;
        }
        self.scene.assets = assets;
        self.timelines.clear();
        self.swap_theme(theme);
        self.look_again_unless(before);
    }

    /// [`Self::replace`] keeping the assets: an edit of the theme. A
    /// framing edit keeps the host decoding; a screen that stores videos
    /// looks again when the edit changes the copy it looks for.
    pub fn replace_theme(&mut self, theme: Theme) {
        let before = self.device_video();
        let asset = |theme: &Theme| video_of(theme).map(|(asset, _)| asset.clone());
        if asset(&theme) != asset(&self.scene.theme) {
            self.info = None;
        }
        self.swap_theme(theme);
        self.look_again_unless(before);
    }

    /// Puts `theme` in place, keeping the history of sensors still graphed;
    /// another video goes back to [`VideoState::NotStarted`].
    fn swap_theme(&mut self, theme: Theme) {
        let before = self.refresh();
        let after = refresh_interval(theme.refresh_seconds, self.slowest);
        if before != after
            && let Some(origin) = self.cadence.origin
        {
            // Keep the original phase when changing the period. Keeping the
            // old deadline can leave a 1 s refresh halfway between a graph's
            // 1 s points forever, taking two samples per second.
            let last = self.cadence.refreshed;
            let phase = last.saturating_sub(origin).as_nanos() % after.as_nanos();
            let phase = Duration::new(
                (phase / 1_000_000_000) as u64,
                (phase % 1_000_000_000) as u32,
            );
            self.cadence.sample = Some(last + after - phase);
        }
        self.scene.revision = content_revision();
        self.recent_frame = None;
        let mut histories = Histories::new(&theme.graph_series());
        histories.adopt(&self.scene.histories);
        if video_of(&theme) != video_of(&self.scene.theme)
            || stored_video(&theme) != stored_video(&self.scene.theme)
        {
            self.video = unstarted(&theme);
            self.host = None;
        }
        self.transitions.edited(&theme, &self.scene.snapshot);
        self.scene.theme = theme;
        self.scene.histories = histories;
        self.update_wanted();
    }

    /// The theme's video as probed by the caller (`None`: unknown), for
    /// framing it: Auto turns a video of the panel's native size in a theme
    /// turned a quarter from the panel ([`VideoFraming::resolve`]), the
    /// geometry crops and pads from its size, and a copy sent as it is is
    /// recognised by its bytes. Hand it over before [`Self::start_video`]
    /// (and again after the video changes); without it Auto turns nothing.
    /// A screen that stores videos looks again when it changes the copy
    /// looked for.
    pub fn set_video_info(&mut self, info: Option<MediaInfo>) {
        let before = self.device_video();
        self.info = info;
        self.look_again_unless(before);
    }

    /// The theme's video as probed ([`Self::set_video_info`]).
    pub fn video_info(&self) -> Option<&MediaInfo> {
        self.info.as_ref()
    }

    /// Goes back to [`VideoState::NotStarted`] when a screen that stores
    /// videos was looking for another copy than `before`.
    fn look_again_unless(&mut self, before: Option<DeviceVideo>) {
        let on_device = matches!(
            self.video,
            VideoState::OnDevice(_) | VideoState::VideoMissing(_)
        );
        if on_device && self.device_video() != before {
            self.video = unstarted(&self.scene.theme);
        }
    }

    /// The panel Auto frames the video for: the screen's once the video was
    /// started on one, else the one the canvas is drawn for
    /// ([`PanelLayout::for_theme`]).
    pub fn video_panel(&self) -> Option<PanelLayout> {
        PanelLayout::for_theme(self.screen, self.scene.theme.canvas)
    }

    /// The theme's framing of its video on its canvas, Auto decided from
    /// the probed size ([`Self::set_video_info`]) for [`Self::video_panel`].
    /// `None` without a video background.
    pub fn video_framing(&self) -> Option<ResolvedFraming> {
        video_of(&self.scene.theme)?;
        let size = self.info.as_ref().and_then(|info| info.dimensions);
        let theme = &self.scene.theme;
        Some(framing_of(theme).resolve(size, theme.orientation, self.video_panel()))
    }

    /// How to decode the theme's video on the host at `fps`: the whole
    /// picture at its probed size, at most twice the canvas
    /// ([`StreamSpec::raw`]); the canvas's size while its size is unknown.
    pub fn video_stream(&self, fps: u32) -> StreamSpec {
        let canvas = self.scene.theme.canvas;
        let size = self.info.as_ref().and_then(|info| info.dimensions);
        let source = size.filter(|size| size.area() > 0).unwrap_or(canvas);
        StreamSpec::raw(source, canvas, fps)
    }

    /// A picture of the theme's video decoded on the host
    /// ([`Self::video_stream`]) framed onto the canvas as the theme says
    /// ([`frame_picture`]; the picture itself when it already is that).
    pub fn framed_video<'p>(&self, picture: &'p Frame) -> Cow<'p, Frame> {
        let framing = self.video_framing().unwrap_or(ResolvedFraming::plain(0));
        frame_picture(picture, &framing, self.scene.theme.canvas)
    }

    /// The theme's video on the screen it was started on, when that screen
    /// stores and plays videos: the copy's name and conversion
    /// ([`framed_options`]) and which stored file is it. A video in the
    /// screen's profile whose framing leaves it as it is is sent as it is,
    /// so only a file of its bytes is it; a converted copy's size is
    /// unknown, so any file under its name is.
    fn device_video(&self) -> Option<DeviceVideo> {
        let model = self.screen?;
        let profile = UploadProfile::for_model(model)?;
        if !model.capabilities.video_playback {
            return None;
        }
        let (asset, orientation) = video_of(&self.scene.theme)?;
        let framing = self.video_framing()?;
        let on_panel = framing.turned(orientation.quarter_turns_to(model.native_orientation));
        let options = match &self.info {
            Some(info) => framed_options(model, orientation, info, &framing),
            None => {
                let turned = FramingGeometry::turning(on_panel.turns, profile.video_size);
                ConvertOptions::from_geometry(&turned)
            }
        };
        let as_is = self.info.as_ref().filter(|info| {
            options.is_identity() && profile.mismatches(MediaKind::Video, info).is_empty()
        });
        Some(DeviceVideo {
            name: device_video_name(asset, &on_panel, &profile),
            options,
            expect: as_is.map_or(Expect::Any, |info| Expect::Bytes(info.bytes)),
        })
    }

    /// Forgets the screen the video was started on (closed, handed back or
    /// lost): frames show the poster again ([`VideoState::NotStarted`]) and
    /// the host decoding stops. Nothing is sent: the screen is gone. What
    /// [`Self::set_video_info`] said stays.
    pub fn forget_screen(&mut self) {
        self.video = unstarted(&self.scene.theme);
        self.screen = None;
        self.host = None;
        self.playing = None;
        self.cadence.drawn = None;
        self.transitions.forget_screen();
    }

    /// Chooses how the theme's video reaches `screen` and starts it. Call it
    /// once the screen is open, after [`Self::replace`] changed the video and
    /// after the video was sent to the screen.
    ///
    /// - A screen that plays stored videos and has storage over this link:
    ///   the copy framed as the theme says ([`device_video_name`] of the
    ///   framing turned to the panel, Auto decided for the screen's panel)
    ///   is looked for in its video folders with size queries only (never a
    ///   listing, never an upload) and looped ([`VideoState::OnDevice`]).
    ///   A copy that is the asset as it is counts only at the asset's size
    ///   ([`Self::set_video_info`]). Absent, [`VideoState::VideoMissing`].
    /// - Any other screen: the host decodes the whole video when `host` is
    ///   offered and its converter is ready ([`Self::video_stream`]) and
    ///   every frame frames its picture ([`VideoState::Host`]); otherwise
    ///   [`VideoState::NoConverter`] or [`VideoState::NoPlayback`].
    ///
    /// A stored video this runtime looped and no longer shows is stopped. On
    /// error the poster stays ([`VideoState::NotStarted`]).
    pub fn start_video(
        &mut self,
        screen: &mut dyn ScreenLink,
        host: Option<HostVideo<'_>>,
    ) -> Result<&VideoState> {
        self.host = None;
        self.video = unstarted(&self.scene.theme);
        self.screen = Some(screen.identity().model);
        if let Some((path, repeat)) = stored_video(&self.scene.theme) {
            let path = path.clone();
            let repeat = *repeat;
            let state = if !screen.identity().model.capabilities.video_playback {
                VideoState::NoPlayback
            } else if let Some(storage) = screen.storage() {
                match presence(storage, &path)? {
                    Presence::Absent => VideoState::StoredMissing(path),
                    Presence::Stored(_) => {
                        storage.resume_video(&path, repeat)?;
                        self.playing = Some(path.clone());
                        VideoState::OnDevice(path)
                    }
                }
            } else {
                VideoState::NoPlayback
            };
            if !matches!(state, VideoState::OnDevice(_)) {
                self.stop_played(screen)?;
            }
            self.video = state;
            return Ok(&self.video);
        }
        let video = match video_of(&self.scene.theme) {
            Some((asset, _)) => asset.clone(),
            None => {
                self.stop_played(screen)?;
                return Ok(&self.video);
            }
        };
        let state = match (self.device_video(), screen.storage()) {
            (Some(device), Some(storage)) => self.on_device(storage, video, device)?,
            _ => self.on_host(host)?,
        };
        if !matches!(state, VideoState::OnDevice(_)) {
            self.stop_played(screen)?;
        }
        self.video = state;
        Ok(&self.video)
    }

    /// Loops the stored copy of the video, or says where it belongs.
    fn on_device(
        &mut self,
        storage: &mut dyn ScreenStorage,
        asset: AssetRef,
        device: DeviceVideo,
    ) -> Result<VideoState> {
        match find_video(storage, device.name, device.expect)? {
            Lookup::Stored(path) => {
                storage.resume_video(&path, Repeat::Loop)?;
                self.playing = Some(path.clone());
                Ok(VideoState::OnDevice(path))
            }
            Lookup::Absent(path) => Ok(VideoState::VideoMissing(MissingVideo {
                asset,
                path,
                options: device.options,
            })),
        }
    }

    /// Starts an offline scene's video on the host through the same decoder
    /// path used by a screen without device playback. The caller owns its clock.
    pub fn start_host_video(&mut self, host: HostVideo<'_>) -> Result<()> {
        self.video = self.on_host(Some(host))?;
        Ok(())
    }

    /// Decodes the video on the host when offered and possible.
    fn on_host(&mut self, host: Option<HostVideo<'_>>) -> Result<VideoState> {
        let Some(host) = host else {
            return Ok(VideoState::NoPlayback);
        };
        if let MediaTools::Missing { install_hints } = host.media.tools() {
            return Ok(VideoState::NoConverter { install_hints });
        }
        let spec = self.video_stream(host.fps);
        self.host = Some(host.media.stream(&host.source, spec)?);
        Ok(VideoState::Host)
    }

    /// Stops the stored video this runtime looped, if any.
    fn stop_played(&mut self, screen: &mut dyn ScreenLink) -> Result<()> {
        if self.playing.take().is_some()
            && let Some(storage) = screen.storage()
        {
            storage.stop()?;
        }
        Ok(())
    }

    /// Tells the sensors what is shown ([`Self::wanted`]), samples them and
    /// records a point in every graph history (a sample with no clock: a
    /// one-off frame; a loop samples with [`Self::sample_on_time`]).
    pub fn sample(&mut self, sensors: &mut dyn SensorSource) -> Result<()> {
        self.sample_at(sensors, Tick::Untimed)
    }

    /// [`Self::sample`] at `tick`: the graph histories it is for take a point.
    fn sample_at(&mut self, sensors: &mut dyn SensorSource, tick: Tick) -> Result<()> {
        self.scene.revision = content_revision();
        self.recent_frame = None;
        sensors.want(&self.wanted);
        if self.scene.quantities.is_empty() {
            // Units of sensor text; a catalog failure only costs the units.
            if let Ok(catalog) = sensors.catalog() {
                self.use_catalog(&catalog);
            }
        }
        let snapshot = match sensors.sample() {
            Ok(snapshot) => snapshot,
            Err(e) => {
                // A failed sample waits for the next point of each graph too.
                self.scene.histories.skip(tick);
                return Err(e);
            }
        };
        // A hardware reader may become ready after the first sample. Refresh
        // units for those new keys as well, including raw Libre power sensors.
        if snapshot
            .iter()
            .any(|(key, _)| self.scene.quantities.get(key).is_none())
            && let Ok(catalog) = sensors.catalog()
        {
            self.use_catalog(&catalog);
        }
        self.scene
            .histories
            .push_with_refresh(&snapshot, tick, self.refresh());
        // A graph's independent point must not change the other elements or
        // restart their value transitions before the theme's refresh is due.
        if !matches!(tick, Tick::At { refresh: false, .. }) {
            let now = match tick {
                Tick::At { now, .. } => Some(now),
                Tick::Untimed => None,
            };
            self.transitions.retarget(&self.scene.theme, &snapshot, now);
            self.scene.snapshot = snapshot.clone();
        }
        self.latest = snapshot;
        Ok(())
    }

    /// Renders one frame from the last sample, animated images as they are
    /// at the clock of the last [`Self::render_at`] or [`Self::preview`].
    /// `video` is how long the host-decoded video has played
    /// ([`VideoState::Host`]; ignored otherwise): a loop streaming it
    /// renders at the video's rate and samples at the theme's. Its picture
    /// is framed as the theme says now ([`Self::framed_video`]).
    pub fn render(
        &mut self,
        renderer: &mut dyn FrameRenderer,
        time: LocalTime,
        video: Duration,
    ) -> Result<Frame> {
        self.advance_cards(self.clock);
        self.learn_animations(renderer);
        if matches!(self.video, VideoState::NoVideo) {
            let clock = self.clock;
            return self.draw_frame(renderer, time, clock, Backdrop::Poster, View::Screen);
        }
        self.recent_frame = None;
        self.render_reused = false;
        let framing = self.video_framing().unwrap_or(ResolvedFraming::plain(0));
        let canvas = self.scene.theme.canvas;
        let shown = self.shown(self.clock, View::Screen);
        let picture = match (&self.video, self.host.as_mut()) {
            (VideoState::Host, Some(frames)) => {
                Some(frame_picture(frames.frame_at(video)?, &framing, canvas))
            }
            _ => None,
        };
        let backdrop = match (&self.video, &picture) {
            (VideoState::OnDevice(_), _) => Backdrop::OnDevice,
            (_, Some(picture)) => Backdrop::Frame(picture),
            _ => Backdrop::Poster,
        };
        let moment = Moment {
            time,
            animation: self.clock,
        };
        let frame = self.scene.draw(
            renderer,
            &self.card_timers.theme(&self.scene.theme),
            moment,
            backdrop,
            &shown,
        )?;
        self.renderer_change = renderer.next_change();
        Ok(frame)
    }

    /// Renders one frame from the last sample over `backdrop`, whatever the
    /// screen shows: an editor's preview passes [`Backdrop::Poster`].
    /// Animated images show their frame of the last clock given.
    pub fn render_with(
        &self,
        renderer: &mut dyn FrameRenderer,
        time: LocalTime,
        backdrop: Backdrop<'_>,
    ) -> Result<Frame> {
        let moment = Moment {
            time,
            animation: self.clock,
        };
        self.scene.draw(
            renderer,
            &self.card_timers.theme(&self.scene.theme),
            moment,
            backdrop,
            &self.shown(self.clock, View::Preview),
        )
    }

    // ------------------------------------------------------- live cadence --
    //
    // `now` is the caller's monotonic clock: the time since any origin that
    // stays the same while the runtime lives (the loop's or the session's
    // start). The caller sleeps until [`Self::next_due`].

    /// Samples at least every `slowest_seconds` whatever the theme asks
    /// (the driving adapter's limit; [`DEFAULT_SLOWEST_REFRESH`] until set).
    pub fn limit_refresh(&mut self, slowest_seconds: f32) {
        self.slowest = slowest_seconds;
    }

    /// Time between two samples: the theme's `refresh_seconds`, at least
    /// `MIN_REFRESH_SECONDS`, at most the caller's limit.
    pub fn refresh(&self) -> Duration {
        refresh_interval(self.scene.theme.refresh_seconds, self.slowest)
    }

    /// Whether the theme's refresh is due at `now`.
    fn refresh_due(&self, now: Duration) -> bool {
        self.cadence.sample.is_none_or(|due| now >= due)
    }

    /// Whether a sample is due at `now`: the theme's refresh, or the next
    /// point of a graph with its own interval
    /// (D-2026-10-15-studio-redesign-fidelity-16).
    pub fn sample_due(&self, now: Duration) -> bool {
        self.refresh_due(now)
            || self
                .scene
                .histories
                .next_due()
                .is_some_and(|due| now >= due)
    }

    /// When the next sample is due (at once before the first): the next
    /// refresh, or sooner the next point of a graph with its own interval.
    pub fn next_sample(&self) -> Duration {
        let refresh = self.cadence.sample.unwrap_or_default();
        self.scene
            .histories
            .next_due()
            .map_or(refresh, |graph| graph.min(refresh))
    }

    /// Samples the sensors ([`Self::sample`]) when a sample is due at `now`
    /// ([`Self::sample_due`]): once per refresh and when a graph's interval
    /// asks for a point, never per animation frame. Whether it sampled. The
    /// next refresh is due a refresh after this one was due; a loop that
    /// fell further behind starts the count again from `now`. A failed
    /// sample waits for the next refresh too. A graph without its own
    /// interval takes a point only at a refresh.
    pub fn sample_on_time(
        &mut self,
        sensors: &mut dyn SensorSource,
        now: Duration,
    ) -> Result<bool> {
        if !self.sample_due(now) {
            return Ok(false);
        }
        let refresh = self.refresh_due(now);
        if refresh {
            self.cadence.origin.get_or_insert(now);
            let every = self.refresh();
            let due = self.cadence.sample.unwrap_or(now);
            let next = due + every;
            self.cadence.refreshed = if next > now { due } else { now };
            self.cadence.sample = Some(if next > now {
                next
            } else {
                self.cadence.origin = Some(now);
                now + every
            });
        }
        self.sample_at(sensors, Tick::At { now, refresh })?;
        Ok(true)
    }

    /// Renders the screen's frame at `now` ([`Self::render`]: animated
    /// images as they are at `now`) and counts it as drawn for
    /// [`Self::next_due`].
    pub fn render_at(
        &mut self,
        renderer: &mut dyn FrameRenderer,
        time: LocalTime,
        now: Duration,
        video: Duration,
    ) -> Result<Frame> {
        self.clock = now;
        let frame = self.render(renderer, time, video)?;
        self.cadence.drawn = Some(if self.render_reused {
            self.recent_frame.as_ref().map_or(now, |recent| recent.at)
        } else {
            now
        });
        Ok(frame)
    }

    /// One frame of a live loop at `now`: a sample when one is due
    /// ([`Self::sample_on_time`]), then the frame of `now` from the last
    /// sample ([`Self::render_at`]). Show it, then wait until
    /// [`Self::next_due`].
    pub fn live_frame(
        &mut self,
        sensors: &mut dyn SensorSource,
        renderer: &mut dyn FrameRenderer,
        time: LocalTime,
        now: Duration,
    ) -> Result<Frame> {
        self.sample_on_time(sensors, now)?;
        self.render_at(renderer, time, now, Duration::ZERO)
    }

    /// When the screen's next frame is due: the next sample, or the next
    /// frame of a visible animated image, no sooner than `MIN_FRAME_STEP`
    /// after the last frame drawn. At once before the first frame. A time
    /// already past means at once: the frame drawn then shows the images as
    /// they are then, so a send slower than the animation skips frames
    /// instead of queuing them.
    pub fn next_due(&self) -> Duration {
        let sample = self.next_sample();
        let Some(drawn) = self.cadence.drawn else {
            return Duration::ZERO;
        };
        let due = match self.change_after(drawn, View::Screen) {
            Some(change) => sample.min(change.max(drawn + MIN_FRAME_STEP)),
            None => sample,
        };
        if self.graph_animating(drawn) {
            due.min(drawn + GRAPH_FRAME_STEP)
        } else {
            due
        }
    }

    /// Whether a visible graph has recorded motion left at this clock.
    pub fn graph_animating(&self, at: Duration) -> bool {
        let theme = self.card_timers.theme(&self.scene.theme);
        theme.elements.iter().any(|element| {
            theme.is_visible(element)
                && match &element.kind {
                    crate::domain::theme::ElementKind::Graph {
                        binding,
                        interval_seconds,
                        ..
                    } => {
                        self.scene
                            .histories
                            .scroll_offset(&SeriesId::graph(&binding.key, *interval_seconds), at)
                            > 0.
                    }
                    _ => false,
                }
        })
    }

    /// When the visible animated images or an easing value next change
    /// after `at` in a preview; `None` while nothing moves. Known for the
    /// images the last render met.
    pub fn next_animation_change(&self, at: Duration) -> Option<Duration> {
        self.change_after(at, View::Preview)
    }

    /// When what `view` shows next changes by itself after `at`: an
    /// animated image, the renderer's card transition, a card timer, or the
    /// next step of a value transition (at `view`'s pace); for a preview
    /// that eases a value, also the next sample, where a transition starts.
    fn change_after(&self, at: Duration, view: View) -> Option<Duration> {
        let theme = self.card_timers.theme(&self.scene.theme);
        let sample = match view {
            View::Preview => self.sample_wake(&theme, at),
            View::Screen => None,
        };
        image_change(&theme, &self.timelines, at)
            .into_iter()
            .chain(theme.elements.iter().any(|e| theme.is_visible(e) && matches!(&e.kind,crate::domain::theme::ElementKind::Text{content:crate::domain::theme::TextContent::Spectrum(s),..} if s.demo)).then_some(at+Duration::from_millis(50)))
            .chain(self.renderer_change)
            .chain(self.card_timers.next)
            .chain(self.transitions.next_change(&theme, at, view))
            .chain(self.graph_animating(at).then_some(at + GRAPH_FRAME_STEP))
            .chain(sample)
            .min()
    }

    /// When a preview of `theme` drawn at `at` that eases a value wants its
    /// next frame for the next sample: when it is due, or a transition step
    /// later while the loop has not taken one that is overdue. `None`
    /// without an eased value, or before a loop samples on time (nothing
    /// would ever start a transition).
    fn sample_wake(&self, theme: &Theme, at: Duration) -> Option<Duration> {
        self.cadence.sample?;
        if !self.transitions.eases_in_previews(theme) {
            return None;
        }
        let due = self.next_sample();
        Some(if due > at {
            due
        } else {
            at + MIN_TRANSITION_STEP
        })
    }

    /// An editor's preview at `now`: the frame over the poster with the
    /// animated images as they are at `now`, and when they next change
    /// (`None`: nothing animates). Not a frame of the screen.
    pub fn preview(
        &mut self,
        renderer: &mut dyn FrameRenderer,
        time: LocalTime,
        now: Duration,
    ) -> Result<(Frame, Option<Duration>)> {
        self.preview_over(renderer, time, now, Backdrop::Poster)
    }

    /// [`Self::preview`] over `picture`, a picture of the theme's video an
    /// editor decoded ([`Self::video_stream`]), framed as the theme says
    /// now ([`Self::framed_video`]): a framing edit shows at the next
    /// picture, the decoder untouched. When the next picture is due is the
    /// caller's. Not a frame of the screen.
    pub fn preview_video(
        &mut self,
        renderer: &mut dyn FrameRenderer,
        time: LocalTime,
        now: Duration,
        picture: &Frame,
    ) -> Result<(Frame, Option<Duration>)> {
        let framed = self.framed_video(picture);
        self.preview_over(renderer, time, now, Backdrop::Frame(&framed))
    }

    /// An editor's preview at `now` over `backdrop`.
    fn preview_over(
        &mut self,
        renderer: &mut dyn FrameRenderer,
        time: LocalTime,
        now: Duration,
        backdrop: Backdrop<'_>,
    ) -> Result<(Frame, Option<Duration>)> {
        self.advance_cards(now);
        self.learn_animations(renderer);
        self.clock = now;
        let frame = self.draw_frame(renderer, time, now, backdrop, View::Preview)?;
        Ok((frame, self.next_animation_change(now)))
    }

    /// An editor's preview of `theme`, another theme than the one running
    /// (one element of it set apart, say), drawn at `now` from this
    /// runtime's last sample, graph histories, units, assets and language:
    /// the frame and when its animated images or the renderer's transitions
    /// next change (`None`: nothing moves). The runtime's own clock, cards
    /// and recent frame stay as they are; the renderer gets no content
    /// revision, so it reuses nothing drawn for the running theme.
    pub fn preview_detached(
        &mut self,
        renderer: &mut dyn FrameRenderer,
        theme: &Theme,
        time: LocalTime,
        now: Duration,
    ) -> Result<(Frame, Option<Duration>)> {
        self.preview_detached_sample(renderer, theme, time, now, None)
    }

    /// Detached inspector trial with temporary readings and histories. Neither
    /// the running scene nor its transitions, sensors or histories are edited.
    pub fn preview_detached_sample(
        &mut self,
        renderer: &mut dyn FrameRenderer,
        theme: &Theme,
        time: LocalTime,
        now: Duration,
        sample: Option<(&Snapshot, &Histories)>,
    ) -> Result<(Frame, Option<Duration>)> {
        for asset in unknown_images(theme, &self.timelines) {
            let timeline = renderer.animation(&asset, &self.scene.assets);
            self.timelines.insert(asset, timeline);
        }
        renderer.set_content_revision(0);
        let shown = if sample.is_some() {
            Default::default()
        } else {
            self.transitions.shown(theme, now, View::Preview)
        };
        let moment = Moment {
            time,
            animation: now,
        };
        let mut context = self.scene.context(moment, Backdrop::Poster, &shown);
        if let Some((snapshot, histories)) = sample {
            context.snapshot = snapshot;
            context.histories = histories;
        }
        let frame = renderer.render(theme, &self.scene.assets, context)?;
        let change = image_change(theme, &self.timelines, now)
            .into_iter()
            .chain(renderer.next_change())
            .chain(self.transitions.next_change(theme, now, View::Preview))
            .chain(self.sample_wake(theme, now))
            .min();
        Ok((frame, change))
    }

    /// Measures content with the same sample, units and shown values as previews.
    pub fn measure_element(
        &mut self,
        renderer: &mut dyn FrameRenderer,
        id: ElementId,
        time: LocalTime,
        now: Duration,
    ) -> Result<Option<BoxF>> {
        let theme = &self.scene.theme;
        let shown = self.transitions.shown(theme, now, View::Preview);
        let context = self.scene.context(
            Moment {
                time,
                animation: now,
            },
            Backdrop::Poster,
            &shown,
        );
        renderer.set_content_revision(0);
        renderer.measure_element(theme, id, &self.scene.assets, context)
    }

    /// Asks the renderer for the frame times of shown images it has not
    /// told yet.
    fn learn_animations(&mut self, renderer: &mut dyn FrameRenderer) {
        let unknown = unknown_images(&self.card_timers.theme(&self.scene.theme), &self.timelines);
        for asset in unknown {
            let timeline = renderer.animation(&asset, &self.scene.assets);
            self.timelines.insert(asset, timeline);
        }
    }

    /// Samples the sensors, records histories and renders one frame
    /// ([`Self::sample`] then [`Self::render`]; a host-decoded video shows
    /// its first picture).
    pub fn frame(
        &mut self,
        sensors: &mut dyn SensorSource,
        renderer: &mut dyn FrameRenderer,
        time: LocalTime,
    ) -> Result<Frame> {
        self.sample(sensors)?;
        self.render(renderer, time, Duration::ZERO)
    }

    /// One refresh: [`Self::frame`] then present it on `screen`.
    pub fn show(
        &mut self,
        sensors: &mut dyn SensorSource,
        renderer: &mut dyn FrameRenderer,
        screen: &mut dyn ScreenLink,
        time: LocalTime,
    ) -> Result<()> {
        let frame = self.frame(sensors, renderer, time)?;
        screen.present(&frame)
    }
}

#[cfg(test)]
mod tests {
    //! The video decisions run through the adapters' fakes in
    //! `tests/runtime_video.rs`; here only what needs no port.

    use super::*;
    use crate::domain::catalog::model_by_id;
    use crate::domain::device::ModelId;

    #[test]
    fn device_video_names_follow_the_vendor() {
        let profile = |id| {
            let model = model_by_id(ModelId(id)).expect("model");
            UploadProfile::for_model(model).expect("storage")
        };
        let (rev_c, usb) = (profile("turing-8.8"), profile("turing-usb-8.8"));
        let name = |asset: &str, turns, profile: &UploadProfile| {
            let plain = ResolvedFraming::plain(turns);
            device_video_name(&AssetRef(asset.into()), &plain, profile).to_string()
        };
        assert_eq!(name("assets/AMD.mp4", 0, &rev_c), "amd.mp4");
        assert_eq!(name("assets/AMD.mp4", 1, &rev_c), "amd_90.mp4");
        assert_eq!(name("assets/my.clip.MOV", 2, &rev_c), "my_clip_180.mp4");
        assert_eq!(name("assets/bg.mp4", 3, &usb), "bg_270.h264");
        assert_eq!(
            name("C:\\clips\\Fundo Azul.mp4", 4, &rev_c),
            "fundo_azul.mp4"
        );
    }

    #[test]
    fn a_missing_video_names_its_upload() {
        let missing = MissingVideo {
            asset: AssetRef("assets/Clip.MP4".into()),
            path: RemotePath::parse("sd/video/clip_90.mp4").expect("path"),
            options: ConvertOptions {
                quarter_turns: 1,
                ..ConvertOptions::default()
            },
        };
        let source = MediaLocation("/themes/clip/assets/Clip.MP4".into());
        let request = missing.upload_request(source.clone());
        assert_eq!(request.source, source);
        assert_eq!(request.name, "clip_90.mp4");
        assert_eq!(request.location, missing.path.location);
        assert_eq!(request.options, missing.options);
    }

    #[test]
    fn a_new_runtime_shows_the_poster_until_its_video_starts() {
        let plain = Theme::blank(
            "plain",
            crate::domain::geometry::Size::new(480, 1920),
            Orientation::Portrait,
        );
        let mut video = plain.clone();
        video.background = Background::Video {
            asset: AssetRef("assets/clip.mp4".into()),
            poster: None,
            framing: None,
        };
        let mut runtime = ThemeRuntime::new(plain.clone(), BTreeMap::new(), Language::English);
        assert_eq!(runtime.video(), &VideoState::NoVideo);
        runtime.replace(video.clone(), BTreeMap::new());
        assert_eq!(runtime.video(), &VideoState::NotStarted);
        assert_eq!(runtime.theme().name, "plain");
        assert!(format!("{runtime:?}").contains("NotStarted"));
        runtime.replace(plain, BTreeMap::new());
        assert_eq!(runtime.video(), &VideoState::NoVideo);
    }

    #[test]
    fn the_probed_video_is_forgotten_with_its_bytes() {
        use crate::domain::geometry::Size;
        let theme = |asset: &str| {
            let mut theme = Theme::blank("clip", Size::new(480, 1920), Orientation::Landscape);
            theme.background = Background::Video {
                asset: AssetRef(asset.into()),
                poster: None,
                framing: None,
            };
            theme
        };
        let clip = AssetRef("assets/clip.mp4".into());
        let assets = BTreeMap::from([(clip.clone(), vec![1, 2, 3])]);
        let info = MediaInfo {
            format: crate::domain::media::MediaFormat::Mp4,
            bytes: 3,
            dimensions: Some(Size::new(480, 1920)),
            video: None,
            has_audio: false,
        };
        let mut runtime =
            ThemeRuntime::new(theme("assets/clip.mp4"), assets.clone(), Language::English);
        let probed = |runtime: &mut ThemeRuntime| {
            runtime.set_video_info(Some(info.clone()));
            assert_eq!(runtime.video_framing().map(|f| f.turns), Some(3));
        };
        probed(&mut runtime);
        // The same bytes keep it; other bytes, another file or theme drop it.
        runtime.add_asset(clip.clone(), vec![1, 2, 3]);
        runtime.replace(theme("assets/clip.mp4"), assets.clone());
        runtime.add_asset(AssetRef("assets/other.png".into()), vec![9]);
        assert_eq!(runtime.video_info(), Some(&info));
        runtime.add_asset(clip.clone(), vec![4]);
        assert_eq!(runtime.video_info(), None);
        assert_eq!(runtime.video_framing().map(|f| f.turns), Some(0));
        probed(&mut runtime);
        runtime.replace(theme("assets/clip.mp4"), assets.clone());
        assert_eq!(runtime.video_info(), None, "new bytes");
        probed(&mut runtime);
        runtime.replace_theme(theme("assets/other.mp4"));
        assert_eq!(runtime.video_info(), None, "another video");
        assert_eq!(runtime.video_stream(10).size, Size::new(1920, 480));
        assert_eq!(
            runtime.video_panel().map(|p| p.native),
            Some(Size::new(480, 1920))
        );
    }
}
