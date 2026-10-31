//! Cover transitions between display profiles: an opaque panel the host
//! paints over the whole scene closes it, the new theme is swapped in under
//! it, and the panel opens again.
//!
//! Only opaque pixels are painted, so a cover hides a video the screen plays
//! on its own as well as the host's own frame: the host never needs the
//! pixels of what it covers. Every measure follows the canvas (the short side
//! sets the edge, the strips and the tiles), so the same cover fits any
//! screen, wide, tall or round.

use super::frame::{Frame, Rect, Rgba};

/// How the panel closes and opens.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoverEffect {
    /// Two leaves from opposite edges meet in the middle and part again.
    Curtain,
    /// One panel crosses the screen: it covers coming in and uncovers
    /// leaving on the other side.
    Wipe,
    /// Strips close one after another in a wave and open the same way.
    Blinds,
    /// Small tiles light up in a wave that sweeps across the screen until
    /// all is covered, and go out in the same wave.
    Mosaic,
    /// A circle closes on the middle behind a ring of light, and opens again.
    Iris,
    /// An old television switching off: the picture squeezes into a line of
    /// light that shrinks to a dot; it switches on the same way back.
    Tv,
    /// Small square pixels flash in, scattered, until all is covered.
    Pixels,
}

impl CoverEffect {
    /// Whether it moves in a direction (the others ignore it).
    pub const fn directed(self) -> bool {
        matches!(
            self,
            CoverEffect::Curtain | CoverEffect::Wipe | CoverEffect::Blinds | CoverEffect::Mosaic
        )
    }
}

/// Where the cover moves. Curtain uses the axis only; iris, television and
/// pixels ignore it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoverDirection {
    /// Towards the left edge.
    Left,
    /// Towards the right edge.
    Right,
    /// Towards the top edge.
    Up,
    /// Towards the bottom edge.
    Down,
}

impl CoverDirection {
    const fn horizontal(self) -> bool {
        matches!(self, CoverDirection::Left | CoverDirection::Right)
    }
}

/// Whether the panel is closing over the old scene or opening on the new.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoverStage {
    /// The panel comes in over the old scene.
    Closing,
    /// The panel goes away from the new scene.
    Opening,
}

/// One cover: its motion and its two colors (painted opaque).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cover {
    /// The motion.
    pub effect: CoverEffect,
    /// Its direction.
    pub direction: CoverDirection,
    /// The panel.
    pub panel: Rgba,
    /// The moving edges and the mosaic's front.
    pub accent: Rgba,
}

/// Panel color of a cover nobody chose.
pub const DEFAULT_PANEL: Rgba = Rgba::opaque(0x12, 0x0b, 0x13);
/// Accent color of a cover nobody chose.
pub const DEFAULT_ACCENT: Rgba = Rgba::opaque(0xf2, 0x8c, 0x28);

/// Delay of the last strip of a blind, as a share of the closing.
const BLINDS_STAGGER: f32 = 1.2;
/// Share of the motion over which a mosaic tile grows, lit with the accent.
const MOSAIC_FRONT: f32 = 0.22;
/// Share of a tile's growth lit with the accent: the crest of the wave.
const LIT: f32 = 0.5;
/// Mosaic tiles across the short side.
const MOSAIC_TILES: f32 = 12.0;
/// How much a mosaic's wave leans, and how ragged its front is, as shares of
/// the sweep.
const WAVE_LEAN: f32 = 0.3;
const WAVE_RAGGED: f32 = 0.12;
/// How close (in short sides) a curtain's leaves light the gap between them.
const CURTAIN_FLASH: f32 = 0.5;
/// Pixels across the short side, and the share of the motion each flashes.
const PIXEL_TILES: f32 = 16.0;
const PIXEL_FRONT: f32 = 0.12;
/// Share of a television's closing spent squeezing the picture into a line.
const TV_SQUEEZE: f32 = 0.6;

/// Smooth start and end (cubic), for `t` in 0..=1.
pub fn ease(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    if t < 0.5 {
        4.0 * t * t * t
    } else {
        1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
    }
}

