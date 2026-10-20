//! The inspector's preview of one element on its own
//! (D-2026-10-09-studio-redesign-fidelity-8): the edited theme cut down to
//! that element (a group with its children, a card with its base and the
//! members of its active face) on a transparent canvas the size of what it
//! draws, then drawn by the session's renderer like any theme
//! ([`crate::Engine::inspect`]) and reduced to fit the size
//! the UI asks for. Nothing here draws: the cut theme goes through the same
//! renderer and readings as the preview of the whole theme.

use bezel_core::domain::frame::{Frame, Rgba};
use bezel_core::domain::geometry::Size;
use bezel_core::domain::theme::{Background, BoxF, Element, ElementId, Theme};
use image::RgbaImage;
use image::imageops::{self, FilterType};

/// The longest side of an element's preview when the UI names none, pixels.
pub const DEFAULT_SIDE: u32 = 512;

/// The longest side an element's preview may be asked for, pixels: the UI's
/// `max` is clamped to `MIN_SIDE..=MAX_SIDE`.
pub const MAX_SIDE: u32 = 2048;

/// The shortest longest side an element's preview may be asked for, pixels.
pub const MIN_SIDE: u32 = 16;

/// The longest side asked for, clamped ([`DEFAULT_SIDE`] when none).
pub fn side_of(max: Option<u32>) -> u32 {
    max.unwrap_or(DEFAULT_SIDE).clamp(MIN_SIDE, MAX_SIDE)
}

/// Whether `element` belongs to an element of `kept` (its group or its
/// card), leaving out the members of a card's faces other than the active
/// one.
fn owned_by(theme: &Theme, element: &Element, kept: &[ElementId]) -> bool {
    if element.group_parent.is_some_and(|p| kept.contains(&p)) {
        return true;
    }
    let Some(member) = &element.card_member else {
        return false;
    };
    if !kept.contains(&member.parent) {
        return false;
    }
    let active = theme
        .element(member.parent)
        .and_then(|p| p.card.as_ref())
        .map(|c| c.active_face);
    member.face.is_none_or(|face| Some(face) == active)
}

/// The ids of `root` and everything it holds, in no particular order.
fn subtree(theme: &Theme, root: ElementId) -> Vec<ElementId> {
    let mut kept = vec![root];
    loop {
        let more: Vec<ElementId> = theme
            .elements
            .iter()
            .filter(|e| !kept.contains(&e.id) && owned_by(theme, e, &kept))
            .map(|e| e.id)
            .collect();
        if more.is_empty() {
            return kept;
        }
        kept.extend(more);
    }
}

/// The smallest whole-pixel box around the boxes of `elements` that draw
/// (groups draw nothing); `fallback` when none does.
fn bounds(elements: &[Element], fallback: BoxF) -> (f32, f32, f32, f32) {
    let boxes: Vec<BoxF> = elements
        .iter()
        .filter(|e| !e.is_group)
        .map(|e| {
            if e.sensor_motion.is_some() && e.supports_sensor_motion() {
                let side = e.frame.width.hypot(e.frame.height);
                let (cx, cy) = e.frame.center();
                BoxF::new(cx - side / 2., cy - side / 2., side, side)
            } else {
                bezel_core::domain::rotation::rotated_bounds(e.frame, e.applied_rotation())
            }
        })
        .collect();
    let boxes = if boxes.is_empty() {
        vec![fallback]
    } else {
        boxes
    };
    let left = boxes.iter().map(|b| b.x).fold(f32::INFINITY, f32::min);
    let top = boxes.iter().map(|b| b.y).fold(f32::INFINITY, f32::min);
    let right = boxes
        .iter()
        .map(|b| b.x + b.width)
        .fold(f32::NEG_INFINITY, f32::max);
    let bottom = boxes
        .iter()
        .map(|b| b.y + b.height)
        .fold(f32::NEG_INFINITY, f32::max);
    (left.floor(), top.floor(), right.ceil(), bottom.ceil())
}

