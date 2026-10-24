//! `gpu.fps` and related frame metrics: the frame rate of the game being
//! played, read from what already measures it. Bezel never hooks or
//! injects into a game: on Windows it prefers Intel PresentMon (started
//! on demand), falling back to RivaTuner Statistics Server.

#[cfg(windows)]
pub(crate) mod presentmon;
pub(crate) mod rtss;

use crate::SensorOptions;
use crate::provider::{Provider, describe, put};
use bezel_core::domain::sensor::{Category, Quantity, Reading, SensorInfo, Snapshot, Wanted, keys};
use std::time::{Duration, Instant};

/// Validated on real hardware 2026-10-18: acceptance tests E1-E9 passed
/// (AMD Ryzen 9 9950X3D, RTX 4090, PresentMon from Program Files).
pub(crate) const HARDWARE_VALIDATED: bool = true;

/// Readings older than this are stale: the game is paused, minimized or
/// closed, or logging stopped.
pub(crate) const MAX_AGE: Duration = Duration::from_secs(3);

/// Aggregated frame metrics from a source. Fields that a source cannot
/// provide are `None` and read as unavailable with a reason.
#[derive(Debug)]
pub(crate) struct FrameMetrics {
    pub fps: f64,
    pub frametime: Option<f64>,
    pub fps_low_1p: Option<f64>,
    pub fps_low_01p: Option<f64>,
    pub fps_min: Option<f64>,
    pub fps_max: Option<f64>,
    pub source: Option<String>,
    pub game: Option<String>,
    pub limiter: Option<String>,
}

/// Where the frame rate comes from.
pub(crate) trait FrameRateSource: Send {
    /// What the catalog shows as the sensor's source.
    fn describe(&self) -> String;
    /// The current frame metrics, or why there is none.
    fn read(&mut self) -> Result<FrameMetrics, String>;
    /// Called when the visibility of sensors changes. Sources that run
    /// external processes override this; the default ignores it.
    fn want(&mut self, wanted: bool) {
        let _ = wanted;
    }
}

/// The `gpu.fps` provider over one [`FrameRateSource`].
pub(crate) struct Fps {
    source: Box<dyn FrameRateSource>,
    is_wanted: bool,
}

impl Fps {
    /// `gpu.fps` read from `source`.
    pub(crate) fn new(source: impl FrameRateSource + 'static) -> Self {
        Self {
            source: Box::new(source),
            is_wanted: false,
        }
    }
}

impl Provider for Fps {
    fn catalog(&self) -> Vec<SensorInfo> {
        let mut source = self.source.describe();
        if !HARDWARE_VALIDATED {
            source.push_str(" (not validated on hardware)");
        }
        vec![
            describe(
                keys::GPU_FPS,
                Category::Gpu,
                "Game FPS (average)",
                Quantity::Number,
                source.clone(),
            ),
            describe(
                keys::GPU_FRAMETIME,
                Category::Gpu,
                "Game frame time (ms)",
                Quantity::Number,
                source.clone(),
            ),
            describe(
                keys::GPU_FPS_LOW_1P,
                Category::Gpu,
                "Game 1% low FPS",
                Quantity::Number,
                source.clone(),
            ),
            describe(
                keys::GPU_FPS_LOW_01P,
                Category::Gpu,
                "Game 0.1% low FPS",
                Quantity::Number,
                source.clone(),
            ),
            describe(
                keys::GPU_FPS_MIN,
                Category::Gpu,
                "Game minimum FPS",
                Quantity::Number,
                source.clone(),
            ),
            describe(
                keys::GPU_FPS_MAX,
                Category::Gpu,
                "Game maximum FPS",
                Quantity::Number,
                source.clone(),
            ),
            describe(
                keys::GPU_FPS_SOURCE,
                Category::Gpu,
                "Game FPS source",
                Quantity::Text,
                source.clone(),
            ),
            describe(
                keys::GPU_GAME,
                Category::Gpu,
                "Game name",
                Quantity::Text,
                source.clone(),
            ),
            describe(
                keys::GPU_LIMITER,
                Category::Gpu,
                "Estimated frame-rate limit",
                Quantity::Text,
                source,
            ),
        ]
        .into_iter()
        .flatten()
        .collect()
    }