impl Cover {
    /// Paints the cover over `frame` with `coverage` of it drawn: 0 paints
    /// nothing, 1 covers every pixel with the panel.
    pub fn paint(&self, frame: &mut Frame, stage: CoverStage, coverage: f32) {
        let c = coverage.clamp(0.0, 1.0);
        if c <= 0.0 {
            return;
        }
        let size = frame.size();
        if c >= 1.0 {
            frame.fill_rect(Rect::of(size), opaque(self.panel));
            return;
        }
        let mut painter = Painter::new(frame, self);
        match self.effect {
            CoverEffect::Curtain => painter.curtain(self.direction, c),
            CoverEffect::Wipe => painter.wipe(self.direction, stage, c),
            CoverEffect::Blinds => painter.blinds(self.direction, c),
            CoverEffect::Mosaic => painter.mosaic(
                Order::Wave(self.direction, stage),
                c,
                MOSAIC_TILES,
                MOSAIC_FRONT,
                true,
            ),
            CoverEffect::Iris => painter.iris(c),
            CoverEffect::Tv => painter.tv(c),
            CoverEffect::Pixels => {
                painter.mosaic(Order::Scattered(stage), c, PIXEL_TILES, PIXEL_FRONT, false)
            }
        }
    }
}

const fn opaque(color: Rgba) -> Rgba {
    Rgba::opaque(color.r, color.g, color.b)
}

/// `accent` over `panel` by `amount`, opaque.
fn mix(panel: Rgba, accent: Rgba, amount: f32) -> Rgba {
    let channel =
        |p: u8, a: u8| (f32::from(p) + (f32::from(a) - f32::from(p)) * amount).round() as u8;
    Rgba::opaque(
        channel(panel.r, accent.r),
        channel(panel.g, accent.g),
        channel(panel.b, accent.b),
    )
}

/// The side of an edge the panel lies on.
#[derive(Clone, Copy)]
enum Inside {
    Before,
    After,
}

struct Painter<'a> {
    frame: &'a mut Frame,
    width: f32,
    height: f32,
    short: f32,
    panel: Rgba,
    accent: Rgba,
    edge: u32,
}

impl<'a> Painter<'a> {
    fn new(frame: &'a mut Frame, cover: &Cover) -> Self {
        let size = frame.size();
        let short = size.width.min(size.height).max(1) as f32;
        Self {
            frame,
            width: size.width as f32,
            height: size.height as f32,
            short,
            panel: opaque(cover.panel),
            accent: opaque(cover.accent),
            edge: (short / 160.0).round().max(2.0) as u32,
        }
    }

    fn length(&self, horizontal: bool) -> f32 {
        if horizontal { self.width } else { self.height }
    }

    /// Fills `start..end` along the axis (columns when `horizontal`).
    fn band(&mut self, horizontal: bool, start: f32, end: f32, color: Rgba) {
        let a = start.round().max(0.0) as u32;
        let b = end.round().max(0.0) as u32;
        if b <= a {
            return;
        }
        let rect = if horizontal {
            Rect::new(a, 0, b - a, self.height as u32)
        } else {
            Rect::new(0, a, self.width as u32, b - a)
        };
        self.frame.fill_rect(rect, color);
    }

    /// The lit edge at `at` along the axis, with its glow inside the panel.
    fn edge(&mut self, horizontal: bool, at: f32, inside: Inside) {
        let glow = (self.short / 48.0).max(2.0);
        self.lit_edge(horizontal, at, inside, glow, &[0.55, 0.3, 0.12]);
    }

    /// The lit edge at `at`, with a glow `glow` long inside the panel fading
    /// through `amounts` of the accent, and its light ahead on the scene.
    fn lit_edge(&mut self, horizontal: bool, at: f32, inside: Inside, glow: f32, amounts: &[f32]) {
        let line = self.edge as f32;
        let steps = amounts.len() as f32;
        let (line_start, glow_sign) = match inside {
            Inside::Before => (at - line, -1.0),
            Inside::After => (at, 1.0),
        };
        let glow_from = match inside {
            Inside::Before => line_start,
            Inside::After => line_start + line,
        };
        for (step, &amount) in amounts.iter().enumerate() {
            let near = glow_from + glow_sign * glow * step as f32 / steps;
            let far = glow_from + glow_sign * glow * (step + 1) as f32 / steps;
            let color = mix(self.panel, self.accent, amount);
            self.band(horizontal, near.min(far), near.max(far), color);
        }
        self.band(horizontal, line_start, line_start + line, self.accent);
        // The light spills a little onto the scene ahead of the edge.
        let (outside, sign) = match inside {
            Inside::Before => (at, 1.0),
            Inside::After => (at, -1.0),
        };
        let spill = (self.short / 32.0).max(3.0);
        for (step, amount) in [0.35, 0.18, 0.07].into_iter().enumerate() {
            let near = outside + sign * spill * step as f32 / 3.0;
            let far = outside + sign * spill * (step + 1) as f32 / 3.0;
            self.glaze(horizontal, near.min(far), near.max(far), amount);
        }
    }

