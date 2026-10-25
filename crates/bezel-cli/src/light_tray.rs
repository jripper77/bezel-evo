//! Windows tray runtime, without a WebView. Rendering runs off the UI thread.
use std::ffi::OsString;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use anyhow::Context;
use bezel_cli::{Cli, Command};
use bezel_power::handoff;
use bezel_sensors::memory::{self, Scope};
use clap::Parser;
use tao::event::Event;
use tao::event_loop::{ControlFlow, EventLoopBuilder};
use tao::platform::run_return::EventLoopExtRunReturn;
use tray_icon::menu::{CheckMenuItem, Menu, MenuEvent, MenuItem, Submenu};
use tray_icon::{Icon, MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};

/// The tray icon's pictures, shared with Studio's tray.
#[path = "../../../apps/bezel-studio/src-tauri/src/tray_icon.rs"]
mod tray_picture;

static OWNER_CONNECTOR: std::sync::OnceLock<bezel_power::owner::OwnerConnector> =
    std::sync::OnceLock::new();

pub(super) fn connector() -> Option<bezel_power::owner::OwnerConnector> {
    OWNER_CONNECTOR.get().cloned()
}

/// On-demand server, including a fresh Studio installation with no saved theme.
pub(super) fn run_owner() -> anyhow::Result<String> {
    let config = std::env::var_os("APPDATA").context("no user config directory")?;
    let settings = Path::new(&config).join("io.github.slipalison.bezel/settings.json");
    let mut args = vec![
        std::env::current_exe()?.into_os_string(),
        "run".into(),
        "__owner_idle__".into(),
        "--tray".into(),
        "--screens-config".into(),
        settings.into_os_string(),
    ];
    // follow_saved_theme below selects the configured theme when one exists.
    let mut cli = Cli::try_parse_from(&args)?;
    follow_saved_theme(&mut cli);
    if let Command::Run { theme, .. } = cli.command {
        args[2] = theme.into_os_string();
    }
    run(args)
}

enum Action {
    Menu(MenuEvent),
    Tray(TrayIconEvent),
}

// After editing, follow the newly saved Studio theme and screen. A standalone
// tray command without Studio settings keeps its original arguments.
fn follow_saved_theme(cli: &mut Cli) {
    let Some(appdata) = std::env::var_os("APPDATA") else {
        return;
    };
    let path = Path::new(&appdata).join("io.github.slipalison.bezel/settings.json");
    let Ok(bytes) = std::fs::read(path) else {
        return;
    };
    let Ok(settings) = serde_json::from_slice::<serde_json::Value>(&bytes) else {
        return;
    };
    let Command::Run {
        theme,
        target,
        ffmpeg,
        settings: sensor_settings,
        ..
    } = &mut cli.command
    else {
        return;
    };
    if let Some(saved) = settings["lastTheme"].as_str()
        && Path::new(saved).is_file()
    {
        *theme = saved.into();
    }
    if let Some(screen) = settings["liveScreen"].as_str() {
        target.screen = Some(screen.into());
    }
    *ffmpeg = settings["ffmpegPath"].as_str().map(Into::into);
    sensor_settings.ping_host = settings["pingHost"].as_str().map(Into::into);
    sensor_settings.presentmon_path = settings["presentmonPath"].as_str().map(Into::into);
}

