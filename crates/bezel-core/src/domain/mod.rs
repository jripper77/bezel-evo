//! Domain types: device catalog, geometry, discovery, themes, sensors,
//! stored media, the framing of a video background, the archive of what
//! Bezel sent, the cleanup assistant, long-running jobs, GIF and sticker
//! search with the user's collection, what a screen does when the computer
//! shuts down, and the frame handed from one Bezel program to another.

pub mod animation;
pub mod archive;
pub mod catalog;
pub mod cleanup;
pub mod clock;
pub mod cover;
pub mod device;
pub mod discovery;
pub mod error;
pub mod frame;
pub mod framing;
pub mod geometry;
pub mod gifs;
pub mod handoff;
pub mod history;
pub mod job;
pub mod media;
pub mod needle;
pub mod pattern;
pub mod poster;
pub mod reconnect;
pub mod screen;
pub mod sensor;
pub mod standby;
pub mod storage;
pub mod theme;
pub mod transition;
pub mod visualization;

pub mod weather;

pub mod gradient;

pub mod playback;
pub mod spectrum;

pub mod value_color;

pub mod rotation;
pub mod sensor_motion;
pub mod text_case;
