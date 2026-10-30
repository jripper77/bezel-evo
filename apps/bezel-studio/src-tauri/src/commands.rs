//! The `#[tauri::command]`s the UI invokes: each runs its [`Backend`] method
//! on a blocking thread (screens and files block) and opens the native file
//! dialogs the method needs.
//!
//! The window enters a command only through IPC: no code names a command
//! function but its definition and `generate_handler!` in `run`, only the
//! GIF commands `search_gifs`, `gif_preview` and `collect_gif` take the
//! invocation (`Request`), nothing here prints, logs or panics
//! (D-2026-10-01-gif-sticker-search-11, -12: only [`crate::diag`] says
//! anything, fixed text only), and only `open_fixed`, which `open_link` and
//! `open_guide` call, uses the system opener (-15); the source guard
//! `tests::nothing_in_the_app_forges_an_invocation` checks it.

use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use bezel_core::domain::job::Progress;
use bezel_core::domain::screen::Confirm;
use bezel_core::ports::ThemeLocation;
use bezel_themes::dto::ThemeDto;
use bezel_themes::native::EXTENSION;
use tauri::ipc::{Request, Response};
use tauri::{AppHandle, Manager as _, Runtime, State, WebviewWindow};
use tauri_plugin_dialog::DialogExt as _;
use tauri_plugin_opener::OpenerExt as _;

use crate::backend::{Backend, guide_url, link_url};
use crate::clock::now;
use crate::dto::{
    AddedDto, AddedMediaDto, AlbumAddedDto, AssetDto, CollectedDto, CollectedUsersDto, DevicesDto,
    FontCatalogDto, GifPageDto, ImportedDto, JobDto, KeyDto, MediaToolsDto, MonitorModeDto,
    PreferencesDto, PrepareDto, ProgressDto, RestartedDto, SampleDto, SavedDto, SensorDto,
    SessionDto, StandbyDto, StorageDto, ThemeEntryDto, VideoAutoDto, parse_orientation,
};
use crate::emit_progress;
use crate::gifs::{Gifs, KlipyKey, SharedGifs, Target, UserAsked};
use crate::manager::{
    Ask, CacheDto, CandidatesDto, ClearedDto, ConfirmedFileDto, DeleteReportDto, ManagedFileDto,
    ManagerOverviewDto, PlanDto, RunDto,
};
use crate::media::{BACKGROUND_EXTENSIONS, IMAGE_EXTENSIONS, MEDIA_EXTENSIONS};
use crate::messages::{ErrorCode, UiError, UiResult};
use crate::standby::{Asked, PHOTO_EXTENSIONS, PhotoAsked};
use crate::storage::ProgressThrottle;
use crate::studio::Motion;
use crate::tray::{LiveItem, TrayMenu};

/// State managed by Tauri.
pub type Shared = Arc<Backend>;

/// Picks one desktop pixel through Windows, independently of WebView2 support.
#[tauri::command]
pub async fn pick_screen_color(window: WebviewWindow) -> UiResult<Option<String>> {
    let owner = window.hwnd().map_err(UiError::system)?.0 as usize;
    tauri::async_runtime::spawn_blocking(move || {
        bezel_power::eyedropper::pick(owner).map_err(UiError::system)
    })
    .await
    .map_err(UiError::system)?
}

/// Runs `work` on the blocking pool with the backend.
async fn blocking<T: Send + 'static>(
    state: &State<'_, Shared>,
    work: impl FnOnce(&Backend) -> UiResult<T> + Send + 'static,
) -> UiResult<T> {
    let backend = Arc::clone(state);
    tauri::async_runtime::spawn_blocking(move || work(&backend))
        .await
        .map_err(UiError::system)?
}

/// The UI's `motion`: a video background plays and cards move unless it
/// says `false` (motion reduced, or the window hidden).
fn motion_of(motion: Option<bool>) -> Motion {
    match motion {
        Some(false) => Motion::Reduced,
        Some(true) | None => Motion::Allowed,
    }
}

/// A file chosen in the native open dialog, or `None` when cancelled.
fn pick_file<R: Runtime>(
    app: &AppHandle<R>,
    filter: &str,
    extensions: &[&str],
) -> UiResult<Option<PathBuf>> {
    app.dialog()
        .file()
        .add_filter(filter, extensions)
        .blocking_pick_file()
        .map(|p| p.into_path().map_err(UiError::system))
        .transpose()
}

/// Lists the connected screens and the panels in desktop mode (read-only).
#[tauri::command]
pub async fn list_devices(state: State<'_, Shared>) -> UiResult<DevicesDto> {
    blocking(&state, Backend::devices).await
}

/// Private memory of this installation/user, cached for at least five seconds.
#[tauri::command]
pub async fn memory_usage(
    state: State<'_, Shared>,
) -> UiResult<bezel_sensors::memory::MemoryReport> {
    blocking(&state, |_| {
        Ok(bezel_sensors::memory::sample(
            bezel_sensors::memory::Scope::Studio,
        ))
    })
    .await
}

/// Switches a panel in desktop mode back to USB monitor mode; `confirmed`
/// comes from the dialog that names it and says it is not validated on
/// hardware (D-2026-09-30-release-polish-8).
#[tauri::command]
pub async fn leave_desktop_mode(
    state: State<'_, Shared>,
    key: String,
    confirmed: bool,
) -> UiResult<MonitorModeDto> {
    blocking(&state, move |b| {
        b.leave_desktop_mode(&key, confirm_of(confirmed))
    })
    .await
}

/// Restarts a hung screen without a USB replug (about 10 s); `screen` comes
/// from the dialog that says what stops (D-2026-09-30-release-polish-13).
/// The tray's live item follows.
#[tauri::command]
pub async fn restart_screen(
    app: AppHandle,
    state: State<'_, Shared>,
    screen: String,
) -> UiResult<RestartedDto> {
    let result = blocking(&state, move |b| b.restart_screen(&screen, now())).await;
    let live = state.studio().any_live();
    if let Some(item) = app.try_state::<LiveItem>() {
        item.sync(live);
    }
    result
}

/// The machine's sensors.
#[tauri::command]
pub async fn sensor_catalog(state: State<'_, Shared>) -> UiResult<Vec<SensorDto>> {
    blocking(&state, Backend::catalog).await
}