/// A whole number of pixels from 1 to [`MAX_SIDE`] (the renderer's canvas
/// stays within its limit whatever the element's box says).
fn pixels(length: f32) -> u32 {
    if length.is_finite() {
        length.clamp(1.0, MAX_SIDE as f32) as u32
    } else {
        1
    }
}

/// `theme` cut down to the element `id` with its properties as they are:
/// only it and what it holds (a group's children, a card's base and the
/// members of its active face), moved onto a transparent canvas just the
/// size of their boxes, in the theme's orientation. The element itself
/// shows even when hidden or inside a hidden group (it is the one asked
/// for); what it holds keeps its own visibility. A card keeps its active
/// face: no rotation, trigger or transition moves it in the preview.
/// `None` when the theme has no element `id`.
pub fn isolate(theme: &Theme, id: ElementId) -> Option<Theme> {
    let root = theme.element(id)?;
    let kept = subtree(theme, id);
    let mut elements: Vec<Element> = theme
        .elements
        .iter()
        .filter(|e| kept.contains(&e.id))
        .cloned()
        .collect();
    let (left, top, right, bottom) = bounds(&elements, root.frame);
    for element in &mut elements {
        element.rotation = element.applied_rotation();
        element.frame.x -= left;
        element.frame.y -= top;
        if let Some(card) = element.card.as_mut() {
            card.triggers.clear();
            card.rotation_seconds = None;
            card.transition = None;
        }
        if element.id == id {
            element.group_parent = None;
            element.card_member = None;
            element.visible = true;
        }
    }
    Some(Theme {
        name: theme.name.clone(),
        canvas: Size::new(pixels(right - left), pixels(bottom - top)),
        orientation: theme.orientation,
        background: Background::Color(Rgba::default()),
        elements,
        refresh_seconds: theme.refresh_seconds,
        palette: Vec::new(),
    })
}

/// Multiplies each pixel's color by its alpha (`undo`: divides it back), so
/// a reduction blends no color of the transparent pixels around an element.
fn premultiply(pixels: &mut [u8], undo: bool) {
    for px in pixels.as_chunks_mut::<4>().0 {
        let alpha = u32::from(px[3]);
        for channel in &mut px[..3] {
            let value = u32::from(*channel);
            *channel = if undo {
                (value * 255 + alpha / 2).checked_div(alpha).unwrap_or(0)
            } else {
                (value * alpha + 127) / 255
            }
            .min(255) as u8;
        }
    }
}