    fn sample(&mut self, _now: Instant, out: &mut Snapshot) {
        match self.source.read() {
            Ok(m) => {
                let presentmon = m.source.as_deref() == Some("PresentMon");
                let statistic_reason = if presentmon {
                    "fps.statistics:Collecting enough recent frames"
                } else {
                    "fps.presentmonOnly:RTSS supplies FPS only"
                };
                put(out, keys::GPU_FPS, Reading::Value(m.fps));
                put(
                    out,
                    keys::GPU_FRAMETIME,
                    m.frametime.map_or(
                        Reading::Unavailable("fps.presentmonOnly:RTSS supplies FPS only".into()),
                        Reading::Value,
                    ),
                );
                put(
                    out,
                    keys::GPU_FPS_LOW_1P,
                    m.fps_low_1p.map_or(
                        Reading::Unavailable(statistic_reason.into()),
                        Reading::Value,
                    ),
                );
                put(
                    out,
                    keys::GPU_FPS_LOW_01P,
                    m.fps_low_01p.map_or(
                        Reading::Unavailable(statistic_reason.into()),
                        Reading::Value,
                    ),
                );
                put(
                    out,
                    keys::GPU_FPS_MIN,
                    m.fps_min.map_or(
                        Reading::Unavailable(statistic_reason.into()),
                        Reading::Value,
                    ),
                );
                put(
                    out,
                    keys::GPU_FPS_MAX,
                    m.fps_max.map_or(
                        Reading::Unavailable(statistic_reason.into()),
                        Reading::Value,
                    ),
                );
                put(
                    out,
                    keys::GPU_FPS_SOURCE,
                    m.source.map_or(
                        Reading::Unavailable("no source is measuring right now".into()),
                        Reading::Text,
                    ),
                );
                put(
                    out,
                    keys::GPU_GAME,
                    m.game.map_or(
                        Reading::Unavailable("fps.presentmonOnly:RTSS supplies FPS only".into()),
                        Reading::Text,
                    ),
                );
                put(
                    out,
                    keys::GPU_LIMITER,
                    m.limiter.map_or(
                        Reading::Unavailable(if presentmon {
                            "fps.estimate:No clear limiting component".into()
                        } else {
                            "fps.presentmonOnly:RTSS supplies FPS only".into()
                        }),
                        Reading::Text,
                    ),
                );
            }
            Err(why) => {
                let reason = if why.starts_with("fps.") {
                    why
                } else if why.to_ascii_lowercase().contains("access denied")
                    || why.contains("Performance Log Users")
                    || why.contains("Permission denied")
                {
                    format!("fps.permissions:{why}")
                } else {
                    format!("fps.unavailable:{why}")
                };
                put(out, keys::GPU_FPS, Reading::Unavailable(reason.clone()));
                put(
                    out,
                    keys::GPU_FRAMETIME,
                    Reading::Unavailable(reason.clone()),
                );
                put(
                    out,
                    keys::GPU_FPS_LOW_1P,
                    Reading::Unavailable(reason.clone()),
                );
                put(
                    out,
                    keys::GPU_FPS_LOW_01P,
                    Reading::Unavailable(reason.clone()),
                );
                put(out, keys::GPU_FPS_MIN, Reading::Unavailable(reason.clone()));
                put(out, keys::GPU_FPS_MAX, Reading::Unavailable(reason.clone()));
                put(
                    out,
                    keys::GPU_FPS_SOURCE,
                    Reading::Unavailable(reason.clone()),
                );
                put(out, keys::GPU_GAME, Reading::Unavailable(reason.clone()));
                put(out, keys::GPU_LIMITER, Reading::Unavailable(reason));
            }
        }
    }

    fn want(&mut self, wanted: &Wanted) {
        let wanted_now = wanted.contains(keys::GPU_FPS)
            || wanted.contains(keys::GPU_FRAMETIME)
            || wanted.contains(keys::GPU_FPS_LOW_1P)
            || wanted.contains(keys::GPU_FPS_LOW_01P)
            || wanted.contains(keys::GPU_FPS_MIN)
            || wanted.contains(keys::GPU_FPS_MAX)
            || wanted.contains(keys::GPU_FPS_SOURCE)
            || wanted.contains(keys::GPU_GAME)
            || wanted.contains(keys::GPU_LIMITER);

        if self.is_wanted != wanted_now {
            self.is_wanted = wanted_now;
            self.source.want(wanted_now);
        }
    }
}

/// This platform's `gpu.fps` provider. PresentMon falls back to RTSS
/// internally when it cannot start.
#[cfg(windows)]
pub(crate) fn provider(options: &SensorOptions) -> Fps {
    Fps::new(presentmon::PresentMon::new(options.presentmon_path.clone()))
}