#[tauri::command]
pub async fn select_screen(state: State<'_, Shared>, screen: String) -> UiResult<SessionDto> {
    blocking(&state, move |b| b.select_screen(&screen)).await
}

/// The latest readings and the live screen's state.
#[tauri::command]
pub async fn sample_sensors(state: State<'_, Shared>) -> UiResult<SampleDto> {
    blocking(&state, |b| Ok(b.sample())).await
}

/// Restart the configured Windows hardware reader on the blocking pool.
#[tauri::command]
pub async fn restart_libre_reader(state: State<'_, Shared>) -> UiResult<()> {
    blocking(&state, |b| {
        bezel_sensors::restart_libre_reader().map_err(UiError::system)?;
        if let Some(engine) = &b.engine {
            engine.request(
                "__options__",
                bezel_runtime::wire::RuntimeOp::SensorRestarted,
            )?;
        }
        Ok(())
    })
    .await
}

/// Search weather locations on the blocking pool, only on explicit user search.
#[tauri::command]
pub async fn search_weather_cities(
    state: State<'_, Shared>,
    query: String,
) -> UiResult<Vec<bezel_sensors::weather::City>> {
    blocking(&state, move |_| {
        bezel_sensors::weather::search_cities(&query).map_err(UiError::system)
    })
    .await
}

/// The sensors the library's list shows now (empty while it is hidden):
/// measured with the theme's, `net.ping` only while one shows it.
#[tauri::command]
pub async fn show_sensors(state: State<'_, Shared>, keys: Vec<String>) -> UiResult<()> {
    blocking(&state, move |b| {
        b.show_sensors(&keys);
        Ok(())
    })
    .await
}

/// The theme being edited.
#[tauri::command]
pub async fn editor_session(state: State<'_, Shared>) -> UiResult<SessionDto> {
    blocking(&state, |b| Ok(b.session())).await
}

/// Renders the UI's theme; the body is a 12-byte header (size, and when
/// the next picture is due) and RGBA. With `motion` (the default) a video
/// background plays; without it the poster shows and no decoder starts.
#[tauri::command]
pub async fn render_preview(
    state: State<'_, Shared>,
    theme: ThemeDto,
    motion: Option<bool>,
    card_rotation: Option<bool>,
    max: Option<u32>,
    trial_ms: Option<u32>,
    trial_hold: Option<bezel_runtime::wire::TrialHold>,
) -> UiResult<Response> {
    let motion = motion_of(motion);
    blocking(&state, move |b| {
        if trial_ms.is_some() || trial_hold.is_some() {
            return b
                .render_trial(
                    &theme,
                    now(),
                    motion,
                    max,
                    (trial_ms.unwrap_or(0), trial_hold),
                )
                .map(Response::new);
        }
        b.render_canvas(
            &theme,
            now(),
            std::time::Instant::now(),
            motion,
            card_rotation.unwrap_or(true),
            max,
        )
        .map(Response::new)
    })
    .await
}

/// Draws one element of the UI's theme on its own for the inspector
/// (D-2026-10-09-studio-redesign-fidelity-8): the same 12-byte header and
/// RGBA as `render_preview`, the element alone on a transparent canvas, its
/// longest side at most `max` (512 when absent, 16 to 2048).
#[tauri::command]
pub async fn render_element_preview(
    state: State<'_, Shared>,
    theme: ThemeDto,
    element: u32,
    max: Option<u32>,
    motion: Option<bool>,
    trial_ms: Option<u32>,
) -> UiResult<Response> {
    let motion = motion_of(motion);
    blocking(&state, move |b| {
        b.render_element_with(
            &theme,
            element,
            (max, trial_ms),
            now(),
            std::time::Instant::now(),
            motion,
        )
        .map(Response::new)
    })
    .await
}

/// Natural frame of content, measured with the native preview renderer.
#[tauri::command]
pub async fn measure_element(
    state: State<'_, Shared>,
    theme: ThemeDto,
    element: u32,
) -> UiResult<Option<bezel_themes::dto::BoxDto>> {
    blocking(&state, move |b| {
        b.measure_element(&theme, element, now(), std::time::Instant::now())
    })
    .await
}

/// What Auto turns the UI's theme's video background, and the video's size.
#[tauri::command]
pub async fn video_auto(state: State<'_, Shared>, theme: ThemeDto) -> UiResult<VideoAutoDto> {
    blocking(&state, move |b| b.video_auto(&theme)).await
}

/// Opens a page of the user guide (`page` in `language`) in the system's
/// browser, off the main thread: only the fixed addresses [`guide_url`]
/// knows, so the webview never navigates.
#[tauri::command]
pub async fn open_guide<R: Runtime>(
    app: AppHandle<R>,
    page: String,
    language: String,
) -> UiResult<()> {
    open_fixed(app, guide_url(&page, &language)?).await
}

/// Opens a page of the fixed list [`LINKS`](crate::backend::LINKS) (`link`
/// names it: `klipyPartnerPanel`) in the system's browser, off the main
/// thread.
#[tauri::command]
pub async fn open_link<R: Runtime>(app: AppHandle<R>, link: String) -> UiResult<()> {
    open_fixed(app, link_url(&link)?.to_string()).await
}

/// Opens only the shared app-owned diagnostics folder on an explicit click.
#[tauri::command]
pub async fn open_logs_folder<R: Runtime>(app: AppHandle<R>) -> UiResult<()> {
    open_fixed(app, "bezel:logs".into()).await
}

/// Opens a fixed app address or the app-owned diagnostics folder.
/// The sole system opener, called by [`open_link`], [`open_guide`] and
/// [`open_logs_folder`] only on an explicit user click
/// (D-2026-10-01-gif-sticker-search-15; the source guard checks it).
async fn open_fixed<R: Runtime>(app: AppHandle<R>, url: String) -> UiResult<()> {
    tauri::async_runtime::spawn_blocking(move || {
        if url == "bezel:logs" || url == "bezel:fonts" {
            let folder = if url == "bezel:fonts" {
                bezel_render::user_fonts_directory().ok_or("APPDATA is unavailable")?
            } else {
                bezel_power::diagnostics::logs_directory()
            };
            std::fs::create_dir_all(&folder).map_err(|e| e.to_string())?;
            app.opener()
                .open_path(folder.to_string_lossy().into_owned(), None::<&str>)
                .map_err(|e| e.to_string())
        } else {
            app.opener()
                .open_url(url, None::<&str>)
                .map_err(|e| e.to_string())
        }
    })
    .await
    .map_err(UiError::system)?
    .map_err(UiError::system)
}