/// `frame` reduced so its longest side is at most `side` (never enlarged),
/// keeping its proportions and its transparency.
pub fn fit_within(frame: Frame, side: u32) -> Frame {
    let size = frame.size();
    let longest = size.width.max(size.height);
    if longest <= side {
        return frame;
    }
    let scale = f64::from(side) / f64::from(longest);
    let reduced = |length: u32| ((f64::from(length) * scale).round() as u32).max(1);
    let (width, height) = (reduced(size.width), reduced(size.height));
    let mut data = frame.as_rgba().to_vec();
    premultiply(&mut data, false);
    let Some(image) = RgbaImage::from_raw(size.width, size.height, data) else {
        return frame;
    };
    let mut small = imageops::resize(&image, width, height, FilterType::Triangle).into_raw();
    premultiply(&mut small, true);
    Frame::from_rgba(Size::new(width, height), small).unwrap_or(frame)
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use bezel_core::domain::geometry::Orientation;
    use bezel_core::domain::playback::{CardTrigger, TriggerSource};
    use bezel_core::domain::theme::{
        Card, CardDirection, CardEffect, CardMember, CardTransition, ElementKind, Paint, ShapeKind,
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

    fn theme(elements: Vec<Element>) -> Theme {
        let mut theme = Theme::blank("t", Size::new(480, 1920), Orientation::Landscape);
        theme.elements = elements;
        theme
    }

    fn ids(theme: &Theme) -> Vec<u32> {
        theme.elements.iter().map(|e| e.id.0).collect()
    }

    #[test]
    fn a_rotated_element_gets_a_canvas_of_its_rotated_box() {
        let mut e = shape(1, BoxF::new(50., 45., 60., 30.), Rgba::WHITE);
        e.rotation = 30.;
        let alone = isolate(&theme(vec![e]), ElementId(1)).unwrap();
        assert_eq!(alone.canvas, Size::new(68, 56));
        assert_eq!(alone.elements[0].frame, BoxF::new(4., 13., 60., 30.));
        assert_eq!(alone.elements[0].rotation, 30.);
    }

    #[test]
    fn a_sensor_rotor_preview_has_room_for_every_angle() {
        let mut e = shape(1, BoxF::new(50., 45., 60., 30.), Rgba::WHITE);
        e.sensor_motion = Some(bezel_core::domain::sensor_motion::SensorMotion {
            sensor: bezel_core::domain::sensor::SensorKey::new("gpu.fan").unwrap(),
            reference: 3000.,
            turns_per_second: 1.,
            inertia_ms: 600,
            reverse: false,
        });
        let alone = isolate(&theme(vec![e]), ElementId(1)).unwrap();
        assert!(alone.canvas.width >= 68 && alone.canvas.height >= 68);
        let (cx, cy) = alone.elements[0].frame.center();
        assert!((cx - alone.canvas.width as f32 / 2.).abs() <= 1.);
        assert!((cy - alone.canvas.height as f32 / 2.).abs() <= 1.);
    }

    #[test]
    fn one_element_alone_on_a_transparent_canvas_of_its_size() {
        let red = Rgba::opaque(255, 0, 0);
        let mut hidden = shape(1, BoxF::new(10.5, 20.25, 100.0, 50.0), red);
        hidden.visible = false;
        let t = theme(vec![
            hidden,
            shape(2, BoxF::new(0.0, 0.0, 300.0, 300.0), red),
        ]);
        let alone = isolate(&t, ElementId(1)).unwrap();
        assert_eq!(ids(&alone), [1], "the overlapping element is left out");
        assert_eq!(
            alone.canvas,
            Size::new(101, 51),
            "whole pixels around its box"
        );
        assert_eq!(alone.elements[0].frame, BoxF::new(0.5, 0.25, 100.0, 50.0));
        assert!(alone.elements[0].visible, "the one asked for shows");
        assert_eq!(alone.background, Background::Color(Rgba::default()));
        assert_eq!(alone.orientation, t.orientation, "no screen rotation");
        assert!(isolate(&t, ElementId(9)).is_none());
    }

    #[test]
    fn a_group_keeps_its_children_and_a_child_leaves_its_group() {
        let c = Rgba::WHITE;
        let mut group = shape(1, BoxF::new(0.0, 0.0, 999.0, 999.0), c);
        group.is_group = true;
        group.visible = false;
        let mut a = shape(2, BoxF::new(10.0, 10.0, 20.0, 20.0), c);
        a.group_parent = Some(ElementId(1));
        let mut b = shape(3, BoxF::new(50.0, 40.0, 10.0, 10.0), c);
        b.group_parent = Some(ElementId(1));
        b.visible = false;
        let t = theme(vec![
            group,
            a,
            b,
            shape(4, BoxF::new(0.0, 0.0, 5.0, 5.0), c),
        ]);
        let whole = isolate(&t, ElementId(1)).unwrap();
        assert_eq!(ids(&whole), [1, 2, 3]);
        assert_eq!(
            whole.canvas,
            Size::new(50, 40),
            "around the children, not the group"
        );
        assert!(!whole.elements[2].visible, "a hidden child stays hidden");
        let child = isolate(&t, ElementId(2)).unwrap();
        assert_eq!(ids(&child), [2]);
        assert_eq!(child.elements[0].group_parent, None);
        let mut empty = t.clone();
        empty.elements.retain(|e| e.id.0 == 1);
        assert_eq!(
            isolate(&empty, ElementId(1)).unwrap().canvas,
            Size::new(999, 999)
        );
    }

    #[test]
    fn a_card_keeps_its_base_and_its_active_face_only() {
        let c = Rgba::WHITE;
        let mut card = shape(1, BoxF::new(0.0, 0.0, 100.0, 100.0), c);
        card.card = Some(Card {
            triggers: vec![CardTrigger {
                interval_seconds: None,
                next_face: false,
                source: TriggerSource::Foreground,
                app: "game".into(),
                face: 0,
                priority: 1,
                return_seconds: 0,
                return_face: None,
            }],
            faces: vec!["A".into(), "B".into()],
            active_face: 1,
            rotation_seconds: Some(5),
            transition: Some(CardTransition {
                effect: CardEffect::Fade,
                direction: CardDirection::Left,
                duration_ms: 300,
                include_base: false,
            }),
        });
        let member = |id, face| {
            let mut e = shape(id, BoxF::new(10.0, 10.0, 10.0, 10.0), c);
            e.card_member = Some(CardMember {
                parent: ElementId(1),
                face,
            });
            e
        };
        let mut inside_b = shape(6, BoxF::new(5.0, 5.0, 5.0, 5.0), c);
        inside_b.group_parent = Some(ElementId(4));
        let mut inside_a = shape(7, BoxF::new(5.0, 5.0, 5.0, 5.0), c);
        inside_a.group_parent = Some(ElementId(3));
        let t = theme(vec![
            card,
            member(2, None),
            member(3, Some(0)),
            member(4, Some(1)),
            inside_b,
            inside_a,
        ]);
        let shown = isolate(&t, ElementId(1)).unwrap();
        assert_eq!(
            ids(&shown),
            [1, 2, 4, 6],
            "base and face B with what it holds"
        );
        let settings = shown.elements[0].card.as_ref().unwrap();
        assert!(settings.triggers.is_empty());
        assert_eq!(
            (settings.rotation_seconds, settings.transition),
            (None, None)
        );
        assert_eq!(settings.active_face, 1);
        let alone = isolate(&t, ElementId(3)).unwrap();
        assert_eq!(
            ids(&alone),
            [3, 7],
            "a member of another face, asked for, shows"
        );
        assert_eq!(alone.elements[0].card_member, None);
    }

    #[test]
    fn a_box_without_size_or_bounds_still_makes_a_canvas() {
        let t = theme(vec![shape(
            1,
            BoxF::new(0.0, 0.0, 0.0, f32::NAN),
            Rgba::WHITE,
        )]);
        assert_eq!(isolate(&t, ElementId(1)).unwrap().canvas, Size::new(1, 1));
        let huge = theme(vec![shape(1, BoxF::new(0.0, 0.0, 1e9, 2.0), Rgba::WHITE)]);
        assert_eq!(
            isolate(&huge, ElementId(1)).unwrap().canvas,
            Size::new(MAX_SIDE, 2)
        );
    }

    #[test]
    fn the_side_asked_for_is_clamped() {
        assert_eq!(side_of(None), DEFAULT_SIDE);
        assert_eq!(side_of(Some(1)), MIN_SIDE);
        assert_eq!(side_of(Some(100_000)), MAX_SIDE);
        assert_eq!(side_of(Some(300)), 300);
    }

    #[test]
    fn a_frame_is_reduced_to_fit_keeping_its_transparency() {
        let mut frame = Frame::filled(Size::new(400, 100), Rgba::default());
        for y in 0..100 {
            for x in 0..200 {
                let at = ((y * 400 + x) * 4) as usize;
                frame.as_rgba_mut()[at..at + 4].copy_from_slice(&[255, 0, 0, 255]);
            }
        }
        let small = fit_within(frame.clone(), 100);
        assert_eq!(small.size(), Size::new(100, 25));
        assert_eq!(small.pixel(10, 10), Some(Rgba::opaque(255, 0, 0)));
        assert_eq!(small.pixel(90, 10).map(|p| p.a), Some(0));
        let edge = small.pixel(50, 10).unwrap();
        assert!(edge.a == 0 || edge.r > 200, "no dark fringe: {edge:?}");
        assert_eq!(
            fit_within(frame.clone(), 400).size(),
            frame.size(),
            "never enlarged"
        );
    }
}