    /// Tints `start..end` along the axis with the accent by `amount`, on the
    /// host's opaque pixels only: where the screen's own video shows through
    /// (alpha below 255) nothing changes, since the host never has its pixels.
    fn glaze(&mut self, horizontal: bool, start: f32, end: f32, amount: f32) {
        let a = start.round().max(0.0) as u32;
        let b = (end.round().max(0.0) as u32).min(self.length(horizontal) as u32);
        if b <= a {
            return;
        }
        let width = self.width as usize;
        let (columns, rows) = if horizontal {
            (a as usize..b as usize, 0..self.height as usize)
        } else {
            (0..width, a as usize..b as usize)
        };
        let accent = self.accent;
        let pixels = self.frame.as_rgba_mut();
        for y in rows {
            for x in columns.clone() {
                let i = (y * width + x) * 4;
                let p = &mut pixels[i..i + 4];
                if p[3] == 255 {
                    let tinted = mix(Rgba::opaque(p[0], p[1], p[2]), accent, amount);
                    p[..3].copy_from_slice(&[tinted.r, tinted.g, tinted.b]);
                }
            }
        }
    }

    fn curtain(&mut self, direction: CoverDirection, c: f32) {
        let horizontal = direction.horizontal();
        let length = self.length(horizontal);
        let half = length / 2.0 * c;
        // As the leaves meet, the light between them flares up.
        let gap = length - half * 2.0;
        let flash = self.short * CURTAIN_FLASH;
        if gap < flash {
            let amount = 0.6 * (1.0 - gap / flash).powi(2);
            self.glaze(horizontal, half, length - half, amount);
        }
        self.band(horizontal, 0.0, half, self.panel);
        self.band(horizontal, length - half, length, self.panel);
        self.edge(horizontal, half, Inside::Before);
        self.edge(horizontal, length - half, Inside::After);
    }

    fn wipe(&mut self, direction: CoverDirection, stage: CoverStage, c: f32) {
        let horizontal = direction.horizontal();
        let length = self.length(horizontal);
        // Eased once more: it sets off slowly, rushes across and brakes.
        let reach = length * ease(c);
        // A long trail of light behind the edge.
        let trail = (self.short / 10.0).max(4.0);
        const TRAIL: [f32; 6] = [0.6, 0.42, 0.28, 0.17, 0.09, 0.04];
        // Towards the start (left, up): it enters at the end and leaves at the start.
        let toward_start = matches!(direction, CoverDirection::Left | CoverDirection::Up);
        match (toward_start, stage) {
            (true, CoverStage::Closing) | (false, CoverStage::Opening) => {
                self.band(horizontal, length - reach, length, self.panel);
                self.lit_edge(horizontal, length - reach, Inside::After, trail, &TRAIL);
            }
            (true, CoverStage::Opening) | (false, CoverStage::Closing) => {
                self.band(horizontal, 0.0, reach, self.panel);
                self.lit_edge(horizontal, reach, Inside::Before, trail, &TRAIL);
            }
        }
    }

    fn blinds(&mut self, direction: CoverDirection, c: f32) {
        let horizontal = direction.horizontal();
        let length = self.length(horizontal);
        let count = ((length / (self.short / 2.0)).round() as usize).max(4);
        let line = self.edge as f32;
        for i in 0..count {
            let mut order = i as f32 / (count - 1) as f32;
            if matches!(direction, CoverDirection::Left | CoverDirection::Up) {
                order = 1.0 - order;
            }
            let closed = (c * (1.0 + BLINDS_STAGGER) - BLINDS_STAGGER * order).clamp(0.0, 1.0);
            if closed <= 0.0 {
                continue;
            }
            let start = (i as f32 * length / count as f32).round();
            let end = ((i + 1) as f32 * length / count as f32).round();
            let span = (end - start) * closed;
            let from = start + (end - start - span) / 2.0;
            self.band(horizontal, from, from + span, self.panel);
            if closed < 1.0 {
                // Each border lit, its glow fading inwards.
                let room = span / 2.0;
                let border = line.min(room);
                let glow = (self.short / 64.0).max(2.0).min(room - border);
                for (step, amount) in [0.45, 0.2].into_iter().enumerate() {
                    let near = border + glow * step as f32 / 2.0;
                    let far = border + glow * (step + 1) as f32 / 2.0;
                    let color = mix(self.panel, self.accent, amount);
                    self.band(horizontal, from + near, from + far, color);
                    self.band(horizontal, from + span - far, from + span - near, color);
                }
                self.band(horizontal, from, from + border, self.accent);
                self.band(horizontal, from + span - border, from + span, self.accent);
            }
        }
    }

