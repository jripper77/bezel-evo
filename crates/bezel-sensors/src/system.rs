//! [`SystemSensors`]: the machine's real sensors behind the port, and the
//! [`SensorOptions`] of the sources that take settings.

use std::path::PathBuf;
use std::time::Instant;

use bezel_core::Result;
use bezel_core::domain::sensor::{SensorInfo, Snapshot, Wanted};
use bezel_core::ports::SensorSource;

use crate::gpu::{Alias, FoundGpu, number};
use crate::provider::Provider;

/// Settings of the sources that take them (the studio's settings and the
/// CLI's flags fill them in).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SensorOptions {
    /// Host `net.ping` measures the round trip to, only while a shown
    /// value uses it ([`SensorSource::want`]).
    pub ping_host: String,
    /// Path of Intel PresentMon's console executable read for `gpu.fps` on
    /// Windows; `None` searches the usual places.
    pub presentmon_path: Option<PathBuf>,
}

impl SensorOptions {
    /// The default `ping_host`: a public resolver that answers ICMP and TCP.
    pub const DEFAULT_PING_HOST: &str = "8.8.8.8";
}

impl Default for SensorOptions {
    fn default() -> Self {
        Self {
            ping_host: Self::DEFAULT_PING_HOST.to_string(),
            presentmon_path: None,
        }
    }
}

/// Measures this machine by composing the platform's providers. Discovery
/// starts when it is built; providers can add sensors when hardware becomes
/// ready later. Previously discovered keys remain available in the catalog.
/// `net.ping` sends packets only while [`SensorSource::want`] says it is
/// shown; until the first `want`, nothing is.
pub struct SystemSensors {
    providers: Vec<Box<dyn Provider>>,
    aliases: Vec<Alias>,
    catalog: Vec<SensorInfo>,
}

impl SystemSensors {
    /// Discovers every sensor this machine offers, with the default
    /// [`SensorOptions`]. Never fails: whatever cannot be read shows up as
    /// unavailable, with the reason.
    pub fn new() -> Self {
        Self::with_options(SensorOptions::default())
    }

    /// Like [`SystemSensors::new`], with `options` for the sources that
    /// take settings.
    pub fn with_options(options: SensorOptions) -> Self {
        let (mut providers, gpus) = platform();
        providers.push(Box::new(crate::fps::provider(&options)));
        providers.push(Box::new(crate::ping::Ping::new(&options.ping_host)));
        providers.push(Box::new(crate::weather::WeatherProvider::default()));
        Self::assemble(providers, gpus)
    }

    fn assemble(mut providers: Vec<Box<dyn Provider>>, gpus: Vec<FoundGpu>) -> Self {
        let gpus = number(gpus);
        providers.extend(gpus.providers);
        let mut catalog: Vec<SensorInfo> = providers
            .iter()
            .flat_map(|p| p.catalog())
            .chain(gpus.catalog)
            .collect();
        // Headline sensors first in each category, per-device ones after
        // (the order is presentation only; keys are what themes bind to).
        catalog.sort_by_key(|info| (info.category, rank(info.key.as_str())));
        Self {
            providers,
            aliases: gpus.aliases,
            catalog,
        }
    }
}

impl Default for SystemSensors {
    fn default() -> Self {
        Self::new()
    }
}

/// 0 for summary keys (`cpu.usage`, `disk.root.used`), 1 for per-device keys
/// (`cpu.3.usage`, `net.eno1.down`), 2 for raw chip sensors (`hwmon.*`).
fn rank(key: &str) -> u8 {
    if key.starts_with("hwmon.") || key.starts_with("lhm.") {
        return 2;
    }
    let second = key.split('.').nth(1).unwrap_or_default();
    u8::from(second.chars().any(|c| c.is_ascii_digit()))
}

fn platform() -> (Vec<Box<dyn Provider>>, Vec<FoundGpu>) {
    (crate::windows::providers(), crate::nvidia::discover())
}

impl SensorSource for SystemSensors {
    fn catalog(&mut self) -> Result<Vec<SensorInfo>> {
        let mut known: std::collections::HashSet<_> =
            self.catalog.iter().map(|info| info.key.clone()).collect();
        for info in self
            .providers
            .iter()
            .flat_map(|provider| provider.catalog())
        {
            if known.insert(info.key.clone()) {
                self.catalog.push(info);
            }
        }
        self.catalog
            .sort_by_key(|info| (info.category, rank(info.key.as_str())));
        Ok(self.catalog.clone())
    }

