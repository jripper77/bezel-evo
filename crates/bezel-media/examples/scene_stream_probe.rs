//! Reproducible, hardware-free mixed-output experiment. Uses Bezel's renderer,
//! persistent H.264 encoder, per-link capability selection and independent clocks.
//! All readings are synthetic. No USB/serial device is opened by this program.
//! cargo run --release -p bezel-media --example scene_stream_probe -- <ffmpeg.exe> <output-directory> [seconds]
use bezel_core::{
    BezelError, Result,
    domain::{
        catalog::model_by_id,
        clock::{Language, LocalTime},
        device::ModelId,
        frame::Frame,
        geometry::Orientation,
        history::{Histories, Tick},
        screen::{Brightness, ScreenIdentity},
        sensor::{Quantities, Reading, SensorKey, Snapshot},
        storage::FileName,
        theme::ElementKind,
    },
    ports::{Backdrop, EncodedScene, FrameRenderer, RenderContext, ScreenLink, ThemeStore},
};
use bezel_media::scene_output::{OutputMode, SceneOutput};
use bezel_render::SkiaRenderer;
use bezel_themes::native::{FsThemeStore, native_location};
use std::{
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant},
};

type AnyResult<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;
#[derive(Default)]
struct Stats {
    frames: u64,
    bytes: u64,
}
struct ProbeLink {
    identity: ScreenIdentity,
    encoded: Option<File>,
    counts: Arc<Mutex<Stats>>,
    delay: Duration,
}
impl ScreenLink for ProbeLink {
    fn identity(&self) -> &ScreenIdentity {
        &self.identity
    }
    fn set_brightness(&mut self, _: Brightness) -> Result<()> {
        Ok(())
    }
    fn set_orientation(&mut self, _: Orientation) -> Result<()> {
        Ok(())
    }
    fn encoded_scene(&mut self) -> Option<&mut dyn EncodedScene> {
        self.encoded.is_some().then_some(self)
    }
    fn present(&mut self, frame: &Frame) -> Result<()> {
        thread::sleep(self.delay);
        let mut stats = self
            .counts
            .lock()
            .map_err(|_| BezelError::Transport("probe lock".into()))?;
        stats.frames += 1;
        stats.bytes += frame.as_rgba().len() as u64;
        Ok(())
    }
    fn screen_off(&mut self) -> Result<()> {
        Ok(())
    }
    fn release(&mut self) -> Result<()> {
        Ok(())
    }
}
impl EncodedScene for ProbeLink {
    fn begin(&mut self, _: u8, _: &FileName) -> Result<usize> {
        Ok(bezel_media::h264::MAX_PICTURE)
    }
    fn send_picture(&mut self, data: &[u8]) -> Result<()> {
        self.encoded
            .as_mut()
            .ok_or_else(|| BezelError::Unsupported("no video input".into()))?
            .write_all(data)
            .map_err(|e| BezelError::Transport(e.to_string()))?;
        let mut stats = self
            .counts
            .lock()
            .map_err(|_| BezelError::Transport("probe lock".into()))?;
        stats.frames += 1;
        stats.bytes += data.len() as u64;
        Ok(())
    }
    fn end(&mut self) -> Result<()> {
        Ok(())
    }
}
fn snapshot(t: f64) -> Snapshot {
    let mut out = Snapshot::default();
    for (key, base, amplitude, period) in [
        ("cpu.usage", 48., 32., 2.4),
        ("gpu.usage", 55., 30., 3.1),
        ("memory.percent", 42., 12., 4.3),
        ("cpu.temperature", 55., 14., 2.4),
        ("gpu.temperature", 60., 12., 3.1),
        ("cpu.frequency", 4200., 500., 2.4),
    ] {
        if let Some(key) = SensorKey::new(key) {
            // Demo sensor readings arrive at 1 Hz. Linear interpolation is
            // display-only and creates motion between those synthetic readings.
            let a = t.floor();
            let f = t - a;
            let value = |x: f64| base + amplitude * (x / period).sin();
            out.insert(key, Reading::Value(value(a - 1.) * (1. - f) + value(a) * f));
        }
    }
    out
}
fn save(frame: &Frame, path: &Path) -> AnyResult<()> {
    image::RgbaImage::from_raw(
        frame.size().width,
        frame.size().height,
        frame.as_rgba().to_vec(),
    )
    .ok_or("invalid preview frame")?
    .save(path)?;
    Ok(())
}
fn run(
    ffmpeg: &Path,
    directory: &Path,
    root: &Path,
    name: &str,
    fps: u8,
    seconds: u32,
    origin: Instant,
) -> AnyResult<serde_json::Value> {
    let wide = name == "wide";
    let theme_path = root.join(if wide {
        "clarity-monitor-wide"
    } else {
        "clarity-monitor-round"
    });
    let (mut theme, assets) = FsThemeStore.load(&native_location(&theme_path))?;
    for element in &mut theme.elements {
        if let ElementKind::Text {
            content: bezel_core::domain::theme::TextContent::Static(text),
            ..
        } = &mut element.kind
        {
            if text == "CLARITY" {
                *text = "H264 DEMO".into();
            } else if text.contains("90 SEC") {
                *text = "12 SEC / DEMO HISTORY".into();
            }
        }
        if let ElementKind::Graph {
            history,
            interval_seconds,
            ..
        } = &mut element.kind
        {
            *history = 600;
            *interval_seconds = None;
        }
    }
    let counts = Arc::new(Mutex::new(Stats::default()));
    let model = model_by_id(ModelId(if wide { "turing-usb-8.8" } else { "turing-2.1" }))
        .ok_or("probe model missing")?;
    let link = ProbeLink {
        identity: ScreenIdentity {
            model,
            firmware: Some("SIMULATED, NOT HARDWARE".into()),
        },
        encoded: if wide {
            Some(File::create(directory.join("wide.h264"))?)
        } else {
            None
        },
        counts: counts.clone(),
        delay: if wide {
            Duration::ZERO
        } else {
            Duration::from_millis(65)
        },
    };
    // Both request the same high-rate path. The serial model falls back before
    // creating an encoder or issuing any video commands.
    let mut output = SceneOutput::h264(
        Box::new(link),
        theme.orientation,
        ffmpeg,
        fps,
        &FileName::parse("standby.h264").map_err(|e| format!("{e:?}"))?,
    )?;
    let initial_mode = output.mode();
    let mut renderer = SkiaRenderer::new();
    let mut histories = Histories::new(&theme.graph_series());
    for n in -600..0 {
        histories.push(&snapshot(f64::from(n) / f64::from(fps)), Tick::Untimed);
    }
    let quantities = Quantities::new();
    let start = Instant::now();
    let stop = start + Duration::from_secs(u64::from(seconds));
    let mut last_index = None;
    let mut last = None;
    let mut rendered = 0_u64;
    let mut render_seconds = 0.;
    let mut deliver_seconds = 0.;
    while Instant::now() < stop {
        let elapsed = start.elapsed().as_secs_f64();
        let index = (elapsed * f64::from(fps)).floor() as u64;
        if last_index == Some(index) {
            thread::sleep(Duration::from_millis(1));
            continue;
        }
        // Skip obsolete unencoded frames. Each display follows its own pacing.
        let first_index = last_index.map_or(index, |i| i + 1);
        for point in first_index..=index {
            histories.push(&snapshot(point as f64 / f64::from(fps)), Tick::Untimed);
        }
        last_index = Some(index);
        let readings = snapshot(index as f64 / f64::from(fps));
        let began = Instant::now();
        let frame = renderer.render(
            &theme,
            &assets,
            RenderContext {
                snapshot: &readings,
                histories: &histories,
                quantities: &quantities,
                time: LocalTime {
                    year: 2026,
                    month: 10,
                    day: 18,
                    hour: 12,
                    minute: 0,
                    second: (index / u64::from(fps) % 60) as u8,
                    weekday: 6,
                    millisecond: 0,
                },
                animation: origin.elapsed(),
                language: Language::English,
                backdrop: Backdrop::Poster,
                shown: &bezel_core::domain::transition::NOTHING_SHOWN,
            },
        )?;
        render_seconds += began.elapsed().as_secs_f64();
        let began = Instant::now();
        output.present(&frame)?;
        deliver_seconds += began.elapsed().as_secs_f64();
        if rendered == 0 {
            save(&frame, &directory.join(format!("{name}-first.png")))?;
        }
        rendered += 1;
        last = Some(frame);
    }
    let elapsed = start.elapsed().as_secs_f64();
    if let Some(frame) = last {
        save(&frame, &directory.join(format!("{name}-last.png")))?;
    }
    output.release()?;
    let mode = output.mode();
    let counts = counts.lock().map_err(|_| "probe lock")?;
    if wide && initial_mode != OutputMode::H264 || !wide && mode != OutputMode::UnsupportedFallback
    {
        return Err("wrong mixed-output selection".into());
    }
    Ok(
        serde_json::json!({"output": name, "model": model.id.0, "requestedFps": fps, "initialMode": format!("{initial_mode:?}"), "mode": format!("{mode:?}"), "frames": counts.frames, "bytes": counts.bytes, "elapsedSeconds": elapsed, "completedFps": counts.frames as f64 / elapsed, "meanRenderMs": render_seconds * 1000. / rendered as f64, "meanOutputMs": deliver_seconds * 1000. / rendered as f64, "hardware": false, "fallbackError": output.last_error()}),
    )
}
fn main() -> AnyResult<()> {
    let mut args = std::env::args_os().skip(1);
    let ffmpeg = PathBuf::from(args.next().ok_or("expected ffmpeg.exe path")?);
    let directory = PathBuf::from(args.next().ok_or("expected output directory")?);
    let seconds: u32 = args
        .next()
        .map(|s| s.to_string_lossy().parse())
        .transpose()?
        .unwrap_or(8);
    if !(1..=60).contains(&seconds) {
        return Err("duration must be 1..=60 seconds".into());
    }
    fs::create_dir_all(&directory)?;
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../themes");
    let origin = Instant::now();
    let reports = thread::scope(|scope| {
        let a = scope.spawn(|| run(&ffmpeg, &directory, &root, "wide", 50, seconds, origin));
        let b = scope.spawn(|| run(&ffmpeg, &directory, &root, "round", 50, seconds, origin));
        Ok::<_, Box<dyn std::error::Error + Send + Sync>>(vec![
            a.join().map_err(|_| "wide worker panic")??,
            b.join().map_err(|_| "round worker panic")??,
        ])
    })?;
    let report = serde_json::json!({"synthetic": true, "hardware": false, "description": "Independent display workers; slow native output deliberately delayed 65 ms", "outputs": reports});
    fs::write(
        directory.join("report.json"),
        serde_json::to_vec_pretty(&report)?,
    )?;
    println!("{}", serde_json::to_string_pretty(&report)?);
    if report["outputs"][0]["mode"] != "H264" {
        return Err("the wide encoder fell back; inspect report.json".into());
    }
    Ok(())
}