/// This platform's `gpu.fps` provider: none is read here.
#[cfg(not(windows))]
pub(crate) fn provider(_options: &SensorOptions) -> Fps {
    Fps::new(Unsupported)
}

/// No frame-rate overlay Bezel reads on this system.
#[cfg(any(not(windows), test))]
pub(crate) struct Unsupported;

#[cfg(any(not(windows), test))]
impl FrameRateSource for Unsupported {
    fn describe(&self) -> String {
        "none on this system".to_string()
    }
    fn read(&mut self) -> Result<FrameMetrics, String> {
        Err("game FPS is read from PresentMon/RTSS on Windows".to_string())
    }
}

/// Test support: decodes the `.hex` fixtures (see `fixtures/rtss/`).
#[cfg(test)]
pub(crate) fn hex_fixture(text: &str) -> Vec<u8> {
    let number = |t: &str| usize::from_str_radix(t.trim_start_matches("0x"), 16).unwrap();
    let mut image = Vec::new();
    let mut at = 0usize;
    for line in text.lines() {
        let line = line.split('#').next().unwrap_or_default().trim();
        if let Some(size) = line.strip_prefix("size ") {
            image = vec![0u8; number(size.trim())];
            continue;
        }
        for token in line.split_whitespace() {
            if let Some(offset) = token.strip_prefix('@') {
                at = number(offset);
            } else {
                image[at] = u8::from_str_radix(token, 16).unwrap();
                at += 1;
            }
        }
    }
    image
}

#[cfg(test)]
mod tests {
    use super::*;
    use bezel_core::domain::sensor::SensorKey;

    struct Scripted(Result<f64, String>);

    impl FrameRateSource for Scripted {
        fn describe(&self) -> String {
            "a script".to_string()
        }
        fn read(&mut self) -> Result<FrameMetrics, String> {
            self.0.clone().map(|fps| FrameMetrics {
                fps,
                frametime: None,
                fps_low_1p: None,
                fps_low_01p: None,
                fps_min: None,
                fps_max: None,
                source: None,
                game: None,
                limiter: None,
            })
        }
    }

    fn sample(fps: &mut Fps) -> Reading {
        let mut out = Snapshot::default();
        fps.sample(Instant::now(), &mut out);
        assert_eq!(out.len(), 9);
        out.get(&SensorKey::new(keys::GPU_FPS).unwrap())
    }

    #[test]
    fn gpu_fps_lists_its_source() {
        let fps = Fps::new(Scripted(Ok(60.0)));
        let catalog = fps.catalog();
        assert_eq!(catalog.len(), 9);
        assert_eq!(catalog[0].key.as_str(), keys::GPU_FPS);
        assert_eq!(catalog[0].category, Category::Gpu);
        assert_eq!(catalog[0].quantity, Quantity::Number);
        assert_eq!(catalog[0].source, "a script");
    }

    #[test]
    fn values_and_reasons_pass_through() {
        assert_eq!(
            sample(&mut Fps::new(Scripted(Ok(0.0)))),
            Reading::Value(0.0)
        );
        assert_eq!(
            sample(&mut Fps::new(Scripted(Err("off".into())))),
            Reading::Unavailable("fps.unavailable:off".into())
        );
        let mut none = Fps::new(Unsupported);
        assert!(
            matches!(sample(&mut none), Reading::Unavailable(why) if why.contains("PresentMon"))
        );
        assert_eq!(none.catalog()[0].source, "none on this system");
    }

    #[test]
    fn hex_fixtures_decode_offsets_and_bytes() {
        let image = hex_fixture("size 0x8\n@0x2 # skip\nab cd\n@6 01\n");
        assert_eq!(image, vec![0, 0, 0xab, 0xcd, 0, 0, 1, 0]);
    }

    #[test]
    fn permission_diagnostics_have_a_stable_code_for_the_localized_ui() {
        let reason = sample(&mut Fps::new(Scripted(Err(
            "PresentMon: access denied; Performance Log Users".into(),
        ))));
        assert!(
            matches!(reason, Reading::Unavailable(message) if message.starts_with("fps.permissions:"))
        );
        let stale = sample(&mut Fps::new(Scripted(Err(
            "fps.stale:No recent game frames".into(),
        ))));
        assert_eq!(
            stale,
            Reading::Unavailable("fps.stale:No recent game frames".into())
        );
    }
}