    /// Fills the parts of row `y` between the circles of radius `inner` and
    /// `outer` around `centre`.
    fn ring_row(&mut self, y: u32, centre: (f32, f32), inner: f32, outer: f32, color: Rgba) {
        let (cx, cy) = centre;
        let dy = y as f32 + 0.5 - cy;
        if outer <= inner || dy.abs() >= outer {
            return;
        }
        let half = |r: f32| {
            if dy.abs() < r {
                (r * r - dy * dy).sqrt()
            } else {
                0.0
            }
        };
        let (near, far) = (half(inner), half(outer));
        if near > 0.0 {
            self.rect(cx - far, y as f32, far - near, 1.0, color);
            self.rect(cx + near, y as f32, far - near, 1.0, color);
        } else {
            self.rect(cx - far, y as f32, far * 2.0, 1.0, color);
        }
    }

    fn iris(&mut self, c: f32) {
        let centre = (self.width / 2.0, self.height / 2.0);
        let line = self.edge as f32;
        let glow = (self.short / 48.0).max(2.0);
        // The ring and its glow start outside the screen and end in the middle.
        let reach = (self.width * self.width + self.height * self.height).sqrt() / 2.0;
        let radius = (reach + line + glow) * (1.0 - c);
        let far = reach + line + glow + 1.0;
        let (panel, accent) = (self.panel, self.accent);
        for y in 0..self.height as u32 {
            self.ring_row(y, centre, radius + line + glow, far, panel);
            for (step, amount) in [0.55, 0.3, 0.12].into_iter().enumerate() {
                let from = radius + line + glow * step as f32 / 3.0;
                let to = radius + line + glow * (step + 1) as f32 / 3.0;
                self.ring_row(y, centre, from, to, mix(panel, accent, amount));
            }
            self.ring_row(y, centre, radius, radius + line, accent);
        }
    }

    fn tv(&mut self, c: f32) {
        let line = (self.edge * 2) as f32;
        if c < TV_SQUEEZE {
            // The picture squeezes: bands close from the top and the bottom.
            let squeezed = c / TV_SQUEEZE;
            let open = (self.height - line) * (1.0 - squeezed) + line;
            let top = (self.height - open) / 2.0;
            self.glaze(false, top, top + open, 0.35 * squeezed);
            self.band(false, 0.0, top, self.panel);
            self.band(false, top + open, self.height, self.panel);
            self.edge(false, top, Inside::Before);
            self.edge(false, top + open, Inside::After);
            return;
        }
        // Then all is dark but a line of light shrinking to a dot.
        let shrunk = (c - TV_SQUEEZE) / (1.0 - TV_SQUEEZE);
        self.band(false, 0.0, self.height, self.panel);
        let length = (self.width * (1.0 - shrunk)).max(line);
        let (x, y) = ((self.width - length) / 2.0, (self.height - line) / 2.0);
        let glow = (self.short / 48.0).max(2.0);
        for (step, amount) in [0.12, 0.3, 0.55].into_iter().enumerate() {
            let grow = glow * (3 - step) as f32 / 3.0;
            let color = mix(self.panel, self.accent, amount);
            self.rect(
                x - grow,
                y - grow,
                length + grow * 2.0,
                line + grow * 2.0,
                color,
            );
        }
        self.rect(x, y, length, line, self.accent);
    }

    /// Fills a rectangle in canvas pixels, clipped to the frame.
    fn rect(&mut self, x: f32, y: f32, width: f32, height: f32, color: Rgba) {
        let (left, top) = (x.max(0.0).round() as u32, y.max(0.0).round() as u32);
        let right = (x + width).min(self.width).round().max(0.0) as u32;
        let bottom = (y + height).min(self.height).round().max(0.0) as u32;
        if right > left && bottom > top {
            self.frame
                .fill_rect(Rect::new(left, top, right - left, bottom - top), color);
        }
    }