pub(super) fn run(arguments: Vec<OsString>) -> anyhow::Result<String> {
    let executable = std::env::current_exe()?;
    let directory = executable
        .parent()
        .context("no executable directory")?
        .to_path_buf();
    let runtime_directory = handoff::runtime_directory(&executable)?;
    let Some(_instance) = handoff::light_instance(&runtime_directory)? else {
        return Ok(String::new());
    };
    let initial = Cli::try_parse_from(&arguments)?;
    let settings = match &initial.command {
        Command::Run {
            screens_config: Some(path),
            ..
        } => path.clone(),
        _ => Path::new(&std::env::var_os("APPDATA").unwrap_or_default())
            .join("io.github.slipalison.bezel/settings.json"),
    };
    let _log = bezel_power::diagnostics::LightLog::start(settings)
        .map_err(|_| eprintln!("bezel light: diagnostic log unavailable"))
        .ok();
    let physical: Arc<dyn bezel_core::ports::ScreenConnector + Send + Sync> = if initial.fake {
        Arc::new(crate::fake_connector())
    } else {
        Arc::new(bezel_devices::SystemConnector)
    };
    let bus: Arc<dyn bezel_core::ports::DeviceBus + Send + Sync> = if initial.fake {
        Arc::new(crate::fake_bus())
    } else {
        Arc::new(bezel_devices::SystemBus)
    };
    let owner = bezel_power::owner::Owner::new(physical);
    let _ = OWNER_CONNECTOR.set(owner.client(true));
    let mut screen_guard = handoff::light_screen(&runtime_directory)?;
    owner.set_enabled(screen_guard.is_some());
    let engine = Arc::new(std::sync::Mutex::new(make_engine(
        &initial,
        Arc::clone(&bus),
        &owner,
        &runtime_directory,
    )?));
    let pipe_engine: Arc<std::sync::Mutex<dyn bezel_core::ports::RuntimeEndpoint>> = engine.clone();
    let server = bezel_power::owner_pipe::Server::bind(&runtime_directory)?;
    let pipe_owner = owner.clone();
    let server_stop = Arc::new(AtomicBool::new(false));
    let pipe_stop = Arc::clone(&server_stop);
    let _server = std::thread::Builder::new()
        .name("bezel-light-pipe".into())
        .spawn(move || {
            if let Err(error) = server.serve_runtime(pipe_owner, bus, pipe_stop, Some(pipe_engine))
            {
                bezel_power::diagnostics::failure("server", &error);
                eprintln!("bezel light: screen owner stopped: {error}");
            }
        })?;
    let quit_file = runtime_directory.join("bezel-light-quit");
    // Only the instance owning the lock clears requests left from earlier runs.
    if quit_file.exists() {
        std::fs::remove_file(&quit_file)?;
    }
    let finite = matches!(
        initial.command,
        Command::Run {
            frames: Some(_),
            ..
        }
    );
    let simulate = initial.fake;
    let mut event_loop = EventLoopBuilder::<Action>::with_user_event().build();
    let menu = Menu::new();
    let locale = sys_locale::get_locale().unwrap_or_else(|| "en".into());
    let italian = locale.starts_with("it");
    let open = MenuItem::new(
        if italian {
            "Apri Studio"
        } else {
            "Open Studio"
        },
        true,
        None,
    );
    let quit = MenuItem::new(if italian { "Esci" } else { "Quit" }, true, None);
    let memory_item = MenuItem::new(
        memory::light_label(&memory::sample(Scope::Light), &locale),
        false,
        None,
    );
    menu.append_items(&[&open, &memory_item, &quit])?;
    let profile_file = match &initial.command {
        Command::Run {
            screens_config: Some(path),
            ..
        } => Some(bezel_runtime::profiles::ProfileFile::beside(path)),
        _ => None,
    };
    let language = match locale.as_str() {
        s if s.starts_with("it") => bezel_core::domain::clock::Language::Italian,
        s if s.starts_with("pt") => bezel_core::domain::clock::Language::PortugueseBr,
        _ => bezel_core::domain::clock::Language::English,
    };
    let labels = bezel_runtime::profiles::menu_labels(language);
    let profiles_menu = Submenu::new(labels[0], true);
    if profile_file.is_some() {
        menu.insert(&profiles_menu, 1)?;
    }
    let mut shown_profiles = None;
    let mut profiles_poll = Instant::now();
    let scale = event_loop
        .primary_monitor()
        .map_or(1.0, |monitor| monitor.scale_factor());
    let (rgba, width, height) =
        tray_picture::tray_rgba(scale).context("the tray icon's picture cannot be read")?;
    let icon = Icon::from_rgba(rgba, width, height)?;
    let tray = TrayIconBuilder::new()
        .with_id("bezel-light")
        .with_tooltip("Bezel Light")
        .with_icon(icon)
        .with_menu(Box::new(menu))
        // Explicit display lets the informational item refresh BEFORE the
        // native menu's blocking message loop starts.
        .with_menu_on_left_click(false)
        .with_menu_on_right_click(false)
        .build()?;
    let proxy = event_loop.create_proxy();
    MenuEvent::set_event_handler(Some(move |event| {
        let _ = proxy.send_event(Action::Menu(event));
    }));
    let proxy = event_loop.create_proxy();
    TrayIconEvent::set_event_handler(Some(move |event| {
        let _ = proxy.send_event(Action::Tray(event));
    }));
    let quit_requested = Arc::new(AtomicBool::new(false));
    let flag = Arc::clone(&quit_requested);
    ctrlc::set_handler(move || flag.store(true, Ordering::SeqCst))?;
    let mut worker: Option<std::thread::JoinHandle<anyhow::Result<String>>> = None;
    let mut stop = Arc::new(AtomicBool::new(false));
    let shutdown = Arc::new(AtomicBool::new(false));
    let mut pending_open = false;
    let mut pending_profiles = false;
    let mut opening: Option<(std::process::Child, Instant)> = None;
    let mut visible = true;
    let mut retry_at = Instant::now();
    let mut reload_saved = false;
    let mut reopen_after_pause = false;
    let studio_exe = directory.join("bezel-studio.exe");
    let mut fatal = None;
    let mut pause_watch = bezel_cli::light_recovery::PauseWatch::new(std::time::SystemTime::now());
    let mut sensor_recovery: Option<std::thread::JoinHandle<std::io::Result<()>>> = None;
    event_loop.run_return(|event, _, flow| {
        // Tao delivers LoopDestroyed synchronously inside WM_ENDSESSION,
        // then exits the process. Finish BEFORE returning from this callback.
        // The query/cancel phase does not deliver it and must leave Light live.
        if matches!(event, Event::LoopDestroyed) && bezel_power::session_ending() {
            shutdown.store(true, Ordering::SeqCst);
            stop.store(true, Ordering::SeqCst);
            let deadline = Instant::now() + Duration::from_millis(3500);
            while worker.as_ref().is_some_and(|thread| !thread.is_finished())
                && Instant::now() < deadline
            {
                std::thread::sleep(Duration::from_millis(10));
            }
            if worker.as_ref().is_some_and(|thread| !thread.is_finished()) {
                eprintln!("bezel light: shutdown deadline reached; screen sleep timer remains the fallback");
            } else if let Some(thread) = worker.take() {
                match thread.join() {
                    Ok(Ok(summary)) => eprint!("{summary}"),
                    Ok(Err(error)) => eprintln!("bezel light: shutdown failed: {error:#}"),
                    Err(_) => eprintln!("bezel light: shutdown worker panicked"),
                }
            }
            server_stop.store(true, Ordering::SeqCst);
            if let Err(error) = owner.finish_until(deadline, &mut bezel_cli::finish_shutdown) {
                eprintln!("bezel light: owner shutdown failed: {error}");
            }
            return;
        }
        *flow = ControlFlow::WaitUntil(Instant::now() + Duration::from_millis(100));
        if Instant::now() >= profiles_poll {
            profiles_poll = Instant::now() + Duration::from_secs(1);
            if let Some(file) = &profile_file && let Ok(overview) = file.overview()
                && shown_profiles.as_ref() != Some(&overview) {
                let config = &overview.config;
                while profiles_menu.remove_at(0).is_some() {}
                let automatic = CheckMenuItem::with_id("profile:auto", labels[1], true, config.automatic, None);
                let default = CheckMenuItem::with_id("profile:default", labels[2], true, !config.automatic && config.manual.is_none(), None);
                let _ = profiles_menu.append_items(&[&automatic, &default]);
                for p in &config.profiles {
                    let item = CheckMenuItem::with_id(format!("profile:{}", p.id), &p.name, true, !config.automatic && config.manual.as_ref() == Some(&p.id), None);
                    let _ = profiles_menu.append(&item);
                }
                let _ = profiles_menu.append(&MenuItem::with_id("profile:manage", labels[3], true, None));
                shown_profiles = Some(overview);
            }
        }
        if let Event::UserEvent(Action::Menu(ref event)) = event
            && let Some(id) = event.id.as_ref().strip_prefix("profile:")
            && id != "manage" && let Some(file) = &profile_file {
                if let Err(error) = file.select(if id == "auto" || id == "default" { None } else { Some(id) }, id == "auto") {
                    eprintln!("bezel light: profile selection failed: {error}");
                }
                profiles_poll = Instant::now();
        }
        if pause_watch.poll(std::time::SystemTime::now()) && !simulate && !finite
            && !quit_requested.load(Ordering::SeqCst)
        {
            bezel_power::diagnostics::light(bezel_power::diagnostics::LightEvent::SensorRestart,0,0.0,"resume");
            eprintln!("bezel light: long pause detected; reopening screens and hardware sensors");
            reopen_after_pause = true;
            stop.store(true, Ordering::SeqCst);
            reload_saved = true;
            if sensor_recovery.is_none() && directory.join("sensors/bezel-sensors-helper.exe").is_file() {
                match std::thread::Builder::new().name("bezel-resume-sensors".into())
                    .spawn(bezel_sensors::restart_libre_reader) {
                    Ok(thread) => sensor_recovery = Some(thread),
                    Err(error) => eprintln!("bezel light: sensor recovery could not start: {error}"),
                }
            }
        }
        if sensor_recovery.as_ref().is_some_and(|thread| thread.is_finished())
            && let Some(thread) = sensor_recovery.take() {
                match thread.join() {
                    Ok(Ok(())) => eprintln!("bezel light: sensor reader recovered after pause; fresh snapshot verified"),
                    Ok(Err(error)) => eprintln!("bezel light: sensor recovery failed: {error}"),
                    Err(_) => eprintln!("bezel light: sensor recovery thread panicked"),
                }
        }
        if matches!(event, Event::UserEvent(Action::Tray(TrayIconEvent::Click {
            button: MouseButton::Left | MouseButton::Right,
            button_state: MouseButtonState::Up,
            ..
        }))) {
            memory_item.set_text(memory::light_label(&memory::sample(Scope::Light), &locale));
            tray.show_menu();
        }
        let wants_open = match event {
            Event::UserEvent(Action::Menu(ref event)) if event.id.as_ref() == "profile:manage" => {
                pending_profiles = true;
                true
            }
            Event::UserEvent(Action::Menu(ref event)) if event.id == *quit.id() => {
                quit_requested.store(true, Ordering::SeqCst);
                false
            }
            Event::UserEvent(Action::Menu(ref event)) => event.id == *open.id(),
            Event::UserEvent(Action::Tray(TrayIconEvent::DoubleClick {
                button: MouseButton::Left,
                ..
            })) => true,
            _ => false,
        };
        if wants_open && studio_exe.is_file() {
            pending_open = true;
        }
        if quit_file.exists() {
            quit_requested.store(true, Ordering::SeqCst);
        }
        let studio_active = match handoff::studio_active(&runtime_directory) {
            Ok(active) => active,
            Err(error) => {
                fatal = Some(anyhow::Error::from(error));
                quit_requested.store(true, Ordering::SeqCst);
                true
            }
        };
        if studio_active && !reload_saved {bezel_power::diagnostics::observed_fallback(&runtime_directory);}
        if studio_active {
            reload_saved = true;
            stop.store(true, Ordering::SeqCst);
        }
        if studio_active && worker.is_none() && screen_guard.is_some() {
            owner.set_enabled(false);
            if let Ok(mut engine)=engine.lock() { engine.disconnect_links(); }
            match owner.clear() {
                Ok(()) => screen_guard = None,
                Err(error) => {
                    fatal = Some(anyhow::Error::from(error));
                    quit_requested.store(true, Ordering::SeqCst);
                }
            }
        } else if !studio_active && screen_guard.is_none() {
            screen_guard = handoff::light_screen(&runtime_directory).ok().flatten();
            owner.set_enabled(screen_guard.is_some());
        }
        if studio_active || quit_requested.load(Ordering::SeqCst) {
            stop.store(true, Ordering::SeqCst);
        }
        if worker.as_ref().is_some_and(|thread| thread.is_finished()) {
            if let Some(thread) = worker.take() {
                match thread.join() {
                    Ok(Ok(summary)) => {
                        eprint!("{summary}");
                    }
                    Ok(Err(error)) => {
                        eprintln!("bezel light: {error:#}");
                    }
                    Err(_) => {
                        eprintln!("bezel light: rendering worker panicked");
                    }
                }
            }
            retry_at = Instant::now() + if reload_saved { Duration::ZERO } else { Duration::from_secs(3) };
            if finite {
                quit_requested.store(true, Ordering::SeqCst);
            }
        }
        if reopen_after_pause && worker.is_none() {
            if let Ok(mut engine)=engine.lock() { engine.disconnect_links(); }
            if let Err(error) = owner.clear() {
                fatal = Some(anyhow::Error::from(error));
                quit_requested.store(true, Ordering::SeqCst);
            }
            reopen_after_pause = false;
        }
        if pending_open && !quit_requested.load(Ordering::SeqCst) {
            match std::process::Command::new(&studio_exe)
                .args(if pending_profiles { vec!["--profiles"] } else { vec![] })
                .current_dir(&directory)
                .spawn()
            {
                Ok(child) => opening = Some((child, Instant::now() + Duration::from_secs(30))),
                Err(error) => eprintln!("bezel light: Studio could not start: {error}"),
            }
            pending_open = false;
            pending_profiles = false;
        }
        if let Some((child, deadline)) = opening.as_mut()
            && (studio_active
                || child.try_wait().ok().flatten().is_some()
                || Instant::now() >= *deadline)
        {
            opening = None;
        }
        if visible == (studio_active || owner.editing()) {
            let _ = tray.set_visible(!(studio_active || owner.editing()));
            visible = !(studio_active || owner.editing());
        }
        if quit_requested.load(Ordering::SeqCst) {
            if worker.is_none() {
                *flow = ControlFlow::Exit;
            }
        } else if !studio_active
            && screen_guard.is_some()
            && !pending_open
            && opening.is_none()
            && worker.is_none()
            && sensor_recovery.is_none()
            && Instant::now() >= retry_at
        {
            if reload_saved {
                let reload=(|| -> anyhow::Result<()> {
                    let mut cli=Cli::try_parse_from(&arguments)?;
                    if !simulate { follow_saved_theme(&mut cli); }
                    let mut engine=engine.lock().map_err(|_|anyhow::anyhow!("runtime lock poisoned"))?;
                    load_saved_documents(&mut engine,&cli)
                })();
                if let Err(error)=reload { eprintln!("bezel light: saved document recovery: {error}"); }
            }
            reload_saved = false;
            owner.saved_ready();
            stop = Arc::new(AtomicBool::new(false));
            let flag = Arc::clone(&stop);
            let ending = Arc::clone(&shutdown);
            let engine = Arc::clone(&engine);
            let wake=engine.lock().map(|engine|engine.wake()).unwrap_or_default();
            match std::thread::Builder::new()
                .name("bezel-light-render".into())
                .spawn(move || {
                    while !flag.load(Ordering::SeqCst) && !ending.load(Ordering::SeqCst) {
                        let version=wake.version();
                        let delay = engine.lock().map_err(|_| anyhow::anyhow!("runtime lock poisoned"))?.tick();
                        match delay {
                            Ok(delay) => wake.wait(version,delay.min(Duration::from_millis(50))),
                            Err(_) => { eprintln!("bezel light: runtime update failed"); std::thread::sleep(Duration::from_millis(50)); }
                        }
                    }
                    Ok(String::new())
                }) {
                Ok(thread) => worker = Some(thread),
                Err(error) => {
                    fatal = Some(anyhow::Error::from(error));
                    quit_requested.store(true, Ordering::SeqCst);
                }
            }
        }
    });
    server_stop.store(true, Ordering::SeqCst);
    if !shutdown.load(Ordering::SeqCst) {
        owner.finish_until(Instant::now() + Duration::from_secs(5), &mut |link| {
            link.release()
        })?;
        owner.clear()?;
    }
    MenuEvent::set_event_handler(None::<fn(MenuEvent)>);
    TrayIconEvent::set_event_handler(None::<fn(TrayIconEvent)>);
    if let Some(error) = fatal {
        return Err(error);
    }
    Ok(String::new())
}

