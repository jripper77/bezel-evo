//! What each UI command does, without Tauri: the commands module only adds
//! threads, dialogs and IPC around these methods, so they run on fakes in
//! tests. Errors reach the UI as codes with arguments ([`UiError`]).

use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Condvar, Mutex, MutexGuard, PoisonError};
use std::time::{Duration, Instant};

use bezel_core::app::handoff::{HandedOver, hand_over};
use bezel_core::app::{
    choose_screen, connect_screen, discover_devices, discover_screens, leave_desktop_mode,
    reopen_screen, restart_screen,
};
use bezel_core::domain::catalog::model_by_id;
use bezel_core::domain::clock::{Language, LocalTime};
use bezel_core::domain::device::DeviceModel;
use bezel_core::domain::discovery::Screen;
use bezel_core::domain::geometry::Orientation;
use bezel_core::domain::screen::{Brightness, Confirm};
use bezel_core::domain::sensor::{SensorKey, Wanted};
use bezel_core::domain::theme::{Background, ElementId, MIN_REFRESH_SECONDS, Theme};
use bezel_core::ports::{
    DesktopModeHid, DeviceBus, HandoffStore, ScreenConnector, ScreenLink, SensorSource,
    ThemeLocation, ThemeStore,
};
use bezel_render::{FontCatalog, FontFace};
use bezel_sensors::SensorOptions;
use bezel_themes::dto::{BackgroundDto, ThemeDto};
use bezel_themes::import::{ImportOptions, import_path_with};
use bezel_themes::native::{is_native, native_location};

use crate::diag::{self, DiagCode};
use crate::dto::{
    ActivityDto, AddedDto, AssetDto, DevicesDto, FontCatalogDto, ImportedDto, LiveVideoDto,
    MonitorModeDto, PreferencesDto, ReconnectingDto, RestartedDto, SampleDto, SavedDto, SensorDto,
    SessionDto, ThemeEntryDto, ThemeFilterDto, VideoAutoDto,
};
use crate::element_preview::{fit_within, side_of};
use crate::library::ThemeLibrary;
use crate::media::{extension_of, is_animated_gif, kind_of, thumbnail_data_url};
pub use crate::messages::UiResult;
use crate::messages::{ErrorCode, UiError};
use crate::settings::{SettingsFile, THEME_AXES, THEME_SCOPES};
use crate::storage::StorageState;
pub use crate::studio::MAX_REFRESH;
use crate::studio::{Delivery, Motion, Studio, TakenPoster, VideoProbe, Wait};
use crate::texts::{Texts, language_slug, parse_language, texts};
use crate::thumbnails::Thumbnails;

/// Largest file accepted as an image or theme, bytes.
pub const MAX_FILE_BYTES: u64 = 64 * 1024 * 1024;

/// Builds the machine's sensors with their options (they take effect when
/// the source is built).
pub type SensorFactory = Arc<dyn Fn(SensorOptions) -> Box<dyn SensorSource> + Send + Sync>;

/// The ports and state behind the window.
pub struct Backend {
    /// Authoritative Light runtime, with the same engine on recovery.
    pub engine: Option<Arc<crate::engine::EngineBridge>>,
    /// Where screens are discovered.
    pub bus: Arc<dyn DeviceBus + Send + Sync>,
    /// How screens are opened.
    pub connector: Arc<dyn ScreenConnector + Send + Sync>,
    /// The HID interface of panels in desktop mode.
    pub hid: Arc<dyn DesktopModeHid + Send + Sync>,
    /// Theme files.
    pub store: Arc<dyn ThemeStore + Send + Sync>,
    /// The theme folders.
    pub library: ThemeLibrary,
    /// Remembered choices.
    pub settings: SettingsFile,
    /// The system's language, which the app follows unless the user chose
    /// another in the settings.
    pub system_language: Language,
    /// Builds the sensors again when their options change.
    pub make_sensors: SensorFactory,
    /// The font families themes can use, from the renderer's font database.
    pub fonts: FontCatalog,
    /// The editing session.
    pub studio: Session,
    /// The screen's files: the media converter and the running operation.
    pub storage: StorageState,
    /// The library's thumbnails, drawn off the session's lock.
    pub thumbnails: Thumbnails,
}

/// The editing session behind its lock, and the signal that the live
/// screen's link came back from showing a frame: the screen's I/O happens
/// outside the lock, so previews render while the screen works.
pub struct Session {
    studio: Arc<Mutex<Studio>>,
    link_back: Arc<Condvar>,
    delivery_epoch: Arc<AtomicU64>,
    completed: crate::deliveries::Completion,
    workers: Mutex<crate::deliveries::Workers>,
}

/// Longest wait for the live link to come back from showing a frame; after
/// it the link counts as in use.
const LINK_BACK_WAIT: Duration = Duration::from_secs(10);

impl Session {
    /// The session of `studio`.
    pub fn new(studio: Studio) -> Self {
        let studio = Arc::new(Mutex::new(studio));
        let link_back = Arc::new(Condvar::new());
        let delivery_epoch = Arc::new(AtomicU64::new(0));
        let completed: crate::deliveries::Completion = {
            let studio = Arc::clone(&studio);
            let link_back = Arc::clone(&link_back);
            let epoch = Arc::clone(&delivery_epoch);
            Arc::new(move |delivery, outcome| {
                let unwanted = studio
                    .lock()
                    .unwrap_or_else(PoisonError::into_inner)
                    .presented(delivery, outcome, Instant::now());
                drop(unwanted);
                // Pair notification with the waiter's lock: no lost wake-up
                // between its predicate and the condvar wait.
                let signal = studio.lock().unwrap_or_else(PoisonError::into_inner);
                epoch.fetch_add(1, Ordering::Release);
                link_back.notify_all();
                drop(signal);
            })
        };
        Self {
            studio,
            link_back,
            delivery_epoch,
            workers: Mutex::new(crate::deliveries::Workers::new(Arc::clone(&completed))),
            completed,
        }
    }

    /// The session, even after a panic in another command.
    pub fn lock(&self) -> MutexGuard<'_, Studio> {
        self.studio.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// The session once the live link is back from showing a frame.
    fn idle(&self) -> MutexGuard<'_, Studio> {
        let waited = self
            .link_back
            .wait_timeout_while(self.lock(), LINK_BACK_WAIT, |s| s.presenting());
        waited.unwrap_or_else(PoisonError::into_inner).0
    }
}

/// Longest sleep of the refresh loop between two looks at the session: an
/// edit or a screen going live is picked up this soon.
pub const LOOK_AGAIN: Duration = Duration::from_millis(250);

/// How long the refresh loop sleeps at `now` for the refresh due at `due`:
/// until then (nothing when it is past), at most [`LOOK_AGAIN`].
pub fn sleep_until(due: Instant, now: Instant) -> Duration {
    due.saturating_duration_since(now).min(LOOK_AGAIN)
}

/// Says that a preview does not change by itself (no animated GIF shows,
/// no video plays).
pub const STILL: u32 = u32::MAX;

/// A preview frame for the UI: a 12-byte header (width, height, and the
/// milliseconds until its next picture is due: its animated GIFs change or
/// its video background shows its next picture, [`STILL`] when nothing
/// moves; u32 little-endian), then the RGBA pixels.
pub fn frame_bytes(frame: &bezel_core::domain::frame::Frame, change: Option<Duration>) -> Vec<u8> {
    let size = frame.size();
    let next = change.map_or(STILL, |d| u32::try_from(d.as_millis()).unwrap_or(STILL - 1));
    let mut out = Vec::with_capacity(12 + frame.as_rgba().len());
    out.extend_from_slice(&size.width.to_le_bytes());
    out.extend_from_slice(&size.height.to_le_bytes());
    out.extend_from_slice(&next.to_le_bytes());
    out.extend_from_slice(frame.as_rgba());
    out
}

fn theme_of(dto: &ThemeDto) -> UiResult<Theme> {
    check_rotation(&dto.background)?;
    Theme::try_from(dto).map_err(|e| UiError::new(ErrorCode::InvalidTheme).arg("detail", e.0))
}

/// A video background's framing turns its video by 0, 90, 180 or 270
/// degrees, or leaves it to Auto (no `rotation`): another rotation is
/// `invalidInput`, not `invalidTheme`
/// (D-2026-10-01-video-background-framing-2); the rotations are the theme
/// format's ([`bezel_themes::dto::FramingDto::quarter_turns`]). Its other
/// numbers are clamped when the theme is read.
fn check_rotation(background: &BackgroundDto) -> UiResult<()> {
    match background {
        BackgroundDto::Video {
            framing: Some(framing),
            ..
        } => framing
            .quarter_turns()
            .map(drop)
            .map_err(|e| UiError::new(ErrorCode::InvalidInput).arg("detail", e.0)),
        _ => Ok(()),
    }
}

/// Pages of the user guide the UI opens in the system's browser.
pub const GUIDE_PAGES: &[&str] = &["ffmpeg", "gifs-and-stickers"];

/// The other pages the UI opens in the system's browser, by the name it
/// asks for: KLIPY's Partner Panel, where a key is made
/// (D-2026-10-01-gif-sticker-search-3).
pub const LINKS: &[(&str, &str)] = &[("klipyPartnerPanel", "https://partner.klipy.com")];

/// The fixed address of the link named `link` ([`LINKS`]), so the UI can
/// open no other.
pub fn link_url(link: &str) -> UiResult<&'static str> {
    LINKS
        .iter()
        .find(|(name, _)| *name == link)
        .map(|(_, url)| *url)
        .ok_or_else(|| {
            UiError::new(ErrorCode::InvalidInput).arg("detail", format!("link \"{link}\""))
        })
}

/// Where the guide's `page` is in `language` (`en` or `pt-BR`): a fixed
/// address on the project's site, so the UI can open no other.
pub fn guide_url(page: &str, language: &str) -> UiResult<String> {
    let folder = match language {
        "en" => Some(""),
        "pt-BR" => Some("pt-BR/"),
        _ => None,
    };
    match folder {
        Some(folder) if GUIDE_PAGES.contains(&page) => Ok(format!(
            "https://github.com/slipalison/bezel/blob/main/docs/user/{folder}{page}.md"
        )),
        _ => Err(UiError::new(ErrorCode::InvalidInput)
            .arg("detail", format!("guide page \"{page}\" in \"{language}\""))),
    }
}

/// Orientation of a new theme for `model` when none was used with its screen
/// yet: horizontal for bar-shaped panels (the long side at least twice the
/// short one, like the 8.8"), else the model's native orientation.
pub fn default_orientation(model: &DeviceModel) -> Orientation {
    let panel = model.panel.portrait();
    if u64::from(panel.height) >= 2 * u64::from(panel.width) {
        Orientation::Landscape
    } else {
        model.native_orientation
    }
}

/// Whether `text` can name a host to ping: a name or an IPv4 or IPv6
/// address (letters, digits, `.`, `-` and `:`), not an option.
fn is_host(text: &str) -> bool {
    !text.is_empty()
        && text.len() <= 253
        && !text.starts_with(['-', '.'])
        && text
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | ':'))
}

fn read_file(path: &Path) -> UiResult<Vec<u8>> {
    read_limited(path, MAX_FILE_BYTES)
}

/// The content of the file at `path`, refused over `limit` bytes.
pub(crate) fn read_limited(path: &Path, limit: u64) -> UiResult<Vec<u8>> {
    let unreadable = |e| UiError::file(path.display(), e);
    let size = std::fs::metadata(path).map_err(unreadable)?.len();
    if size > limit {
        return Err(UiError::new(ErrorCode::FileTooLarge)
            .arg("file", path.display())
            .arg("size", size / (1024 * 1024))
            .arg("limit", limit / (1024 * 1024)));
    }
    std::fs::read(path).map_err(unreadable)
}

impl Backend {
    pub(crate) fn sync_engine_options(&self) -> bezel_core::Result<()> {
        if let Some(engine) = &self.engine {
            let settings = self.settings.load();
            let options = settings.sensor_options();
            engine.options(bezel_runtime::wire::Options {
                debug_overlay: (settings.debug && settings.debug_show_readings)
                    .then_some(settings.debug_corner),
                audio_capture: settings.audio_capture.0,
                ping_host: options.ping_host,
                presentmon_path: settings.presentmon_path,

                ffmpeg: settings.ffmpeg_path,
            })?;
        }
        Ok(())
    }
    fn sync_engine(&self, saved: bool) -> bezel_core::Result<String> {
        let studio = self.studio();
        let key = studio.selected_screen().unwrap_or("__editor__").to_owned();
        if let Some(engine) = &self.engine {
            let revision = studio.document_revision();
            if !engine.needs_sync(&key, revision, saved) {
                return Ok(key);
            }
            let document = bezel_runtime::Document {
                theme: studio.theme().clone(),
                assets: studio.assets().clone(),
            };
            drop(studio);
            engine.sync(&key, revision, document, saved)?;
        }
        Ok(key)
    }

    /// The font picker's families and the fallback for a missing one.
    pub fn font_catalog(&self) -> FontCatalogDto {
        let mut dto = FontCatalogDto::from(&self.current_fonts());
        let studio = self.studio();
        for (asset, bytes) in studio.assets() {
            if kind_of(asset) != "font" || !bezel_render::valid_font(bytes) {
                continue;
            }
            let renderer = bezel_render::SkiaRenderer::with_fonts(
                vec![bytes.clone()],
                bezel_render::SystemFonts::Skip,
            );
            let mut fonts = FontCatalogDto::from(&renderer.font_catalog());
            for family in &mut fonts.families {
                family.asset = Some(asset.0.clone());
                family.bundled = false;
                family.origin = "theme";
            }
            dto.families.extend(fonts.families);
        }
        dto
    }

    /// A catalogued font file for the picker preview; empty for unknown identities.
    pub fn font_face(&self, family: &str, weight: u16, italic: bool) -> Vec<u8> {
        let face = FontFace { weight, italic };
        self.fonts
            .bundled_face(family, face)
            .map(<[u8]>::to_vec)
            .or_else(|| self.current_fonts().portable_face(family, face))
            .unwrap_or_default()
    }

    fn current_fonts(&self) -> FontCatalog {
        self.fonts.with_user_directory()
    }

    /// Only font assets of the current document; never an arbitrary filesystem path.
    pub fn theme_font_face(&self, asset: &str) -> Vec<u8> {
        let studio = self.studio();
        let key = bezel_core::domain::theme::AssetRef(asset.into());
        studio
            .assets()
            .get(&key)
            .filter(|b| kind_of(&key) == "font" && bezel_render::valid_font(b))
            .cloned()
            .unwrap_or_default()
    }

    /// Include a catalogued user/system file. Bundled fonts are refused server-side.
    pub fn include_font(&self, family: &str, weight: u16, italic: bool) -> UiResult<AddedDto> {
        let bytes = self
            .current_fonts()
            .portable_face(family, FontFace { weight, italic })
            .ok_or_else(|| UiError::new(ErrorCode::InvalidInput))?;
        let ext = if bytes.starts_with(b"ttcf") {
            "ttc"
        } else if bytes.starts_with(b"OTTO") {
            "otf"
        } else {
            "ttf"
        };
        let reference = self
            .studio()
            .add_asset(
                &format!("{family}-{weight}{}.{ext}", if italic { "i" } else { "" }),
                bytes,
            )
            .0;
        Ok(AddedDto { reference })
    }