/// Shows the UI's theme on the live screen now.
#[tauri::command]
pub async fn push_theme(state: State<'_, Shared>, theme: ThemeDto) -> UiResult<()> {
    blocking(&state, move |b| b.push(&theme, now())).await
}

/// Turns live mode on (on `screen`) or off; the tray's live item follows.
#[tauri::command]
pub async fn set_live(
    app: AppHandle,
    state: State<'_, Shared>,
    on: bool,
    screen: Option<String>,
) -> UiResult<()> {
    let result = blocking(&state, move |b| b.set_live(on, screen.as_deref(), now())).await;
    let live = state.studio().any_live();
    if let Some(item) = app.try_state::<LiveItem>() {
        item.sync(live);
    }
    result
}

/// Sets a screen's brightness, 0 to 100.
#[tauri::command]
pub async fn set_brightness(state: State<'_, Shared>, screen: String, percent: u8) -> UiResult<()> {
    blocking(&state, move |b| b.set_brightness(&screen, percent)).await
}

/// Hands a screen back to its own mode.
#[tauri::command]
pub async fn release_screen(state: State<'_, Shared>, screen: String) -> UiResult<()> {
    blocking(&state, move |b| b.release(&screen)).await
}

/// Saves the UI's theme; `save_as` asks where first. `None` when cancelled.
#[tauri::command]
pub async fn save_theme<R: Runtime>(
    app: AppHandle<R>,
    state: State<'_, Shared>,
    theme: ThemeDto,
    save_as: bool,
) -> UiResult<Option<SavedDto>> {
    let target = if save_as {
        let chosen = app
            .dialog()
            .file()
            .add_filter("Bezel", &[EXTENSION])
            .set_directory(state.library.user_dir())
            .set_file_name(format!("{}.{EXTENSION}", theme.name))
            .blocking_save_file();
        match chosen {
            None => return Ok(None),
            Some(path) => {
                let path = path.into_path().map_err(UiError::system)?;
                let location = ThemeLocation(path.display().to_string());
                // The user picked it: the window may save there from now on.
                state.library.grant(&location);
                Some(location)
            }
        }
    } else {
        None
    };
    blocking(&state, move |b| b.save(&theme, target).map(Some)).await
}

/// The theme library.
#[tauri::command]
pub async fn list_themes(state: State<'_, Shared>) -> UiResult<Vec<ThemeEntryDto>> {
    blocking(&state, |b| Ok(b.themes())).await
}

/// Copies a library theme into the user's folder as a new theme named
/// `name` (the UI translates "Copy of …"); a bundled theme is only read.
#[tauri::command]
pub async fn duplicate_theme(
    state: State<'_, Shared>,
    location: String,
    name: String,
) -> UiResult<ThemeEntryDto> {
    blocking(&state, move |b| b.duplicate_theme(&location, &name)).await
}

/// Exports the selected saved theme to a user-picked native package; cancellation writes nothing.
#[tauri::command]
pub async fn export_theme<R: Runtime>(
    app: AppHandle<R>,
    state: State<'_, Shared>,
    location: String,
) -> UiResult<Option<String>> {
    let source = location.clone();
    let name = blocking(&state, move |b| {
        b.library
            .entry(&ThemeLocation(source.clone()))
            .map(|e| e.theme.name)
            .ok_or_else(|| UiError::new(ErrorCode::NotInLibrary).arg("location", source))
    })
    .await?;
    let chosen = app
        .dialog()
        .file()
        .add_filter("Bezel", &[EXTENSION])
        .set_file_name(format!("{name}.{EXTENSION}"))
        .blocking_save_file();
    let Some(chosen) = chosen else {
        return Ok(None);
    };
    let target = ThemeLocation(
        chosen
            .into_path()
            .map_err(UiError::system)?
            .with_extension(EXTENSION)
            .display()
            .to_string(),
    );
    state.library.grant(&target);
    blocking(&state, move |b| {
        b.export_theme(&location, &target).map(Some)
    })
    .await
}

/// Removes an unused personal theme from the library.
#[tauri::command]
pub async fn delete_theme(state: State<'_, Shared>, location: String) -> UiResult<()> {
    blocking(&state, move |b| b.delete_theme(&location)).await
}

/// The thumbnail of a library theme as a PNG `data:` URL, drawn off the
/// session's lock when it is not kept yet; `None` when the theme cannot be
/// drawn (the gallery shows its placeholder).
#[tauri::command]
pub async fn theme_thumbnail(
    state: State<'_, Shared>,
    location: String,
) -> UiResult<Option<String>> {
    blocking(&state, move |b| b.thumbnail(&location, now())).await
}

/// Opens a theme of the library (or one picked in a dialog this session).
#[tauri::command]
pub async fn open_theme(state: State<'_, Shared>, location: String) -> UiResult<ThemeDto> {
    blocking(&state, move |b| b.open(&location)).await
}

/// Starts a blank theme sized for `screen`, in `orientation` (a theme.json
/// name) or the one [`Backend::new_theme`] picks.
#[tauri::command]
pub async fn new_theme(
    state: State<'_, Shared>,
    screen: Option<String>,
    name: Option<String>,
    orientation: Option<String>,
) -> UiResult<ThemeDto> {
    let orientation = orientation
        .as_deref()
        .map(|o| {
            parse_orientation(o)
                .ok_or_else(|| UiError::new(ErrorCode::UnknownOrientation).arg("orientation", o))
        })
        .transpose()?;
    blocking(&state, move |b| {
        let untitled = b.texts().untitled;
        b.new_theme(
            screen.as_deref(),
            name.as_deref().unwrap_or(untitled),
            orientation,
        )
    })
    .await
}

/// Theme files the import dialog offers: Bezel's own, the TURZX app's and
/// turing-smart-screen-python's `theme.yaml` (its folder comes along).
const IMPORT_EXTENSIONS: [&str; 4] = [EXTENSION, "turtheme", "yaml", "yml"];

