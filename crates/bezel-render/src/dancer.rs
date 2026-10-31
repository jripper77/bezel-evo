//! Pixel dancers of the `Dancer` spectrum style, mirrored by
//! apps/bezel-studio/src/dancer-pixel.js (a fixture of every pose keeps
//! both equal to the design): a 24×24 character built from its head and
//! torso, with arms, legs and tail drawn along the paths of each pose, on
//! whole pixels.
use crate::layer::Layer;
use bezel_core::domain::frame::Rgba;
use bezel_core::domain::spectrum::{DancePose, Dancer};
use bezel_core::domain::theme::BoxF;
use tiny_skia::{PathBuilder, Rect};

/// Cells on each side of the grid.
const GRID: i32 = 24;
/// Column of the character's left edge.
const LEFT: i32 = 8;
/// Row the feet stand on; the shadow lies on the one below.
const FLOOR: i32 = 22;
/// Length of a standing leg, foot included.
const LEG: i32 = 4;

const fn rgb(r: u8, g: u8, b: u8) -> Rgba {
    Rgba { r, g, b, a: 255 }
}

/// One character per colour; `.` is transparent.
const PALETTE: [(char, Rgba); 6] = [
    ('W', rgb(242, 237, 230)),
    ('O', rgb(255, 146, 72)),
    ('G', rgb(48, 191, 139)),
    ('M', rgb(138, 133, 124)),
    ('D', rgb(60, 58, 54)),
    ('K', rgb(16, 16, 16)),
];

/// What a dancer is made of.
struct Character {
    head: &'static [&'static str],
    torso: &'static [&'static str],
    arm: char,
    hand: char,
    leg: char,
    foot: char,
    tail: Option<char>,
}

const ROBOT: Character = Character {
    head: &[
        "...OO...", "....W...", "WWWWWWWW", "WKKKKKKW", "WKGKKGKW", "WKKKKKKW", "WWWWWWWW",
    ],
    torso: &[".MWWWWM.", ".WOOOOW.", ".WOGGOW.", ".WOOOOW.", ".MWWWWM."],
    arm: 'M',
    hand: 'W',
    leg: 'M',
    foot: 'W',
    tail: None,
};
const HEADPHONES: Character = Character {
    head: &[
        ".OOOOOO.", "O.WWWW.O", "OWWWWWWO", "OWKWWKWO", "OWWWWWWO", ".WWKKWW.", "..WWWW..",
    ],
    torso: &[".GGGGGG.", ".GGGGGG.", ".GGWWGG.", ".GGGGGG.", ".MMMMMM."],
    arm: 'G',
    hand: 'W',
    leg: 'M',
    foot: 'O',
    tail: None,
};
const CAT: Character = Character {
    head: &[
        "O......O", "OO....OO", "OOOOOOOO", "OGOOOOGO", "OOOWWOOO", "OOWKKWOO", ".OOOOOO.",
    ],
    torso: &[".OOOOOO.", ".OWWWWO.", ".OWWWWO.", ".OWWWWO.", ".OOOOOO."],
    arm: 'O',
    hand: 'W',
    leg: 'O',
    foot: 'W',
    tail: Some('O'),
};

type Path = &'static [(i32, i32)];

/// Arm paths from the shoulder, for the left arm; the right one is mirrored.
const ARM_DOWN: Path = &[(0, 1), (0, 2), (0, 3)];
const ARM_OUT: Path = &[(-1, 0), (-2, 0), (-3, -1)];
const ARM_UP: Path = &[(-1, 0), (-2, -1), (-2, -2), (-2, -3), (-2, -4)];
/// Tail paths from the back of the hip.
const TAIL_DOWN: Path = &[(1, 0), (2, 1), (2, 2)];
const TAIL_OUT: Path = &[(1, 0), (2, 0), (3, -1)];
const TAIL_UP: Path = &[(1, 0), (2, -1), (2, -2), (3, -3)];

/// How the body stands in a pose: its offset (down is positive), the left
/// and right arms, the left and right legs from the hips (none: the left
/// one mirrored) and the tail.
struct Shape {
    dy: i32,
    arms: [Path; 2],
    legs: (Path, Option<Path>),
    tail: Path,
}