    /// The editing session, even after a panic in another command.
    pub fn studio(&self) -> MutexGuard<'_, Studio> {
        self.studio.lock()
    }

    /// The editing session once the live link is back from showing a frame
    /// (for whatever needs the link).
    pub(crate) fn idle_studio(&self) -> MutexGuard<'_, Studio> {
        self.studio.idle()
    }

    /// Shows the frame the session prepared, outside its lock, then gives
    /// the link back to the session.
    fn deliver(&self, delivered: bezel_core::Result<Option<Delivery>>) -> bezel_core::Result<()> {
        let Some(mut delivery) = delivered? else {
            return Ok(());
        };
        let outcome = delivery.present();
        (self.studio.completed)(delivery, &outcome);
        outcome
    }

    fn dispatch(&self, delivered: bezel_core::Result<Option<Delivery>>) {
        match delivered {
            Ok(Some(delivery)) => {
                let submitted = self
                    .studio
                    .workers
                    .lock()
                    .unwrap_or_else(PoisonError::into_inner)
                    .submit(delivery);
                if let Err((delivery, error)) = submitted {
                    (self.studio.completed)(*delivery, &Err(error));
                    diag::report(DiagCode::LiveFrameFailed);
                }
            }
            Ok(None) => {}
            Err(_) => diag::report(DiagCode::LiveFrameFailed),
        }
    }

    #[cfg(test)]
    pub(crate) fn tick_and_wait(&self, time: LocalTime, now: Instant) -> Instant {
        self.tick(time, now);
        let studio = self.idle_studio();
        assert!(
            !studio.presenting(),
            "delivery worker did not return its link"
        );
        studio.next_due_all()
    }

    /// Capture before ticking, so a completion during rendering is not missed.
    pub(crate) fn delivery_epoch(&self) -> u64 {
        self.studio.delivery_epoch.load(Ordering::Acquire)
    }

    /// Wait until rendering is due or any display has completed its frame.
    pub(crate) fn wait_refresh(&self, due: Instant, epoch: u64) {
        let timeout = sleep_until(due, Instant::now());
        let waited = self
            .studio
            .link_back
            .wait_timeout_while(self.studio.lock(), timeout, |_| {
                self.delivery_epoch() == epoch
            });
        drop(waited.unwrap_or_else(PoisonError::into_inner));
    }

    /// Shows the edited theme on the live screen now (its video probed
    /// first when the screen is to start it).
    pub(crate) fn show_now(&self, time: LocalTime) -> bezel_core::Result<()> {
        if let Some(engine) = &self.engine {
            let key = self.sync_engine(false)?;
            return engine.live(&key, self.studio().is_live(&key));
        }
        let probes = self.studio().live_videos_to_probe();
        for probe in probes {
            self.learn_video(Some(probe), Wait::No);
        }
        let delivered = self.idle_studio().frame_for_screen(time, Instant::now());
        self.deliver(delivered)
    }

    /// Runs `probe` of the theme's video outside the session's lock (ffprobe
    /// reads a container that is not MP4 or GIF) and hands what it said to
    /// the session; with [`Wait::No`], nothing while a storage job holds the
    /// converter.
    fn learn_video(&self, probe: Option<VideoProbe>, wait: Wait) {
        if let Some(probed) = probe.and_then(|probe| probe.run(wait)) {
            self.studio().probed(probed);
        }
    }

    fn find_screen(&self, key: &str) -> UiResult<Screen> {
        Ok(choose_screen(
            discover_screens(self.bus.as_ref())?,
            Some(key),
        )?)
    }

    /// Opens the screen `key`: under the claim of the storage operation that
    /// needs it ([`StorageState::claim`]), never in the final state of a
    /// shutdown (`busy`; D-2026-10-03-power-off-standby-3).
    pub(crate) fn connect(&self, key: &str) -> UiResult<Box<dyn ScreenLink>> {
        self.storage.refuse_while_shutting_down()?;
        let screen = self.find_screen(key)?;
        Ok(self.connector.connect(&screen)?)
    }

    // -------------------------------------------------------- preferences --

    /// The app's language: the one chosen in the settings, else the system's.
    pub fn language(&self) -> Language {
        self.settings
            .load()
            .language()
            .unwrap_or(self.system_language)
    }

    /// The backend's own texts, in the app's language.
    pub fn texts(&self) -> Texts {
        texts(self.language())
    }

    /// What the preferences show.
    pub fn set_debug(
        &self,
        on: bool,
        show_readings: bool,
        corner: crate::performance::Corner,
    ) -> std::io::Result<()> {
        crate::debug::configure(
            &bezel_power::diagnostics::logs_directory().join("studio.log"),
            on,
            show_readings,
            corner,
        )?;
        self.settings.update(|s| {
            s.debug = on;
            s.debug_show_readings = show_readings;
            s.debug_corner = corner;
        });
        self.sync_engine_options().map_err(std::io::Error::other)?;
        Ok(())
    }

    pub fn restore_debug(&self) {
        let settings = self.settings.load();
        let _ = crate::debug::configure(
            &bezel_power::diagnostics::logs_directory().join("studio.log"),
            settings.debug,
            settings.debug_show_readings,
            settings.debug_corner,
        );
        let _ = self.sync_engine_options();
    }

    pub fn preferences(&self) -> PreferencesDto {
        let settings = self.settings.load();
        PreferencesDto {
            audio_capture: settings.audio_capture.0,
            debug: settings.debug,
            debug_show_readings: settings.debug_show_readings,
            debug_corner: settings.debug_corner,
            debug_log_path: bezel_power::diagnostics::logs_directory()
                .join("studio.log")
                .display()
                .to_string(),
            light_runtime: cfg!(windows),
            theme_filter: ThemeFilterDto {
                scope: settings.themes_shown(),
                axis: settings.themes_axis(),
            },
            language: settings.language().map(language_slug),
            system_language: language_slug(self.system_language),
            ping_host: settings.sensor_options().ping_host,
            default_ping_host: SensorOptions::DEFAULT_PING_HOST,
        }
    }

    pub fn profiles(&self) -> UiResult<bezel_runtime::profiles::Overview> {
        use bezel_runtime::profiles::Display;
        let mut overview =
            bezel_runtime::profiles::ProfileFile::beside(self.settings.path()).overview()?;
        for screen in self.devices()?.screens {
            if !overview.displays.iter().any(|d| d.key == screen.key) {
                overview.displays.push(Display {
                    key: screen.key.clone(),
                    name: screen.key.clone(),
                    theme: None,
                    theme_name: None,
                    connected: true,
                });
            }
            if let Some(display) = overview.displays.iter_mut().find(|d| d.key == screen.key) {
                display.connected = true;
                display.name = screen
                    .models
                    .first()
                    .map_or_else(|| screen.family.to_owned(), |m| m.name.to_owned());
            }
        }
        for (key, (name, location)) in self.studio().profile_documents() {
            if let Some(display) = overview.displays.iter_mut().find(|d| d.key == key) {
                display.theme_name = Some(name);
                display.theme = location;
            }
        }
        overview.displays.sort_by(|a, b| {
            b.connected
                .cmp(&a.connected)
                .then_with(|| a.key.cmp(&b.key))
        });
        // The active profile and why come from the engine that applies them.
        overview.status = match &self.engine {
            Some(engine) => engine.profile_status().ok().flatten(),
            None => None,
        };
        Ok(overview)
    }

    pub fn save_profiles(&self, config: bezel_runtime::profiles::Config) -> UiResult<()> {
        config.validate()?;
        // Refuse unreadable themes before persisting a profile the user cannot use.
        for profile in &config.profiles {
            for theme in profile.themes.values() {
                let _ = self
                    .store
                    .load(&bezel_core::ports::ThemeLocation(theme.clone()))?;
            }
        }
        Ok(bezel_runtime::profiles::ProfileFile::beside(self.settings.path()).save(&config)?)
    }

    /// Plays `transition` on the live screens towards `themes` (the profile
    /// selected in the manager, by screen key, saved or not) and back after a
    /// moment, saving nothing; answers how many screens it covers.
    pub fn preview_transition(
        &self,
        transition: bezel_runtime::profiles::TransitionSettings,
        themes: std::collections::BTreeMap<String, String>,
    ) -> UiResult<usize> {
        let config = bezel_runtime::profiles::Config {
            transition,
            ..Default::default()
        };
        config.validate()?;
        match &self.engine {
            Some(engine) => Ok(engine.preview_cover(&config.transition, &themes)?),
            None => Ok(0),
        }
    }

    /// A temporary thirty-second display trial; does not write the profile file.
    pub fn try_themes(
        &self,
        themes: std::collections::BTreeMap<String, String>,
        profile: Option<String>,
    ) -> UiResult<usize> {
        if let Some(id) = &profile {
            let config =
                bezel_runtime::profiles::ProfileFile::beside(self.settings.path()).load()?;
            if !config.profiles.iter().any(|p| &p.id == id) {
                return Err(UiError::new(ErrorCode::InvalidInput).arg("detail", "unknown profile"));
            }
        }
        for location in themes.values() {
            if !self.library.allows(&ThemeLocation(location.clone())) {
                return Err(UiError::new(ErrorCode::NotInLibrary).arg("location", location));
            }
        }
        match &self.engine {
            Some(engine) => Ok(engine.try_themes(&themes, profile)?),
            None => Ok(0),
        }
    }

    pub fn select_profile(&self, id: Option<&str>, automatic: bool) -> UiResult<()> {
        Ok(
            bezel_runtime::profiles::ProfileFile::beside(self.settings.path())
                .select(id, automatic)?,
        )
    }

    /// Remembers `ping_host` (empty: the default) and rebuilds the sensors
    /// outside the session's lock.
    pub fn set_sensor_options(&self, ping_host: &str) -> UiResult<()> {
        let host = match ping_host.trim() {
            "" | SensorOptions::DEFAULT_PING_HOST => None,
            host if is_host(host) => Some(host.to_string()),
            host => return Err(UiError::new(ErrorCode::InvalidHost).arg("host", host)),
        };
        self.settings.update(|s| {
            s.ping_host = host;
        });
        if self.engine.is_some() {
            self.sync_engine_options()?;
            return Ok(());
        }
        let sensors = (self.make_sensors)(self.settings.load().sensor_options());
        self.studio().replace_sensors(sensors)?;
        Ok(())
    }

    /// Remembers which themes the Themes tab lists: `scope` (`screen`,
    /// `all`, or `None` for the screen's when one is known) and `axis`
    /// (`all`, `vertical` or `horizontal`).
    pub fn set_theme_filter(&self, scope: Option<&str>, axis: &str) -> UiResult<()> {
        let unknown = |text: &str| {
            UiError::new(ErrorCode::InvalidInput).arg("detail", format!("theme filter \"{text}\""))
        };
        if let Some(scope) = scope.filter(|s| !THEME_SCOPES.contains(s)) {
            return Err(unknown(scope));
        }
        if !THEME_AXES.contains(&axis) {
            return Err(unknown(axis));
        }
        self.settings.update(|s| {
            s.themes_shown = scope.map(str::to_string);
            s.themes_axis = Some(axis.to_string());
        });
        Ok(())
    }

    /// Uses `language` (`pt-BR` or `en`) from now on, or the system's for
    /// `None`, and remembers the choice. Returns the language in use.
    pub fn set_language(&self, language: Option<&str>) -> UiResult<Language> {
        let chosen = language
            .map(|slug| {
                parse_language(slug)
                    .ok_or_else(|| UiError::new(ErrorCode::UnknownLanguage).arg("language", slug))
            })
            .transpose()?;
        self.settings
            .update(|s| s.language = chosen.map(|l| language_slug(l).to_string()));
        let language = chosen.unwrap_or(self.system_language);
        self.studio().set_language(self.system_language);
        Ok(language)
    }

    // ------------------------------------------------------------ screens --

    /// The connected screens and the panels in desktop mode (read-only).
    pub fn devices(&self) -> UiResult<DevicesDto> {
        Ok(DevicesDto::from(&discover_devices(self.bus.as_ref())?))
    }

    /// Switches the panel in desktop mode at `key` back to USB monitor mode
    /// (D-2026-09-30-release-polish-8, not validated on hardware). Without
    /// `Confirm::Yes` nothing is sent; in the final state of a shutdown
    /// neither (`busy`, D-2026-10-03-power-off-standby-3).
    pub fn leave_desktop_mode(&self, key: &str, confirm: Confirm) -> UiResult<MonitorModeDto> {
        self.storage.refuse_while_shutting_down()?;
        let switched =
            leave_desktop_mode(self.bus.as_ref(), self.hid.as_ref(), Some(key), confirm)?;
        Ok(MonitorModeDto::from(&switched))
    }

    /// Restarts the hung screen `key` without a USB replug
    /// (D-2026-09-30-release-polish-13): live mode on it (by either of its
    /// ports) stops first (its port closes), its MCU restarts it and Bezel
    /// waits until it is back (about 10 s); a screen that was live shows the
    /// theme live again, under its new key. Storage operations, live mode
    /// and brightness answer `busy` meanwhile.
    pub fn restart_screen(&self, key: &str, time: LocalTime) -> UiResult<RestartedDto> {
        let restarted = {
            let _claim = self.storage.claim()?;
            let was_live = {
                let mut studio = self.idle_studio();
                let live = studio.is_live(key);
                if live {
                    drop(studio.stop_live());
                }
                live
            };
            // A shutdown that started meanwhile waits for this claim: the MCU
            // restarts nothing then.
            self.storage.refuse_while_shutting_down()?;
            let screen = restart_screen(self.bus.as_ref(), self.connector.as_ref(), Some(key))?;
            (key_of(&screen, key), was_live)
        };
        let (key, was_live) = restarted;
        let live = was_live
            && self
                .set_live(true, Some(&key), time)
                .inspect_err(|_| diag::report(DiagCode::LiveNotResumed))
                .is_ok();
        Ok(RestartedDto { key, live })
    }

    fn pause_live_for_open(&self) -> UiResult<()> {
        let previous = {
            let mut studio = self.idle_studio();
            if studio.shutting_down() {
                return Err(UiError::new(ErrorCode::Busy));
            }
            studio.stop_live()
        };
        drop(previous);
        self.finish_live_off()
    }

    fn finish_live_off(&self) -> UiResult<()> {
        if let Some(engine) = &self.engine {
            let key = self.sync_engine(false)?;
            engine.live(&key, false)?;
        }
        let selected = self.studio().selected_screen().map(str::to_owned);
        self.settings.update(|s| {
            if let Some(key) = selected {
                s.live_screens.retain(|k| *k != key);
            }
            s.live_screen = s.live_screens.last().cloned();
        });
        Ok(())
    }

    /// Starts (`screen` given, by either of its ports) or stops showing the
    /// edited theme live. The screen goes live under its one key, the
    /// address it is listed by once connected (its display's;
    /// D-2026-10-01-live-screen-controls-2), which is remembered with the
    /// orientation; `screen` only when it cannot be listed again. The screen
    /// is opened under the claim on the screens ([`StorageState::claim`]).
    /// In the final state of a shutdown, `busy`: live mode and the
    /// remembered live screen stay as they are.
    pub fn set_live(&self, on: bool, screen: Option<&str>, time: LocalTime) -> UiResult<()> {
        if on && let Some(key) = screen {
            self.select_screen(key)?;
        }
        // Stop first: a screen can only be opened once.
        let previous = {
            let mut studio = self.idle_studio();
            if studio.shutting_down() {
                return Err(UiError::new(ErrorCode::Busy));
            }
            let previous = studio.stop_live();
            let alias = screen.and_then(|key| studio.stop_live_screen(key));
            (previous, alias)
        };
        drop(previous);
        if !on {
            return self.finish_live_off();
        }
        let asked = screen.ok_or_else(|| UiError::new(ErrorCode::NoScreenChosen))?;
        let (key, orientation) = {
            let _claim = self.storage.claim()?;
            // Opening wakes the screen (seconds); the session stays usable
            // meanwhile.
            let (found, link) =
                connect_screen(self.bus.as_ref(), self.connector.as_ref(), Some(asked))?;
            let key = key_of(&found, asked);
            let mut studio = self.studio();
            if !studio.go_live_on(key.clone(), found, link) {
                return Err(UiError::new(ErrorCode::Busy));
            }
            studio.canonicalize_selected_screen(&key);
            (key, studio.theme().orientation)
        };
        self.show_now(time)?;
        self.settings.update(|s| {
            s.live_screen = Some(key.clone());
            s.selected_screen = Some(key.clone());
            if !s.live_screens.contains(&key) {
                s.live_screens.push(key.clone());
            }
            s.remember_orientation(&key, orientation);
        });
        Ok(())
    }

    /// The tray's live switch: off when a screen is live, else on for the
    /// first connected screen (an awake one first). Whether a screen is live
    /// after.
    pub fn toggle_live(&self, time: LocalTime) -> UiResult<bool> {
        if self.studio().any_live() {
            if let Some(engine) = &self.engine {
                for key in self.settings.load().live_screens {
                    engine.live(&key, false)?;
                }
            }
            self.idle_studio().stop_all();
            self.settings.update(|s| {
                s.live_screens.clear();
                s.live_screen = None;
            });
            return Ok(false);
        }
        let screen =
            discover_screens(self.bus.as_ref()).and_then(|screens| choose_screen(screens, None))?;
        let key = screen
            .address()
            .ok_or_else(|| UiError::new(ErrorCode::NoScreenAddress))?;
        self.set_live(true, Some(&key.0), time)?;
        Ok(true)
    }

    /// Remembers `orientation` as the last one used with `screen` (the file
    /// is written only when it changes).
    fn remember_orientation(&self, screen: &str, orientation: Orientation) {
        if self.settings.load().orientation_for(screen) != Some(orientation) {
            self.settings
                .update(|s| s.remember_orientation(screen, orientation));
        }
    }

    /// Sets a screen's brightness (through the live link when it is live,
    /// named by either of its ports; else the screen opened under the claim
    /// on the screens).
    pub fn set_brightness(&self, screen: &str, percent: u8) -> UiResult<()> {
        let brightness =
            Brightness::new(percent).ok_or_else(|| UiError::new(ErrorCode::BrightnessRange))?;
        if self.idle_studio().live_brightness(screen, brightness)? {
            return Ok(());
        }
        let _claim = self.storage.claim()?;
        Ok(self.connect(screen)?.set_brightness(brightness)?)
    }

    /// Hands the live screens to Bezel Light: live mode stops (the settings
    /// keep the live screens, which Light reads), each screen's last
    /// confirmed frame is left in `store`, dated `now` (since the Unix
    /// epoch), and the links close without sending anything more, so the
    /// screens keep showing that frame and Light's first frame is a
    /// difference. How many frames were left; one that cannot be left only
    /// means a full first frame from Light.
    pub fn hand_over_live(&self, store: &dyn HandoffStore, now: Duration) -> usize {
        if let Some(engine) = &self.engine {
            engine.suspend(true);
            if !engine.wait_idle(Instant::now() + Duration::from_secs(5)) {
                return 0;
            }
        }
        let links = self.idle_studio().take_all_live();
        let mut left = 0;
        for (address, link) in links {
            match hand_over(link.as_ref(), address, store, now) {
                Ok(HandedOver::Baseline) => left += 1,
                Ok(HandedOver::Nothing) => {}
                Err(_) => diag::report(DiagCode::HandoffFrameNotLeft),
            }
        }
        left
    }

    /// Hands a screen back to its own mode; a live one (named by either of
    /// its ports) stops live mode and is released through its live link.
    /// Under the claim on the screens.
    pub fn release(&self, screen: &str) -> UiResult<()> {
        if let Some(engine) = &self.engine {
            let key = self.sync_engine(false)?;
            engine.live(&key, false)?;
        }
        let _claim = self.storage.claim()?;
        let live = {
            let mut studio = self.idle_studio();
            if studio.is_live(screen) {
                studio.stop_live()
            } else {
                None
            }
        };
        let mut link = match live {
            Some(link) => {
                self.settings.update(|s| s.live_screen = None);
                link
            }
            None => self.connect(screen)?,
        };
        Ok(link.release()?)
    }

    // ------------------------------------------------------------ sensors --

    /// The sensor catalog, re-read.
    pub fn catalog(&self) -> UiResult<Vec<SensorDto>> {
        if let Some(engine) = &self.engine {
            let key = self.sync_engine(false)?;
            let (value, _) = engine.request(
                &key,
                bezel_runtime::wire::RuntimeOp::Sensors { listed: None },
            )?;
            return Ok(bezel_runtime::wire::catalog(&value)
                .iter()
                .map(SensorDto::from)
                .collect());
        }
        let mut studio = self.studio();
        Ok(studio
            .refresh_catalog()?
            .iter()
            .map(SensorDto::from)
            .collect())
    }

    /// The sensors the library's list shows now (none while it is hidden):
    /// the refresh loop measures them with the theme's
    /// (D-2026-09-30-release-polish-11). Keys that are not sensor keys are
    /// left out.
    pub fn show_sensors(&self, keys: &[String]) {
        if let Some(engine) = &self.engine {
            if let Ok(key) = self.sync_engine(false) {
                let _ = engine.request(
                    &key,
                    bezel_runtime::wire::RuntimeOp::Sensors {
                        listed: Some(keys.to_vec()),
                    },
                );
            }
            return;
        }
        let listed: Wanted = keys
            .iter()
            .filter_map(|k| SensorKey::new(k.as_str()))
            .collect();
        self.studio().show_sensors(listed);
    }

    /// The latest readings (sampled by the refresh loop), with the live
    /// screen's frame rate and the applications and media sessions.
    pub fn sample(&self) -> SampleDto {
        if let Some(engine) = &self.engine {
            let key = self
                .studio()
                .selected_screen()
                .unwrap_or("__editor__")
                .to_owned();
            let status = engine.request(
                &key,
                bezel_runtime::wire::RuntimeOp::Sensors { listed: None },
            );
            let value = status
                .as_ref()
                .map(|s| s.0.clone())
                .unwrap_or(serde_json::Value::Null);
            let catalog = bezel_runtime::wire::catalog(&value);
            let snapshot = bezel_runtime::wire::snapshot(&value);
            let video = value["video"]["state"]
                .as_str()
                .and_then(|s| {
                    [
                        "notStarted",
                        "onDevice",
                        "storedMissing",
                        "missing",
                        "host",
                        "noConverter",
                        "noPlayback",
                    ]
                    .into_iter()
                    .find(|slug| *slug == s)
                })
                .map(|state| LiveVideoDto {
                    state,
                    path: value["video"]["path"].as_str().map(str::to_owned),
                });
            return SampleDto {
                live_fps: value["fps"].as_f64(),
                activity: serde_json::from_value(value["activity"].clone()).ok(),
                sample_millis: 0.0,
                readings: SampleDto::readings(
                    &snapshot,
                    &bezel_core::domain::sensor::Quantities::from_catalog(&catalog),
                ),
                live: self.studio().live_key().map(str::to_owned),
                live_error: status.err().map(UiError::from).or_else(|| {
                    value["error"]
                        .as_str()
                        .map(|e| UiError::from(bezel_core::BezelError::Transport(e.into())))
                }),
                video,
                reconnecting: None,
            };
        }
        let studio = self.studio();
        let (snapshot, millis) = studio.readings();
        SampleDto {
            live_fps: studio.live_fps(Instant::now()),
            activity: ActivityDto::of(snapshot),
            sample_millis: millis,
            readings: SampleDto::readings(snapshot, studio.quantities()),
            live: studio.live_key().map(str::to_string),
            live_error: studio.live_error().cloned(),
            video: studio.live_video().and_then(LiveVideoDto::of),
            reconnecting: studio.reconnecting().map(ReconnectingDto::from),
        }
    }

    // ------------------------------------------------------------- themes --

    /// The edited theme and its location.
    pub fn session(&self) -> SessionDto {
        let studio = self.studio();
        SessionDto {
            version: crate::EVO_VERSION.into(),
            screen: studio.selected_screen().map(str::to_owned),
            theme: ThemeDto::from(studio.theme()),
            location: studio.location().map(|l| l.0.clone()),
            min_refresh_seconds: MIN_REFRESH_SECONDS,
        }
    }

    pub fn select_screen(&self, key: &str) -> UiResult<SessionDto> {
        let screen = self.find_screen(key)?;
        let model = screen
            .candidates
            .first()
            .copied()
            .ok_or_else(|| UiError::new(ErrorCode::NoModel))?;
        let settings = self.settings.load();
        let orientation = settings
            .orientation_for(key)
            .unwrap_or_else(|| default_orientation(model));
        let blank = Theme::blank(self.texts().untitled, model.panel, orientation);
        let fresh = self.studio().select_screen(key, blank);
        if fresh && let Some(path) = settings.screen_themes.get(key) {
            let location = ThemeLocation(path.clone());
            self.library.grant(&location);
            if self.open_at(location).is_err() {
                diag::report(DiagCode::LastThemeNotReopened);
            }
        }
        let location = self.studio().location().cloned();
        if let Some(location) = location {
            self.remember_document(&location);
        }
        self.settings
            .update(|s| s.selected_screen = Some(key.into()));
        Ok(self.session())
    }

    fn remember_document(&self, location: &ThemeLocation) {
        let selected = self.studio().selected_screen().map(str::to_owned);
        self.settings.update(|s| {
            s.last_theme = Some(location.0.clone());
            if let Some(key) = selected {
                s.screen_themes.insert(key, location.0.clone());
            }
        });
    }

    /// Takes the UI's theme and renders it at `now` ([`frame_bytes`]: its
    /// size, when its next picture is due, then RGBA). A video background
    /// plays with `motion` ([`Studio::preview`]), else shows its poster; a
    /// video not probed yet is probed first, outside the session's lock
    /// (while a storage job holds the converter, the poster shows and the UI
    /// asks again soon).
    pub fn render(
        &self,
        theme: &ThemeDto,
        time: LocalTime,
        now: Instant,
        motion: Motion,
    ) -> UiResult<Vec<u8>> {
        self.render_cards(theme, time, now, motion, true)
    }

    /// Render with the editor's transient card-playback policy.
    /// Canvas transfer bounded by the displayed viewport.
    pub fn render_canvas(
        &self,
        theme: &ThemeDto,
        time: LocalTime,
        now: Instant,
        motion: Motion,
        cards: bool,
        max: Option<u32>,
    ) -> UiResult<Vec<u8>> {
        if let Some(engine) = &self.engine {
            self.studio().set_theme(theme_of(theme)?);
            let key = self.sync_engine(false)?;
            return Ok(engine.canvas(&key, max.unwrap_or(2320).clamp(16, 2320), cards)?);
        }
        self.render_cards(theme, time, now, motion, cards)
    }
    /// The canvas with trial inputs instead of the sensors (the canvas's trial
    /// values): at `ms` of the trial, or with `hold` one element's sensor held
    /// at a share of its scale. Nothing reaches a screen or the document.
    pub fn render_trial(
        &self,
        theme: &ThemeDto,
        time: LocalTime,
        motion: Motion,
        max: Option<u32>,
        (ms, hold): (u32, Option<bezel_runtime::wire::TrialHold>),
    ) -> UiResult<Vec<u8>> {
        let max = max.unwrap_or(2320).clamp(16, 2320);
        if let Some(engine) = &self.engine {
            self.studio().set_theme(theme_of(theme)?);
            let key = self.sync_engine(false)?;
            let (value, pixels) = engine.request(
                &key,
                bezel_runtime::wire::RuntimeOp::Trial {
                    max,
                    motion: motion == Motion::Allowed,
                    ms,
                    hold,
                },
            )?;
            bezel_runtime::wire::frame(&value, pixels.clone())?;
            return Ok(crate::engine::bytes(&value, pixels));
        }
        let theme = theme_of(theme)?;
        let mut studio = self.studio();
        studio.set_theme(theme);
        let hold = hold.map(|h| bezel_runtime::inspector_trial::Hold {
            element: ElementId(h.element),
            fraction: h.fraction,
        });
        let frame = studio.canvas_trial(time, motion, ms, hold)?;
        drop(studio);
        Ok(frame_bytes(
            &fit_within(frame, max),
            Some(Duration::from_millis(
                bezel_runtime::inspector_trial::FRAME_MS,
            )),
        ))
    }
    pub fn render_cards(
        &self,
        theme: &ThemeDto,
        time: LocalTime,
        now: Instant,
        motion: Motion,
        card_rotation: bool,
    ) -> UiResult<Vec<u8>> {
        if let Some(engine) = &self.engine {
            self.studio().set_theme(theme_of(theme)?);
            let key = self.sync_engine(false)?;
            return Ok(engine.canvas(&key, 2320, card_rotation)?);
        }
        let started = Instant::now();
        let theme = theme_of(theme)?;
        let waiting = Instant::now();
        let mut studio = self.studio();
        let lock_ms = waiting.elapsed().as_secs_f64() * 1000.0;
        studio.set_theme(theme);
        studio.automatic_cards(card_rotation);
        if let Some(probe) = studio.video_to_probe() {
            drop(studio);
            self.learn_video(Some(probe), Wait::No);
            // The theme the UI sent last, which may be a newer one by now.
            studio = self.studio();
        }
        let (frame, change) = studio.preview(time, now, motion)?;
        let reused = studio.render_reused();
        drop(studio);
        let bytes = frame_bytes(&frame, change);
        if crate::debug::enabled() {
            crate::performance::preview_render(
                started.elapsed().as_secs_f64() * 1000.0,
                lock_ms,
                reused,
            );
        }
        Ok(bytes)
    }

    /// Measures the edited element through the native preview renderer.
    pub fn measure_element(
        &self,
        theme: &ThemeDto,
        element: u32,
        time: LocalTime,
        now: Instant,
    ) -> UiResult<Option<bezel_themes::dto::BoxDto>> {
        if let Some(engine) = &self.engine {
            self.studio().set_theme(theme_of(theme)?);
            let key = self.sync_engine(false)?;
            let (value, _) =
                engine.request(&key, bezel_runtime::wire::RuntimeOp::Measure { element })?;
            return serde_json::from_value(value)
                .map_err(|e| UiError::from(bezel_core::BezelError::InvalidInput(e.to_string())));
        }
        let theme = theme_of(theme)?;
        let mut studio = self.studio();
        studio.set_theme(theme);
        Ok(studio
            .measure_element(ElementId(element), time, now)?
            .map(|b| bezel_themes::dto::BoxDto {
                x: b.x,
                y: b.y,
                width: b.width,
                height: b.height,
            }))
    }

    /// Takes the UI's theme and draws its element `element` on its own, as
    /// the inspector previews it (D-2026-10-09-studio-redesign-fidelity-8):
    /// [`frame_bytes`] of the element alone on a transparent canvas, its
    /// longest side at most `max` ([`side_of`]), drawn by the session's
    /// renderer from the last sample at `now`
    /// ([`Studio::element_preview`]). `invalidInput` when the theme has no
    /// such element.
    pub fn render_element(
        &self,
        theme: &ThemeDto,
        element: u32,
        max: Option<u32>,
        time: LocalTime,
        now: Instant,
        motion: Motion,
    ) -> UiResult<Vec<u8>> {
        self.render_element_with(theme, element, (max, None), time, now, motion)
    }

    /// Preview options are transport-only; trial time is never part of a theme.
    pub fn render_element_with(
        &self,
        theme: &ThemeDto,
        element: u32,
        (max, trial_ms): (Option<u32>, Option<u32>),
        time: LocalTime,
        now: Instant,
        motion: Motion,
    ) -> UiResult<Vec<u8>> {
        if let Some(engine) = &self.engine {
            self.studio().set_theme(theme_of(theme)?);
            let key = self.sync_engine(false)?;
            let (value, pixels) = engine.request(
                &key,
                bezel_runtime::wire::RuntimeOp::Inspect {
                    element,
                    max: side_of(max),
                    motion: motion == Motion::Allowed,
                    trial_ms,
                },
            )?;
            bezel_runtime::wire::frame(&value, pixels.clone())?;
            return Ok(crate::engine::bytes(&value, pixels));
        }
        let theme = theme_of(theme)?;
        let mut studio = self.studio();
        studio.set_theme(theme);
        let (frame, change) =
            studio.element_trial(ElementId(element), time, now, motion, trial_ms)?;
        drop(studio);
        Ok(frame_bytes(&fit_within(frame, side_of(max)), change))
    }

    /// Takes the UI's theme and says what Auto turns its video background
    /// and the video's own size (D-2026-10-01-video-background-framing-2):
    /// probed once (an MP4's header needs no ffmpeg), outside the session's
    /// lock, waiting for a storage job that holds the converter.
    pub fn video_auto(&self, theme: &ThemeDto) -> UiResult<VideoAutoDto> {
        let theme = theme_of(theme)?;
        let probe = {
            let mut studio = self.studio();
            studio.set_theme(theme);
            studio.video_to_probe()
        };
        self.learn_video(probe, Wait::Yes);
        Ok(VideoAutoDto::of(self.studio().video_auto()))
    }

    /// Takes the UI's theme and shows it on the live screen now, in the
    /// theme's orientation (remembered for that screen).
    pub fn push(&self, theme: &ThemeDto, time: LocalTime) -> UiResult<()> {
        let theme = theme_of(theme)?;
        let orientation = theme.orientation;
        let live = {
            let mut studio = self.studio();
            studio.set_theme(theme);
            studio.live_key().map(str::to_string)
        };
        self.show_now(time)?;
        if let Some(key) = live {
            self.remember_orientation(&key, orientation);
        }
        Ok(())
    }

    /// The library's themes.
    pub fn themes(&self) -> Vec<ThemeEntryDto> {
        self.library
            .list()
            .iter()
            .map(ThemeEntryDto::from)
            .collect()
    }

    /// Copies the library theme at `location` into the user's folder as a
    /// new theme named `name` (the UI's translated "Copy of …"), assets and
    /// all, and lists the copy. The original is only read: a bundled theme
    /// is never written. `notInLibrary` for a theme the library does not
    /// allow, `invalidInput` for an empty name.
    pub fn duplicate_theme(&self, location: &str, name: &str) -> UiResult<ThemeEntryDto> {
        let source = ThemeLocation(location.to_string());
        if !self.library.allows(&source) {
            return Err(UiError::new(ErrorCode::NotInLibrary).arg("location", source.0));
        }
        let name = name.trim();
        if name.is_empty() {
            return Err(UiError::new(ErrorCode::InvalidInput).arg("detail", "an empty theme name"));
        }
        let (mut theme, assets) = self.store.load(&source)?;
        theme.name = name.to_string();
        let copy = self.library.new_location(name);
        self.store.save(&copy, &theme, &assets)?;
        self.library
            .entry(&copy)
            .map(|entry| ThemeEntryDto::from(&entry))
            .ok_or_else(|| UiError::new(ErrorCode::ThemeFile).arg("detail", copy.0))
    }

    /// Exports a saved theme and every referenced asset without opening it in Studio.
    pub fn export_theme(&self, location: &str, target: &ThemeLocation) -> UiResult<String> {
        let source = ThemeLocation(location.into());
        if !self.library.allows(&source) {
            return Err(UiError::new(ErrorCode::NotInLibrary).arg("location", location));
        }
        if !self.library.allows(target) || self.library.is_bundled(target) {
            return Err(UiError::new(ErrorCode::NotPicked).arg("location", &target.0));
        }
        if Path::new(&target.0)
            .extension()
            .is_none_or(|e| e != bezel_themes::native::EXTENSION)
        {
            return Err(
                UiError::new(ErrorCode::InvalidInput).arg("detail", "export needs .bezeltheme")
            );
        }
        let (theme, assets) = self.store.load(&source)?;
        for asset in theme.assets() {
            if !assets.contains_key(&asset) {
                return Err(UiError::new(ErrorCode::ThemeFile)
                    .arg("detail", format!("missing asset: {}", asset.0)));
            }
        }
        self.store.save(target, &theme, &assets)?;
        Ok(target.0.clone())
    }

    /// Removes an unused personal theme, after resolving it strictly inside the user library.
    pub fn delete_theme(&self, location: &str) -> UiResult<()> {
        let source = ThemeLocation(location.into());
        if !self.library.is_user(&source) || self.library.entry(&source).is_none() {
            return Err(UiError::new(ErrorCode::NotInLibrary).arg("location", location));
        }
        let path = std::fs::canonicalize(location).map_err(UiError::system)?;
        let root = std::fs::canonicalize(self.library.user_dir()).map_err(UiError::system)?;
        if path == root || !path.starts_with(&root) {
            return Err(UiError::new(ErrorCode::NotInLibrary).arg("location", location));
        }
        let overview = self.profiles()?;
        let used = overview
            .config
            .profiles
            .iter()
            .any(|p| p.themes.values().any(|v| v == location))
            || overview
                .displays
                .iter()
                .any(|d| d.theme.as_deref() == Some(location))
            || overview
                .status
                .as_ref()
                .and_then(|s| s.preview.as_ref())
                .is_some_and(|p| p.themes.values().any(|v| v == location))
            || self.studio().location() == Some(&source);
        if used {
            return Err(UiError::new(ErrorCode::InvalidInput).arg("detail", "theme still in use"));
        }
        if path.is_dir() {
            std::fs::remove_dir_all(path).map_err(UiError::system)?;
        } else {
            std::fs::remove_file(path).map_err(UiError::system)?;
        }
        self.thumbnails.forget(&source);
        Ok(())
    }

    /// The thumbnail of a library theme as a PNG `data:` URL, drawn at
    /// `time` when it is not kept yet, outside the session's lock. `None`
    /// when the theme cannot be drawn: the gallery shows its placeholder.
    pub fn thumbnail(&self, location: &str, time: LocalTime) -> UiResult<Option<String>> {
        let location = ThemeLocation(location.to_string());
        if !self.library.allows(&location) {
            return Err(UiError::new(ErrorCode::NotInLibrary).arg("location", location.0));
        }
        let language = self.language();
        Ok(self
            .thumbnails
            .get(self.store.as_ref(), &location, language, time))
    }

    /// Opens a theme of the library, or a theme file the user picked in a
    /// dialog during this session.
    pub fn open(&self, location: &str) -> UiResult<ThemeDto> {
        let location = ThemeLocation(location.to_string());
        if !self.library.allows(&location) {
            return Err(UiError::new(ErrorCode::NotInLibrary).arg("location", location.0));
        }
        let (theme, assets) = self.store.load(&location)?;
        self.pause_live_for_open()?;
        self.studio().start(theme, assets, Some(location.clone()));
        self.remember_document(&location);
        self.sync_engine_after_document_change();
        Ok(ThemeDto::from(self.studio().theme()))
    }

    /// Tells Light about a document that already changed (opened or saved).
    /// Failing here must not say the open or the save failed: the editor
    /// would keep the previous theme while the session points at the new
    /// file, and its next Save would write that theme over the file. Light
    /// gets the document again at the next sync.
    fn sync_engine_after_document_change(&self) {
        if self.sync_engine(true).is_err() {
            diag::report(DiagCode::DocumentNotSynced);
        }
    }

    /// Opens the theme at `location`, which the app chose itself.
    fn open_at(&self, location: ThemeLocation) -> UiResult<ThemeDto> {
        let mut studio = self.studio();
        studio.open(self.store.as_ref(), location.clone())?;
        let theme = ThemeDto::from(studio.theme());
        drop(studio);
        self.remember_document(&location);
        self.sync_engine_after_document_change();
        Ok(theme)
    }

    /// Saves the UI's theme: to `target` when given (a file picked in the
    /// save dialog, [`ThemeLibrary::grant`]ed first), else where it was
    /// opened from when the library allows it (otherwise, and for a bundled
    /// theme, a copy in the user folder).
    pub fn save(&self, theme: &ThemeDto, target: Option<ThemeLocation>) -> UiResult<SavedDto> {
        self.save_shown(theme, target, None)
    }

    /// [`Self::save`] of the theme the editor shows as opened from `shown`
    /// (`Some(None)`: a theme never saved). A plain Save whose session
    /// document is another file is refused: the editor and the session
    /// disagree on the document, and writing would replace a theme the user
    /// is not looking at. Save As (`target`) writes where the user chose.
    pub fn save_shown(
        &self,
        theme: &ThemeDto,
        target: Option<ThemeLocation>,
        shown: Option<Option<&str>>,
    ) -> UiResult<SavedDto> {
        let theme = theme_of(theme)?;
        if let Some(target) = target.as_ref().filter(|t| !self.library.allows(t)) {
            return Err(UiError::new(ErrorCode::NotPicked).arg("location", &target.0));
        }
        let poster = self.retake_poster(&theme);
        let mut studio = self.studio();
        if target.is_none()
            && let Some(shown) = shown
            && studio.location().map(|l| l.0.as_str()) != shown
        {
            return Err(UiError::new(ErrorCode::DocumentChanged)
                .arg("location", studio.location().map_or("", |l| l.0.as_str())));
        }
        let location =
            target.unwrap_or_else(|| self.library.save_location(studio.location(), &theme.name));
        studio.set_theme(theme);
        if let Some(poster) = poster {
            studio.poster_taken(poster);
        }
        studio.save(self.store.as_ref(), location.clone())?;
        drop(studio);
        // Its gallery card shows it as saved.
        self.thumbnails.forget(&location);
        self.remember_document(&location);
        self.sync_engine_after_document_change();
        Ok(SavedDto {
            location: location.0,
        })
    }

    /// The poster of `theme`'s video background taken again when the theme
    /// frames the video otherwise than the poster shows
    /// (D-2026-10-01-video-background-framing-3), the video probed first if
    /// it is not yet: both outside the session's lock, so the refresh, the
    /// live screen and the preview go on meanwhile. `None` when no new
    /// poster is needed or none could be taken (without ffmpeg, or while a
    /// storage job holds the converter, the poster stays as it is).
    fn retake_poster(&self, theme: &Theme) -> Option<TakenPoster> {
        let probe = {
            let mut studio = self.studio();
            studio.set_theme(theme.clone());
            studio.video_to_probe()
        };
        self.learn_video(probe, Wait::No);
        let retake = {
            let mut studio = self.studio();
            studio.set_theme(theme.clone());
            studio.poster_to_take()
        };
        retake?.run()
    }

    /// A blank theme sized for `screen` (or the 8.8" when none is known) in
    /// `orientation`, which is then remembered for that screen. Without one:
    /// the orientation last used with the screen, else
    /// [`default_orientation`].
    pub fn new_theme(
        &self,
        screen: Option<&str>,
        name: &str,
        orientation: Option<Orientation>,
    ) -> UiResult<ThemeDto> {
        let model = screen
            .and_then(|key| self.find_screen(key).ok())
            .and_then(|s| s.candidates.first().copied())
            .or_else(|| model_by_id(DEFAULT_MODEL))
            .ok_or_else(|| UiError::new(ErrorCode::NoModel))?;
        let orientation = match (orientation, screen) {
            (Some(chosen), Some(key)) => {
                self.remember_orientation(key, chosen);
                chosen
            }
            (Some(chosen), None) => chosen,
            (None, key) => key
                .and_then(|k| self.settings.load().orientation_for(k))
                .unwrap_or_else(|| default_orientation(model)),
        };
        let theme = Theme::blank(name, model.panel, orientation);
        let mut studio = self.studio();
        studio.start(theme, Default::default(), None);
        Ok(ThemeDto::from(studio.theme()))
    }

    /// Imports a theme: Bezel's own (`.bezeltheme`, a folder with a
    /// `theme.json`, or that `theme.json`) as it is; another app's (a TURZX `.turtheme`, a
    /// turing-smart-screen-python `theme.yaml` or its folder) converted, with
    /// what had no exact equivalent as warnings.
    pub fn import(&self, path: &Path) -> UiResult<ImportedDto> {
        let (theme, assets, warnings) = if is_native(path) {
            let (theme, assets) = self.store.load(&native_location(path))?;
            (theme, assets, Vec::new())
        } else {
            let fonts = self.current_fonts();
            // A temporarily unreachable engine must not prevent file import.
            // Without its catalog, leave the bindings available for rebinding.
            let sensors = self.catalog().ok();
            let installed_font = |name: &str| {
                fonts.families.iter().any(|family| {
                    family.name.eq_ignore_ascii_case(name)
                        || family
                            .aliases
                            .iter()
                            .any(|alias| alias.eq_ignore_ascii_case(name))
                })
            };
            let measured = |key: &SensorKey| {
                sensors
                    .as_ref()
                    .is_none_or(|catalog| catalog.iter().any(|sensor| sensor.key == key.as_str()))
            };
            let options = ImportOptions {
                installed_font: &installed_font,
                measured: &measured,
            };
            let (theme, assets, report) = import_path_with(path, &options)?;
            (
                theme,
                assets,
                report.warnings.iter().map(Into::into).collect(),
            )
        };
        self.pause_live_for_open()?;
        let mut studio = self.studio();
        // A copy: saving writes to the user folder, not over the imported file.
        studio.start(theme, assets, None);
        Ok(ImportedDto {
            theme: ThemeDto::from(studio.theme()),
            warnings,
        })
    }

    // -------------------------------------------------------------- media --

    /// Adds the image at `path` to the theme. An animated GIF also gets its
    /// poster, ready to be a video background.
    pub fn add_image(&self, path: &Path) -> UiResult<AddedDto> {
        let bytes = read_file(path)?;
        if is_animated_gif(&bytes) {
            let added = self.add_moving(path, bytes)?;
            return Ok(AddedDto {
                reference: added.reference,
            });
        }
        self.add_image_bytes(path, bytes)
    }

    /// Adds the image file at `path`, whose content is `bytes`.
    pub(crate) fn add_image_bytes(&self, path: &Path, bytes: Vec<u8>) -> UiResult<AddedDto> {
        if image::guess_format(&bytes).is_err() && bezel_render::svg_size(&bytes).is_none() {
            return Err(UiError::new(ErrorCode::NotAnImage).arg("file", path.display()));
        }
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        let asset = self.studio().add_asset(&name, bytes);
        Ok(AddedDto { reference: asset.0 })
    }

    /// Adds a vector icon chosen in the offline media catalog.
    pub fn add_icon(&self, name: &str, svg: String) -> UiResult<AddedDto> {
        if name.is_empty()
            || name.len() > 100
            || !name.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-')
            || svg.len() > 128 * 1024
            || bezel_render::svg_size(svg.as_bytes()).is_none()
        {
            return Err(UiError::new(ErrorCode::NotAnImage).arg("file", name));
        }
        if name.starts_with("mdi-") {
            let Some(root_end) = svg
                .find("<svg")
                .and_then(|start| svg[start..].find('>').map(|end| start + end + 1))
            else {
                return Err(UiError::new(ErrorCode::NotAnImage).arg("file", name));
            };
            let mut svg = svg;
            svg.insert_str(
                root_end,
                &format!(
                    "<metadata><![CDATA[{}\n{}]]></metadata>",
                    include_str!("../../src/assets/mdi/LICENSE"),
                    include_str!("../../src/assets/mdi/NOTICE")
                ),
            );
            self.add_image_bytes(Path::new(&format!("{name}.svg")), svg.into_bytes())
        } else {
            self.add_image_bytes(
                Path::new(&format!("tabler-{name}.svg")),
                format!(
                    "<!--\n{}\n-->\n{svg}",
                    include_str!("../../src/assets/tabler/LICENSE")
                )
                .into_bytes(),
            )
        }
    }

    /// Keeps the media of copied elements for a paste on any screen.
    pub fn copy_media(&self, assets: Vec<String>) {
        let assets: Vec<_> = assets
            .into_iter()
            .map(bezel_core::domain::theme::AssetRef)
            .collect();
        self.studio().copy_media(&assets);
    }

    /// Adds the copied media to this screen's document: copied reference to
    /// the reference here.
    pub fn paste_media(&self) -> std::collections::BTreeMap<String, String> {
        self.studio()
            .paste_media()
            .into_iter()
            .map(|(copied, here)| (copied.0, here.0))
            .collect()
    }

    /// Removes unused media, preserving bytes required by editor Undo.
    pub fn remove_asset(&self, asset: &str, theme: &ThemeDto) -> UiResult<bool> {
        let theme = theme_of(theme)?;
        let mut studio = self.studio();
        studio.set_theme(theme.clone());
        Ok(studio.remove_asset(&bezel_core::domain::theme::AssetRef(asset.into()), &theme))
    }

    /// The theme's assets with previews; videos are not decoded for the list.
    pub fn assets(&self) -> Vec<AssetDto> {
        let listed: Vec<(AssetDto, Option<Vec<u8>>)> = {
            let studio = self.studio();
            let background = match &studio.theme().background {
                Background::Video { asset, poster, .. } => Some((asset.clone(), poster.clone())),
                _ => None,
            };
            studio
                .assets()
                .iter()
                .filter(|(asset, _)| studio.asset_visible(asset))
                .map(|(asset, bytes)| {
                    let kind = kind_of(asset);
                    let known = studio.added_video(asset);
                    let named = background
                        .as_ref()
                        .filter(|(video, _)| video == asset)
                        .and_then(|(_, poster)| poster.clone());
                    let dto = AssetDto {
                        reference: asset.0.clone(),
                        kind,
                        data_url: None,
                        icon_svg: None,
                        bytes: bytes.len() as u64,
                        animated: false,
                        poster: known.and_then(|k| k.poster.clone()).or(named).map(|p| p.0),
                        duration_ms: known
                            .and_then(|k| k.duration)
                            .map(|d| u64::try_from(d.as_millis()).unwrap_or(u64::MAX)),
                    };
                    (dto, (kind == "image").then(|| bytes.clone()))
                })
                .collect()
        };
        listed
            .into_iter()
            .map(|(dto, image)| match image {
                Some(bytes) => AssetDto {
                    data_url: thumbnail_data_url(&bytes),
                    icon_svg: ((dto.reference.starts_with("assets/tabler-")
                        || dto.reference.starts_with("assets/mdi-"))
                        && extension_of(&dto.reference) == "svg"
                        && bytes.len() <= 132 * 1024)
                        .then(|| String::from_utf8(bytes.clone()).ok())
                        .flatten(),
                    animated: extension_of(&dto.reference) == "gif" && is_animated_gif(&bytes),
                    ..dto
                },
                None => dto,
            })
            .collect()
    }

    /// Preview only assets belonging to the current theme, never an arbitrary path.
    pub fn media_video(&self, asset: &str) -> Option<String> {
        let studio = self.studio();
        let bytes = studio
            .assets()
            .get(&bezel_core::domain::theme::AssetRef(asset.into()))?;
        crate::media::video_data_url(asset, bytes)
    }

    // -------------------------------------------------------------- start --

    /// The theme the window starts with: the last one when it still opens,
    /// else a blank one for the first connected screen (in the orientation
    /// [`Self::new_theme`] picks for it).
    pub fn restore_theme(&self) {
        let settings = self.settings.load();
        let selected = settings
            .selected_screen
            .as_ref()
            .or(settings.live_screen.as_ref());
        if let Some(key) = selected {
            let blank = self.studio().theme().clone();
            self.studio().select_screen(key, blank);
        }
        if let Some(last) = selected
            .and_then(|k| settings.screen_themes.get(k).cloned())
            .or(settings.last_theme)
        {
            // The app's own record of a theme the window was allowed to open
            // or save last time: saving writes back to it again.
            let location = ThemeLocation(last.clone());
            self.library.grant(&location);
            match self.open_at(location) {
                Ok(_) => return,
                Err(_) => diag::report(DiagCode::LastThemeNotReopened),
            }
        }
        let screen = discover_screens(self.bus.as_ref())
            .and_then(|screens| choose_screen(screens, None))
            .ok();
        let key = screen
            .as_ref()
            .and_then(Screen::address)
            .map(|a| a.0.clone());
        let blank = match self.new_theme(key.as_deref(), self.texts().untitled, None) {
            Ok(theme) => theme,
            Err(_) => {
                diag::report(DiagCode::NoStartingTheme);
                return;
            }
        };
        // First run: a bundled theme made for this screen and orientation is a
        // better start than an empty canvas (saving it writes a copy).
        let fitting = self.library.list().into_iter().find(|e| {
            e.bundled
                && e.theme.canvas.width == blank.canvas.width
                && e.theme.canvas.height == blank.canvas.height
                && crate::dto::orientation_slug(e.theme.orientation) == blank.orientation
        });
        if let Some(entry) = fitting
            && self.open_at(entry.location.clone()).is_err()
        {
            diag::report(DiagCode::BundledThemeNotOpened);
        }
    }

    /// Shows the theme live again on the screen that was live when the app
    /// last ran, when it is connected. A failure leaves live mode off. A key
    /// saved by an older version as the MCU's port is replaced by the
    /// display's ([`Self::set_live`]; D-2026-10-01-live-screen-controls-2).
    pub fn restore_live(&self, time: LocalTime) {
        let settings = self.settings.load();
        let keys = if settings.live_screens.is_empty() {
            settings.live_screen.into_iter().collect()
        } else {
            settings.live_screens
        };
        for key in keys {
            if self
                .select_screen(&key)
                .and_then(|_| self.set_live(true, Some(&key), time))
                .is_err()
            {
                diag::report(DiagCode::LiveNotRestored);
            }
        }
        if let Some(selected) = settings.selected_screen {
            let canonical = self
                .find_screen(&selected)
                .ok()
                .and_then(|screen| screen.address().map(|address| address.0.clone()))
                .unwrap_or(selected);
            let _ = self.select_screen(&canonical);
        }
    }

    /// One refresh of the session at `now`: an attempt to connect a live
    /// screen that failed when one is due, the probe of the theme's video
    /// when the live screen is to start it, a sample when one is due, and
    /// the live screen's frame when it is due (the screen's I/O and the
    /// probe outside the session's lock). Returns when the next refresh is
    /// due.
    pub fn tick(&self, time: LocalTime, now: Instant) -> Instant {
        if let Some(engine) = &self.engine {
            return now + engine.drive().unwrap_or(Duration::from_millis(100));
        }
        let started = Instant::now();
        self.reconnect(now);
        let probes = self.studio().live_videos_to_probe();
        for probe in probes {
            self.learn_video(Some(probe), Wait::No);
        }
        let waiting = Instant::now();
        let mut studio = self.studio();
        let lock_ms = waiting.elapsed().as_secs_f64() * 1000.0;
        let deliveries = studio.tick_all(time, now);
        drop(studio);
        let frames = deliveries
            .iter()
            .filter(|r| matches!(r, Ok(Some(_))))
            .count();
        for delivered in deliveries {
            self.dispatch(delivered);
        }
        let due = self.studio().next_due_all();
        if crate::debug::enabled() && (frames > 0 || started.elapsed().as_millis() > 50) {
            crate::performance::refresh_tick(
                started.elapsed().as_secs_f64() * 1000.0,
                lock_ms,
                Instant::now().saturating_duration_since(now).as_secs_f64() * 1000.0,
                frames,
            );
        }
        due
    }

    /// Connects a live screen whose link failed again, when an attempt is
    /// due (T-7.11): found by identity (a rev C SoC comes back under a new
    /// device name), restarted through its MCU by the connection when it
    /// hung, outside the session's lock. Back, it shows the theme again
    /// under its new key, which is remembered.
    fn reconnect(&self, now: Instant) {
        let attempts = self.studio().reconnects_due(now);
        for (key, attempt) in attempts {
            let outcome =
                reopen_screen(self.bus.as_ref(), self.connector.as_ref(), attempt.screen());
            let unwanted =
                self.studio()
                    .reconnect_document(key.as_deref(), attempt, outcome, Instant::now());
            drop(unwanted);
        }
        let back = {
            let studio = self.studio();
            studio
                .reconnecting()
                .is_none()
                .then(|| studio.live_key().map(str::to_string))
                .flatten()
        };
        if let Some(key) = back
            && self.settings.load().live_screen.as_deref() != Some(key.as_str())
        {
            self.settings.update(|s| s.live_screen = Some(key));
        }
    }
}

