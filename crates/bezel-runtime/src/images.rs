//! Latest immutable frames; cloning a snapshot never takes the engine mutex.
use crate::{element_preview, invalid, wire};
use bezel_core::{Result, domain::frame::Frame, ports::RuntimeImages};
use bezel_power::diagnostics::{LightEvent, light};
use std::{
    collections::BTreeMap,
    sync::{Arc, RwLock},
};
pub(crate) struct Snapshot {
    pub frame: Arc<Frame>,
    pub sequence: u64,
    pub next_ms: u32,
    pub render_millis: f64,
}
#[derive(Default)]
pub struct Images {
    frames: RwLock<BTreeMap<String, Arc<Snapshot>>>,
    viewed: std::sync::Mutex<BTreeMap<String, std::time::Instant>>,
}
impl Images {
    pub(crate) fn viewed(&self, key: &str) -> bool {
        self.viewed
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get(key)
            .is_some_and(|at| at.elapsed() < std::time::Duration::from_secs(2))
    }
    pub(crate) fn publish(&self, key: &str, snapshot: Snapshot) {
        self.frames
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .insert(key.into(), Arc::new(snapshot));
    }
    pub(crate) fn forget(&self, key: &str) {
        self.frames
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .remove(key);
    }
    pub fn latest(&self, key: &str, after: u64, max: u32) -> Result<wire::Image> {
        if max == 0 || max > 2320 {
            return Err(invalid("invalid canvas bound"));
        }
        let snapshot = self
            .frames
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get(key)
            .cloned();
        if snapshot.is_some() {
            let mut viewed = self
                .viewed
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            viewed.retain(|_, at| at.elapsed() < std::time::Duration::from_secs(2));
            if viewed.len() < 32 || viewed.contains_key(key) {
                viewed.insert(key.into(), std::time::Instant::now());
            }
        }
        light(LightEvent::Canvas, 0, 0.0, "request");
        let Some(snapshot) = snapshot else {
            return Ok(wire::Image {
                render_millis: 0.0,
                sequence: 0,
                next_ms: 16,
                frame: None,
            });
        };
        Ok(wire::Image {
            render_millis: snapshot.render_millis,
            sequence: snapshot.sequence,
            next_ms: snapshot.next_ms.min(33),
            frame: if after == snapshot.sequence {
                None
            } else {
                Some(element_preview::fit_within(
                    snapshot.frame.as_ref().clone(),
                    max,
                ))
            },
        })
    }
}
impl RuntimeImages for Images {
    fn image(&self, metadata: &[u8]) -> Result<(Vec<u8>, Vec<u8>)> {
        let answer = (|| {
            if metadata.len() > wire::MAX_METADATA {
                return Err(invalid("image metadata limit"));
            }
            let request: wire::RuntimeMessage =
                serde_json::from_slice(metadata).map_err(|_| invalid("invalid image request"))?;
            if request.runtime != 2 {
                return Err(invalid("runtime version"));
            }
            let wire::RuntimeOp::Image { after, max, .. } = request.op else {
                return Err(invalid("not an image request"));
            };
            self.latest(&request.scene, after, max)
        })();
        let (reply, pixels) = match answer {
            Ok(image) => {
                let (mut value, pixels) = wire::image(image.frame, image.sequence, image.next_ms);
                value["renderMillis"] = serde_json::json!(image.render_millis);
                (wire::Reply { value, error: None }, pixels)
            }
            Err(e) => (
                wire::Reply {
                    value: serde_json::Value::Null,
                    error: Some(e.to_string()),
                },
                Vec::new(),
            ),
        };
        Ok((
            serde_json::to_vec(&reply).map_err(|_| invalid("image response"))?,
            pixels,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bezel_core::domain::{frame::Rgba, geometry::Size};
    #[test]
    fn render_time_is_the_snapshot_measurement_even_when_pixels_are_unchanged() {
        let images = Images::default();
        images.publish(
            "screen",
            Snapshot {
                frame: Arc::new(Frame::filled(Size::new(2, 2), Rgba::BLACK)),
                sequence: 1,
                next_ms: 33,
                render_millis: 12.5,
            },
        );
        for after in [0, 1] {
            let request = wire::request(
                "screen",
                wire::RuntimeOp::Image {
                    after,
                    max: 32,
                    cards: true,
                },
            )
            .unwrap();
            let (metadata, pixels) = images.image(&request).unwrap();
            let value = wire::response(&metadata).unwrap();
            assert_eq!(value["renderMillis"], 12.5);
            assert_eq!(pixels.is_empty(), after == 1);
        }
        assert_eq!(images.latest("unknown", 0, 32).unwrap().render_millis, 0.0);
    }
}