    /// Tiles (`across` on the short side) cover in `order`, each over
    /// `front` of the motion, lit with the accent meanwhile; they grow from
    /// their middle when `grow`.
    fn mosaic(&mut self, order: Order, c: f32, across: f32, front: f32, grow: bool) {
        let tile = self.short / across;
        let columns = ((self.width / tile).round() as usize).max(4);
        let rows = ((self.height / tile).round() as usize).max(4);
        let count = columns * rows;
        let rank = match order {
            Order::Scattered(stage) => scattered(
                count,
                match stage {
                    CoverStage::Closing => 0x2026,
                    CoverStage::Opening => 0x0911,
                },
            ),
            // Opening, the wave goes on the same way: its first tiles go first.
            Order::Wave(direction, CoverStage::Closing) => wave(columns, rows, direction),
            Order::Wave(direction, CoverStage::Opening) => wave(columns, rows, direction)
                .into_iter()
                .map(|rank| count - 1 - rank)
                .collect(),
        };
        for (cell, &order) in rank.iter().enumerate() {
            // Each tile grows from its middle over MOSAIC_FRONT of the motion,
            // lit while it grows; the last one is whole when the cover is.
            let threshold = order as f32 / count as f32 * (1.0 - front);
            if c <= threshold {
                continue;
            }
            let grown = ((c - threshold) / front).min(1.0);
            // Lit while young (the crest), then dark; a pixel stays lit until whole.
            let lit = if grow { LIT } else { 1.0 };
            let color = if grown < lit { self.accent } else { self.panel };
            let scale = if grow {
                ease(0.5 + grown / 2.0) * 2.0 - 1.0
            } else {
                1.0
            };
            let (column, row) = (cell % columns, cell / columns);
            let x0 = column as f32 * self.width / columns as f32;
            let x1 = (column + 1) as f32 * self.width / columns as f32;
            let y0 = row as f32 * self.height / rows as f32;
            let y1 = (row + 1) as f32 * self.height / rows as f32;
            let (dx, dy) = (
                (x1 - x0) * (1.0 - scale) / 2.0,
                (y1 - y0) * (1.0 - scale) / 2.0,
            );
            let (left, top) = ((x0 + dx).round() as u32, (y0 + dy).round() as u32);
            let (right, bottom) = ((x1 - dx).round() as u32, (y1 - dy).round() as u32);
            self.frame.fill_rect(
                Rect::new(
                    left,
                    top,
                    right.saturating_sub(left),
                    bottom.saturating_sub(top),
                ),
                color,
            );
        }
    }
}

/// The order tiles cover in.
#[derive(Clone, Copy)]
enum Order {
    /// Scattered, another order to close than to open.
    Scattered(CoverStage),
    /// A leaning, ragged wave sweeping towards the direction, closing and
    /// opening alike.
    Wave(CoverDirection, CoverStage),
}

/// The position of each of `columns` × `rows` cells (row by row) in a wave
/// sweeping towards `direction`, leaning and ragged.
fn wave(columns: usize, rows: usize, direction: CoverDirection) -> Vec<usize> {
    let count = columns * rows;
    let ragged = scattered(count, 0x0a7e);
    let share = |i: usize, n: usize| i as f32 / n.saturating_sub(1).max(1) as f32;
    let keys: Vec<f32> = (0..count)
        .map(|cell| {
            let (column, row) = (share(cell % columns, columns), share(cell / columns, rows));
            let (along, across) = match direction {
                CoverDirection::Left => (1.0 - column, row),
                CoverDirection::Right => (column, row),
                CoverDirection::Up => (1.0 - row, column),
                CoverDirection::Down => (row, column),
            };
            along + WAVE_LEAN * across + WAVE_RAGGED * ragged[cell] as f32 / count as f32
        })
        .collect();
    let mut cells: Vec<usize> = (0..count).collect();
    cells.sort_by(|a, b| keys[*a].total_cmp(&keys[*b]));
    let mut rank = vec![0; count];
    for (position, cell) in cells.into_iter().enumerate() {
        rank[cell] = position;
    }
    rank
}