/// The key `screen` goes by: the address it is listed by (its display's
/// when awake), else `asked`, the key it was asked for.
fn key_of(screen: &Screen, asked: &str) -> String {
    screen
        .address()
        .map_or_else(|| asked.to_string(), |a| a.0.clone())
}

/// Model a new theme is sized for when no screen is connected.
pub const DEFAULT_MODEL: bezel_core::domain::device::ModelId =
    bezel_core::domain::device::ModelId("turing-8.8");

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manager::Copies;
    use crate::storage::tests::{Call, FakeMedia, fixture_on};
    use crate::studio::CONVERTER_BUSY;
    use bezel_core::domain::frame::{Frame, Rgba};
    use bezel_core::domain::framing::{VideoFit, VideoFraming, Zoom};
    use bezel_core::domain::geometry::{Orientation, Size};
    use bezel_core::domain::theme::AssetRef;
    use bezel_devices::fake::FakeStorage;
    use bezel_devices::{FakeBus, FakeConnector, FakeHid};
    use bezel_media::archive::MemoryArchive;
    use bezel_render::{SkiaRenderer, SystemFonts};
    use bezel_sensors::FakeSensors;
    use bezel_themes::FsThemeStore;
    use std::collections::BTreeMap;
    use std::path::PathBuf;
    use std::sync::mpsc;

    /// JetBrains Mono NL Regular, the renderer's test font (SIL OFL 1.1).
    const TEST_FONT: &[u8] =
        include_bytes!("../../../../crates/bezel-render/tests/fonts/JetBrainsMonoNL-Regular.ttf");

    #[test]
    fn the_font_catalog_lists_the_renderers_families_and_hands_out_only_bundled_faces() {
        let f = fixture("font-catalog");
        let catalog = f.backend.font_catalog();
        assert_eq!(catalog.fallback.as_deref(), Some("JetBrains Mono NL"));
        // Fonts the user dropped into %APPDATA%\Bezel Evo\fonts are listed too;
        // they belong to this machine, not to the fixture, so look at bundled ones.
        let bundled: Vec<_> = catalog.families.iter().filter(|f| f.bundled).collect();
        assert_eq!(bundled.len(), 1);
        let family = bundled[0];
        assert_eq!(family.name, "JetBrains Mono NL");
        assert_eq!(
            family.faces,
            vec![crate::dto::FontFaceDto {
                weight: 400,
                italic: false
            }]
        );
        assert_eq!(
            (family.category, family.monospace, family.bundled),
            ("mono", true, true)
        );
        let json = serde_json::to_value(&catalog).unwrap();
        let entry = json["families"]
            .as_array()
            .unwrap()
            .iter()
            .find(|f| f["name"] == "JetBrains Mono NL")
            .unwrap();
        assert_eq!(entry["faces"][0]["weight"], 400);
        assert!(entry.get("aliases").is_none(), "no aliases, not written");
        assert_eq!(
            f.backend.font_face("jetbrains mono nl", 400, false),
            TEST_FONT
        );
        assert!(
            f.backend
                .font_face("JetBrains Mono NL", 400, true)
                .is_empty()
        );
        assert!(f.backend.font_face("Arial", 400, false).is_empty());
    }

    #[test]
    fn theme_fonts_are_catalogued_and_only_catalogued_nonbundled_files_can_be_included() {
        let mut f = fixture("font-assets");
        assert!(
            f.backend
                .include_font("JetBrains Mono NL", 400, false)
                .is_err()
        );
        assert!(f.backend.include_font("../../private", 400, false).is_err());
        let asset = f
            .backend
            .studio()
            .add_asset("local.ttf", TEST_FONT.to_vec());
        f.backend.studio().add_asset("bad.otf", b"broken".to_vec());
        let catalog = f.backend.font_catalog();
        let theme_font = catalog
            .families
            .iter()
            .find(|font| font.asset.is_some())
            .unwrap();
        assert_eq!(theme_font.asset.as_deref(), Some(asset.0.as_str()));
        assert_eq!(theme_font.origin, "theme");
        assert!(!theme_font.bundled);
        assert_eq!(f.backend.theme_font_face(&asset.0), TEST_FONT);
        assert!(f.backend.theme_font_face("../../private.ttf").is_empty());
        assert!(f.backend.theme_font_face("assets/bad.otf").is_empty());
        // Windows' system catalog supplies the source; callers cannot submit a path.
        f.backend.fonts = SkiaRenderer::new().font_catalog();
        let catalog = f.backend.current_fonts();
        let system = catalog
            .families
            .iter()
            .find(|font| font.origin == bezel_render::FontOrigin::System)
            .unwrap();
        let face = system.faces[0];
        let added = f
            .backend
            .include_font(&system.name, face.weight, face.italic)
            .unwrap();
        assert!(bezel_render::valid_font(
            &f.backend.theme_font_face(&added.reference)
        ));
        assert!(f.backend.include_font("Inter", 400, false).is_err());
        // Saving and reopening carries the bytes through the existing font.asset field.
        use bezel_core::domain::theme::{
            BoxF, Element, ElementKind, FontSpec, TextContent, TextStyle,
        };
        let mut theme = f.backend.studio().theme().clone();
        theme.elements.push(Element {
            id: ElementId(1),
            name: "Portable text".into(),
            frame: BoxF::new(0., 0., 200., 60.),
            opacity: 1.,
            visible: true,
            locked: false,
            rotation: 0.0,
            is_group: false,
            group_parent: None,
            card_member: None,
            card: None,
            visualization: None,
            value_transition: None,
            sensor_motion: None,
            color_by_value: None,
            kind: ElementKind::Text {
                content: TextContent::Static("Portable".into()),
                style: TextStyle {
                    font: FontSpec {
                        family: system.name.clone(),
                        weight: face.weight,
                        italic: face.italic,
                        asset: Some(bezel_core::domain::theme::AssetRef(added.reference.clone())),
                    },
                    ..TextStyle::default()
                },
            },
        });
        f.backend.studio().set_theme(theme);
        let saved = f.backend.save(&f.backend.session().theme, None).unwrap();
        let (theme, assets) = f
            .backend
            .store
            .load(&ThemeLocation(saved.location))
            .unwrap();
        assert_eq!(theme.assets().len(), 1);
        assert!(bezel_render::valid_font(assets.values().next().unwrap()));
        let mut isolated = Studio::new(
            Box::new(FakeSensors::demo()),
            Box::new(SkiaRenderer::with_fonts(Vec::new(), SystemFonts::Skip)),
            Language::English,
            theme.clone(),
        );
        isolated.start(theme, assets, None);
        assert!(isolated.render(TIME).is_ok());
    }

    const TIME: LocalTime = LocalTime {
        year: 2026,
        month: 9,
        day: 30,
        hour: 21,
        minute: 5,
        second: 0,
        weekday: 2,
        millisecond: 0,
    };
    const KEY: &str = "/dev/ttyACM1";

    struct Fixture {
        backend: Backend,
        connector: FakeConnector,
        root: PathBuf,
        _temporary: tempfile::TempDir,
    }

    fn fixture(name: &str) -> Fixture {
        let temporary = tempfile::Builder::new()
            .prefix(&format!("bezel-backend-{name}-"))
            .tempdir()
            .unwrap();
        let root = temporary.path().to_path_buf();
        let connector = FakeConnector::default();
        let theme = Theme::blank("Start", Size::new(480, 1920), Orientation::ReversePortrait);
        let studio = Studio::new(
            Box::new(FakeSensors::demo()),
            Box::new(SkiaRenderer::with_fonts(Vec::new(), SystemFonts::Skip)),
            Language::English,
            theme,
        );
        let backend = Backend {
            engine: None,
            bus: Arc::new(FakeBus::turing_88()),
            connector: Arc::new(connector.clone()),
            hid: Arc::new(FakeHid::answering(0x88)),
            store: Arc::new(FsThemeStore),
            library: ThemeLibrary::new(root.join("themes"), vec![]),
            settings: SettingsFile::new(root.join("settings.json")),
            system_language: Language::English,
            make_sensors: Arc::new(|_| Box::new(FakeSensors::demo())),

            fonts: SkiaRenderer::with_fonts(vec![TEST_FONT.to_vec()], SystemFonts::Skip)
                .font_catalog(),
            studio: Session::new(studio),
            storage: StorageState::new(
                Box::new(crate::storage::tests::FakeMedia::ready()),
                crate::manager::Copies::in_memory(bezel_media::archive::MemoryArchive::new()),
                root.join("scratch"),
            ),
            thumbnails: crate::thumbnails::tests::thumbnails(root.join("thumbnails")),
        };
        Fixture {
            backend,
            connector,
            root,
            _temporary: temporary,
        }
    }

    #[test]
    fn the_first_run_opens_the_bundled_theme_for_the_screen() {
        let mut f = fixture("first-run");
        let bundled = f.root.join("bundled");
        for (name, size, orientation) in [
            ("Wide", Size::new(480, 1920), Orientation::Landscape),
            ("Tall", Size::new(480, 1920), Orientation::Portrait),
            ("Small", Size::new(320, 480), Orientation::Landscape),
        ] {
            let at = ThemeLocation(
                bundled
                    .join(format!("{name}.bezeltheme"))
                    .display()
                    .to_string(),
            );
            FsThemeStore
                .save(
                    &at,
                    &Theme::blank(name, size, orientation),
                    &Default::default(),
                )
                .unwrap();
        }
        f.backend.library = ThemeLibrary::new(f.root.join("themes"), vec![bundled]);
        f.backend.restore_theme();
        let session = f.backend.session();
        assert_eq!(session.theme.name, "Wide", "the 8.8\" starts horizontal");
        assert!(session.location.unwrap().ends_with("Wide.bezeltheme"));
        // Saving it writes a copy in the user folder, never over the bundled file.
        let saved = f.backend.save(&session.theme, None).unwrap();
        assert!(
            Path::new(&saved.location).starts_with(f.root.join("themes")),
            "{}",
            saved.location
        );
    }

    #[test]
    fn profile_overview_names_each_displays_selected_theme_independently() {
        use bezel_runtime::profiles::{Config, Profile};
        let f = fixture("profile-display-names");
        std::fs::create_dir_all(&f.root).unwrap();
        let mut paths = std::collections::BTreeMap::new();
        for (key, name) in [(KEY, "Wide dashboard"), ("COM9", "Round clock")] {
            let path = f.root.join(format!(
                "{}.bezeltheme",
                if key == KEY { "wide" } else { "round" }
            ));
            let theme = Theme::blank(name, Size::new(480, 1920), Orientation::Portrait);
            f.backend
                .store
                .save(
                    &ThemeLocation(path.display().to_string()),
                    &theme,
                    &Default::default(),
                )
                .unwrap();
            paths.insert(key.to_owned(), path.display().to_string());
        }
        f.backend.settings.update(|s| {
            s.screen_themes = paths.clone();
            s.selected_screen = Some("COM9".into());
            s.last_theme = Some(paths[KEY].clone()); // Must not override COM9's own choice.
        });
        let overview = f.backend.profiles().unwrap();
        assert_eq!(
            overview
                .displays
                .iter()
                .find(|d| d.key == KEY)
                .unwrap()
                .theme_name
                .as_deref(),
            Some("Wide dashboard")
        );
        assert_eq!(
            overview
                .displays
                .iter()
                .find(|d| d.key == "COM9")
                .unwrap()
                .theme_name
                .as_deref(),
            Some("Round clock")
        );
        f.backend
            .save_profiles(Config {
                profiles: vec![Profile {
                    id: "gaming".into(),
                    name: "Gaming".into(),
                    themes: paths.clone(),
                    rules: vec![],
                    combine: Default::default(),
                }],
                ..Config::default()
            })
            .unwrap();
        assert_eq!(
            f.backend.profiles().unwrap().config.profiles[0].themes,
            paths
        );
    }

    #[test]
    fn a_transition_preview_is_checked_and_never_saved() {
        use bezel_runtime::profiles::TransitionSettings;
        let f = fixture("transition-preview");
        let wipe = TransitionSettings {
            effect: "wipe".into(),
            direction: "up".into(),
            duration_ms: 900,
        };
        // No live screen in the fixture: nothing to cover.
        assert_eq!(
            f.backend
                .preview_transition(wipe, Default::default())
                .unwrap(),
            0
        );
        assert!(
            f.backend
                .preview_transition(
                    TransitionSettings {
                        duration_ms: 20,
                        ..Default::default()
                    },
                    Default::default()
                )
                .is_err()
        );
        assert_eq!(
            f.backend.profiles().unwrap().config.transition,
            TransitionSettings::default()
        );
    }

    #[test]
    fn the_language_follows_the_system_until_one_is_chosen() {
        let f = fixture("language");
        assert_eq!(f.backend.language(), Language::English);
        let prefs = f.backend.preferences();
        assert_eq!((prefs.language, prefs.system_language), (None, "en"));
        assert_eq!(f.backend.texts().untitled, "Untitled");

        let chosen = f.backend.set_language(Some("pt-BR")).unwrap();
        assert_eq!(chosen, Language::PortugueseBr);
        assert_eq!(f.backend.preferences().language, Some("pt-BR"));
        assert_eq!(f.backend.texts().untitled, "Sem título");
        assert_eq!(f.backend.studio().language(), Language::English);
        let json = std::fs::read_to_string(f.backend.settings.path()).unwrap();
        assert!(json.contains("\"language\": \"pt-BR\""), "{json}");

        let error = f.backend.set_language(Some("klingon")).unwrap_err();
        assert_eq!(error.code(), "unknownLanguage");
        assert_eq!(f.backend.language(), Language::PortugueseBr, "unchanged");

        assert_eq!(f.backend.set_language(None).unwrap(), Language::English);
        assert_eq!(f.backend.preferences().language, None);
        assert_eq!(f.backend.studio().language(), Language::English);
    }

    #[test]
    fn the_gallery_filter_is_checked_and_remembered() {
        let f = fixture("theme-filter");
        let prefs = f.backend.preferences();
        assert_eq!(
            prefs.theme_filter,
            ThemeFilterDto {
                scope: None,
                axis: "all"
            }
        );
        f.backend
            .set_theme_filter(Some("all"), "horizontal")
            .unwrap();
        let filter = f.backend.preferences().theme_filter;
        assert_eq!((filter.scope, filter.axis), (Some("all"), "horizontal"));
        for (scope, axis) in [(Some("mine"), "all"), (None, "diagonal")] {
            let error = f.backend.set_theme_filter(scope, axis).unwrap_err();
            assert_eq!(error.code(), "invalidInput", "{scope:?} {axis}");
        }
        assert_eq!(f.backend.preferences().theme_filter, filter, "unchanged");
        f.backend.set_theme_filter(None, "vertical").unwrap();
        let filter = f.backend.preferences().theme_filter;
        assert_eq!((filter.scope, filter.axis), (None, "vertical"));
    }

    #[test]
    fn library_themes_get_thumbnails_and_name_their_screens() {
        let f = fixture("thumbnails");
        let theme = Theme::blank("Mine", Size::new(480, 1920), Orientation::Landscape);
        let saved = f.backend.save(&ThemeDto::from(&theme), None).unwrap();
        let listed = f.backend.themes();
        let entry = listed
            .iter()
            .find(|e| e.location == saved.location)
            .unwrap();
        assert_eq!(
            entry.models,
            vec!["turing-8.8", "turing-usb-8.8", "turing-usb-9.2"]
        );
        assert_eq!(entry.diagonal_hundredths, None);
        let first = f.backend.thumbnail(&saved.location, TIME).unwrap();
        assert!(first.unwrap().starts_with("data:image/png;base64,"));
        let kept = || std::fs::read_dir(f.backend.thumbnails.dir()).map_or(0, Iterator::count);
        assert_eq!(kept(), 1);
        // Saving it again through the studio forgets the old thumbnail, and
        // the list says it changed.
        let mut renamed = theme.clone();
        renamed.name = "Mine again".into();
        f.backend.save(&ThemeDto::from(&renamed), None).unwrap();
        assert_eq!(kept(), 0, "forgotten on save");
        let again = f.backend.themes();
        let after = again.iter().find(|e| e.location == saved.location).unwrap();
        assert_ne!(after.revision, entry.revision);
        assert!(
            f.backend
                .thumbnail(&saved.location, TIME)
                .unwrap()
                .is_some()
        );
        assert_eq!(kept(), 1);
        // Only the library's themes.
        let error = f.backend.thumbnail("/etc/passwd", TIME).unwrap_err();
        assert_eq!(error.code(), "notInLibrary");
        // A broken file of the library: no thumbnail, no error.
        let broken = f.root.join("themes").join("broken.bezeltheme");
        std::fs::write(&broken, b"not a zip").unwrap();
        let none = f.backend.thumbnail(&broken.display().to_string(), TIME);
        assert_eq!(none.unwrap(), None);
    }

    #[test]
    fn a_panel_in_desktop_mode_switches_back_only_when_confirmed() {
        let mut f = fixture("desktop-mode");
        let hid = FakeHid::answering(0x88);
        f.backend.hid = Arc::new(hid.clone());
        f.backend.bus = Arc::new(FakeBus::turing_88().and(FakeBus::desktop_mode()));
        let found = f.backend.devices().unwrap();
        assert_eq!((found.screens.len(), found.desktop_mode.len()), (1, 1));
        let key = found.desktop_mode[0].key.clone();

        let refused = f.backend.leave_desktop_mode(&key, Confirm::No).unwrap_err();
        assert_eq!(refused.code(), "notConfirmed");
        assert!(hid.calls().is_empty(), "nothing without Confirm::Yes");

        let done = f.backend.leave_desktop_mode(&key, Confirm::Yes).unwrap();
        assert_eq!(done.model, Some("Turing 8.8\" V1.x (USB)"));
        assert_eq!(hid.calls().len(), 2, "the model query, then the switch");
        let gone = f
            .backend
            .leave_desktop_mode("hid:/dev/hidraw9", Confirm::Yes);
        assert_eq!(gone.unwrap_err().code(), "screenNotFound");

        // Review W9 (iteration 1), D-2026-10-03-power-off-standby-3: in the
        // final state of a shutdown no panel is switched either (`busy`).
        let calls = hid.calls().len();
        f.backend.enter_final_state();
        let busy = f.backend.leave_desktop_mode(&key, Confirm::Yes);
        assert_eq!(busy.unwrap_err().code(), "busy");
        assert_eq!(hid.calls().len(), calls, "nothing sent in the final state");
        f.backend.leave_final_state();
        f.backend.leave_desktop_mode(&key, Confirm::Yes).unwrap();
    }

    /// The guide's pages open at their fixed addresses, pages of this
    /// repository; nothing else does.
    #[test]
    fn the_guide_opens_only_at_its_fixed_addresses() {
        let site = "https://github.com/slipalison/bezel/blob/main/";
        let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
        for (language, page) in [
            ("en", "docs/user/ffmpeg.md"),
            ("pt-BR", "docs/user/pt-BR/ffmpeg.md"),
        ] {
            assert_eq!(
                guide_url("ffmpeg", language).unwrap(),
                format!("{site}{page}")
            );
            assert!(repo.join(page).is_file(), "{page}");
        }
        for (page, language) in [
            ("ffmpeg", "fr"),
            ("ffmpeg", "pt-br"),
            ("../../etc/passwd", "en"),
            ("https://example.com", "en"),
            ("", "pt-BR"),
        ] {
            let error = guide_url(page, language).unwrap_err();
            assert_eq!(error.code(), "invalidInput", "{page} {language}");
            assert!(error.to_string().contains(page), "{error}");
        }
    }

    /// D-2026-10-01-gif-sticker-search-3: KLIPY's Partner Panel is the one
    /// link the UI may open besides the guide's pages (the GIF guide among
    /// them); the app's code names no other address and the window gets no
    /// permission to open one itself.
    #[test]
    fn partner_panel_is_the_only_new_link() {
        assert_eq!(
            LINKS,
            [("klipyPartnerPanel", "https://partner.klipy.com")],
            "one link"
        );
        assert_eq!(
            link_url("klipyPartnerPanel").unwrap(),
            "https://partner.klipy.com"
        );
        for link in [
            "",
            "klipy",
            "KlipyPartnerPanel",
            "https://partner.klipy.com",
            "https://example.com",
            "../ffmpeg",
        ] {
            let error = link_url(link).unwrap_err();
            assert_eq!(error.code(), "invalidInput", "{link}");
            assert!(error.to_string().contains(link), "{error}");
        }
        assert_eq!(GUIDE_PAGES, ["ffmpeg", "gifs-and-stickers"]);
        let site = "https://github.com/slipalison/bezel/blob/main/";
        let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
        for (language, page) in [
            ("en", "docs/user/gifs-and-stickers.md"),
            ("pt-BR", "docs/user/pt-BR/gifs-and-stickers.md"),
        ] {
            assert_eq!(
                guide_url("gifs-and-stickers", language).unwrap(),
                format!("{site}{page}")
            );
            assert!(repo.join(page).is_file(), "{page}");
        }
        // Every address the app's own code holds: the guide's site and the
        // panel.
        let code = [
            include_str!("backend.rs"),
            include_str!("commands.rs"),
            include_str!("gifs.rs"),
            include_str!("gifs/key.rs"),
            include_str!("lib.rs"),
        ];
        let mut addresses: Vec<&str> = code
            .iter()
            .map(|source| {
                source
                    .split("#[cfg(test)]\nmod tests {")
                    .next()
                    .unwrap()
                    .split("#[cfg(test)]\r\nmod tests {")
                    .next()
                    .unwrap()
            })
            .flat_map(|source| {
                source
                    .match_indices("https://")
                    .map(|(at, _)| &source[at..])
            })
            .map(|from| from.split(['"', '{', ')', ' ']).next().unwrap())
            .collect();
        addresses.sort_unstable();
        addresses.dedup();
        assert_eq!(
            addresses,
            [
                "https://github.com/slipalison/bezel/blob/main/docs/user/",
                "https://partner.klipy.com",
            ]
        );
        let capability = include_str!("../capabilities/default.json");
        assert!(!capability.contains("\"opener:"), "no opener permission");
    }

    #[test]
    fn render_preview_returns_the_canvas_size() {
        let f = fixture("render");
        let theme = f.backend.session().theme;
        let bytes = f
            .backend
            .render(&theme, TIME, Instant::now(), Motion::Allowed)
            .unwrap();
        assert_eq!(&bytes[..8], &[224, 1, 0, 0, 128, 7, 0, 0]);
        assert_eq!(&bytes[8..12], &STILL.to_le_bytes(), "nothing animates");
        let json = serde_json::to_value(f.backend.session()).unwrap();
        assert_eq!(json["minRefreshSeconds"], MIN_REFRESH_SECONDS);
        assert_eq!(bytes.len(), 12 + 480 * 1920 * 4);
        let mut bad = theme.clone();
        bad.orientation = "sideways".into();
        assert!(
            f.backend
                .render(&bad, TIME, Instant::now(), Motion::Allowed)
                .is_err()
        );
    }

    /// The element preview's pixels, from [`frame_bytes`].
    fn element_pixels(f: &Fixture, theme: &Theme, id: u32, max: Option<u32>) -> UiResult<Frame> {
        let bytes = f.backend.render_element(
            &ThemeDto::from(theme),
            id,
            max,
            TIME,
            Instant::now(),
            Motion::Reduced,
        )?;
        let width = u32::from_le_bytes(bytes[0..4].try_into().unwrap());
        let height = u32::from_le_bytes(bytes[4..8].try_into().unwrap());
        assert_eq!(&bytes[8..12], &STILL.to_le_bytes(), "nothing moves");
        Ok(Frame::from_rgba(Size::new(width, height), bytes[12..].to_vec()).unwrap())
    }

    /// How many pixels of `frame` are `color`.
    fn count(frame: &Frame, color: Rgba) -> usize {
        frame
            .as_rgba()
            .as_chunks::<4>()
            .0
            .iter()
            .filter(|p| **p == [color.r, color.g, color.b, color.a])
            .count()
    }

    fn landscape(elements: Vec<bezel_core::domain::theme::Element>) -> Theme {
        let mut theme = Theme::blank("Parts", Size::new(480, 1920), Orientation::Landscape);
        theme.background = Background::Color(Rgba::opaque(9, 9, 9));
        theme.elements = elements;
        theme
    }

    #[test]
    fn inspector_try_renders_full_scale_and_returns_without_changing_the_theme() {
        use bezel_core::domain::theme::{Binding, BoxF, Direction, ElementKind, Paint};
        let f = fixture("inspector-trial");
        f.backend.studio().sample().unwrap();
        let fill = Rgba::opaque(0, 200, 0);
        let mut bar = crate::element_preview::tests::shape(1, BoxF::new(0., 0., 200., 10.), fill);
        bar.kind = ElementKind::Bar {
            binding: Binding {
                key: SensorKey::new("cpu.temperature").unwrap(),
                min: 0.,
                max: 100.,
            },
            direction: Direction::LeftToRight,
            fill: Paint::solid(fill),
            track: None,
            radius: 0.,
            segments: None,
        };
        let theme = landscape(vec![bar]);
        let dto = ThemeDto::from(&theme);
        for (trial_ms, motion, expected, next) in [
            (None, Motion::Allowed, 920, STILL),
            (Some(1500), Motion::Allowed, 2000, 33),
            (Some(3000), Motion::Allowed, 920, STILL),
            (Some(1500), Motion::Reduced, 920, STILL),
        ] {
            let bytes = f
                .backend
                .render_element_with(&dto, 1, (None, trial_ms), TIME, Instant::now(), motion)
                .unwrap();
            assert_eq!(u32::from_le_bytes(bytes[8..12].try_into().unwrap()), next);
            let frame = Frame::from_rgba(Size::new(200, 10), bytes[12..].to_vec()).unwrap();
            assert_eq!(count(&frame, fill), expected);
            assert_eq!(*f.backend.studio().theme(), theme);
        }
    }

    #[test]
    fn an_element_preview_draws_only_that_element_on_transparency() {
        use crate::element_preview::tests::shape;
        use bezel_core::domain::theme::{BoxF, ElementKind, ShapeKind};
        let (red, blue) = (Rgba::opaque(255, 0, 0), Rgba::opaque(0, 0, 255));
        let mut ring = shape(1, BoxF::new(100.0, 100.0, 50.0, 40.0), red);
        if let ElementKind::Shape { shape, .. } = &mut ring.kind {
            *shape = ShapeKind::Ellipse;
        }
        let over = shape(2, BoxF::new(90.0, 90.0, 200.0, 200.0), blue);
        let f = fixture("element-alone");
        let theme = landscape(vec![ring.clone(), over]);
        let frame = element_pixels(&f, &theme, 1, None).unwrap();
        assert_eq!(frame.size(), Size::new(50, 40), "just the element's box");
        assert_eq!(frame.pixel(25, 20), Some(red), "its own color");
        assert_eq!(
            frame.pixel(0, 0).map(|p| p.a),
            Some(0),
            "transparent around it"
        );
        assert_eq!(count(&frame, blue), 0, "the element above it is not drawn");
        assert!(
            !frame
                .as_rgba()
                .as_chunks::<4>()
                .0
                .iter()
                .any(|p| p[..3] == [9, 9, 9]),
            "nor the theme's background"
        );
        // Its properties apply: another color, another size.
        let mut green = ring.clone();
        if let ElementKind::Shape { fill, .. } = &mut green.kind {
            *fill = Some(bezel_core::domain::theme::Paint::solid(Rgba::opaque(
                0, 255, 0,
            )));
        }
        green.frame.width = 80.0;
        let changed = element_pixels(&f, &landscape(vec![green]), 1, None).unwrap();
        assert_eq!(changed.size(), Size::new(80, 40));
        assert_eq!(changed.pixel(40, 20), Some(Rgba::opaque(0, 255, 0)));
        assert_eq!(count(&changed, red), 0);
        // At most `max` pixels on its longest side, never enlarged.
        let small = element_pixels(&f, &theme, 1, Some(20)).unwrap();
        assert_eq!(small.size(), Size::new(20, 16));
        assert_eq!(small.pixel(10, 8), Some(red));
        let err = element_pixels(&f, &theme, 7, None).unwrap_err();
        assert_eq!(err.code(), "invalidInput");
    }

    #[test]
    fn an_element_preview_shows_the_value_its_binding_reads() {
        use bezel_core::domain::theme::{
            Binding, BoxF, Direction, Element, ElementId, ElementKind, Paint,
        };
        let f = fixture("element-value");
        f.backend.studio().sample().unwrap();
        let fill = Rgba::opaque(0, 200, 0);
        let bar = |max: f64| Element {
            color_by_value: None,
            visualization: None,
            value_transition: None,
            sensor_motion: None,
            rotation: 0.0,
            is_group: false,
            group_parent: None,
            card: None,
            card_member: None,
            id: ElementId(1),
            name: "CPU".into(),
            frame: BoxF::new(0.0, 0.0, 200.0, 10.0),
            opacity: 1.0,
            visible: true,
            locked: false,
            kind: ElementKind::Bar {
                binding: Binding {
                    key: SensorKey::new(bezel_core::domain::sensor::keys::CPU_TEMPERATURE).unwrap(),
                    min: 0.0,
                    max,
                },
                direction: Direction::LeftToRight,
                fill: Paint::solid(fill),
                track: None,
                radius: 0.0,
                segments: None,
            },
        };
        let full = count(
            &element_pixels(&f, &landscape(vec![bar(100.0)]), 1, None).unwrap(),
            fill,
        );
        let tenth = count(
            &element_pixels(&f, &landscape(vec![bar(1000.0)]), 1, None).unwrap(),
            fill,
        );
        assert_eq!(full, 92 * 10, "46 °C of 100 fills 46 % of the bar");
        assert!(tenth < full, "a wider range fills less: {tenth} < {full}");
    }

    #[test]
    fn a_canvas_trial_sweeps_the_whole_scene_and_holds_one_bar_without_touching_it() {
        use bezel_core::domain::theme::{
            Binding, BoxF, Direction, Element, ElementId, ElementKind, Paint,
        };
        let f = fixture("canvas-trial");
        f.backend.studio().sample().unwrap();
        let fill = Rgba::opaque(0, 200, 0);
        let bar = Element {
            color_by_value: None,
            visualization: None,
            value_transition: None,
            sensor_motion: None,
            rotation: 0.0,
            is_group: false,
            group_parent: None,
            card: None,
            card_member: None,
            id: ElementId(1),
            name: "CPU".into(),
            frame: BoxF::new(0.0, 0.0, 200.0, 10.0),
            opacity: 1.0,
            visible: true,
            locked: false,
            kind: ElementKind::Bar {
                binding: Binding {
                    key: SensorKey::new(bezel_core::domain::sensor::keys::CPU_TEMPERATURE).unwrap(),
                    min: 0.0,
                    max: 100.0,
                },
                direction: Direction::LeftToRight,
                fill: Paint::solid(fill),
                track: None,
                radius: 0.0,
                segments: None,
            },
        };
        // A small canvas keeps the four renders light beside timing-sensitive tests.
        let mut small = landscape(vec![bar]);
        small.canvas = Size::new(240, 60);
        let theme = ThemeDto::from(&small);
        let trial = |ms, hold| {
            let bytes = f
                .backend
                .render_trial(&theme, TIME, Motion::Reduced, Some(2320), (ms, hold))
                .unwrap();
            let width = u32::from_le_bytes(bytes[0..4].try_into().unwrap());
            let height = u32::from_le_bytes(bytes[4..8].try_into().unwrap());
            assert_eq!(&bytes[8..12], &33u32.to_le_bytes(), "the next trial frame");
            Frame::from_rgba(Size::new(width, height), bytes[12..].to_vec()).unwrap()
        };
        let start = trial(0, None);
        assert_eq!(start.size(), Size::new(240, 60), "the whole canvas");
        assert_eq!(count(&start, fill), 0, "the sweep starts at the minimum");
        assert_eq!(
            count(&trial(1500, None), fill),
            200 * 10,
            "and reaches the maximum"
        );
        let held = trial(
            0,
            Some(bezel_runtime::wire::TrialHold {
                element: 1,
                fraction: 0.25,
            }),
        );
        assert_eq!(count(&held, fill), 50 * 10, "a held bar shows its share");
        // The live preview still shows the reading (46 °C of 100).
        let live = f
            .backend
            .render(&theme, TIME, Instant::now(), Motion::Reduced)
            .unwrap();
        let width = u32::from_le_bytes(live[0..4].try_into().unwrap());
        let height = u32::from_le_bytes(live[4..8].try_into().unwrap());
        let live = Frame::from_rgba(Size::new(width, height), live[12..].to_vec()).unwrap();
        assert_eq!(count(&live, fill), 92 * 10);
    }

    #[test]
    fn a_card_preview_shows_its_base_and_active_face_only() {
        use crate::element_preview::tests::shape;
        use bezel_core::domain::theme::{BoxF, Card, CardMember, ElementId};
        let (base, a, b) = (
            Rgba::opaque(40, 40, 40),
            Rgba::opaque(255, 0, 0),
            Rgba::opaque(0, 255, 0),
        );
        let mut card = shape(1, BoxF::new(10.0, 10.0, 100.0, 100.0), base);
        card.card = Some(Card {
            triggers: Vec::new(),
            faces: vec!["A".into(), "B".into()],
            active_face: 1,
            rotation_seconds: Some(5),
            transition: None,
        });
        let member = |id, face, color| {
            let mut e = shape(id, BoxF::new(30.0, 30.0, 20.0, 20.0), color);
            e.card_member = Some(CardMember {
                parent: ElementId(1),
                face: Some(face),
            });
            e
        };
        let f = fixture("element-card");
        let theme = landscape(vec![card, member(2, 0, a), member(3, 1, b)]);
        let frame = element_pixels(&f, &theme, 1, None).unwrap();
        assert_eq!(frame.size(), Size::new(100, 100));
        assert_eq!(frame.pixel(30, 30), Some(b), "the active face");
        assert_eq!(count(&frame, a), 0, "not the other face");
        assert_eq!(frame.pixel(5, 5), Some(base), "over the card's base");
    }

    /// Frames left for Light, in memory; `broken` refuses to keep them.
    #[derive(Default)]
    struct LeftFrames {
        left: Mutex<Vec<bezel_core::domain::handoff::HandedFrame>>,
        broken: bool,
    }

    impl HandoffStore for LeftFrames {
        fn save(
            &self,
            handed: &bezel_core::domain::handoff::HandedFrame,
        ) -> bezel_core::Result<()> {
            if self.broken {
                return Err(bezel_core::BezelError::Transport("disk full".into()));
            }
            self.left.lock().unwrap().push(handed.clone());
            Ok(())
        }
        fn take(
            &self,
            _: &bezel_core::domain::discovery::DeviceAddress,
        ) -> bezel_core::Result<Option<bezel_core::domain::handoff::HandedFrame>> {
            Ok(None)
        }
    }

    #[test]
    fn handing_over_leaves_the_last_frame_and_sends_nothing_more() {
        let f = fixture("hand-over");
        let store = LeftFrames::default();
        assert_eq!(
            f.backend.hand_over_live(&store, Duration::from_secs(9)),
            0,
            "nothing live"
        );
        f.backend.set_live(true, Some(KEY), TIME).unwrap();
        let shown = f.connector.log().frames;
        let last = shown.last().cloned().unwrap();
        assert_eq!(f.backend.hand_over_live(&store, Duration::from_secs(9)), 1);
        let left = store.left.lock().unwrap().clone();
        assert_eq!(left.len(), 1);
        assert_eq!(left[0].key.address.0, KEY);
        assert_eq!(left[0].key.model, "turing-8.8");
        assert_eq!(left[0].saved_at, Duration::from_secs(9));
        assert_eq!(left[0].baseline.frame, last, "the frame the screen shows");
        assert_eq!(left[0].baseline.orientation, Orientation::ReversePortrait);
        assert!(!f.backend.studio().any_live());
        let seen = f.connector.log();
        assert_eq!(
            seen.releases, 0,
            "no END_UPDATE_BITMAP: the stream goes on in Light"
        );
        assert_eq!(
            seen.frames.len(),
            shown.len(),
            "no frame after the one left"
        );
        assert_eq!(
            f.backend.settings.load().live_screen.as_deref(),
            Some(KEY),
            "Light reads the live screens from the settings"
        );
        // A store that cannot keep it only means a full first frame.
        f.backend.set_live(true, Some(KEY), TIME).unwrap();
        let broken = LeftFrames {
            broken: true,
            ..LeftFrames::default()
        };
        assert_eq!(f.backend.hand_over_live(&broken, Duration::from_secs(9)), 0);
        assert!(!f.backend.studio().any_live());
    }

    #[test]
    fn samples_tell_the_live_frame_rate_and_the_activity() {
        let f = fixture("fps");
        let quiet = f.backend.sample();
        assert_eq!(
            (quiet.live_fps, &quiet.activity),
            (None, &None),
            "nothing live, nothing read"
        );
        let json = serde_json::to_value(&quiet).unwrap();
        assert!(json["liveFps"].is_null() && json["activity"].is_null());
        f.backend.set_live(true, Some(KEY), TIME).unwrap();
        let theme = f.backend.session().theme;
        for _ in 0..3 {
            f.backend.push(&theme, TIME).unwrap();
        }
        let fps = f.backend.sample().live_fps;
        assert!(
            fps.is_some_and(|fps| fps > 0.0),
            "frames back to back: {fps:?}"
        );
        f.backend.set_live(false, None, TIME).unwrap();
        assert_eq!(f.backend.sample().live_fps, None, "nothing live");
    }

    #[test]
    fn duplicating_copies_into_the_user_folder_and_never_writes_a_bundled_theme() {
        let mut f = fixture("duplicate");
        let bundled = f.root.join("bundled");
        let shipped = ThemeLocation(bundled.join("Aurora.bezeltheme").display().to_string());
        let mut theme = Theme::blank("Aurora", Size::new(480, 1920), Orientation::Landscape);
        theme.refresh_seconds = 2.0;
        FsThemeStore
            .save(&shipped, &theme, &Default::default())
            .unwrap();
        f.backend.library = ThemeLibrary::new(f.root.join("themes"), vec![bundled]);
        let before = (
            crate::library::revision(&shipped),
            std::fs::read(&shipped.0).unwrap(),
        );
        let copy = f
            .backend
            .duplicate_theme(&shipped.0, " Copia di Aurora ")
            .unwrap();
        assert_eq!(copy.name, "Copia di Aurora");
        assert!(!copy.bundled);
        assert!(copy.modified_ms.is_some());
        assert!(
            Path::new(&copy.location).starts_with(f.root.join("themes")),
            "{}",
            copy.location
        );
        assert_ne!(copy.location, shipped.0);
        let after = (
            crate::library::revision(&shipped),
            std::fs::read(&shipped.0).unwrap(),
        );
        assert_eq!(before, after, "the bundled theme is untouched");
        let opened = f.backend.open(&copy.location).unwrap();
        assert_eq!(
            (opened.name.as_str(), opened.refresh_seconds),
            ("Copia di Aurora", 2.0)
        );
        let again = f
            .backend
            .duplicate_theme(&shipped.0, "Copia di Aurora")
            .unwrap();
        assert_ne!(again.location, copy.location, "a free name each time");
        assert_eq!(f.backend.themes().len(), 3);
        let json = serde_json::to_value(&copy).unwrap();
        assert!(json["modifiedMs"].is_u64());
        let elsewhere = f.root.join("Desktop").join("x.bezeltheme");
        let refused = f
            .backend
            .duplicate_theme(&elsewhere.display().to_string(), "X")
            .unwrap_err();
        assert_eq!(refused.code(), "notInLibrary");
        let empty = f.backend.duplicate_theme(&shipped.0, "  ").unwrap_err();
        assert_eq!(empty.code(), "invalidInput");
        let missing = bundled_missing(&f);
        assert!(f.backend.duplicate_theme(&missing, "M").is_err());
    }

    #[test]
    fn a_theme_trial_checks_locations_and_profile_ids_without_writing_config() {
        let f = fixture("gallery-trial");
        let config = f.backend.profiles().unwrap().config;
        assert_eq!(f.backend.try_themes(Default::default(), None).unwrap(), 0);
        assert!(
            f.backend
                .try_themes(Default::default(), Some("missing".into()))
                .is_err()
        );
        let unknown =
            std::collections::BTreeMap::from([("COM9".into(), "C:/outside.bezeltheme".into())]);
        assert_eq!(
            f.backend.try_themes(unknown, None).unwrap_err().code(),
            "notInLibrary"
        );
        assert_eq!(f.backend.profiles().unwrap().config, config);
    }

    #[test]
    fn gallery_delete_only_removes_unused_personal_themes_and_metadata_is_real() {
        let f = fixture("gallery-delete");
        let location = f.backend.library.new_location("Personal");
        let theme = f.backend.studio().theme().clone();
        FsThemeStore
            .save(&location, &theme, &Default::default())
            .unwrap();
        let entry = f
            .backend
            .themes()
            .into_iter()
            .find(|e| e.location == location.0)
            .unwrap();
        assert_eq!(entry.objects, theme.elements.len());
        assert_eq!(
            entry.sensors,
            theme
                .sensor_keys()
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
        );
        let outside = f.root.join("outside.bezeltheme");
        FsThemeStore
            .save(
                &ThemeLocation(outside.display().to_string()),
                &theme,
                &Default::default(),
            )
            .unwrap();
        assert!(
            f.backend
                .delete_theme(&outside.display().to_string())
                .is_err()
        );
        assert!(outside.exists());
        assert!(
            f.backend
                .delete_theme(&f.backend.library.user_dir().display().to_string())
                .is_err()
        );
        f.backend.open(&location.0).unwrap();
        assert!(
            f.backend.delete_theme(&location.0).is_err(),
            "the open document is protected"
        );
        f.backend
            .studio()
            .set_theme(Theme::blank("Unsaved", theme.canvas, theme.orientation));
        // Opening another saved document changes its location; set_theme alone deliberately does not.
        let another = f.backend.library.new_location("Another");
        FsThemeStore
            .save(&another, &theme, &Default::default())
            .unwrap();
        f.backend.open(&another.0).unwrap();
        f.backend.delete_theme(&location.0).unwrap();
        assert!(!Path::new(&location.0).exists());
        assert!(Path::new(&another.0).exists());
    }

    #[test]
    fn exporting_the_selected_saved_theme_keeps_all_assets_and_the_open_document() {
        use bezel_core::domain::theme::{AssetRef, Fit};
        let f = fixture("theme-export");
        let source = ThemeLocation(
            f.backend
                .library
                .user_dir()
                .join("Selected")
                .display()
                .to_string(),
        );
        let mut theme = f.backend.studio().theme().clone();
        theme.name = "Selected saved theme".into();
        let asset = AssetRef("assets/cover.png".into());
        theme.background = Background::Image {
            asset: asset.clone(),
            fit: Fit::Contain,
        };
        let assets = std::collections::BTreeMap::from([(asset.clone(), vec![1, 2, 3, 4])]);
        FsThemeStore.save(&source, &theme, &assets).unwrap();
        let before = std::fs::read(Path::new(&source.0).join("theme.json")).unwrap();
        let mut unsaved = f.backend.studio().theme().clone();
        unsaved.name = "Unsaved Studio document".into();
        f.backend.studio().set_theme(unsaved.clone());
        let target = ThemeLocation(
            f.root
                .join("exports")
                .join("selected.bezeltheme")
                .display()
                .to_string(),
        );
        assert_eq!(
            f.backend
                .export_theme(&source.0, &target)
                .unwrap_err()
                .code(),
            "notPicked"
        );
        assert!(!Path::new(&target.0).exists());
        f.backend.library.grant(&target);
        assert_eq!(
            f.backend.export_theme(&source.0, &target).unwrap(),
            target.0
        );
        let (exported, exported_assets) = FsThemeStore.load(&target).unwrap();
        assert_eq!(exported, theme);
        assert_eq!(exported_assets, assets);
        assert_eq!(*f.backend.studio().theme(), unsaved);
        assert_eq!(f.backend.studio().location(), None);
        assert_eq!(
            std::fs::read(Path::new(&source.0).join("theme.json")).unwrap(),
            before
        );
        let unknown = f.root.join("outside.bezeltheme");
        assert_eq!(
            f.backend
                .export_theme(&unknown.display().to_string(), &target)
                .unwrap_err()
                .code(),
            "notInLibrary"
        );
        std::fs::remove_file(Path::new(&source.0).join(&asset.0)).unwrap();
        let missing = ThemeLocation(f.root.join("missing.bezeltheme").display().to_string());
        f.backend.library.grant(&missing);
        assert!(
            f.backend.export_theme(&source.0, &missing).is_err(),
            "no incomplete export if an asset is missing"
        );
        assert!(!Path::new(&missing.0).exists());
    }

    /// A location inside the bundled folder where no theme is.
    fn bundled_missing(f: &Fixture) -> String {
        f.root
            .join("bundled")
            .join("gone.bezeltheme")
            .display()
            .to_string()
    }

    #[test]
    fn live_mode_shows_edits_and_is_remembered() {
        let f = fixture("live");
        assert_eq!(f.backend.devices().unwrap().screens.len(), 1);
        f.backend.set_live(true, Some(KEY), TIME).unwrap();
        assert_eq!(f.backend.sample().live.as_deref(), Some(KEY));
        let theme = f.backend.session().theme;
        f.backend.push(&theme, TIME).unwrap();
        let now = Instant::now();
        let due = f.backend.tick_and_wait(TIME, now);
        assert!(due >= now + Duration::from_secs_f32(MIN_REFRESH_SECONDS));
        assert_eq!(f.connector.log().frames.len(), 3);
        f.backend.set_brightness(KEY, 40).unwrap();
        assert!(f.backend.set_brightness(KEY, 101).is_err());
        assert_eq!(f.backend.settings.load().live_screen.as_deref(), Some(KEY));

        f.backend.release(KEY).unwrap();
        assert_eq!(f.connector.log().releases, 1);
        assert_eq!(f.backend.sample().live, None);
        assert_eq!(f.backend.settings.load().live_screen, None);
        f.backend.set_live(false, None, TIME).unwrap();
        assert!(f.backend.set_live(true, None, TIME).is_err());
        assert!(f.backend.set_live(true, Some("COM9"), TIME).is_err());
    }

    #[test]
    fn a_hung_screen_restarts_and_shows_the_theme_live_again() {
        // D-2026-09-30-release-polish-13: live mode stops (the port closes),
        // the MCU restarts the screen, live mode comes back on it.
        let f = fixture("restart-live");
        f.backend.set_live(true, Some(KEY), TIME).unwrap();
        let frames = f.connector.log().frames.len();
        let done = f.backend.restart_screen(KEY, TIME).unwrap();
        assert_eq!(
            done,
            RestartedDto {
                key: KEY.into(),
                live: true
            }
        );
        assert_eq!(f.connector.log().restarts, [KEY]);
        assert_eq!(f.backend.sample().live.as_deref(), Some(KEY));
        assert!(f.connector.log().frames.len() > frames, "a frame at once");
        assert_eq!(f.backend.settings.load().live_screen.as_deref(), Some(KEY));

        // Not live: it only restarts.
        f.backend.set_live(false, None, TIME).unwrap();
        let done = f.backend.restart_screen(KEY, TIME).unwrap();
        assert!(!done.live);
        assert_eq!(f.backend.sample().live, None);
        assert_eq!(f.connector.log().restarts.len(), 2);
        let err = f.backend.restart_screen("COM9", TIME).unwrap_err();
        assert_eq!(err.code(), "screenNotFound");
    }

    #[test]
    fn a_restart_waits_for_storage_and_needs_a_screen_with_an_mcu() {
        let f = fixture("restart-busy");
        let claim = f.backend.storage.claim().unwrap();
        let err = f.backend.restart_screen(KEY, TIME).unwrap_err();
        assert_eq!(err.code(), "busy");
        drop(claim);
        assert!(f.connector.log().restarts.is_empty());

        let mut f = fixture("restart-weact");
        f.backend.bus = Arc::new(FakeBus::new(vec![
            bezel_core::domain::discovery::Endpoint {
                address: bezel_core::domain::discovery::DeviceAddress("/dev/ttyACM0".into()),
                transport: bezel_core::domain::device::Transport::Serial,
                usb: bezel_core::domain::device::UsbId::new(0x1a86, 0xfe0c),
                serial_number: Some("AD0001".into()),
                manufacturer: None,
                product: None,
                location: None,
            },
        ]));
        let err = f.backend.restart_screen("/dev/ttyACM0", TIME).unwrap_err();
        assert_eq!(err.code(), "unsupported");
        assert!(err.to_string().contains("unplug the screen"), "{err}");
        assert!(f.connector.log().restarts.is_empty());
        let screens = f.backend.devices().unwrap().screens;
        assert!(!screens[0].restartable);
    }

    #[test]
    fn the_refresh_loop_sleeps_until_the_next_refresh_is_due() {
        let t0 = Instant::now();
        let ms = Duration::from_millis;
        assert_eq!(sleep_until(t0 + ms(40), t0), ms(40), "a GIF's next frame");
        assert_eq!(sleep_until(t0, t0 + ms(5)), Duration::ZERO, "already due");
        assert_eq!(sleep_until(t0 + ms(2000), t0), LOOK_AGAIN, "edits are seen");
    }

    /// The fixture's frame header for a preview whose GIF changes in
    /// `ms`: `None` when nothing animates.
    fn next_change(bytes: &[u8]) -> Option<u32> {
        let next = u32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]);
        (next != STILL).then_some(next)
    }

    /// A GIF of two 100 ms frames.
    fn blinking_gif() -> Vec<u8> {
        use image::codecs::gif::{GifEncoder, Repeat};
        let mut out = Vec::new();
        {
            let mut encoder = GifEncoder::new(&mut out);
            encoder.set_repeat(Repeat::Infinite).unwrap();
            for red in [0, 255] {
                let pixels = image::RgbaImage::from_pixel(4, 4, image::Rgba([red, 0, 0, 255]));
                let delay = image::Delay::from_numer_denom_ms(100, 1);
                encoder
                    .encode_frame(image::Frame::from_parts(pixels, 0, 0, delay))
                    .unwrap();
            }
        }
        out
    }

    /// T-7.11: the preview says when its GIF changes next, so the UI draws
    /// it at the GIF's pace; a hidden GIF says nothing.
    #[test]
    fn the_preview_says_when_its_gif_changes() {
        let f = fixture("preview-gif");
        std::fs::create_dir_all(&f.root).unwrap();
        let file = f.root.join("blink.gif");
        std::fs::write(&file, blinking_gif()).unwrap();
        let asset = f.backend.add_image(&file).unwrap().reference;
        let mut theme = f.backend.session().theme;
        let element = serde_json::json!({
            "id": 1, "name": "blink", "frame": {"x": 10, "y": 10, "width": 40, "height": 40},
            "opacity": 1, "visible": true, "locked": false,
            "kind": {"type": "image", "asset": asset, "fit": "fill"}
        });
        theme
            .elements
            .push(serde_json::from_value(element).unwrap());
        let bytes = f
            .backend
            .render(&theme, TIME, Instant::now(), Motion::Allowed)
            .unwrap();
        let next = next_change(&bytes).expect("animates");
        assert!((1..=100).contains(&next), "{next} ms");
        theme.elements[0].visible = false;
        let bytes = f
            .backend
            .render(&theme, TIME, Instant::now(), Motion::Allowed)
            .unwrap();
        assert_eq!(next_change(&bytes), None);
    }

    /// T-7.11: a live screen whose link failed (it hung) is connected again
    /// after 2 s by the refresh loop and shows the theme again, a whole frame
    /// on the new link; the UI hears about it meanwhile.
    #[test]
    fn a_live_screen_that_hangs_comes_back_by_itself() {
        let mut f = fixture("reconnect");
        let connector = FakeConnector::default()
            .breaking_after(1, bezel_core::BezelError::Hung("stalled".into()));
        f.backend.connector = Arc::new(connector.clone());
        f.connector = connector;
        f.backend.set_live(true, Some(KEY), TIME).unwrap();
        let later = |ms| Instant::now() + Duration::from_millis(ms);
        f.backend.tick_and_wait(TIME, later(1_000));
        let sample = f.backend.sample();
        assert_eq!(sample.live.as_deref(), Some(KEY), "still live");
        assert_eq!(sample.live_error, None);
        let waiting = sample.reconnecting.expect("reconnecting");
        assert_eq!((waiting.attempt, waiting.attempts), (1, 3));
        let json = serde_json::to_value(&sample).unwrap();
        assert_eq!(json["reconnecting"]["attempt"], 1);
        // The link is gone meanwhile: whoever needs it hears why.
        let busy = f.backend.set_brightness(KEY, 30).unwrap_err();
        assert!(
            busy.to_string().contains(crate::studio::RECONNECTING),
            "{busy}"
        );

        f.backend.tick_and_wait(TIME, later(500));
        assert_eq!(f.connector.log().connects, 1, "not before 2 s");
        f.backend.tick_and_wait(TIME, later(3_000));
        let sample = f.backend.sample();
        assert_eq!(sample.reconnecting, None);
        assert_eq!(sample.live.as_deref(), Some(KEY));
        let log = f.connector.log();
        assert_eq!((log.connects, log.frames.len()), (2, 2), "a frame at once");
        assert_eq!(log.orientations.len(), 2, "turned again");
        assert_eq!(f.backend.settings.load().live_screen.as_deref(), Some(KEY));
    }

    /// After three attempts live mode stops with the error that stopped the
    /// link (a hung screen's card offers the restart); turning live mode off
    /// while the screen is away ends the attempts at once.
    #[test]
    fn reconnecting_gives_up_after_three_attempts_or_when_stopped() {
        let hung = || bezel_core::BezelError::Hung("stalled".into());
        let mut f = fixture("reconnect-gone");
        let connector = FakeConnector::default().breaking_after(1, hung());
        f.backend.connector = Arc::new(connector.clone());
        f.backend.set_live(true, Some(KEY), TIME).unwrap();
        let later = |ms| Instant::now() + Duration::from_millis(ms);
        f.backend.bus = Arc::new(FakeBus::new(Vec::new()));
        f.backend.tick_and_wait(TIME, later(0));
        for (wait, attempt) in [(3_000, 2), (6_000, 3)] {
            f.backend.tick_and_wait(TIME, later(wait));
            let sample = f.backend.sample();
            assert_eq!(sample.reconnecting.map(|r| r.attempt), Some(attempt));
        }
        f.backend.tick_and_wait(TIME, later(11_000));
        let sample = f.backend.sample();
        assert_eq!((sample.live, sample.reconnecting), (None, None));
        assert_eq!(sample.live_error.unwrap().code(), "hung");
        assert_eq!(connector.log().connects, 1, "the screen never came back");

        let mut f = fixture("reconnect-stop");
        let connector = FakeConnector::default().breaking_after(1, hung());
        f.backend.connector = Arc::new(connector.clone());
        f.backend.set_live(true, Some(KEY), TIME).unwrap();
        f.backend.tick_and_wait(TIME, later(0));
        assert!(f.backend.sample().reconnecting.is_some());
        f.backend.set_live(false, None, TIME).unwrap();
        f.backend.tick_and_wait(TIME, later(3_000));
        let sample = f.backend.sample();
        assert_eq!((sample.live, sample.reconnecting), (None, None));
        assert_eq!(sample.live_error, None);
        assert_eq!(connector.log().connects, 1, "no attempt after the stop");
    }

    /// A link whose frames wait until the test lets them through, telling
    /// when one arrived: the screen's I/O in slow motion.
    struct Held {
        inner: Box<dyn ScreenLink>,
        arrived: mpsc::Sender<()>,
        through: mpsc::Receiver<()>,
    }

    impl ScreenLink for Held {
        fn identity(&self) -> &bezel_core::domain::screen::ScreenIdentity {
            self.inner.identity()
        }
        fn set_brightness(&mut self, brightness: Brightness) -> bezel_core::Result<()> {
            self.inner.set_brightness(brightness)
        }
        fn set_orientation(&mut self, orientation: Orientation) -> bezel_core::Result<()> {
            self.inner.set_orientation(orientation)
        }
        fn present(&mut self, frame: &bezel_core::domain::frame::Frame) -> bezel_core::Result<()> {
            self.arrived.send(()).unwrap();
            self.through.recv_timeout(Duration::from_secs(5)).unwrap();
            self.inner.present(frame)
        }
        fn screen_off(&mut self) -> bezel_core::Result<()> {
            self.inner.screen_off()
        }
        fn release(&mut self) -> bezel_core::Result<()> {
            self.inner.release()
        }
    }

    #[test]
    fn a_slow_display_does_not_block_fast_deliveries_or_queue_stale_frames() {
        let f = fixture("independent-deliveries");
        let blank = f.backend.studio().theme().clone();
        let (slow_arrived, slow_frames) = mpsc::channel();
        let (slow_release, slow_through) = mpsc::channel();
        let (fast_arrived, fast_frames) = mpsc::channel();
        let (fast_release, fast_through) = mpsc::channel();
        let slow = Held {
            inner: f.backend.connect(KEY).unwrap(),
            arrived: slow_arrived,
            through: slow_through,
        };
        let fast = Held {
            inner: f.backend.connect(KEY).unwrap(),
            arrived: fast_arrived,
            through: fast_through,
        };
        {
            let mut s = f.backend.studio();
            s.select_screen("slow", blank.clone());
            s.go_live("slow".into(), Box::new(slow));
            s.select_screen("fast", blank.clone());
            s.go_live("fast".into(), Box::new(fast));
        }
        let now = Instant::now();
        let epoch = f.backend.delivery_epoch();
        f.backend.tick(TIME, now);
        slow_frames.recv_timeout(Duration::from_secs(2)).unwrap();
        fast_frames.recv_timeout(Duration::from_secs(2)).unwrap();
        // Both sends started, and tick returned while both links were held.
        assert_eq!(f.backend.delivery_epoch(), epoch);
        fast_release.send(()).unwrap();
        f.backend.wait_refresh(now + Duration::from_secs(1), epoch);
        assert!(
            f.backend.delivery_epoch() > epoch,
            "completion wakes rendering"
        );

        let mut blue = blank.clone();
        blue.background = Background::Color(Rgba::opaque(0, 0, 255));
        f.backend.studio().set_theme(blue);
        f.backend.tick(TIME, now + Duration::from_secs(2));
        fast_frames.recv_timeout(Duration::from_secs(2)).unwrap();
        // Fast's worker was reused while slow is still on its first frame.
        assert!(slow_frames.try_recv().is_err());
        let mut red = blank;
        red.background = Background::Color(Rgba::opaque(255, 0, 0));
        f.backend.studio().set_theme(red);
        for seconds in 3..10 {
            f.backend.tick(TIME, now + Duration::from_secs(seconds));
        }
        assert!(
            fast_frames.try_recv().is_err(),
            "one outstanding frame, no backlog"
        );
        assert_eq!(f.connector.log().frames.len(), 1);

        let epoch = f.backend.delivery_epoch();
        fast_release.send(()).unwrap();
        f.backend
            .wait_refresh(Instant::now() + Duration::from_secs(1), epoch);
        f.backend.tick(TIME, now + Duration::from_secs(11));
        fast_frames.recv_timeout(Duration::from_secs(2)).unwrap();
        slow_release.send(()).unwrap();
        fast_release.send(()).unwrap();
        assert!(!f.backend.idle_studio().presenting());
        let frames = f.connector.log().frames;
        assert_eq!(frames.len(), 4, "three fast frames and one slow frame");
        assert_eq!(
            frames
                .iter()
                .filter(|frame| frame.pixel(0, 0) == Some(Rgba::opaque(255, 0, 0)))
                .count(),
            1,
            "latest edit replaces skipped frames"
        );
        assert_eq!(
            frames
                .iter()
                .filter(|frame| frame.pixel(0, 0) == Some(Rgba::opaque(0, 0, 255)))
                .count(),
            1
        );
    }

    #[test]
    fn a_delivery_completed_before_waiting_does_not_lose_its_wakeup() {
        let f = fixture("delivery-wakeup");
        f.backend
            .studio()
            .go_live(KEY.into(), f.backend.connect(KEY).unwrap());
        let epoch = f.backend.delivery_epoch();
        f.backend.tick_and_wait(TIME, Instant::now());
        assert!(f.backend.delivery_epoch() > epoch);
        let start = Instant::now();
        f.backend
            .wait_refresh(start + Duration::from_secs(1), epoch);
        assert!(
            start.elapsed() < Duration::from_millis(100),
            "do not wait the 250 ms polling interval after completion"
        );
    }

    #[test]
    fn four_display_workers_start_independently_and_one_failure_is_isolated() {
        let f = fixture("four-deliveries");
        let blank = f.backend.studio().theme().clone();
        let mut gates = Vec::new();
        for index in 0..4 {
            let (arrived, frames) = mpsc::channel();
            let (release, through) = mpsc::channel();
            let inner = if index == 0 {
                let failing = FakeConnector::default()
                    .breaking_after(0, bezel_core::BezelError::Hung("test stall".into()));
                connect_screen(f.backend.bus.as_ref(), &failing, Some(KEY))
                    .unwrap()
                    .1
            } else {
                f.backend.connect(KEY).unwrap()
            };
            let key = format!("display-{index}");
            let mut s = f.backend.studio();
            s.select_screen(&key, blank.clone());
            s.go_live(
                key,
                Box::new(Held {
                    inner,
                    arrived,
                    through,
                }),
            );
            gates.push((frames, release));
        }
        f.backend.tick(TIME, Instant::now());
        for (frames, _) in &gates {
            frames.recv_timeout(Duration::from_secs(2)).unwrap();
        }
        assert!(f.backend.studio().presenting());
        for (_, release) in &gates {
            release.send(()).unwrap();
        }
        assert!(!f.backend.idle_studio().presenting());
        assert_eq!(f.connector.log().frames.len(), 3);
        assert!(f.backend.studio().any_live(), "other displays stay live");
        f.backend.studio().select_screen("display-0", blank);
        assert!(f.backend.studio().live_error().is_some());
        assert_eq!(f.backend.studio().live_key(), None);
    }

    #[test]
    fn previews_render_while_the_screen_shows_a_frame() {
        let f = fixture("held");
        let (arrived, frame_arrived) = mpsc::channel();
        let (let_through, through) = mpsc::channel();
        let inner = f.backend.connect(KEY).unwrap();
        let held = Held {
            inner,
            arrived,
            through,
        };
        f.backend.studio().go_live(KEY.into(), Box::new(held));
        let theme = f.backend.session().theme;
        let backend = &f.backend;
        std::thread::scope(|scope| {
            let started = Instant::now();
            let ticking = scope.spawn(|| backend.tick_and_wait(TIME, Instant::now()));
            frame_arrived.recv().unwrap();
            // The screen is busy with a frame: the session is not.
            assert!(
                backend
                    .render(&theme, TIME, Instant::now(), Motion::Allowed)
                    .is_ok()
            );
            assert_eq!(backend.sample().live.as_deref(), Some(KEY));
            // Whoever needs the link waits for it.
            let dimming = scope.spawn(|| backend.set_brightness(KEY, 30));
            std::thread::sleep(Duration::from_millis(50));
            assert!(f.connector.log().brightness.is_empty(), "after the frame");
            let_through.send(()).unwrap();
            assert!(ticking.join().unwrap() > started);
            dimming.join().unwrap().unwrap();
        });
        let log = f.connector.log();
        assert_eq!((log.frames.len(), log.brightness.len()), (1, 1));
    }

    const DRAGON: &str = "assets/dragon.mp4";
    const DRAGON_POSTER: &str = "assets/poster-195.png";
    /// Longest wait for a held call of the converter to arrive (it arrives
    /// at once; this only keeps a broken test from hanging).
    const ARRIVAL: Duration = Duration::from_secs(30);

    /// Live on the fake 8.8" (a blank theme first), then editing the Dragon
    /// Ball theme: 1920x480, its video the vendor's pre-turned 480x1920
    /// `dragon.mp4` (not probed yet) with its poster. The session probes and
    /// takes posters with `media`, the storage tab's converter, as in the
    /// app.
    fn dragon_live(name: &str, media: FakeMedia) -> (crate::storage::tests::Fixture, Theme) {
        let copies = Copies::in_memory(MemoryArchive::new());
        let f = fixture_on(
            name,
            FakeBus::turing_88(),
            FakeStorage::default(),
            media,
            copies,
        );
        f.backend.set_live(true, Some(KEY), TIME).unwrap();
        let mut theme = Theme::blank("Dragon Ball", Size::new(480, 1920), Orientation::Landscape);
        theme.background = Background::Video {
            asset: AssetRef(DRAGON.into()),
            poster: Some(AssetRef(DRAGON_POSTER.into())),
            framing: None,
        };
        let poster = Frame::filled(Size::new(1920, 480), Rgba::opaque(9, 9, 9));
        let assets = BTreeMap::from([
            (AssetRef(DRAGON.into()), vec![9; 4096]),
            (
                AssetRef(DRAGON_POSTER.into()),
                crate::media::png_of(&poster).unwrap(),
            ),
        ]);
        f.backend.studio().start(theme.clone(), assets, None);
        (f, theme)
    }

    /// When a preview ([`frame_bytes`]) changes next, milliseconds.
    fn next_ms(preview: &[u8]) -> u32 {
        u32::from_le_bytes(preview[8..12].try_into().unwrap())
    }

    /// While a call holds the converter, the session goes on: its lock is
    /// free, the refresh `ms` from now draws the live screen's frame, a push
    /// shows one at once, and the preview renders `theme` (its poster, asked
    /// again soon).
    fn the_session_goes_on(f: &crate::storage::tests::Fixture, theme: &ThemeDto, ms: u64) {
        assert!(
            f.backend.studio.studio.try_lock().is_ok(),
            "the lock is free"
        );
        let frames = || f.connector.log().frames.len();
        let before = frames();
        f.backend
            .tick_and_wait(TIME, Instant::now() + Duration::from_millis(ms));
        assert_eq!(frames(), before + 1, "the refresh drew a frame");
        f.backend.push(theme, TIME).unwrap();
        assert_eq!(frames(), before + 2, "a push shows one at once");
        let preview = f
            .backend
            .render(theme, TIME, Instant::now(), Motion::Allowed)
            .unwrap();
        let busy = u32::try_from(CONVERTER_BUSY.as_millis()).unwrap();
        assert_eq!(next_ms(&preview), busy, "the poster, asked again soon");
    }

    /// Review W2: probing the theme's video (ffprobe, for a container that
    /// is not MP4 or GIF) runs outside the session's lock; the live screen
    /// starts the video once it is probed.
    #[test]
    fn the_session_goes_on_while_the_video_is_probed() {
        let (media, arrivals, go) = FakeMedia::ready().holding(Call::Probe);
        let (f, theme) = dragon_live("probing", media);
        let dto = ThemeDto::from(&theme);
        let backend = &f.backend;
        std::thread::scope(|scope| {
            // Dropped if the test fails: the held probe goes on.
            let go = go;
            let rendering =
                scope.spawn(|| backend.render(&dto, TIME, Instant::now(), Motion::Allowed));
            assert_eq!(arrivals.recv_timeout(ARRIVAL), Ok(Call::Probe));
            the_session_goes_on(&f, &dto, 60_000);
            let video = backend.sample().video.unwrap();
            assert_eq!(video.state, "notStarted", "not before the probe");
            go.send(()).unwrap();
            let preview = rendering.join().unwrap().unwrap();
            assert_eq!(next_ms(&preview), 67, "the video plays");
        });
        let auto = f.backend.video_auto(&dto).unwrap();
        let size = auto.size.map(|s| (s.width, s.height));
        assert_eq!((auto.rotation, size), (270, Some((480, 1920))));
        f.backend
            .tick_and_wait(TIME, Instant::now() + Duration::from_secs(120));
        let video = f.backend.sample().video.unwrap();
        assert_eq!(
            (video.state, video.path.as_deref()),
            ("missing", Some("internal/video/dragon.mp4"))
        );
    }

    /// Review W2: the poster taken again on save (ffmpeg, up to 30 s) runs
    /// outside the session's lock; it reaches the saved theme.
    #[test]
    fn the_session_goes_on_while_the_poster_is_taken() {
        let (media, arrivals, go) = FakeMedia::ready().holding(Call::Poster);
        let (f, mut theme) = dragon_live("posters", media);
        // The refresh probes the video for the live screen.
        f.backend
            .tick_and_wait(TIME, Instant::now() + Duration::from_secs(60));
        assert_eq!(f.backend.sample().video.unwrap().state, "missing");
        let poster = AssetRef(DRAGON_POSTER.into());
        let before = f.backend.studio().assets()[&poster].clone();
        if let Background::Video { framing, .. } = &mut theme.background {
            *framing = Some(VideoFraming {
                fit: VideoFit::Contain,
                zoom: Zoom::from_percent(125),
                ..VideoFraming::default()
            });
        }
        let dto = ThemeDto::from(&theme);
        let backend = &f.backend;
        let saved = std::thread::scope(|scope| {
            // Dropped if the test fails: the held poster goes on.
            let go = go;
            let saving = scope.spawn(|| backend.save(&dto, None));
            assert_eq!(arrivals.recv_timeout(ARRIVAL), Ok(Call::Poster));
            the_session_goes_on(&f, &dto, 120_000);
            go.send(()).unwrap();
            saving.join().unwrap().unwrap()
        });
        let taken = f.backend.studio().assets()[&poster].clone();
        assert_ne!(taken, before, "taken again");
        let (_, assets) = FsThemeStore.load(&ThemeLocation(saved.location)).unwrap();
        assert_eq!(assets[&poster], taken, "saved with the theme");
        let picture = image::load_from_memory(&taken).unwrap().to_rgba8();
        assert_eq!(picture.dimensions(), (1920, 480));
    }

    #[test]
    fn the_tray_switches_live_mode_on_the_connected_screen() {
        let f = fixture("tray");
        assert!(f.backend.toggle_live(TIME).unwrap());
        assert_eq!(f.backend.sample().live.as_deref(), Some(KEY));
        assert_eq!(f.connector.log().frames.len(), 1);
        assert!(!f.backend.toggle_live(TIME).unwrap());
        assert_eq!(f.backend.sample().live, None);
        assert_eq!(f.backend.settings.load().live_screen, None);

        let mut empty = fixture("tray-empty");
        empty.backend.bus = Arc::new(FakeBus::new(Vec::new()));
        assert!(empty.backend.toggle_live(TIME).is_err());
        assert_eq!(empty.backend.sample().live, None);
    }

    #[test]
    fn brightness_and_release_open_a_screen_that_is_not_live() {
        let f = fixture("offline");
        f.backend.set_brightness(KEY, 10).unwrap();
        f.backend.release(KEY).unwrap();
        let log = f.connector.log();
        assert_eq!((log.brightness.len(), log.releases), (1, 1));
    }

    /// The fake 8.8"'s MCU (its wake chip): the screen's other port.
    const MCU: &str = "/dev/ttyACM0";

    /// What reopening a port this app already holds answers on Linux.
    fn busy() -> bezel_core::BezelError {
        bezel_core::BezelError::Transport(format!("{KEY}: Device or resource busy"))
    }

    /// The fake 8.8" asleep: its MCU listed, its display not.
    fn asleep() -> Vec<bezel_core::domain::discovery::Endpoint> {
        let all = FakeBus::turing_88().endpoints().unwrap();
        all.into_iter().filter(|e| e.address.0 == MCU).collect()
    }

    /// The fake 8.8" asleep until the connector's first connection wakes
    /// its display.
    struct Waking(FakeConnector);

    impl DeviceBus for Waking {
        fn endpoints(&self) -> bezel_core::Result<Vec<bezel_core::domain::discovery::Endpoint>> {
            if self.0.log().connects == 0 {
                return Ok(asleep());
            }
            FakeBus::turing_88().endpoints()
        }
    }

    /// D-2026-10-01-live-screen-controls-2: put live by its MCU's port (the
    /// tray finds a sleeping 8.8" by it), the screen goes live under its
    /// display's key, remembered with its orientation; the MCU's port only
    /// while the display cannot be listed.
    #[test]
    fn a_screen_put_live_by_its_mcu_port_is_keyed_by_its_display() {
        let f = fixture("live-by-mcu");
        f.backend.set_live(true, Some(MCU), TIME).unwrap();
        assert_eq!(f.backend.sample().live.as_deref(), Some(KEY));
        let settings = f.backend.settings.load();
        assert_eq!(settings.live_screen.as_deref(), Some(KEY));
        let turned = Some(Orientation::ReversePortrait);
        assert_eq!(settings.orientation_for(KEY), turned);
        assert_eq!(settings.orientation_for(MCU), None);
        assert_eq!(f.connector.log().connects, 1);

        // The tray on a sleeping 8.8": the connection wakes its display.
        let mut tray = fixture("live-by-mcu-tray");
        tray.backend.bus = Arc::new(Waking(tray.connector.clone()));
        assert!(tray.backend.toggle_live(TIME).unwrap());
        assert_eq!(tray.backend.sample().live.as_deref(), Some(KEY));
        let saved = tray.backend.settings.load().live_screen;
        assert_eq!(saved.as_deref(), Some(KEY));

        // Still asleep once connected: its MCU's port is all there is.
        let mut sleepy = fixture("live-by-mcu-asleep");
        sleepy.backend.bus = Arc::new(FakeBus::new(asleep()));
        sleepy.backend.set_live(true, Some(MCU), TIME).unwrap();
        assert_eq!(sleepy.backend.sample().live.as_deref(), Some(MCU));
        let saved = sleepy.backend.settings.load().live_screen;
        assert_eq!(saved.as_deref(), Some(MCU));
    }

    /// Writes down, in order, what a screen's ports see: the links `inner`
    /// opens and that close, and the restarts.
    #[derive(Clone, Default)]
    struct Tracked {
        inner: FakeConnector,
        seen: Arc<Mutex<Vec<&'static str>>>,
    }

    impl Tracked {
        fn note(&self, what: &'static str) {
            self.seen.lock().unwrap().push(what);
        }
    }

    /// A link of [`Tracked`], noting when it closes.
    struct TrackedLink(Box<dyn ScreenLink>, Tracked);

    impl Drop for TrackedLink {
        fn drop(&mut self) {
            self.1.note("close");
        }
    }

    impl ScreenLink for TrackedLink {
        fn identity(&self) -> &bezel_core::domain::screen::ScreenIdentity {
            self.0.identity()
        }
        fn set_brightness(&mut self, brightness: Brightness) -> bezel_core::Result<()> {
            self.0.set_brightness(brightness)
        }
        fn set_orientation(&mut self, orientation: Orientation) -> bezel_core::Result<()> {
            self.0.set_orientation(orientation)
        }
        fn present(&mut self, frame: &bezel_core::domain::frame::Frame) -> bezel_core::Result<()> {
            self.0.present(frame)
        }
        fn screen_off(&mut self) -> bezel_core::Result<()> {
            self.0.screen_off()
        }
        fn release(&mut self) -> bezel_core::Result<()> {
            self.0.release()
        }
    }

    impl ScreenConnector for Tracked {
        fn connect(&self, screen: &Screen) -> bezel_core::Result<Box<dyn ScreenLink>> {
            let link = self.inner.connect(screen)?;
            self.note("open");
            Ok(Box::new(TrackedLink(link, self.clone())))
        }
        fn restart(&self, screen: &Screen) -> bezel_core::Result<()> {
            self.inner.restart(screen)?;
            self.note("restart");
            Ok(())
        }
    }

    /// D-2026-10-01-live-screen-controls-3: live on the 8.8" (the UI names
    /// it by its display), either of its ports reaches the open live link:
    /// brightness, the storage tab and release go through it and never open
    /// the screen again (which would answer busy); a restart by the MCU's
    /// port stops live mode before the screen restarts and resumes it under
    /// the display's key.
    #[test]
    fn either_port_of_the_live_screen_reaches_its_live_link() {
        let f = fixture("either-port");
        f.backend.set_live(true, Some(KEY), TIME).unwrap();
        let _held = f.connector.clone().refusing_after(1, vec![busy()]);
        for port in [MCU, KEY] {
            f.backend.set_brightness(port, 40).unwrap();
            let overview = f.backend.storage_overview(port, TIME).unwrap();
            assert!(overview.internal.total > 0, "{port}");
        }
        f.backend.release(MCU).unwrap();
        let log = f.connector.log();
        assert_eq!(
            (log.connects, log.brightness.len(), log.releases),
            (1, 2, 1)
        );
        assert_eq!(f.backend.sample().live, None);
        assert_eq!(f.backend.settings.load().live_screen, None);

        let mut f = fixture("either-port-restart");
        let tracked = Tracked {
            inner: f.connector.clone(),
            ..Tracked::default()
        };
        f.backend.connector = Arc::new(tracked.clone());
        f.backend.set_live(true, Some(KEY), TIME).unwrap();
        let done = f.backend.restart_screen(MCU, TIME).unwrap();
        assert_eq!(
            done,
            RestartedDto {
                key: KEY.into(),
                live: true
            }
        );
        let seen = tracked.seen.lock().unwrap().clone();
        assert_eq!(seen, ["open", "close", "restart", "open"]);
        assert_eq!(f.backend.sample().live.as_deref(), Some(KEY));
        assert_eq!(f.backend.settings.load().live_screen.as_deref(), Some(KEY));
    }

    /// D-2026-10-01-live-screen-controls-2: a settings file that kept the
    /// live screen by its MCU's port (0.1.0-dev.287) brings it back live
    /// under its display's key and is rewritten with it: the UI's
    /// brightness, by the display, then goes through the live link.
    #[test]
    fn a_saved_mcu_key_is_restored_under_the_display_key() {
        let f = fixture("restore-mcu");
        f.backend
            .settings
            .update(|s| s.live_screen = Some(MCU.into()));
        f.backend.restore_live(TIME);
        assert_eq!(f.backend.sample().live.as_deref(), Some(KEY));
        let settings = f.backend.settings.load();
        assert_eq!(settings.live_screen.as_deref(), Some(KEY));
        assert_eq!(settings.orientation_for(MCU), None);
        let _held = f.connector.clone().refusing_after(1, vec![busy()]);
        f.backend.set_brightness(KEY, 60).unwrap();
        let log = f.connector.log();
        assert_eq!((log.connects, log.brightness.len()), (1, 1));
    }

    #[test]
    fn restoring_live_does_not_reselect_the_stale_wake_alias_document() {
        let f = fixture("restore-selected-alias");
        f.backend.settings.update(|s| {
            s.selected_screen = Some(MCU.into());
            s.live_screens = vec![KEY.into()];
        });
        f.backend.restore_live(TIME);
        assert_eq!(f.backend.session().screen.as_deref(), Some(KEY));
        assert_eq!(f.backend.sample().live.as_deref(), Some(KEY));
        assert_eq!(f.connector.log().connects, 1);
    }

    #[test]
    fn live_from_wake_document_releases_existing_display_document_before_reopening() {
        let mut f = fixture("live-alias-document");
        let tracked = Tracked {
            inner: f.connector.clone(),
            ..Tracked::default()
        };
        f.backend.connector = Arc::new(tracked.clone());
        f.backend.set_live(true, Some(KEY), TIME).unwrap();
        f.backend.select_screen(MCU).unwrap();
        // The user edited this document; enabling Live must retain those edits.
        let mut edited = f.backend.studio().theme().clone();
        edited.name = "Unsaved card edit".into();
        f.backend.studio().set_theme(edited);
        f.backend.set_live(true, Some(MCU), TIME).unwrap();
        assert_eq!(f.backend.session().theme.name, "Unsaved card edit");
        assert_eq!(f.backend.sample().live.as_deref(), Some(KEY));
        assert_eq!(f.connector.log().connects, 2);
        assert_eq!(*tracked.seen.lock().unwrap(), ["open", "close", "open"]);
        assert_eq!(f.backend.session().screen.as_deref(), Some(KEY));
        assert_eq!(
            f.backend.settings.load().selected_screen.as_deref(),
            Some(KEY)
        );
        f.backend.select_screen(MCU).unwrap();
        f.backend.select_screen(KEY).unwrap();
        assert_eq!(f.backend.session().theme.name, "Unsaved card edit");
    }

    #[test]
    fn sensors_reach_the_ui() {
        let f = fixture("sensors");
        let catalog = f.backend.catalog().unwrap();
        assert!(catalog.iter().any(|s| s.key == "cpu.usage"));
        f.backend.tick_and_wait(TIME, Instant::now());
        let sample = f.backend.sample();
        assert!(sample.readings.contains_key("cpu.usage"));
        assert_eq!(sample.live_error, None);
    }

    #[test]
    fn the_sensor_list_says_what_it_shows() {
        let f = fixture("sensor-list");
        f.backend.restore_theme();
        let theme = f.backend.studio().wanted().clone();
        assert!(!theme.contains("net.ping"), "nothing shows the ping");
        f.backend
            .show_sensors(&["net.ping".to_string(), "not a key".to_string()]);
        let listed = f.backend.studio().wanted().clone();
        let ping: Wanted = SensorKey::new("net.ping").into_iter().collect();
        assert_eq!(listed, theme.union(&ping));
        f.backend.show_sensors(&[]);
        assert_eq!(f.backend.studio().wanted(), &theme, "the list is hidden");
    }

    #[test]
    fn themes_save_list_open_and_restore() {
        let f = fixture("themes");
        let mut theme = f
            .backend
            .new_theme(Some(KEY), "Mine", Some(Orientation::ReversePortrait))
            .unwrap();
        assert_eq!((theme.canvas.width, theme.canvas.height), (480, 1920));
        theme.refresh_seconds = 2.0;
        let saved = f.backend.save(&theme, None).unwrap();
        assert!(
            saved.location.ends_with("Mine.bezeltheme"),
            "{}",
            saved.location
        );
        let again = f.backend.save(&theme, None).unwrap();
        assert_eq!(again.location, saved.location, "saving again overwrites");
        let listed = f.backend.themes();
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].name, "Mine");
        assert_eq!(listed[0].orientation, "reverse-portrait");

        f.backend.new_theme(None, "Other", None).unwrap();
        assert_eq!(
            f.backend.open(&saved.location).unwrap().refresh_seconds,
            2.0
        );
        assert!(f.backend.open("/nope.bezeltheme").is_err());

        f.backend.new_theme(None, "Scratch", None).unwrap();
        f.backend
            .settings
            .update(|s| s.live_screen = Some(KEY.into()));
        f.backend.restore_theme();
        f.backend.restore_live(TIME);
        assert_eq!(f.backend.session().theme.name, "Mine");
        assert_eq!(f.backend.sample().live.as_deref(), Some(KEY));
    }

    #[test]
    fn save_never_writes_over_a_document_the_editor_does_not_show() {
        let f = fixture("shown");
        let mut kept = theme_of(&f.backend.session().theme).unwrap();
        kept.name = "Pixel V2.1".into();
        let mine = f
            .backend
            .save(&ThemeDto::from(&kept), None)
            .unwrap()
            .location;
        // The editor still shows another theme while the session opened Mine
        // (an open the UI saw fail): Save is refused and Mine stays.
        f.backend.open(&mine).unwrap();
        let mut other = kept.clone();
        other.name = "Pixel".into();
        let error = f
            .backend
            .save_shown(&ThemeDto::from(&other), None, Some(None))
            .unwrap_err();
        assert_eq!(error.code(), "documentChanged", "{error}");
        let (on_disk, _) = FsThemeStore.load(&ThemeLocation(mine.clone())).unwrap();
        assert_eq!(on_disk.name, "Pixel V2.1");
        let error = f
            .backend
            .save_shown(
                &ThemeDto::from(&other),
                None,
                Some(Some("elsewhere.bezeltheme")),
            )
            .unwrap_err();
        assert_eq!(error.code(), "documentChanged", "{error}");
        // The editor showing Mine saves it in place.
        let saved = f
            .backend
            .save_shown(&ThemeDto::from(&kept), None, Some(Some(mine.as_str())))
            .unwrap();
        assert_eq!(saved.location, mine);
    }

    #[test]
    fn the_window_opens_and_saves_only_where_allowed() {
        let f = fixture("allowed");
        let theme = f.backend.session().theme;
        let outside = f.root.join("Desktop").join("Mine.bezeltheme");
        let at = ThemeLocation(outside.display().to_string());
        FsThemeStore
            .save(&at, &theme_of(&theme).unwrap(), &Default::default())
            .unwrap();
        let sneaky = f.root.join("themes").join("..").join("Desktop");
        for refused in [
            outside.display().to_string(),
            sneaky.join("Mine.bezeltheme").display().to_string(),
            "/etc/passwd".into(),
        ] {
            let error = f.backend.open(&refused).unwrap_err();
            assert_eq!(error.code(), "notInLibrary", "{error}");
            assert_eq!(error.value("location"), Some(refused.as_str()));
        }
        let error = f.backend.save(&theme, Some(at.clone())).unwrap_err();
        assert_eq!(error.code(), "notPicked", "{error}");
        assert!(error.to_string().contains("not picked"), "{error}");
        assert!(!outside.with_file_name("theme.json").exists());

        // A theme from elsewhere (an import keeps no location; say it had
        // one) is saved as a copy in the user folder.
        f.backend.studio().start(
            theme_of(&theme).unwrap(),
            Default::default(),
            Some(at.clone()),
        );
        let copy = f.backend.save(&theme, None).unwrap().location;
        assert!(
            Path::new(&copy).starts_with(f.root.join("themes")),
            "{copy}"
        );

        // Picked in a dialog: open and save reach it for the session, and
        // the next start reopens it and saves back to it.
        f.backend.library.grant(&at);
        f.backend.open(&at.0).unwrap();
        assert_eq!(f.backend.save(&theme, None).unwrap().location, at.0);
        let next = fixture("allowed-next");
        next.backend
            .settings
            .update(|s| s.last_theme = Some(at.0.clone()));
        next.backend.restore_theme();
        assert_eq!(
            next.backend.session().location.as_deref(),
            Some(at.0.as_str())
        );
        assert_eq!(next.backend.save(&theme, None).unwrap().location, at.0);
    }

    #[test]
    fn new_themes_follow_the_screen_shape_then_the_last_orientation_used() {
        let f = fixture("orientation");
        let model = model_by_id(DEFAULT_MODEL).unwrap();
        assert_eq!(
            default_orientation(model),
            Orientation::Landscape,
            "8.8\" bar"
        );
        let square = model_by_id(bezel_core::domain::device::ModelId("turing-2.1")).unwrap();
        assert_eq!(default_orientation(square), square.native_orientation);
        let five = model_by_id(bezel_core::domain::device::ModelId("usbpcmonitor-5")).unwrap();
        assert_eq!(
            default_orientation(five),
            Orientation::Portrait,
            "5:3 is no bar"
        );

        let first = f.backend.new_theme(Some(KEY), "A", None).unwrap();
        assert_eq!(first.orientation, "landscape");
        assert_eq!((first.canvas.width, first.canvas.height), (1920, 480));
        let chosen = f
            .backend
            .new_theme(Some(KEY), "B", Some(Orientation::ReversePortrait))
            .unwrap();
        assert_eq!((chosen.canvas.width, chosen.canvas.height), (480, 1920));
        let next = f.backend.new_theme(Some(KEY), "C", None).unwrap();
        assert_eq!(
            next.orientation, "reverse-portrait",
            "remembered for the screen"
        );
        let elsewhere = f.backend.new_theme(None, "D", None).unwrap();
        assert_eq!(
            elsewhere.orientation, "landscape",
            "no screen: the 8.8\" rule"
        );
        let unplugged = f
            .backend
            .new_theme(Some("COM9"), "E", Some(Orientation::Portrait))
            .unwrap();
        assert_eq!(
            (unplugged.canvas.width, unplugged.canvas.height),
            (480, 1920)
        );
        assert_eq!(
            f.backend.settings.load().orientation_for("COM9"),
            Some(Orientation::Portrait)
        );

        // With no last theme the app starts with a blank theme for the first
        // screen, in the orientation last used with it.
        f.backend.restore_theme();
        let start = f.backend.session();
        assert_eq!(
            (start.theme.name.as_str(), start.theme.orientation.as_str()),
            ("Untitled", "reverse-portrait")
        );
        assert_eq!(start.location, None);
        f.backend
            .settings
            .update(|s| s.last_theme = Some("/gone.bezeltheme".into()));
        f.backend.restore_theme();
        assert_eq!(
            f.backend.session().theme.name,
            "Untitled",
            "an unreadable last theme"
        );
    }

    #[test]
    fn live_mode_follows_a_turn_to_the_other_orientation() {
        let f = fixture("turn");
        f.backend.set_live(true, Some(KEY), TIME).unwrap();
        assert_eq!(
            f.backend.settings.load().orientation_for(KEY),
            Some(Orientation::ReversePortrait)
        );
        let mut wide = f.backend.session().theme;
        wide.orientation = "landscape".into();
        wide.canvas = bezel_themes::dto::SizeDto {
            width: 1920,
            height: 480,
        };
        f.backend.push(&wide, TIME).unwrap();
        let log = f.connector.log();
        assert_eq!(
            log.orientations,
            vec![Orientation::ReversePortrait, Orientation::Landscape]
        );
        assert_eq!(log.frames.len(), 2);
        assert_eq!(log.frames[0].size(), Size::new(480, 1920));
        assert_eq!(log.frames[1].size(), Size::new(1920, 480));
        assert_eq!(
            f.backend.settings.load().orientation_for(KEY),
            Some(Orientation::Landscape)
        );
        assert_eq!(
            f.backend
                .new_theme(Some(KEY), "Next", None)
                .unwrap()
                .orientation,
            "landscape"
        );
        // Not live: a push shows nothing and remembers nothing.
        f.backend.set_live(false, None, TIME).unwrap();
        let mut tall = wide.clone();
        tall.orientation = "portrait".into();
        tall.canvas = bezel_themes::dto::SizeDto {
            width: 480,
            height: 1920,
        };
        f.backend.push(&tall, TIME).unwrap();
        assert_eq!(f.connector.log().frames.len(), 2);
        assert_eq!(
            f.backend.settings.load().orientation_for(KEY),
            Some(Orientation::Landscape)
        );
    }

    /// A small turing-smart-screen-python theme; the LED color has no
    /// equivalent in a Bezel theme.
    const TINY_PYTHON_THEME: &str = r#"---
