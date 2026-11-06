//! Dimming a screen while the computer's monitors are off and restoring it
//! when they come back (D-2026-11-06-display-idle-saver-1): the backlight
//! only, nothing else is sent. What the screen shows meanwhile (the moving
//! logo, [`crate::domain::idle::logo_box`]) is the runtime's.

use crate::Result;
use crate::domain::idle::{IDLE_BRIGHTNESS, IdleBrightness};
use crate::domain::screen::Brightness;
use crate::ports::ScreenLink;

/// Dims the screen behind `link` to [`IDLE_BRIGHTNESS`]: one
/// `set_brightness`, and nothing at all for a screen without a backlight
/// control. The level sent, if any.
pub fn dim(link: &mut dyn ScreenLink) -> Result<Option<Brightness>> {
    send(link, IDLE_BRIGHTNESS)
}

/// Restores the level of the screen behind `link`: the last one `levels`
/// recorded, else `fallback`; one `set_brightness`, and nothing for a
/// screen without a backlight control. The level sent, if any.
pub fn restore(
    link: &mut dyn ScreenLink,
    levels: &IdleBrightness,
    fallback: Brightness,
) -> Result<Option<Brightness>> {
    send(link, levels.to_restore(fallback))
}

fn send(link: &mut dyn ScreenLink, level: Brightness) -> Result<Option<Brightness>> {
    if !link.identity().model.capabilities.brightness {
        return Ok(None);
    }
    link.set_brightness(level)?;
    Ok(Some(level))
}