/// Asks for a theme file and imports it. `None` when cancelled.
#[tauri::command]
pub async fn import_theme<R: Runtime>(
    app: AppHandle<R>,
    state: State<'_, Shared>,
) -> UiResult<Option<ImportedDto>> {
    let Some(path) = pick_file(&app, state.texts().themes, &IMPORT_EXTENSIONS)? else {
        return Ok(None);
    };
    blocking(&state, move |b| b.import(&path).map(Some)).await
}

/// Asks for an image and adds it to the theme. `None` when cancelled.
#[tauri::command]
pub async fn add_image<R: Runtime>(
    app: AppHandle<R>,
    state: State<'_, Shared>,
) -> UiResult<Option<AddedDto>> {
    let Some(path) = pick_file(&app, state.texts().images, IMAGE_EXTENSIONS)? else {
        return Ok(None);
    };
    blocking(&state, move |b| b.add_image(&path).map(Some)).await
}

/// Adds a video background or a dropped file: the file at `path` (a drop
/// on the canvas or the Media panel), else one asked for in the native
/// dialog (videos and GIFs). `None` when the dialog is cancelled.
#[tauri::command]
pub async fn add_media<R: Runtime>(
    app: AppHandle<R>,
    state: State<'_, Shared>,
    path: Option<String>,
) -> UiResult<Option<AddedMediaDto>> {
    let path = match path {
        Some(path) => PathBuf::from(path),
        None => match pick_file(&app, state.texts().videos, BACKGROUND_EXTENSIONS)? {
            Some(path) => path,
            None => return Ok(None),
        },
    };
    blocking(&state, move |b| b.add_media(&path).map(Some)).await
}

/// Adds an offline vector icon as a theme asset.
#[tauri::command]
pub async fn add_icon(state: State<'_, Shared>, name: String, svg: String) -> UiResult<AddedDto> {
    blocking(&state, move |b| b.add_icon(&name, svg)).await
}

/// The theme's assets with previews.
#[tauri::command]
pub async fn list_assets(state: State<'_, Shared>) -> UiResult<Vec<AssetDto>> {
    blocking(&state, |b| Ok(b.assets())).await
}

/// Keeps the media of copied elements, for a paste on any screen.
#[tauri::command]
pub async fn copy_media(state: State<'_, Shared>, assets: Vec<String>) -> UiResult<()> {
    blocking(&state, move |b| {
        b.copy_media(assets);
        Ok(())
    })
    .await
}

/// Adds the copied media to this screen's document: copied reference to the
/// reference here.
#[tauri::command]
pub async fn paste_media(
    state: State<'_, Shared>,
) -> UiResult<std::collections::BTreeMap<String, String>> {
    blocking(&state, |b| Ok(b.paste_media())).await
}

/// Removes unused media from this document, keeping bytes needed by Undo.
#[tauri::command]
pub async fn remove_asset(
    state: State<'_, Shared>,
    asset: String,
    theme: ThemeDto,
) -> UiResult<bool> {
    blocking(&state, move |b| b.remove_asset(&asset, &theme)).await
}

/// Preview a theme video or a catalogued copy; no device I/O or arbitrary paths.
#[tauri::command]
pub async fn video_preview(
    state: State<'_, Shared>,
    asset: Option<String>,
    screen: Option<String>,
    path: Option<String>,
) -> UiResult<Option<String>> {
    blocking(&state, move |b| match (asset, screen, path) {
        (Some(asset), None, None) => Ok(b.media_video(&asset)),
        (None, Some(screen), Some(path)) => Ok(b.manager_video(&screen, &path)),
        _ => Err(crate::messages::UiError::new(
            crate::messages::ErrorCode::InvalidInput,
        )),
    })
    .await
}

/// The font families themes can use, with their faces, and the family drawn
/// for a missing one (D-2026-10-16-studio-redesign-fidelity-28).
#[tauri::command]
pub async fn font_catalog(state: State<'_, Shared>) -> UiResult<FontCatalogDto> {
    blocking(&state, |b| Ok(b.font_catalog())).await
}

/// Raw bytes of a catalogued face or a validated font asset of this document.
#[tauri::command]
pub async fn font_face(
    state: State<'_, Shared>,
    family: String,
    weight: u16,
    italic: bool,
    asset: Option<String>,
) -> UiResult<Response> {
    blocking(&state, move |b| {
        Ok(Response::new(match asset {
            Some(asset) => b.theme_font_face(&asset),
            None => b.font_face(&family, weight, italic),
        }))
    })
    .await
}

/// Copies only a catalogued face into the current document, after the UI licence notice.
#[tauri::command]
pub async fn include_font(
    state: State<'_, Shared>,
    family: String,
    weight: u16,
    italic: bool,
) -> UiResult<AddedDto> {
    blocking(&state, move |b| b.include_font(&family, weight, italic)).await
}

/// Opens the fixed shared user font folder on an explicit click.
#[tauri::command]
pub async fn open_fonts_folder<R: Runtime>(app: AppHandle<R>) -> UiResult<()> {
    open_fixed(app, "bezel:fonts".into()).await
}

/// Whether the UI holds edits that are not saved: closing the window asks
/// first then ([`crate::on_close`]).
#[derive(Debug, Default)]
pub struct Unsaved(AtomicBool);

impl Unsaved {
    /// Whether edits are unsaved.
    pub fn get(&self) -> bool {
        self.0.load(Ordering::SeqCst)
    }
}

/// The UI tells whether it holds unsaved edits.
#[tauri::command]
pub fn set_unsaved(state: State<'_, Unsaved>, unsaved: bool) {
    state.0.store(unsaved, Ordering::SeqCst);
}

/// Closes the window once the UI settled its unsaved edits: it hides while
/// a screen is live (Bezel stays in the tray), else the app ends.
#[tauri::command]
pub fn close_window<R: Runtime>(
    window: WebviewWindow<R>,
    state: State<'_, Shared>,
) -> UiResult<()> {
    let live = state.studio().any_live();
    crate::prepare_light_on_close(&window, &state).map_err(UiError::system)?;
    match crate::on_close_for_runtime(live, false, crate::returns_to_light(&window)) {
        crate::OnClose::Hide => window.hide(),
        crate::OnClose::Ask | crate::OnClose::Close => window.destroy(),
    }
    .map_err(UiError::system)
}