display:
  DISPLAY_SIZE: 3.5"
  DISPLAY_ORIENTATION: landscape
  DISPLAY_RGB_LED: 0, 120, 255
static_text:
  LABEL:
    TEXT: "CPU"
    X: 20
    Y: 18
    FONT_SIZE: 18
    FONT_COLOR: 255, 255, 255
    BACKGROUND_COLOR: 0, 0, 0
"#;

    /// The theme laid out like the Python repository (`res/themes/<name>`).
    fn python_theme(root: &Path) -> PathBuf {
        let dir = root.join("res/themes/Tiny");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("theme.yaml"), TINY_PYTHON_THEME).unwrap();
        dir
    }

    #[test]
    fn a_theme_json_opens_its_folder_like_the_cli() {
        let f = fixture("import-manifest");
        let folder = f.root.join("Manifest theme");
        let target = ThemeLocation(folder.display().to_string());
        f.backend.library.grant(&target);
        f.backend
            .save(&f.backend.session().theme, Some(target))
            .unwrap();
        let imported = f.backend.import(&folder.join("theme.json")).unwrap();
        assert_eq!(imported.theme.name, "Start");
        assert!(imported.warnings.is_empty());
    }

    #[test]
    fn opening_and_importing_pause_only_the_selected_live_screen_after_loading_succeeds() {
        let f = fixture("open-offline");
        let location = f
            .backend
            .save(&f.backend.session().theme, None)
            .unwrap()
            .location;
        f.backend.set_live(true, Some(KEY), TIME).unwrap();
        let missing = f.root.join("missing.turtheme");
        assert!(f.backend.import(&missing).is_err());
        assert!(
            f.backend.sample().live.is_some(),
            "a failed file read preserves live output"
        );
        f.backend
            .settings
            .update(|s| s.live_screens.push("other-screen".into()));
        f.backend.open(&location).unwrap();
        assert_eq!(f.backend.sample().live, None);
        assert_eq!(f.backend.settings.load().live_screens, ["other-screen"]);
        f.backend.set_live(true, Some(KEY), TIME).unwrap();
        f.backend.import(Path::new(&location)).unwrap();
        assert_eq!(f.backend.sample().live, None);
        assert!(
            !f.backend
                .settings
                .load()
                .live_screens
                .contains(&KEY.to_string())
        );
        assert_eq!(f.backend.session().location, None);
    }

    #[test]
    fn imports_native_and_other_apps_themes() {
        let f = fixture("import");
        let native = f
            .backend
            .save(&f.backend.session().theme, None)
            .unwrap()
            .location;
        let imported = f.backend.import(Path::new(&native)).unwrap();
        assert_eq!(imported.theme.name, "Start");
        assert!(imported.warnings.is_empty());
        assert_eq!(f.backend.session().location, None, "a copy, not the file");
        let folder = f.root.join("Folder theme");
        let target = ThemeLocation(folder.display().to_string());
        // Picked in the save dialog.
        f.backend.library.grant(&target);
        f.backend
            .save(&f.backend.session().theme, Some(target))
            .unwrap();
        assert!(folder.join("theme.json").is_file());
        let imported = f.backend.import(&folder).unwrap();
        assert_eq!(imported.theme.name, "Start");
        assert!(imported.warnings.is_empty());

        let dir = python_theme(&f.root);
        for path in [dir.clone(), dir.join("theme.yaml")] {
            let imported = f.backend.import(&path).unwrap();
            assert_eq!(imported.theme.name, "Tiny", "{}", path.display());
            assert_eq!(imported.theme.orientation, "landscape");
            assert_eq!(
                (imported.theme.canvas.width, imported.theme.canvas.height),
                (480, 320)
            );
            assert!(
                imported
                    .warnings
                    .iter()
                    .any(|w| w.code == "backplateLed" && w.message.contains("LED")),
                "{:?}",
                imported.warnings
            );
            assert_eq!(f.backend.session().theme.name, "Tiny");
        }
        let json = serde_json::to_value(f.backend.import(&dir).unwrap()).unwrap();
        assert!(json["warnings"].as_array().is_some_and(|w| !w.is_empty()));
        assert_eq!(json["warnings"][0]["code"], "backplateLed");

        let junk = f.root.join("notes.turtheme");
        std::fs::write(&junk, b"not a theme").unwrap();
        assert!(f.backend.import(&junk).is_err());
        assert!(f.backend.import(&f.root.join("missing.yaml")).is_err());
        assert_eq!(
            f.backend.session().theme.name,
            "Tiny",
            "a failed import keeps the theme"
        );
    }

    #[test]
    fn images_are_checked_and_previewed() {
        let f = fixture("media");
        std::fs::create_dir_all(&f.root).unwrap();
        let png = f.root.join("Logo Final.png");
        image::RgbaImage::from_pixel(8, 8, image::Rgba([1, 2, 3, 255]))
            .save(&png)
            .unwrap();
        let text = f.root.join("notes.txt");
        std::fs::write(&text, b"hello").unwrap();
        assert_eq!(
            f.backend.add_image(&png).unwrap().reference,
            "assets/logo-final.png"
        );
        assert!(f.backend.add_image(&text).is_err());
        assert!(f.backend.add_image(&f.root.join("missing.png")).is_err());
        let assets = f.backend.assets();
        assert_eq!(assets.len(), 1);
        assert!(
            assets[0]
                .data_url
                .as_deref()
                .unwrap()
                .starts_with("data:image/png;base64,")
        );
        // An animated GIF is an image that can be a video background: it
        // comes with its poster.
        let gif = f.root.join("Ondas.gif");
        std::fs::write(&gif, crate::media::tests::gif(3)).unwrap();
        let added = f.backend.add_image(&gif).unwrap();
        assert_eq!(added.reference, "assets/ondas.gif");
        let listed = f.backend.assets();
        let ondas = listed.iter().find(|a| a.reference == "assets/ondas.gif");
        let ondas = ondas.unwrap();
        assert!(ondas.animated && ondas.data_url.is_some());
        assert_eq!(ondas.poster.as_deref(), Some("assets/ondas-poster.png"));
    }

    #[test]
    fn material_icons_keep_their_own_license_and_render_as_svg() {
        let f = fixture("material-icons");
        let svg = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" data-bezel-icon="mdi-water-pump" fill="#38bdf8"><path d="M4 4h16v16H4z"/></svg>"##;
        let added = f.backend.add_icon("mdi-water-pump", svg.into()).unwrap();
        assert_eq!(added.reference, "assets/mdi-water-pump.svg");
        let studio = f.backend.studio();
        let stored = studio.assets().get(&AssetRef(added.reference)).unwrap();
        assert!(bezel_render::svg_size(stored).is_some());
        let text = std::str::from_utf8(stored).unwrap();
        assert!(text.contains("Apache License"));
        assert!(text.contains("Pictogrammers"));
        assert!(text.contains("SVG root fill changed"));
        drop(studio);
        let assets = f.backend.assets();
        assert!(assets[0].icon_svg.is_some());
        assert!(
            assets[0]
                .data_url
                .as_deref()
                .unwrap()
                .starts_with("data:image/png;base64,")
        );
    }

    #[test]
    fn vector_icons_preserve_the_svg_and_have_rendered_thumbnails() {
        let f = fixture("vector-icons");
        let svg = r##"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24"><circle cx="12" cy="12" r="8" fill="#38bdf8"/></svg>"##;
        let added = f.backend.add_icon("cpu", svg.to_string()).unwrap();
        assert_eq!(added.reference, "assets/tabler-cpu.svg");
        let studio = f.backend.studio();
        let asset = AssetRef(added.reference);
        let stored = studio.assets().get(&asset).unwrap();
        let stored = std::str::from_utf8(stored).unwrap();
        assert!(stored.contains("MIT License"));
        assert!(stored.ends_with(svg));
        drop(studio);
        let assets = f.backend.assets();
        assert_eq!(assets[0].kind, "image");
        assert!(assets[0].icon_svg.as_deref().unwrap().ends_with(svg));
        assert!(
            assets[0]
                .data_url
                .as_deref()
                .unwrap()
                .starts_with("data:image/png;base64,")
        );
        assert!(f.backend.add_icon("../escape", svg.into()).is_err());
        assert!(f.backend.add_icon("broken", "not SVG".into()).is_err());
    }
}