fn shape(pose: DancePose) -> Shape {
    match pose {
        DancePose::Still => Shape {
            dy: 0,
            arms: [ARM_DOWN, ARM_DOWN],
            legs: (&[(0, 1), (0, 2), (0, 3), (0, 4)], None),
            tail: TAIL_DOWN,
        },
        DancePose::Down => Shape {
            dy: 2,
            arms: [ARM_OUT, ARM_OUT],
            legs: (&[(-1, 1), (-1, 2)], None),
            tail: TAIL_OUT,
        },
        DancePose::ArmsUp => Shape {
            dy: 0,
            arms: [ARM_UP, ARM_UP],
            legs: (&[(-1, 1), (-1, 2), (-2, 3), (-2, 4)], None),
            tail: TAIL_UP,
        },
        DancePose::StepLeft => Shape {
            dy: 1,
            arms: [ARM_UP, ARM_DOWN],
            legs: (
                &[(-1, 1), (-1, 2), (-1, 3)],
                Some(&[(1, 0), (2, 0), (2, 1)]),
            ),
            tail: TAIL_OUT,
        },
        DancePose::StepRight => Shape {
            dy: 1,
            arms: [ARM_DOWN, ARM_UP],
            legs: (
                &[(-1, 0), (-2, 0), (-2, 1)],
                Some(&[(1, 1), (1, 2), (1, 3)]),
            ),
            tail: TAIL_UP,
        },
        DancePose::Jump => Shape {
            dy: -4,
            arms: [ARM_UP, ARM_UP],
            legs: (&[(0, 1), (1, 2), (0, 3)], None),
            tail: TAIL_UP,
        },
    }
}

type Cells = [[char; GRID as usize]; GRID as usize];

/// Paints `ch` at (`x`, `y`) when it is a colour inside the grid.
fn put(cells: &mut Cells, x: i32, y: i32, ch: char) {
    if (0..GRID).contains(&x) && (0..GRID).contains(&y) && ch != '.' {
        cells[y as usize][x as usize] = ch;
    }
}

/// Paints `rows` with their top-left corner at (`LEFT`, `top`).
fn sprite(cells: &mut Cells, rows: &[&str], top: i32) {
    for (y, row) in rows.iter().enumerate() {
        for (x, ch) in row.chars().enumerate() {
            put(cells, LEFT + x as i32, top + y as i32, ch);
        }
    }
}

/// The grid of `dancer` in `pose`, rows top to bottom, `.` transparent.
pub(crate) fn grid(dancer: Dancer, pose: DancePose) -> Cells {
    let c = match dancer {
        Dancer::Robot => &ROBOT,
        Dancer::Headphones => &HEADPHONES,
        Dancer::Cat => &CAT,
    };
    let p = shape(pose);
    let mut cells = [['.'; GRID as usize]; GRID as usize];
    let (head, torso) = (c.head.len() as i32, c.torso.len() as i32);
    let top = FLOOR - LEG - torso - head + 1 + p.dy;
    let torso_top = top + head;
    let hip = torso_top + torso - 1;
    // The shadow, smaller while in the air.
    let shadow = if p.dy >= 0 { 6 } else { 4 };
    for x in 12 - shadow / 2..12 + shadow / 2 {
        put(&mut cells, x, GRID - 1, 'D');
    }
    let mirrored: Vec<(i32, i32)> = p.legs.0.iter().map(|&(dx, dy)| (-dx, dy)).collect();
    let right = p.legs.1.unwrap_or(&mirrored);
    for (hx, path, side) in [(LEFT + 2, p.legs.0, -1), (LEFT + 5, right, 1)] {
        for &(dx, dy) in path {
            put(&mut cells, hx + dx, hip + dy, c.leg);
        }
        if let Some(&(fx, fy)) = path.last() {
            put(&mut cells, hx + fx, hip + fy, c.foot);
            put(&mut cells, hx + fx + side, hip + fy, c.foot);
        }
    }
    if let Some(tail) = c.tail {
        for &(dx, dy) in p.tail {
            put(&mut cells, LEFT + 7 + dx, hip - 1 + dy, tail);
        }
    }
    sprite(&mut cells, c.torso, torso_top);
    for (shoulder, flip, path) in [(LEFT, 1, p.arms[0]), (LEFT + 7, -1, p.arms[1])] {
        for &(dx, dy) in path {
            put(&mut cells, shoulder + dx * flip, torso_top + dy, c.arm);
        }
        if let Some(&(dx, dy)) = path.last() {
            put(&mut cells, shoulder + dx * flip, torso_top + dy, c.hand);
        }
    }
    sprite(&mut cells, c.head, top);
    cells
}