/// Ends the app once the UI settled its unsaved edits (the tray's Quit).
#[tauri::command]
pub fn quit_app<R: Runtime>(app: AppHandle<R>) -> UiResult<()> {
    crate::exit_bezel(&app).map_err(UiError::system)
}

/// Whether Light starts at login (the installer's option too). A simulated
/// Studio has no Light: never.
#[tauri::command]
pub fn get_autostart<R: Runtime>(app: AppHandle<R>) -> UiResult<bool> {
    app.try_state::<bezel_power::login::LightLogin>()
        .map_or(Ok(false), |login| login.is_enabled())
        .map_err(UiError::system)
}

/// Starts Light at login (in the tray, showing the last theme live) or not.
#[tauri::command]
pub fn set_autostart<R: Runtime>(app: AppHandle<R>, on: bool) -> UiResult<()> {
    let login = app
        .try_state::<bezel_power::login::LightLogin>()
        .ok_or_else(|| UiError::system("no Light in a simulated Studio"))?;
    login.set(on).map_err(UiError::system)
}

// --------------------------------------------------------- preferences --

/// Explicit local permission to capture process audio for visible spectrum widgets.
#[tauri::command]
pub async fn set_audio_capture(state: State<'_, Shared>, on: bool) -> UiResult<()> {
    blocking(&state, move |b| {
        b.settings
            .update(|s| s.audio_capture = crate::settings::DefaultOn(on));
        b.sync_engine_options().map_err(UiError::from)
    })
    .await
}

/// Enables bounded diagnostic logs and the optional physical-screen overlay.
#[tauri::command]
pub fn set_debug(
    state: State<'_, Shared>,
    on: bool,
    show_readings: bool,
    corner: Option<crate::performance::Corner>,
) -> UiResult<()> {
    state
        .set_debug(on, show_readings, corner.unwrap_or_default())
        .map_err(|e| UiError::from(bezel_core::BezelError::Transport(e.to_string())))
}

/// The language chosen in the settings and the system's.
#[tauri::command]
pub fn preferences(state: State<'_, Shared>) -> PreferencesDto {
    state.preferences()
}

#[tauri::command]
pub async fn profiles(state: State<'_, Shared>) -> UiResult<bezel_runtime::profiles::Overview> {
    blocking(&state, |b| b.profiles()).await
}

#[tauri::command]
pub fn profiles_requested() -> bool {
    crate::OPEN_PROFILES.swap(false, Ordering::SeqCst)
}

#[tauri::command]
pub async fn save_profiles(
    app: AppHandle,
    state: State<'_, Shared>,
    config: bezel_runtime::profiles::Config,
) -> UiResult<()> {
    blocking(&state, move |b| b.save_profiles(config)).await?;
    if let Some(menu) = app.try_state::<TrayMenu>() {
        menu.refresh_profiles(&app);
    }
    Ok(())
}

/// Show saved themes for thirty seconds without modifying the open editor document.
#[tauri::command]
pub async fn try_themes(
    state: State<'_, Shared>,
    themes: std::collections::BTreeMap<String, String>,
    profile: Option<String>,
) -> UiResult<usize> {
    blocking(&state, move |backend| backend.try_themes(themes, profile)).await
}

/// Plays a profile transition once on the live screens, unsaved.
#[tauri::command]
pub async fn preview_transition(
    state: State<'_, Shared>,
    transition: bezel_runtime::profiles::TransitionSettings,
    themes: std::collections::BTreeMap<String, String>,
) -> UiResult<usize> {
    blocking(&state, move |b| b.preview_transition(transition, themes)).await
}

/// Uses `language` (`pt-BR`, `en`, or `None` for the system's) from now on,
/// in the tray too.
#[tauri::command]
pub fn set_language(
    app: AppHandle,
    state: State<'_, Shared>,
    language: Option<String>,
) -> UiResult<()> {
    state.set_language(language.as_deref())?;
    if let Some(tray) = app.try_state::<TrayMenu>() {
        tray.relabel(&state.texts());
        tray.refresh_profiles(&app);
    }
    Ok(())
}

/// Remembers which themes the Themes tab lists: `scope` (`screen`, `all`,
/// or `None`: the screen's when one is known) and `axis` (`all`,
/// `vertical`, `horizontal`).
#[tauri::command]
pub fn set_theme_filter(
    state: State<'_, Shared>,
    scope: Option<String>,
    axis: String,
) -> UiResult<()> {
    state.set_theme_filter(scope.as_deref(), &axis)
}

/// Measures `net.ping` against `ping_host` from now on.
#[tauri::command]
pub async fn set_sensor_options(state: State<'_, Shared>, ping_host: String) -> UiResult<()> {
    blocking(&state, move |b| b.set_sensor_options(&ping_host)).await
}

/// Asks for a folder. `None` when cancelled.
#[tauri::command]
pub async fn pick_folder<R: Runtime>(app: AppHandle<R>) -> UiResult<Option<String>> {
    app.dialog()
        .file()
        .blocking_pick_folder()
        .map(|p| p.into_path().map_err(UiError::system))
        .transpose()
        .map(|p| p.map(|p| p.display().to_string()))
}

// ------------------------------------------------------------- storage --

/// Event carrying a storage job's progress ([`ProgressDto`]): an upload's,
/// a storage manager plan's; `crate::emit_progress` sends it.
pub const PROGRESS_EVENT: &str = "storage-progress";

/// The answer of the UI's confirmation dialog (which names the file) as the
/// core takes it. Only these commands, the human-facing edge, make a
/// [`Confirm`] (D-2026-09-30-storage-video-1).
fn confirm_of(confirmed: bool) -> Confirm {
    if confirmed { Confirm::Yes } else { Confirm::No }
}

/// Capacity and files of a screen.
#[tauri::command]
pub async fn storage_overview(state: State<'_, Shared>, screen: String) -> UiResult<StorageDto> {
    blocking(&state, move |b| b.storage_overview(&screen, now())).await
}

/// Whether ffmpeg can convert videos, with install hints when it cannot.
#[tauri::command]
pub async fn media_tools(state: State<'_, Shared>) -> UiResult<MediaToolsDto> {
    blocking(&state, |b| Ok(b.media_tools())).await
}

