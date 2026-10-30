//! Pixel-art weather icons ([`IconStyle::Pixel`]): 16×16 sprites drawn cell
//! by cell on whole pixels. The demo mirrors them in
//! `apps/bezel-studio/src/weather-pixel.js`; a UI test keeps the two equal.
//!
//! A moving icon takes the vector styles' [`IconPose`] snapped to the grid:
//! the parts move by whole cells, opacities take three steps (0, ½, 1) and
//! the sun's rays alternate two frames instead of turning.
//!
//! [`IconStyle::Pixel`]: bezel_core::domain::weather::IconStyle::Pixel
use crate::layer::Layer;
use crate::weather::faded;
use bezel_core::domain::frame::Rgba;
use bezel_core::domain::theme::BoxF;
use bezel_core::domain::weather::{IconPose, family};
use tiny_skia::{PathBuilder, Rect};

/// Cells on each side of the grid.
const GRID: i32 = 16;
/// Units of the pose (a 64-unit icon) in one cell.
const UNITS_PER_CELL: f32 = 64.0 / GRID as f32;
/// Degrees the vector rays turn while the pixel rays show one frame.
const RAY_FRAME_DEGREES: f32 = 22.5;

const fn rgb(r: u8, g: u8, b: u8) -> Rgba {
    Rgba { r, g, b, a: 255 }
}

/// One character per colour; `.` is transparent.
const PALETTE: [(char, Rgba); 14] = [
    ('k', rgb(38, 48, 74)),
    ('H', rgb(255, 246, 194)),
    ('Y', rgb(255, 213, 79)),
    ('O', rgb(242, 159, 29)),
    ('R', rgb(255, 193, 7)),
    ('W', rgb(245, 251, 255)),
    ('C', rgb(181, 205, 230)),
    ('F', rgb(195, 211, 228)),
    ('B', rgb(55, 192, 245)),
    ('N', rgb(29, 112, 214)),
    ('S', rgb(230, 247, 255)),
    ('L', rgb(255, 224, 66)),
    ('M', rgb(255, 139, 30)),
    ('X', rgb(160, 174, 192)),
];

/// The sprites, rows top to bottom.
const SPRITES: [(&str, &[&str]); 12] = [
    (
        "disc",
        &[
            "..kkkk..", ".kHYYYk.", "kHYYYYOk", "kYYYYYOk", "kYYYYOOk", "kYYYOOOk", ".kOOOOk.",
            "..kkkk..",
        ],
    ),
    (
        "rays-a",
        &[
            "................",
            ".......RR.......",
            ".......RR.......",
            "...R........R...",
            "................",
            "................",
            "................",
            ".RR..........RR.",
            ".RR..........RR.",
            "................",
            "................",
            "................",
            "...R........R...",
            ".......RR.......",
            ".......RR.......",
            "................",
        ],
    ),
    (
        "rays-b",
        &[
            "................",
            "................",
            "..R....RR....R..",
            "...R........R...",
            "................",
            "................",
            "................",
            "..R..........R..",
            "..R..........R..",
            "................",
            "................",
            "................",
            "...R........R...",
            "..R....RR....R..",
            "................",
            "................",
        ],
    ),
    (
        "small-disc",
        &[".kkkk.", "kHYYYk", "kYYYOk", "kYYOOk", "kOOOOk", ".kkkk."],
    ),
    (
        "small-rays-a",
        &[
            ".....RR.....",
            ".....RR.....",
            "..R......R..",
            "............",
            "............",
            "RR........RR",
            "RR........RR",
            "............",
            "............",
            "..R......R..",
            ".....RR.....",
            ".....RR.....",
        ],
    ),
    (
        "small-rays-b",
        &[
            "............",
            ".R...RR...R.",
            "..R......R..",
            "............",
            "............",
            ".R........R.",
            ".R........R.",
            "............",
            "............",
            "..R......R..",
            ".R...RR...R.",
            "............",
        ],
    ),
    (
        "cloud",
        &[
            "....kkk.......",
            "...kWWWk.kk...",
            "..kWWWWWkWWk..",
            ".kWWWWWWWWWWk.",
            "kWWWWWWWWWWWWk",
            "kWWWWWWWWWWWCk",
            "kCCWWWWWWCCCCk",
            ".kkkkkkkkkkkk.",
        ],
    ),
    ("strip", &["FFFFFFFFFFFF"]),
    ("drop", &[".B", ".B", "N."]),
    ("flake", &[".S.", "SSS", ".S."]),
    (
        "bolt",
        &["...LL", "..LL.", ".LLLL", "...LM", "..LM.", "..M.."],
    ),
    (
        "cross",
        &[
            "XX....XX", "XXX..XXX", ".XXXXXX.", "..XXXX..", "..XXXX..", ".XXXXXX.", "XXX..XXX",
            "XX....XX",
        ],
    ),
];

