//! Shared inspector geometry used by the authoritative engine.
pub use bezel_runtime::element_preview::*;

#[cfg(test)]
pub(crate) mod tests {
    use bezel_core::domain::{
        frame::Rgba,
        theme::{BoxF, Element, ElementId, ElementKind, Paint, ShapeKind},
    };
    pub(crate) fn shape(id: u32, frame: BoxF, color: Rgba) -> Element {
        Element {
            color_by_value: None,
            visualization: None,
            value_transition: None,
            sensor_motion: None,
            rotation: 0.0,
            is_group: false,
            group_parent: None,
            card: None,
            card_member: None,
            id: ElementId(id),
            name: format!("shape {id}"),
            frame,
            opacity: 1.0,
            visible: true,
            locked: false,
            kind: ElementKind::Shape {
                image: None,
                video_window: false,
                fade: None,
                shape: ShapeKind::Rect { radius: 0.0 },
                fill: Some(Paint::solid(color)),
                stroke: None,
            },
        }
    }
}
