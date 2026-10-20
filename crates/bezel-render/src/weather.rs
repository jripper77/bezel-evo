//! Small vector condition icons, independent of installed fonts or assets.
//!
//! An icon is built from parts (sun disc, rays, cloud, fog strips, drops or
//! flakes, bolt, cross) in a 24-unit box. At rest the parts are drawn as one
//! icon; a moving icon (D-2026-10-20-studio-ux-v2-09) turns, shifts and
//! fades each part by its [`IconPose`], and the 3D style adds a shadow that
//! follows the cloud, a halo round the sun and a flash with the lightning.
use crate::{layer::Layer, paint::paint_for};
use bezel_core::domain::frame::Rgba;
use bezel_core::domain::theme::{BoxF, Paint};
use bezel_core::domain::weather::{IconPose, IconStyle, family};
use tiny_skia::{Path, PathBuilder, Pixmap, Rect, Stroke, Transform};

/// What a part of an icon is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Role {
    Disc,
    Rays,
    Cloud,
    Strip(usize),
    Drop(usize),
    Bolt,
    Cross,
}

/// A part of an icon with its colours (light and dark for the coloured
/// styles) and whether it is filled.
struct Part {
    role: Role,
    path: Path,
    light: Rgba,
    dark: Rgba,
    fill: bool,
}

const fn color(r: u8, g: u8, b: u8) -> Rgba {
    Rgba { r, g, b, a: 255 }
}

/// Centre of the sun, 24-unit coordinates.
fn sun_center(colored: bool) -> (f32, f32) {
    (12.0, if colored { 9.0 } else { 10.0 })
}

/// The parts of family `f`'s icon in `area`, in drawing order.
fn parts(f: usize, area: BoxF, colored: bool) -> Vec<Part> {
    let (x, y, s) = (area.x, area.y, area.width / 24.0);
    let mut out = Vec::new();
    let mut add = |role, pb: PathBuilder, light, dark, fill| {
        if let Some(path) = pb.finish() {
            out.push(Part {
                role,
                path,
                light,
                dark,
                fill,
            });
        }
    };
    if f == 0 || f == 1 {
        let (cx, cy) = sun_center(colored);
        let mut p = PathBuilder::new();
        p.push_circle(x + cx * s, y + cy * s, 4.0 * s);
        add(
            Role::Disc,
            p,
            color(255, 213, 79),
            color(239, 126, 22),
            true,
        );
        let mut p = PathBuilder::new();
        for i in 0..8 {
            let a = i as f32 * std::f32::consts::FRAC_PI_4;
            p.move_to(x + (cx + a.cos() * 6.0) * s, y + (cy + a.sin() * 6.0) * s);
            p.line_to(x + (cx + a.cos() * 8.0) * s, y + (cy + a.sin() * 8.0) * s);
        }
        add(
            Role::Rays,
            p,
            color(255, 193, 7),
            color(255, 148, 20),
            false,
        );
    }
    if (1..=6).contains(&f) {
        let mut p = PathBuilder::new();
        p.move_to(x + 5.0 * s, y + 16.0 * s);
        p.cubic_to(
            x - s,
            y + 16.0 * s,
            x + s,
            y + 8.0 * s,
            x + 7.0 * s,
            y + 10.0 * s,
        );
        p.cubic_to(
            x + 8.0 * s,
            y + 3.0 * s,
            x + 19.0 * s,
            y + 5.0 * s,
            x + 18.0 * s,
            y + 11.0 * s,
        );
        p.cubic_to(
            x + 24.0 * s,
            y + 10.0 * s,
            x + 24.0 * s,
            y + 16.0 * s,
            x + 19.0 * s,
            y + 16.0 * s,
        );
        if colored {
            p.close();
        } else {
            p.line_to(x + 5.0 * s, y + 16.0 * s);
        }
        add(
            Role::Cloud,
            p,
            color(224, 241, 255),
            color(103, 146, 188),
            true,
        );
        let (light, dark) = match f {
            6 => (color(255, 224, 66), color(255, 139, 30)),
            5 => (color(215, 244, 255), color(120, 204, 240)),
            _ => (color(55, 192, 245), color(29, 112, 214)),
        };
        match f {
            3 => {
                for i in 0..2 {
                    let py = y + (19.0 + i as f32 * 3.0) * s;
                    let mut p = PathBuilder::new();
                    p.move_to(x + 4.0 * s, py);
                    p.line_to(x + 20.0 * s, py);
                    add(Role::Strip(i), p, light, dark, false);
                }
            }
            4 | 5 => {
                for i in 0..3 {
                    let px = x + (6.0 + i as f32 * 6.0) * s;
                    let mut p = PathBuilder::new();
                    p.move_to(px, y + 19.0 * s);
                    p.line_to(px - s, y + 22.0 * s);
                    if f == 5 {
                        p.move_to(px - 1.5 * s, y + 20.0 * s);
                        p.line_to(px + 1.5 * s, y + 20.0 * s);
                    }
                    add(Role::Drop(i), p, light, dark, false);
                }
            }
            6 => {
                let mut p = PathBuilder::new();
                p.move_to(x + 13.0 * s, y + 17.0 * s);
                p.line_to(x + 10.0 * s, y + 21.0 * s);
                p.line_to(x + 14.0 * s, y + 21.0 * s);
                p.line_to(x + 11.0 * s, y + 24.0 * s);
                add(Role::Bolt, p, light, dark, false);
            }
            _ => {}
        }
    }
    if f == 7 {
        let mut p = PathBuilder::new();
        p.move_to(x + 5.0 * s, y + 5.0 * s);
        p.line_to(x + 19.0 * s, y + 19.0 * s);
        p.move_to(x + 19.0 * s, y + 5.0 * s);
        p.line_to(x + 5.0 * s, y + 19.0 * s);
        add(
            Role::Cross,
            p,
            color(160, 174, 192),
            color(80, 98, 120),
            false,
        );
    }
    out
}