fn make_engine(
    cli: &Cli,
    bus: Arc<dyn bezel_core::ports::DeviceBus + Send + Sync>,
    owner: &bezel_power::owner::Owner,
    cache: &Path,
) -> anyhow::Result<bezel_runtime::Engine> {
    let settings = match &cli.command {
        Command::Run { settings, .. } => Some(settings),
        _ => None,
    };
    let mut engine = bezel_runtime::Engine::new(
        Box::new(crate::renderer_for(cli, crate::bundled_dir().as_deref())),
        crate::sensor_source(cli.fake, settings),
        Box::new(crate::FfmpegTranscoder::new(
            cli.command.ffmpeg().map(Path::to_path_buf),
        )),
        bus,
        Arc::new(owner.client(true)),
        bezel_cli::clock::language(),
        cache.join("engine-video"),
        bezel_cli::clock::now,
    )?;
    engine = engine.with_audio(Arc::new(bezel_sensors::audio::ProcessAudio));
    let fake = cli.fake;
    engine = engine.with_handoff(
        Arc::new(bezel_power::handoff::FileHandoffStore::new(cache)),
        || {
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
        },
    );
    engine = engine.with_adapters(
        Box::new(move |options| {
            let sensors: Box<dyn bezel_core::ports::SensorSource> = if fake {
                Box::new(bezel_sensors::FakeSensors::demo())
            } else {
                Box::new(bezel_sensors::SystemSensors::with_options(
                    bezel_sensors::SensorOptions {
                        ping_host: options.ping_host.clone(),
                        presentmon_path: options
                            .presentmon_path
                            .as_ref()
                            .map(std::path::PathBuf::from),
                    },
                ))
            };
            Ok((
                sensors,
                Box::new(crate::FfmpegTranscoder::new(
                    options.ffmpeg.as_ref().map(std::path::PathBuf::from),
                )),
            ))
        }),
        Box::new(bezel_sensors::FakeSensors::demo()),
    );
    // Audio capture is on unless the user's local settings turned it off.
    if let Command::Run {
        screens_config: Some(path),
        ..
    } = &cli.command
    {
        let settings = std::fs::metadata(path)
            .is_ok_and(|m| m.len() <= 1024 * 1024)
            .then(|| std::fs::read(path).ok())
            .flatten()
            .and_then(|bytes| serde_json::from_slice::<serde_json::Value>(&bytes).ok());
        engine.diagnostic_overlay(
            settings
                .as_ref()
                .filter(|s| {
                    s["debug"].as_bool() == Some(true)
                        && s["debugShowReadings"].as_bool() == Some(true)
                })
                .map(|s| serde_json::from_value(s["debugCorner"].clone()).unwrap_or_default()),
        );
        engine.audio_consent(
            settings.is_none_or(|settings| settings["audioCapture"].as_bool() != Some(false)),
        );
    }
    load_saved_documents(&mut engine, cli)?;
    if let Command::Run {
        screens_config: Some(path),
        ..
    } = &cli.command
    {
        engine = engine.with_profiles(bezel_runtime::profiles::ProfileFile::beside(path));
    }
    Ok(engine)
}

