//! Windows session shutdown and installation-scoped runtime ownership.
#![deny(unsafe_code)]

pub mod diagnostics;

#[cfg(windows)]
pub mod eyedropper;

pub mod handoff;

pub mod login;

mod session;

pub use session::session_ending;

pub mod owner;

pub mod owner_pipe;
pub mod owner_protocol;

pub mod owner_client;