/// Draws the icon of condition `code` in `area` at `pose` (at rest:
/// [`IconPose::REST`]).
pub(crate) fn draw(
    layer: &mut Layer<'_>,
    area: BoxF,
    code: u16,
    color: &Paint,
    style: IconStyle,
    pose: &IconPose,
) {
    let colored = matches!(style, IconStyle::Colored | IconStyle::Dimensional);
    let depth = style == IconStyle::Dimensional;
    let f = family(code);
    let parts = parts(f, area, colored);
    if *pose == IconPose::REST {
        if colored {
            draw_colored_at_rest(layer, area, &parts, depth);
        } else {
            draw_plain_at_rest(layer, area, &parts, color, style);
        }
        return;
    }
    faded(layer, area, pose.whole, |layer| {
        if depth {
            backdrop(layer, area, f, pose, &parts);
        }
        for part in &parts {
            let (transform, opacity) = place(part, area, pose, colored);
            let Some(path) = part.path.clone().transform(transform) else {
                continue;
            };
            faded(layer, area, opacity, |layer| {
                if colored {
                    component(layer, area, path, part.light, part.dark, part.fill, depth);
                } else {
                    plain(layer, area, &path, color, style);
                }
            });
        }
        if depth && pose.flash > 0.0 {
            let mut flash = Rgba::WHITE;
            flash.a = (pose.flash.clamp(0.0, 1.0) * 255.0).round() as u8;
            if let Some(rect) = Rect::from_xywh(area.x, area.y, area.width, area.height) {
                let path = PathBuilder::from_rect(rect);
                layer.fill(&path, &crate::paint::solid(flash));
            }
        }
    });
}

/// Where `part` goes and how opaque it is at `pose`.
fn place(part: &Part, area: BoxF, pose: &IconPose, colored: bool) -> (Transform, f32) {
    let (s, k) = (area.width / 24.0, area.width / 64.0);
    let (cx, cy) = sun_center(colored);
    let (cx, cy) = (area.x + cx * s, area.y + cy * s);
    match part.role {
        Role::Disc => (
            Transform::from_translate(cx, cy)
                .pre_scale(1.0 + pose.disc, 1.0 + pose.disc)
                .pre_translate(-cx, -cy),
            1.0,
        ),
        Role::Rays => (Transform::from_rotate_at(pose.rays, cx, cy), 1.0),
        Role::Cloud => (Transform::from_translate(pose.cloud * k, 0.0), 1.0),
        Role::Strip(i) => {
            let (dx, opacity) = pose.strips[i.min(3)];
            (Transform::from_translate(dx * k, 0.0), opacity)
        }
        Role::Drop(i) => {
            let (dx, dy, opacity) = pose.drops[i.min(2)];
            (Transform::from_translate(dx * k, dy * k), opacity)
        }
        Role::Bolt => (Transform::identity(), pose.bolt),
        Role::Cross => (Transform::identity(), 1.0),
    }
}