// Only startup and lock-protected legacy recovery read files again. Pipe editor
// leases restore documents inside the existing scenes and never enter this path.
fn load_saved_documents(engine: &mut bezel_runtime::Engine, cli: &Cli) -> anyhow::Result<()> {
    if let Command::Run {
        theme,
        target,
        screens_config,
        ..
    } = &cli.command
    {
        let settings = screens_config
            .as_ref()
            .and_then(|p| std::fs::read(p).ok())
            .and_then(|b| serde_json::from_slice::<serde_json::Value>(&b).ok());
        let mut themes = Vec::new();
        if let Some(settings) = settings.as_ref()
            && let Some(keys) = settings["liveScreens"].as_array()
        {
            for key in keys.iter().filter_map(serde_json::Value::as_str) {
                if let Some(path) = settings["screenThemes"][key]
                    .as_str()
                    .or_else(|| settings["lastTheme"].as_str())
                {
                    themes.push((key.to_owned(), std::path::PathBuf::from(path)));
                }
            }
        }
        if themes.is_empty()
            && settings
                .as_ref()
                .is_none_or(|s| s["liveScreens"].as_array().is_none())
            && theme.as_os_str() != "__owner_idle__"
            && let Some(key) = &target.screen
        {
            themes.push((key.clone(), theme.clone()));
        }
        engine.follow_saved_live(
            &themes
                .iter()
                .map(|(key, _)| key.clone())
                .collect::<Vec<_>>(),
        );
        for (key, path) in themes {
            match bezel_cli::theme::load(&bezel_themes::FsThemeStore, &path) {
                Ok(loaded) => {
                    engine.document(
                        &key,
                        0,
                        bezel_runtime::Document {
                            theme: loaded.theme,
                            assets: loaded.assets,
                        },
                        true,
                    )?;
                    if let Err(error) = engine.live(&key, true) {
                        eprintln!("bezel light: {key}: {error}");
                    }
                }
                Err(error) => eprintln!("bezel light: saved theme: {error}"),
            }
        }
    }
    Ok(())
}