/// Asks where ffmpeg is and uses it when it works. `None` when cancelled.
#[tauri::command]
pub async fn locate_ffmpeg<R: Runtime>(
    app: AppHandle<R>,
    state: State<'_, Shared>,
) -> UiResult<Option<MediaToolsDto>> {
    let Some(path) = app.dialog().file().blocking_pick_file() else {
        return Ok(None);
    };
    let path = path.into_path().map_err(UiError::system)?;
    blocking(&state, move |b| Ok(Some(b.locate_ffmpeg(&path)))).await
}

/// Asks for an image or a video to send to a screen. `None` when cancelled.
#[tauri::command]
pub async fn pick_media<R: Runtime>(
    app: AppHandle<R>,
    state: State<'_, Shared>,
) -> UiResult<Option<String>> {
    let filter = state.texts().media;
    Ok(pick_file(&app, filter, MEDIA_EXTENSIONS)?.map(|p| p.display().to_string()))
}

/// The preflight of sending the local file `source` to `medium` of `screen`.
#[tauri::command]
pub async fn prepare_upload(
    state: State<'_, Shared>,
    screen: String,
    source: String,
    medium: String,
) -> UiResult<PrepareDto> {
    blocking(&state, move |b| {
        b.prepare_upload(&screen, &PathBuf::from(source), &medium, now())
    })
    .await
}

/// Check an imported background video on storage without starting playback.
#[tauri::command]
pub async fn check_theme_video(
    state: State<'_, Shared>,
    screen: String,
) -> UiResult<Option<crate::dto::ThemeVideoDto>> {
    blocking(&state, move |b| b.check_theme_video(&screen, now())).await
}

/// The preflight of sending the theme's video to the live screen.
#[tauri::command]
pub async fn prepare_theme_video(state: State<'_, Shared>, screen: String) -> UiResult<PrepareDto> {
    blocking(&state, move |b| b.prepare_theme_video(&screen, now())).await
}

/// Runs a prepared upload; progress goes out as [`PROGRESS_EVENT`].
#[tauri::command]
pub async fn run_upload<R: Runtime>(
    app: AppHandle<R>,
    state: State<'_, Shared>,
    ticket: u64,
    overwrite: bool,
) -> UiResult<JobDto> {
    blocking(&state, move |b| {
        let mut throttle = ProgressThrottle::default();
        let mut report = |progress: Progress| {
            if throttle.pass(progress) {
                emit_progress(&app, ProgressDto::from(progress));
            }
        };
        b.run_upload(ticket, confirm_of(overwrite), now(), &mut report)
    })
    .await
}

/// Asks the running upload to stop.
#[tauri::command]
pub fn cancel_job(state: State<'_, Shared>) -> bool {
    state.cancel_job()
}

/// Deletes a stored file; `confirmed` comes from the dialog naming it.
#[tauri::command]
pub async fn delete_stored(
    state: State<'_, Shared>,
    screen: String,
    path: String,
    confirmed: bool,
) -> UiResult<()> {
    blocking(&state, move |b| {
        b.delete_stored(&screen, &path, confirm_of(confirmed), now())
    })
    .await
}

/// Plays a stored file.
#[tauri::command]
pub async fn play_stored(state: State<'_, Shared>, screen: String, path: String) -> UiResult<()> {
    blocking(&state, move |b| b.play_stored(&screen, &path, now())).await
}

/// Stops what the screen plays.
#[tauri::command]
pub async fn stop_playback(state: State<'_, Shared>, screen: String) -> UiResult<()> {
    blocking(&state, move |b| b.stop_playback(&screen, now())).await
}

/// Sets the boot media (`None`: the built-in screen) and the brightness
/// the screen starts with (`None`: the screen's own); `confirmed` comes from
/// the dialog naming both.
#[tauri::command]
pub async fn set_boot_media(
    state: State<'_, Shared>,
    screen: String,
    path: Option<String>,
    confirmed: bool,
    brightness: Option<u8>,
) -> UiResult<()> {
    blocking(&state, move |b| {
        let confirm = confirm_of(confirmed);
        b.set_boot_media(&screen, path.as_deref(), brightness, confirm, now())
    })
    .await
}

// ------------------------------------------ when the computer shuts down --

/// What a screen does when the computer shuts down: its choice, the four
/// options and what it offers (D-2026-10-03-power-off-standby-2).
#[tauri::command]
pub async fn standby_overview(state: State<'_, Shared>, screen: String) -> UiResult<StandbyDto> {
    blocking(&state, move |b| b.standby_overview(&screen, now())).await
}

/// Records the choice of a screen and writes its plan B (`sleep_minutes`
/// with `off`, `file` with `video`); `confirmed` comes from the dialog that
/// says what is written.
#[tauri::command]
pub async fn set_standby(
    state: State<'_, Shared>,
    screen: String,
    choice: String,
    sleep_minutes: Option<u32>,
    file: Option<String>,
    confirmed: bool,
) -> UiResult<StandbyDto> {
    let asked = Asked {
        choice,
        sleep_minutes,
        file,
    };
    blocking(&state, move |b| {
        b.set_standby(&screen, &asked, confirm_of(confirmed), now())
    })
    .await
}

/// Asks for a photo for a screen's album. `None` when cancelled.
#[tauri::command]
pub async fn pick_photo<R: Runtime>(
    app: AppHandle<R>,
    state: State<'_, Shared>,
) -> UiResult<Option<String>> {
    let filter = state.texts().images;
    Ok(pick_file(&app, filter, PHOTO_EXTENSIONS)?.map(|p| p.display().to_string()))
}

/// The photo `source` framed by `fit` (`cover` or `contain`) as the album
/// of `screen` shows it, as a PNG `data:` URL.
#[tauri::command]
pub async fn album_preview(
    state: State<'_, Shared>,
    screen: String,
    source: String,
    fit: String,
) -> UiResult<String> {
    blocking(&state, move |b| {
        b.album_preview(&screen, &PathBuf::from(source), &fit)
    })
    .await
}

/// Sends the photo `source`, framed like its preview, to the album of
/// `screen` as `name`; `confirmed` comes from the dialog that showed it and
/// named it, and `replace` from its own confirmation of replacing a photo
/// of that name (D-2026-10-03-power-off-standby-4 (3)).
#[tauri::command]
pub async fn album_add(
    state: State<'_, Shared>,
    screen: String,
    source: String,
    fit: String,
    name: String,
    confirmed: bool,
    replace: bool,
) -> UiResult<AlbumAddedDto> {
    let asked = PhotoAsked {
        source: PathBuf::from(source),
        fit,
        name,
        replace: confirm_of(replace),
    };
    blocking(&state, move |b| {
        b.album_add(&screen, &asked, confirm_of(confirmed), now())
    })
    .await
}