/// The position of each of `count` cells in a fixed scattered order.
fn scattered(count: usize, seed: u32) -> Vec<usize> {
    let mut state = seed.max(1);
    let mut next = move || {
        state ^= state << 13;
        state ^= state >> 17;
        state ^= state << 5;
        state
    };
    let mut cells: Vec<usize> = (0..count).collect();
    for i in (1..count).rev() {
        let j = next() as usize % (i + 1);
        cells.swap(i, j);
    }
    let mut rank = vec![0; count];
    for (position, cell) in cells.into_iter().enumerate() {
        rank[cell] = position;
    }
    rank
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::geometry::Size;

    const SCENE: Rgba = Rgba::opaque(10, 200, 30);
    const SIZES: [(u32, u32); 5] = [(1920, 480), (320, 480), (480, 480), (800, 480), (7, 5)];
    const EFFECTS: [CoverEffect; 7] = [
        CoverEffect::Curtain,
        CoverEffect::Wipe,
        CoverEffect::Blinds,
        CoverEffect::Mosaic,
        CoverEffect::Iris,
        CoverEffect::Tv,
        CoverEffect::Pixels,
    ];
    const DIRECTIONS: [CoverDirection; 4] = [
        CoverDirection::Left,
        CoverDirection::Right,
        CoverDirection::Up,
        CoverDirection::Down,
    ];

    fn cover(effect: CoverEffect, direction: CoverDirection) -> Cover {
        Cover {
            effect,
            direction,
            panel: Rgba {
                a: 40,
                ..DEFAULT_PANEL
            },
            accent: DEFAULT_ACCENT,
        }
    }

    fn painted(size: Size, cover: &Cover, stage: CoverStage, c: f32) -> Frame {
        let mut frame = Frame::filled(size, Rgba { a: 0, ..SCENE });
        cover.paint(&mut frame, stage, c);
        frame
    }

    fn covered(frame: &Frame) -> usize {
        frame
            .as_rgba()
            .as_chunks::<4>()
            .0
            .iter()
            .filter(|p| p[3] == 255)
            .count()
    }

    #[test]
    fn every_cover_spans_nothing_then_everything_on_any_canvas() {
        for (w, h) in SIZES {
            let size = Size::new(w, h);
            for effect in EFFECTS {
                for direction in DIRECTIONS {
                    let c = cover(effect, direction);
                    for stage in [CoverStage::Closing, CoverStage::Opening] {
                        let empty = painted(size, &c, stage, 0.0);
                        assert_eq!(covered(&empty), 0, "{effect:?} {w}x{h}");
                        let full = painted(size, &c, stage, 1.0);
                        assert!(
                            full.as_rgba()
                                .as_chunks::<4>()
                                .0
                                .iter()
                                .all(|p| *p == [0x12, 0x0b, 0x13, 255]),
                            "{effect:?} {direction:?} {w}x{h} leaves a gap"
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn covers_grow_with_coverage_and_paint_only_opaque_pixels() {
        for (w, h) in SIZES.into_iter().take(4) {
            let size = Size::new(w, h);
            for effect in EFFECTS {
                for direction in DIRECTIONS {
                    let c = cover(effect, direction);
                    let mut last = 0;
                    for step in [0.2, 0.45, 0.7, 0.95] {
                        let frame = painted(size, &c, CoverStage::Closing, step);
                        let now = covered(&frame);
                        // A television is all dark but its line before the end.
                        assert!(
                            now > last || now == (w * h) as usize,
                            "{effect:?} {direction:?} {w}x{h} at {step}"
                        );
                        last = now;
                        for p in frame.as_rgba().as_chunks::<4>().0 {
                            assert!(p[3] == 0 || p[3] == 255);
                            assert!(p[3] == 255 || p[..3] == [SCENE.r, SCENE.g, SCENE.b]);
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn a_wipe_enters_on_one_side_and_leaves_on_the_other() {
        let size = Size::new(1920, 480);
        let c = cover(CoverEffect::Wipe, CoverDirection::Left);
        let closing = painted(size, &c, CoverStage::Closing, 0.3);
        assert_eq!(closing.pixel(1919, 240).unwrap().a, 255);
        assert_eq!(closing.pixel(0, 240).unwrap().a, 0);
        let opening = painted(size, &c, CoverStage::Opening, 0.3);
        assert_eq!(opening.pixel(0, 240).unwrap().a, 255);
        assert_eq!(opening.pixel(1919, 240).unwrap().a, 0);
    }

    #[test]
    fn a_curtain_meets_in_the_middle_with_a_lit_edge() {
        let size = Size::new(1920, 480);
        let frame = painted(
            size,
            &cover(CoverEffect::Curtain, CoverDirection::Right),
            CoverStage::Closing,
            0.5,
        );
        assert_eq!(frame.pixel(0, 0).unwrap().a, 255);
        assert_eq!(frame.pixel(1919, 0).unwrap().a, 255);
        assert_eq!(frame.pixel(960, 240).unwrap().a, 0);
        assert_eq!(frame.pixel(479, 240), Some(DEFAULT_ACCENT));
    }

    #[test]
    fn mosaic_tiles_follow_the_short_side() {
        // 1920 x 480: tiles of 40 px, 48 x 12; the order covers each tile once.
        let rank = wave(48, 12, CoverDirection::Left);
        let mut seen = rank.clone();
        seen.sort_unstable();
        assert_eq!(seen, (0..576).collect::<Vec<_>>());
        let frame = painted(
            Size::new(1920, 480),
            &cover(CoverEffect::Mosaic, CoverDirection::Left),
            CoverStage::Closing,
            0.5,
        );
        for x in (0..1920).step_by(40) {
            for y in (0..480).step_by(40) {
                let corner = frame.pixel(x, y).unwrap();
                let opposite = frame.pixel(x + 39, y + 39).unwrap();
                assert_eq!(corner, opposite);
            }
        }
    }

    #[test]
    fn a_mosaic_sweeps_as_a_wave_towards_its_direction() {
        let size = Size::new(1920, 480);
        for (direction, first, last) in [
            (CoverDirection::Left, (1900, 20), (20, 460)),
            (CoverDirection::Right, (20, 20), (1900, 460)),
            (CoverDirection::Up, (20, 460), (1900, 20)),
            (CoverDirection::Down, (20, 20), (1900, 460)),
        ] {
            let frame = painted(
                size,
                &cover(CoverEffect::Mosaic, direction),
                CoverStage::Closing,
                0.5,
            );
            assert_eq!(
                frame.pixel(first.0, first.1).map(|p| p.a),
                Some(255),
                "{direction:?} starts there"
            );
            assert_eq!(
                frame.pixel(last.0, last.1).map(|p| p.a),
                Some(0),
                "{direction:?} ends there"
            );
        }
        // Opening goes out in the same wave: where it started, it is gone first.
        let opening = painted(
            size,
            &cover(CoverEffect::Mosaic, CoverDirection::Right),
            CoverStage::Opening,
            0.5,
        );
        assert_eq!(opening.pixel(1900, 460).map(|p| p.a), Some(255));
    }

    #[test]
    fn the_edge_light_spills_on_the_hosts_pixels_never_on_the_screens_video() {
        let size = Size::new(1920, 480);
        let wipe = cover(CoverEffect::Wipe, CoverDirection::Left);
        // Left half drawn by the host (opaque), right half the screen's video (alpha 0).
        let mut frame = Frame::filled(size, SCENE);
        frame.fill_rect(Rect::new(960, 0, 960, 480), Rgba { a: 0, ..SCENE });
        wipe.paint(&mut frame, CoverStage::Closing, 0.6);
        // Eased once more, the edge stands at 492 (1920 * (1 - ease(0.6))), the panel to its right.
        let ahead = frame.pixel(485, 100).unwrap();
        assert_eq!(ahead.a, 255);
        assert_ne!(ahead, SCENE, "the opaque scene ahead of the edge is lit");
        assert_eq!(
            frame.pixel(300, 100),
            Some(SCENE),
            "far from the edge it is untouched"
        );
        let mut video = Frame::filled(size, Rgba { a: 0, ..SCENE });
        wipe.paint(&mut video, CoverStage::Closing, 0.6);
        assert_eq!(video.pixel(485, 100), Some(Rgba { a: 0, ..SCENE }));
    }

    #[test]
    fn a_mosaic_tile_grows_from_its_middle() {
        let size = Size::new(480, 480);
        let mosaic = cover(CoverEffect::Mosaic, CoverDirection::Left);
        // 12 x 12 tiles of 40 px; the first in order starts at 0.
        let first = wave(12, 12, CoverDirection::Left)
            .iter()
            .position(|&r| r == 0)
            .unwrap();
        let (x, y) = ((first % 12) as u32 * 40, (first / 12) as u32 * 40);
        let half = painted(size, &mosaic, CoverStage::Closing, 0.06);
        assert_eq!(half.pixel(x + 20, y + 20), Some(DEFAULT_ACCENT));
        assert_eq!(half.pixel(x + 1, y + 1).unwrap().a, 0);
        let whole = painted(size, &mosaic, CoverStage::Closing, 0.3);
        assert_eq!(whole.pixel(x + 1, y + 1).map(|p| p.a), Some(255));
    }

    #[test]
    fn curtain_leaves_light_the_gap_as_they_meet() {
        let size = Size::new(1920, 480);
        let curtain = cover(CoverEffect::Curtain, CoverDirection::Right);
        let mut far = Frame::filled(size, SCENE);
        curtain.paint(&mut far, CoverStage::Closing, 0.5);
        assert_eq!(
            far.pixel(960, 240),
            Some(SCENE),
            "far apart, the middle is untouched"
        );
        let mut near = Frame::filled(size, SCENE);
        curtain.paint(&mut near, CoverStage::Closing, 0.95);
        let middle = near.pixel(960, 240).unwrap();
        assert!(
            middle != SCENE && middle.r > SCENE.r,
            "close, it flares: {middle:?}"
        );
    }

    #[test]
    fn an_iris_closes_on_the_middle_behind_a_ring_of_light() {
        let size = Size::new(1920, 480);
        let iris = cover(CoverEffect::Iris, CoverDirection::Left);
        let frame = painted(size, &iris, CoverStage::Closing, 0.8);
        assert_eq!(
            frame.pixel(0, 0).map(|p| p.a),
            Some(255),
            "corners close first"
        );
        assert_eq!(
            frame.pixel(960, 240).map(|p| p.a),
            Some(0),
            "the middle last"
        );
        let lit = (960..1920)
            .filter_map(|x| frame.pixel(x, 240))
            .any(|p| p == DEFAULT_ACCENT);
        assert!(lit, "a ring of accent around the opening");
        let start = painted(size, &iris, CoverStage::Closing, 0.005);
        assert_eq!(covered(&start), 0, "it starts outside the screen");
    }

    #[test]
    fn a_tv_squeezes_into_a_line_that_shrinks_to_a_dot() {
        let size = Size::new(1920, 480);
        let tv = cover(CoverEffect::Tv, CoverDirection::Left);
        let squeezing = painted(size, &tv, CoverStage::Closing, 0.3);
        assert_eq!(squeezing.pixel(960, 5).map(|p| p.a), Some(255));
        assert_eq!(squeezing.pixel(960, 240).map(|p| p.a), Some(0));
        let line = painted(size, &tv, CoverStage::Closing, 0.8);
        assert_eq!(
            line.pixel(960, 240),
            Some(DEFAULT_ACCENT),
            "the line in the middle"
        );
        assert_ne!(line.pixel(20, 240), Some(DEFAULT_ACCENT), "already shorter");
        assert_eq!(covered(&line), 1920 * 480);
        let dot = painted(size, &tv, CoverStage::Closing, 0.999);
        let lit = dot
            .as_rgba()
            .as_chunks::<4>()
            .0
            .iter()
            .filter(|p| p[..3] == [DEFAULT_ACCENT.r, DEFAULT_ACCENT.g, DEFAULT_ACCENT.b])
            .count();
        assert!(lit > 0 && lit < 400, "a dot of {lit} pixels");
    }

    #[test]
    fn pixels_are_small_whole_squares() {
        // 1920 x 480: pixels of 30 px, 64 x 16; each one is whole or absent.
        let frame = painted(
            Size::new(1920, 480),
            &cover(CoverEffect::Pixels, CoverDirection::Left),
            CoverStage::Closing,
            0.5,
        );
        for x in (0..1920).step_by(30) {
            for y in (0..480).step_by(30) {
                assert_eq!(frame.pixel(x, y), frame.pixel(x + 29, y + 29));
            }
        }
        assert_eq!(EFFECTS.iter().filter(|e| e.directed()).count(), 4);
    }

    #[test]
    fn ease_is_bounded_and_symmetric() {
        assert_eq!(ease(-1.0), 0.0);
        assert_eq!(ease(2.0), 1.0);
        assert!((ease(0.5) - 0.5).abs() < 1e-6);
        assert!((ease(0.25) + ease(0.75) - 1.0).abs() < 1e-6);
    }
}
