//! Bezel sensors: the driven adapter behind the core's
//! [`SensorSource`](bezel_core::ports::SensorSource) port.
//!
//! [`SystemSensors`] composes Windows providers: sysinfo, LibreHardwareMonitor,
//! NVIDIA NVML, RTSS, media sessions and application activity. Ping runs only
//! while a shown value requests it; weather is cached independently.
//! [`FakeSensors`] replays scripted snapshots for tests and demos.
//!
//! What every provider guarantees (D-2026-09-30-sensors-1 and -4):
//! - a value that cannot be measured is `Reading::Unavailable(reason)`,
//!   never a guess;
//! - rates and usages come from counter deltas over the real time between
//!   two `sample` calls, so their first sample reads "warming up";
//! - a sample only reads local files (kernel, logs) and memory, never waits on
//!   the network (the ping thread stores its last result; a sample reads it).
//!
//! `unsafe` is denied (the workspace lint); the exceptions are the read-only
//! Win32 adapters for RTSS shared memory (`fps::rtss`) and installation-scoped
//! process memory (`memory`), each call with its `// SAFETY:` reasoning.
#![deny(unsafe_code)]

pub mod agent;
pub mod audio;
mod claude_usage;
mod fake;
mod fps;
mod gpu;

mod lhm;
mod libre_control;

pub mod memory;
mod nvidia;
mod ping;
mod provider;

mod sensor_log;
mod shared;
mod system;

pub mod weather;

mod windows;

pub use fake::FakeSensors;
pub use libre_control::restart_libre_reader;
pub use shared::SharedSensors;
pub use system::{SensorOptions, SystemSensors};