// ----------------------------------------------------- storage manager --

/// Both media of a screen next to the catalog of what Bezel sent.
#[tauri::command]
pub async fn manager_overview(
    state: State<'_, Shared>,
    screen: String,
) -> UiResult<ManagerOverviewDto> {
    blocking(&state, move |b| b.manager_overview(&screen, now())).await
}

/// A listed file's thumbnail from its local copy, as a `data:` URL.
#[tauri::command]
pub async fn manager_thumbnail(
    state: State<'_, Shared>,
    screen: String,
    path: String,
) -> UiResult<Option<String>> {
    blocking(&state, move |b| Ok(b.manager_thumbnail(&screen, &path))).await
}

/// Runs [`Backend::plan_transfer`] for `ask`.
async fn plan(
    state: State<'_, Shared>,
    screen: String,
    ask: Ask,
    overwrite: Vec<String>,
) -> UiResult<PlanDto> {
    blocking(&state, move |b| {
        b.plan_transfer(&screen, &ask, &overwrite, now())
    })
    .await
}

/// Plans moving `paths` to the medium `to`; `overwrite`: targets whose
/// replacement the user confirmed.
#[tauri::command]
pub async fn plan_move(
    state: State<'_, Shared>,
    screen: String,
    paths: Vec<String>,
    to: String,
    overwrite: Vec<String>,
) -> UiResult<PlanDto> {
    plan(state, screen, Ask::Move { paths, to }, overwrite).await
}

/// Plans copying `paths` to the medium `to`.
#[tauri::command]
pub async fn plan_copy(
    state: State<'_, Shared>,
    screen: String,
    paths: Vec<String>,
    to: String,
    overwrite: Vec<String>,
) -> UiResult<PlanDto> {
    plan(state, screen, Ask::Copy { paths, to }, overwrite).await
}

/// Plans renaming `path` to `new_name`.
#[tauri::command]
pub async fn plan_rename(
    state: State<'_, Shared>,
    screen: String,
    path: String,
    new_name: String,
    overwrite: Vec<String>,
) -> UiResult<PlanDto> {
    plan(state, screen, Ask::Rename { path, new_name }, overwrite).await
}

/// Plans restoring the cataloged entries `ids` onto the medium `to`.
#[tauri::command]
pub async fn plan_restore(
    state: State<'_, Shared>,
    screen: String,
    ids: Vec<String>,
    to: String,
    overwrite: Vec<String>,
) -> UiResult<PlanDto> {
    plan(state, screen, Ask::Restore { ids, to }, overwrite).await
}

/// Runs the plan the user confirmed; `confirmed` comes from the dialog that
/// listed every file. Progress goes out as [`PROGRESS_EVENT`], Cancel is
/// `cancel_job`.
#[tauri::command]
pub async fn run_plan<R: Runtime>(
    app: AppHandle<R>,
    state: State<'_, Shared>,
    ticket: u64,
    confirmed: bool,
) -> UiResult<RunDto> {
    blocking(&state, move |b| {
        let confirm = confirm_of(confirmed);
        b.run_plan(ticket, confirm, now(), &mut |p| emit_progress(&app, p))
    })
    .await
}

/// Deletes the files the user confirmed, one by one, each only while it has
/// the size the dialog listed; `confirmed` comes from that dialog, which
/// listed them and the space freed.
#[tauri::command]
pub async fn delete_files<R: Runtime>(
    app: AppHandle<R>,
    state: State<'_, Shared>,
    screen: String,
    files: Vec<ConfirmedFileDto>,
    confirmed: bool,
) -> UiResult<DeleteReportDto> {
    blocking(&state, move |b| {
        let confirm = confirm_of(confirmed);
        b.delete_files(&screen, &files, confirm, now(), &mut |p| {
            emit_progress(&app, p);
        })
    })
    .await
}

/// Asks for the originals of a screen file: media files, or one folder
/// (`folder`). Empty when cancelled.
#[tauri::command]
pub async fn pick_originals<R: Runtime>(
    app: AppHandle<R>,
    state: State<'_, Shared>,
    folder: bool,
) -> UiResult<Vec<String>> {
    let dialog = app.dialog().file();
    let picked = if folder {
        dialog.blocking_pick_folder().map(|p| vec![p])
    } else {
        let filter = state.texts().media;
        dialog
            .add_filter(filter, MEDIA_EXTENSIONS)
            .blocking_pick_files()
    };
    picked
        .unwrap_or_default()
        .into_iter()
        .map(|p| {
            p.into_path()
                .map(|p| p.display().to_string())
                .map_err(UiError::system)
        })
        .collect()
}

/// The originals of a screen file among the picked `sources`.
#[tauri::command]
pub async fn associate_candidates(
    state: State<'_, Shared>,
    screen: String,
    path: String,
    sources: Vec<String>,
) -> UiResult<CandidatesDto> {
    blocking(&state, move |b| {
        b.associate_candidates(&screen, &path, &sources, now())
    })
    .await
}

/// Associates a screen file with the original the user confirmed.
#[tauri::command]
pub async fn associate_original(
    state: State<'_, Shared>,
    screen: String,
    path: String,
    source: String,
    confirmed: bool,
) -> UiResult<ManagedFileDto> {
    blocking(&state, move |b| {
        b.associate_original(&screen, &path, &source, confirm_of(confirmed), now())
    })
    .await
}

/// The local copies in numbers.
#[tauri::command]
pub async fn cache_info(state: State<'_, Shared>) -> UiResult<CacheDto> {
    blocking(&state, Backend::cache_info).await
}

/// "Clear cache" (`deleted` or `all`); `confirmed` comes from the dialog
/// that listed the copies and their size.
#[tauri::command]
pub async fn clear_cache(
    state: State<'_, Shared>,
    scope: String,
    confirmed: bool,
) -> UiResult<ClearedDto> {
    blocking(&state, move |b| {
        b.clear_cache(&scope, confirm_of(confirmed))
    })
    .await
}