/// Draws `dancer` in `pose` in `area`: the grid as large as whole pixels
/// allow (finer below 24 px), centred across and standing on the bottom.
pub(crate) fn draw(layer: &mut Layer<'_>, area: BoxF, dancer: Dancer, pose: DancePose) {
    let side = area.width.min(area.height);
    if !side.is_finite() || side <= 0.0 {
        return;
    }
    let size = GRID as f32;
    let cell = if side >= size {
        (side / size).floor()
    } else {
        side / size
    };
    let snap = |v: f32| if cell >= 1.0 { (v + 0.5).floor() } else { v };
    let origin = (
        snap(area.x + (area.width - cell * size) / 2.0),
        snap(area.y + area.height - cell * size),
    );
    let mut paths: Vec<(Rgba, PathBuilder)> = Vec::new();
    for (y, row) in grid(dancer, pose).iter().enumerate() {
        for (x, ch) in row.iter().enumerate() {
            let Some(&(_, color)) = PALETTE.iter().find(|(c, _)| c == ch) else {
                continue;
            };
            let Some(rect) = Rect::from_xywh(
                origin.0 + x as f32 * cell,
                origin.1 + y as f32 * cell,
                cell,
                cell,
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
#[allow(clippy::unwrap_used, clippy::panic)]
mod tests {
    use super::*;

    const POSES: [DancePose; 6] = [
        DancePose::Still,
        DancePose::Down,
        DancePose::ArmsUp,
        DancePose::StepLeft,
        DancePose::StepRight,
        DancePose::Jump,
    ];

    #[test]
    fn every_pose_is_the_designed_one() {
        let fixture: serde_json::Value = serde_json::from_str(include_str!(
            "../../../apps/bezel-studio/tests/ui/fixtures/dancers.json"
        ))
        .unwrap();
        for (name, dancer) in [
            ("robot", Dancer::Robot),
            ("headphones", Dancer::Headphones),
            ("cat", Dancer::Cat),
        ] {
            for (i, pose) in POSES.into_iter().enumerate() {
                let drawn: Vec<String> = grid(dancer, pose)
                    .iter()
                    .map(|row| row.iter().collect())
                    .collect();
                let designed: Vec<String> = fixture[name][i]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|row| row.as_str().unwrap().to_string())
                    .collect();
                assert_eq!(drawn, designed, "{name} {pose:?}");
                assert!(
                    drawn
                        .iter()
                        .flat_map(|r| r.chars())
                        .all(|c| c == '.' || PALETTE.iter().any(|(p, _)| *p == c)),
                    "{name} {pose:?} uses palette colours"
                );
            }
        }
    }

    #[test]
    fn a_dancer_stands_on_whole_pixels_and_a_tiny_box_draws_nothing_wrong() {
        let mut pixmap = tiny_skia::Pixmap::new(100, 60).unwrap();
        {
            let mut layer = Layer {
                pixmap: pixmap.as_mut(),
                x: 0,
                y: 0,
            };
            draw(
                &mut layer,
                BoxF::new(0.0, 0.0, 100.0, 60.0),
                Dancer::Cat,
                DancePose::Jump,
            );
            draw(
                &mut layer,
                BoxF::new(0.0, 0.0, 0.0, 10.0),
                Dancer::Cat,
                DancePose::Jump,
            );
            draw(
                &mut layer,
                BoxF::new(0.0, 0.0, 10.0, 10.0),
                Dancer::Robot,
                DancePose::Down,
            );
        }
        let palette: Vec<[u8; 3]> = PALETTE.iter().map(|(_, c)| [c.r, c.g, c.b]).collect();
        let mut big = tiny_skia::Pixmap::new(100, 60).unwrap();
        {
            let mut layer = Layer {
                pixmap: big.as_mut(),
                x: 0,
                y: 0,
            };
            draw(
                &mut layer,
                BoxF::new(0.0, 0.0, 100.0, 60.0),
                Dancer::Robot,
                DancePose::ArmsUp,
            );
        }
        let lit: Vec<_> = big.pixels().iter().filter(|p| p.alpha() > 0).collect();
        assert!(!lit.is_empty());
        assert!(
            lit.iter()
                .all(|p| p.alpha() == 255 && palette.contains(&[p.red(), p.green(), p.blue()])),
            "cells are whole pixels of the palette"
        );
    }
}