/// The sprite called `name`.
fn sprite(name: &str) -> &'static [&'static str] {
    SPRITES
        .iter()
        .find(|(n, _)| *n == name)
        .map_or(&[], |(_, rows)| rows)
}

fn color_of(c: char) -> Option<Rgba> {
    PALETTE
        .iter()
        .find(|(p, _)| *p == c)
        .map(|(_, color)| *color)
}

/// A sprite at its cell, with its opacity.
struct Placed {
    rows: &'static [&'static str],
    x: i32,
    y: i32,
    opacity: f32,
}

/// A pose offset (units of a 64-unit icon) in whole cells.
fn cells(units: f32) -> i32 {
    if units.is_finite() {
        (units / UNITS_PER_CELL + 0.5).floor() as i32
    } else {
        0
    }
}

/// An opacity in three steps: 0, ½ or 1.
fn steps(opacity: f32) -> f32 {
    (opacity.clamp(0.0, 1.0) * 2.0 + 0.5).floor() / 2.0
}

/// Which of the two ray frames shows while the vector rays have turned `degrees`.
fn ray_frame(degrees: f32) -> usize {
    if degrees.is_finite() {
        ((degrees / RAY_FRAME_DEGREES).floor() as i64).rem_euclid(2) as usize
    } else {
        0
    }
}

/// The sprites of family `f`'s icon at `pose`, in drawing order.
fn placed(f: usize, pose: &IconPose) -> Vec<Placed> {
    let frame = ray_frame(pose.rays);
    let cloud = cells(pose.cloud);
    let mut out = Vec::new();
    let mut put = |name: &str, x: i32, y: i32, opacity: f32| {
        out.push(Placed {
            rows: sprite(name),
            x,
            y,
            opacity,
        });
    };
    match f {
        0 => {
            put("disc", 4, 4, 1.0);
            put(["rays-a", "rays-b"][frame], 0, 0, 1.0);
        }
        1 => {
            put("small-disc", 3, 3, 1.0);
            put(["small-rays-a", "small-rays-b"][frame], 0, 0, 1.0);
            put("cloud", 2 + cloud, 7, 1.0);
        }
        2 => put("cloud", 1 + cloud, 4, 1.0),
        3 => {
            put("cloud", 1 + cloud, 2, 1.0);
            for (i, (x, y)) in [(2, 11), (3, 13)].into_iter().enumerate() {
                let (dx, opacity) = pose.strips[i];
                put("strip", x + cells(dx), y, steps(opacity));
            }
        }
        4 | 5 => {
            put("cloud", 1 + cloud, 1, 1.0);
            let (name, at) = if f == 4 {
                ("drop", [(3, 11), (7, 12), (11, 11)])
            } else {
                ("flake", [(2, 11), (7, 12), (12, 11)])
            };
            for (i, (x, y)) in at.into_iter().enumerate() {
                let (dx, dy, opacity) = pose.drops[i];
                put(name, x + cells(dx), y + cells(dy), steps(opacity));
            }
        }
        6 => {
            put("cloud", 1 + cloud, 1, 1.0);
            put("bolt", 6, 9, steps(pose.bolt));
        }
        _ => put("cross", 4, 4, 1.0),
    }
    out
}

/// Draws the pixel icon of condition `code` in `area` at `pose`: whole
/// pixels per cell when the icon is 16 pixels or more, the grid centered.
pub(crate) fn draw(layer: &mut Layer<'_>, area: BoxF, code: u16, pose: &IconPose) {
    let side = area.width.min(area.height);
    if !side.is_finite() || side <= 0.0 {
        return;
    }
    let grid = GRID as f32;
    let cell = if side >= grid {
        (side / grid).floor()
    } else {
        side / grid
    };
    let snap = |v: f32| if cell >= 1.0 { (v + 0.5).floor() } else { v };
    let origin = (
        snap(area.x + (area.width - cell * grid) / 2.0),
        snap(area.y + (area.height - cell * grid) / 2.0),
    );
    faded(layer, area, steps(pose.whole), |layer| {
        for sprite in placed(family(code), pose) {
            faded(layer, area, sprite.opacity, |layer| {
                paint(layer, origin, cell, &sprite);
            });
        }
    });
}