/// Sets the limit of the copies of deleted files, bytes.
#[tauri::command]
pub async fn set_cache_limit(state: State<'_, Shared>, bytes: u64) -> UiResult<CacheDto> {
    blocking(&state, move |b| b.set_cache_limit(bytes)).await
}

// --------------------------------------------------- GIFs and stickers --

/// Runs `work` on the blocking pool with the GIF state and the backend (a
/// request to the provider, files and the theme block).
async fn with_gifs<T: Send + 'static>(
    gifs: &State<'_, SharedGifs>,
    state: &State<'_, Shared>,
    work: impl FnOnce(&Gifs, &Backend) -> UiResult<T> + Send + 'static,
) -> UiResult<T> {
    let gifs = Arc::clone(gifs);
    blocking(state, move |b| work(&gifs, b)).await
}

/// Whether a KLIPY key is saved, and its last 4 characters (never the key).
#[tauri::command]
pub async fn klipy_key(gifs: State<'_, SharedGifs>, state: State<'_, Shared>) -> UiResult<KeyDto> {
    with_gifs(&gifs, &state, |g, _| g.key_status()).await
}

/// Saves the user's KLIPY key; nothing is sent to KLIPY. The window sends
/// it as a string, read straight into a [`KlipyKey`]: one that cannot be a
/// key is `invalidInput` (D-2026-10-01-gif-sticker-search-10).
#[tauri::command]
pub async fn save_klipy_key(
    gifs: State<'_, SharedGifs>,
    state: State<'_, Shared>,
    key: KlipyKey,
) -> UiResult<KeyDto> {
    with_gifs(&gifs, &state, move |g, _| g.save_key(key)).await
}

/// Deletes the saved KLIPY key.
#[tauri::command]
pub async fn remove_klipy_key(
    gifs: State<'_, SharedGifs>,
    state: State<'_, Shared>,
) -> UiResult<KeyDto> {
    with_gifs(&gifs, &state, |g, _| g.remove_key()).await
}

/// A page of GIFs or stickers (`kind`) for `text` (empty: the trending
/// ones), explicit results shown only with `explicit`, in the app's
/// language. The window's invocation (`request`) is the proof the user
/// asked: none, no request to the provider.
#[tauri::command]
pub async fn search_gifs(
    request: Request<'_>,
    gifs: State<'_, SharedGifs>,
    state: State<'_, Shared>,
    kind: String,
    text: String,
    page: u32,
    explicit: Option<bool>,
) -> UiResult<GifPageDto> {
    let asked = UserAsked::of(&request);
    with_gifs(&gifs, &state, move |g, b| {
        let explicit = explicit.unwrap_or(false);
        let query = crate::gifs::query(&kind, &text, page, explicit, b.language())?;
        g.search(&asked, &query)
    })
    .await
}

/// The preview of a result of the last search as a `data:` URL (its still
/// with `still`: a GIF's JPEG, a sticker's PNG), or `None`; the window's
/// invocation (`request`) is the proof the user asked.
#[tauri::command]
pub async fn gif_preview(
    request: Request<'_>,
    gifs: State<'_, SharedGifs>,
    state: State<'_, Shared>,
    id: String,
    still: Option<bool>,
) -> UiResult<Option<String>> {
    let asked = UserAsked::of(&request);
    with_gifs(&gifs, &state, move |g, _| {
        g.preview(&asked, &id, still.unwrap_or(false))
    })
    .await
}

/// Adds a result of the last search to the collection; the window's
/// invocation (`request`) is the proof the user asked.
#[tauri::command]
pub async fn collect_gif(
    request: Request<'_>,
    gifs: State<'_, SharedGifs>,
    state: State<'_, Shared>,
    id: String,
) -> UiResult<CollectedDto> {
    let asked = UserAsked::of(&request);
    with_gifs(&gifs, &state, move |g, _| g.collect(&asked, &id)).await
}

/// The collection, the last added first, previews still with `still`.
#[tauri::command]
pub async fn gif_collection(
    gifs: State<'_, SharedGifs>,
    state: State<'_, Shared>,
    still: Option<bool>,
) -> UiResult<Vec<CollectedDto>> {
    with_gifs(&gifs, &state, move |g, _| g.list(still.unwrap_or(false))).await
}

/// Renames an item of the collection.
#[tauri::command]
pub async fn rename_collected(
    gifs: State<'_, SharedGifs>,
    state: State<'_, Shared>,
    id: String,
    name: String,
) -> UiResult<CollectedDto> {
    with_gifs(&gifs, &state, move |g, _| g.rename(&id, &name)).await
}

/// The user's themes, and whether the open one, that hold an item's bytes.
#[tauri::command]
pub async fn collected_users(
    gifs: State<'_, SharedGifs>,
    state: State<'_, Shared>,
    id: String,
) -> UiResult<CollectedUsersDto> {
    with_gifs(&gifs, &state, move |g, b| g.users(b, &id)).await
}

/// Deletes an item of the collection; `confirmed` comes from the dialog
/// that named the themes using it.
#[tauri::command]
pub async fn delete_collected(
    gifs: State<'_, SharedGifs>,
    state: State<'_, Shared>,
    id: String,
    confirmed: bool,
) -> UiResult<()> {
    with_gifs(&gifs, &state, move |g, _| {
        g.delete(&id, confirm_of(confirmed))
    })
    .await
}

/// Copies an item of the collection into the theme as an image or its
/// background (`target`).
#[tauri::command]
pub async fn use_collected(
    gifs: State<'_, SharedGifs>,
    state: State<'_, Shared>,
    id: String,
    target: String,
) -> UiResult<AddedMediaDto> {
    with_gifs(&gifs, &state, move |g, b| {
        g.use_in_theme(b, &id, Target::parse(&target)?)
    })
    .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_a_confirmed_dialog_is_confirm_yes() {
        assert_eq!(confirm_of(true), Confirm::Yes);
        assert_eq!(confirm_of(false), Confirm::No);
    }

    #[test]
    fn motion_is_allowed_unless_the_ui_says_false() {
        assert_eq!(motion_of(None), Motion::Allowed);
        assert_eq!(motion_of(Some(true)), Motion::Allowed);
        assert_eq!(motion_of(Some(false)), Motion::Reduced);
    }
}