/// The 3D extras under a moving icon: a shadow on the ground that follows
/// the cloud and a halo round the sun.
fn backdrop(layer: &mut Layer<'_>, area: BoxF, f: usize, pose: &IconPose, parts: &[Part]) {
    let k = area.width / 64.0;
    let follow = if (1..=6).contains(&f) {
        pose.cloud
    } else {
        0.0
    };
    let (cx, cy) = (area.x + (33.0 + follow) * k, area.y + 58.0 * k);
    if let Some(oval) = Rect::from_xywh(cx - 16.0 * k, cy - 2.6 * k, 32.0 * k, 5.2 * k)
        && let Some(shadow) = PathBuilder::from_oval(oval)
    {
        layer.fill(
            &shadow,
            &crate::paint::solid(Rgba {
                r: 0,
                g: 0,
                b: 0,
                a: 50,
            }),
        );
    }
    if pose.halo > 0.0 && parts.iter().any(|p| p.role == Role::Disc) {
        let s = area.width / 24.0;
        let (sx, sy) = sun_center(true);
        let mut glow = color(255, 213, 79);
        glow.a = (pose.halo.clamp(0.0, 1.0) * 255.0).round() as u8;
        if let Some(halo) = PathBuilder::from_circle(area.x + sx * s, area.y + sy * s, 5.5 * s) {
            layer.fill(&halo, &crate::paint::solid(glow));
        }
    }
}

/// Runs `draw` at `opacity`: straight onto the layer when opaque, through a
/// buffer round `area` otherwise; nothing when invisible.
pub(crate) fn faded(
    layer: &mut Layer<'_>,
    area: BoxF,
    opacity: f32,
    draw: impl FnOnce(&mut Layer<'_>),
) {
    if opacity >= 0.999 {
        draw(layer);
        return;
    }
    if opacity <= 0.001 {
        return;
    }
    let margin = area.width.max(area.height) * 0.5;
    let (rx, ry) = (
        (area.x - margin).floor() as i32,
        (area.y - margin).floor() as i32,
    );
    let (w, h) = (
        (area.width + 2.0 * margin).ceil() as u32 + 1,
        (area.height + 2.0 * margin).ceil() as u32 + 1,
    );
    let Some(mut buffer) = Pixmap::new(w, h) else {
        return;
    };
    let mut sub = Layer {
        pixmap: buffer.as_mut(),
        x: rx,
        y: ry,
    };
    draw(&mut sub);
    let scale = opacity.clamp(0.0, 1.0);
    for channel in buffer.data_mut() {
        *channel = (f32::from(*channel) * scale).round() as u8;
    }
    layer.blit(buffer.as_ref(), rx, ry);
}

fn plain(layer: &mut Layer<'_>, area: BoxF, path: &Path, color: &Paint, style: IconStyle) {
    let Some(paint) = paint_for(color, area) else {
        return;
    };
    if style == IconStyle::Filled {
        layer.fill(path, &paint);
    }
    let s = area.width / 24.0;
    layer.stroke(
        path,
        &paint,
        &Stroke {
            width: (s * 1.6).max(1.0),
            ..Stroke::default()
        },
    );
}

fn draw_plain_at_rest(
    layer: &mut Layer<'_>,
    area: BoxF,
    parts: &[Part],
    color: &Paint,
    style: IconStyle,
) {
    let mut whole = PathBuilder::new();
    for part in parts {
        whole.push_path(&part.path);
    }
    if let Some(path) = whole.finish() {
        plain(layer, area, &path, color, style);
    }
}

fn component(
    layer: &mut Layer<'_>,
    area: BoxF,
    path: Path,
    light: Rgba,
    dark: Rgba,
    fill: bool,
    depth: bool,
) {
    let s = area.width / 24.;
    if depth && fill {
        let shadow = crate::paint::solid(Rgba {
            r: 0,
            g: 0,
            b: 0,
            a: 65,
        });
        if let Some(p) = path
            .clone()
            .transform(Transform::from_translate(s * 0.5, s * 0.9))
        {
            layer.fill(&p, &shadow);
        }
    }
    let palette = if depth {
        Paint::Linear {
            angle: 65.,
            stops: vec![(0., light), (1., dark)],
        }
    } else {
        Paint::solid(light)
    };
    if let Some(paint) = paint_for(&palette, area) {
        if fill {
            layer.fill(&path, &paint);
        }
        layer.stroke(
            &path,
            &paint,
            &Stroke {
                width: (s * 1.5).max(1.),
                line_cap: tiny_skia::LineCap::Round,
                ..Stroke::default()
            },
        );
    }
}

/// The coloured styles at rest: the cloud's extras (strips, drops, bolt)
/// are one component, as they always were.
fn draw_colored_at_rest(layer: &mut Layer<'_>, area: BoxF, parts: &[Part], depth: bool) {
    let mut extras = PathBuilder::new();
    let mut extra_colors = None;
    for part in parts {
        match part.role {
            Role::Strip(_) | Role::Drop(_) | Role::Bolt => {
                extras.push_path(&part.path);
                extra_colors = Some((part.light, part.dark));
            }
            _ => component(
                layer,
                area,
                part.path.clone(),
                part.light,
                part.dark,
                part.fill,
                depth,
            ),
        }
    }
    if let Some((light, dark)) = extra_colors
        && let Some(path) = extras.finish()
    {
        component(layer, area, path, light, dark, false, depth);
    }
}