/// Fills the cells of `sprite` inside the grid, one path per colour.
fn paint(layer: &mut Layer<'_>, origin: (f32, f32), cell: f32, sprite: &Placed) {
    let mut paths: Vec<(Rgba, PathBuilder)> = Vec::new();
    for (gy, row) in sprite.rows.iter().enumerate() {
        for (gx, c) in row.chars().enumerate() {
            let (x, y) = (sprite.x + gx as i32, sprite.y + gy as i32);
            if !(0..GRID).contains(&x) || !(0..GRID).contains(&y) {
                continue;
            }
            let (Some(color), Some(rect)) = (
                color_of(c),
                Rect::from_xywh(
                    origin.0 + x as f32 * cell,
                    origin.1 + y as f32 * cell,
                    cell,
                    cell,
                ),
            ) else {
                continue;
            };
            match paths.iter_mut().find(|(c, _)| *c == color) {
                Some((_, path)) => path.push_rect(rect),
                None => {
                    let mut path = PathBuilder::new();
                    path.push_rect(rect);
                    paths.push((color, path));
                }
            }
        }
    }
    for (color, path) in paths {
        if let Some(path) = path.finish() {
            let mut paint = crate::paint::solid(color);
            paint.anti_alias = false;
            layer.fill(&path, &paint);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bezel_core::domain::weather::pose;
    use tiny_skia::Pixmap;

    /// Every condition family, by one of its codes.
    const CODES: [u16; 8] = [0, 2, 3, 45, 61, 71, 95, 999];

    fn drawn(side: u32, code: u16, pose: &IconPose) -> Pixmap {
        let mut pixmap = Pixmap::new(side, side).unwrap();
        let mut layer = Layer {
            pixmap: pixmap.as_mut(),
            x: 0,
            y: 0,
        };
        let area = BoxF::new(0.0, 0.0, side as f32, side as f32);
        draw(&mut layer, area, code, pose);
        pixmap
    }

    #[test]
    fn every_sprite_is_rectangular_and_painted_from_the_palette() {
        for (name, rows) in SPRITES {
            assert!(!rows.is_empty(), "{name}");
            let width = rows[0].chars().count();
            for row in rows {
                assert_eq!(row.chars().count(), width, "{name}: {row}");
                assert!(
                    row.chars().all(|c| c == '.' || color_of(c).is_some()),
                    "{name}: {row}"
                );
            }
        }
    }

    #[test]
    fn at_rest_every_pixel_is_a_palette_colour_on_whole_cells() {
        // 48 pixels: three per cell, so a cell never shares a pixel.
        for code in CODES {
            let pixmap = drawn(48, code, &IconPose::REST);
            let mut painted = 0;
            for (i, p) in pixmap.pixels().iter().enumerate() {
                if p.alpha() == 0 {
                    continue;
                }
                painted += 1;
                let color = Rgba {
                    r: p.red(),
                    g: p.green(),
                    b: p.blue(),
                    a: p.alpha(),
                };
                assert!(
                    PALETTE.iter().any(|(_, c)| *c == color),
                    "code {code}: pixel {i} is {color:?}"
                );
                let (x, y) = (i % 48, i / 48);
                let first = pixmap
                    .pixel((x - x % 3) as u32, (y - y % 3) as u32)
                    .unwrap();
                assert_eq!(*p, first, "code {code}: one colour per cell");
            }
            assert!(painted > 40, "code {code} draws its icon");
        }
    }

    #[test]
    fn the_sun_twinkles_between_two_frames_and_rain_falls_by_whole_cells() {
        let sun = |t: f32| drawn(32, 0, &pose(0, t, 1.0, 1.0)).data().to_vec();
        let frames: std::collections::BTreeSet<_> = (0..48).map(|i| sun(i as f32 * 0.5)).collect();
        assert_eq!(frames.len(), 2, "two ray frames and nothing in between");
        assert_eq!(ray_frame(0.0), 0);
        assert_eq!(ray_frame(30.0), 1);
        assert_eq!(ray_frame(-10.0), 1);
        let rain: std::collections::BTreeSet<_> = (0..20)
            .map(|i| {
                drawn(32, 61, &pose(4, i as f32 * 0.05, 1.0, 1.0))
                    .data()
                    .to_vec()
            })
            .collect();
        assert!(rain.len() > 2, "the drops fall");
        assert_eq!(cells(8.0), 2);
        assert_eq!(cells(-6.0), -1);
        assert_eq!(steps(0.3), 0.5);
        assert_eq!(steps(0.2), 0.0);
        assert_eq!(steps(0.8), 1.0);
    }

    #[test]
    fn a_small_or_empty_box_never_fails() {
        drawn(10, 95, &IconPose::REST);
        let mut pixmap = Pixmap::new(4, 4).unwrap();
        let mut layer = Layer {
            pixmap: pixmap.as_mut(),
            x: 0,
            y: 0,
        };
        draw(
            &mut layer,
            BoxF::new(0.0, 0.0, 0.0, 0.0),
            0,
            &IconPose::REST,
        );
        assert!(pixmap.pixels().iter().all(|p| p.alpha() == 0));
    }
}