    /// Samples every provider at once, each on its own thread: a slow
    /// kernel read (a Wi-Fi chip's firmware query, an NVMe SMART log, an
    /// SMBus DIMM sensor) then costs its own latency, not the sum of all.
    fn sample(&mut self) -> Result<Snapshot> {
        let now = Instant::now();
        let parts: Vec<Snapshot> = std::thread::scope(|scope| {
            let running: Vec<_> = self
                .providers
                .iter_mut()
                .map(|provider| {
                    scope.spawn(move || {
                        let mut part = Snapshot::default();
                        provider.sample(now, &mut part);
                        part
                    })
                })
                .collect();
            running
                .into_iter()
                .filter_map(|thread| {
                    thread
                        .join()
                        .map_err(|_| {
                            tracing::error!("a sensor provider panicked; its sensors are missing")
                        })
                        .ok()
                })
                .collect()
        });
        let mut out = Snapshot::default();
        for part in &parts {
            out.media.extend(part.media.iter().cloned());
            out.media_available |= part.media_available;
            if part.activity_available {
                out.activity_available = true;
                out.applications.extend(part.applications.iter().cloned());
                out.foreground.clone_from(&part.foreground);
            }
            for (key, reading) in part.iter() {
                out.insert(key.clone(), reading.clone());
            }
        }
        for alias in &self.aliases {
            let reading = alias.reading(&out);
            out.insert(alias.key.clone(), reading);
        }
        Ok(out)
    }

    /// Passes what is shown to every provider: the ping starts or stops
    /// its probes (D-2026-09-30-release-polish-11).
    fn want(&mut self, wanted: &Wanted) {
        for provider in &mut self.providers {
            provider.want(wanted);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_includes_hardware_ready_after_the_first_sample() {
        use bezel_core::domain::sensor::{Category, Quantity};
        struct Late(bool);
        impl Provider for Late {
            fn catalog(&self) -> Vec<SensorInfo> {
                if !self.0 {
                    return vec![];
                }
                crate::provider::describe(
                    "lhm.psu.corsair.0.power.14",
                    Category::Board,
                    "Total Output",
                    Quantity::Watts,
                    "LibreHardwareMonitor",
                )
                .into_iter()
                .collect()
            }
            fn sample(&mut self, _: Instant, _: &mut Snapshot) {
                self.0 = true;
            }
        }
        let mut sensors = SystemSensors::assemble(vec![Box::new(Late(false))], vec![]);
        let initial = sensors.catalog().unwrap();
        assert!(
            !initial
                .iter()
                .any(|info| info.key.as_str() == "lhm.psu.corsair.0.power.14")
        );
        sensors.sample().unwrap();
        assert_eq!(sensors.catalog().unwrap().len(), initial.len() + 1);
        assert_eq!(sensors.catalog().unwrap().len(), initial.len() + 1);
    }

    #[test]
    fn summary_keys_rank_before_devices_and_chips() {
        assert_eq!(rank("cpu.usage"), 0);
        assert_eq!(rank("cpu.load.15"), 0);
        assert_eq!(rank("disk.root.used"), 0);
        assert_eq!(rank("cpu.3.usage"), 1);
        assert_eq!(rank("net.eno1.down"), 1);
        assert_eq!(rank("hwmon.k10temp.tctl"), 2);
        assert_eq!(rank("nodots"), 0);
    }

    #[test]
    fn options_default_to_the_public_resolver() {
        let options = SensorOptions::default();
        assert_eq!(options.ping_host, "8.8.8.8");
    }

    /// The real machine, read-only: whatever it has, every catalog entry is
    /// answered and nothing outside the catalog is.
    #[test]
    fn this_machine_answers_for_every_catalog_entry() {
        let mut sensors = SystemSensors::default();
        let catalog = sensors.catalog().unwrap();
        sensors.sample().unwrap();
        let snapshot = sensors.sample().unwrap();
        assert_eq!(snapshot.len(), catalog.len());
        for info in &catalog {
            assert!(
                snapshot.iter().any(|(k, _)| *k == info.key),
                "{} not sampled",
                info.key
            );
        }
    }
}
