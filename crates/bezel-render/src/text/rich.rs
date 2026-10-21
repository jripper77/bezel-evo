//! Text in pieces with styles of their own: the parts of a value or of a
//! digital clock (D-2026-10-20-studio-ux-v2-06), digits of equal width
//! (D-2026-10-20-studio-ux-v2-05), lines wider than their box (shrink,
//! ellipsis, scroll) and numbers whose digits roll like wheels.
//!
//! Each piece of a line is shaped on its own and the pieces are set side by
//! side on one baseline; a raised piece sits higher. Digits take the advance
//! of the widest digit: the font's `tnum` when it has one, else the glyphs
//! are spread to that advance.

use std::time::Duration;

use bezel_core::domain::theme::{Baseline, HAlign, Paint, PartStyle, TextOverflow, VAlign};
use cosmic_text::{FeatureTag, FontFeatures, LayoutGlyph};

use super::*;

/// Time between two frames of a scrolling line.
pub(crate) const SCROLL_STEP: Duration = Duration::from_millis(50);
/// How fast a line scrolls, pixels a second.
const SCROLL_SPEED: f32 = 40.0;
/// Space between the end of a scrolling line and its next turn, in text sizes.
const SCROLL_GAP: f32 = 2.0;
/// Height of a capital over the baseline, in text sizes: a raised part's top
/// is level with the number's.
const CAP_HEIGHT: f32 = 0.7;
/// The ellipsis that ends a cut line.
const ELLIPSIS: &str = "…";
/// Most entries kept in the cache of digit glyphs before it is emptied.
const DIGIT_CACHE: usize = 256;

/// A piece of text with one style.
pub(crate) struct Piece<'a> {
    /// What it says.
    pub text: String,
    /// Its own style; `None` follows the text.
    pub part: Option<&'a PartStyle>,
    /// Keeps its place but is not drawn (a blinking separator, off).
    pub hidden: bool,
    /// Wheels of a rolling number, one per ASCII digit of `text`, left to
    /// right: the digit shown and how far it has turned to the next.
    pub wheels: Option<Vec<(u8, f32)>>,
}

impl Piece<'_> {
    /// A piece with the text's style.
    pub fn plain(text: String) -> Self {
        Piece {
            text,
            part: None,
            hidden: false,
            wheels: None,
        }
    }
}

/// Pieces of text to draw in a box.
pub(crate) struct RichJob<'a> {
    /// The pieces, in reading order; a `\n` starts a new line.
    pub pieces: Vec<Piece<'a>>,
    /// The text's style.
    pub style: &'a TextStyle,
    /// The box it is aligned in.
    pub area: BoxF,
    /// Digits of equal width.
    pub tabular: bool,
    /// How long the theme has been showing, for a scrolling line; `None`
    /// holds it still at its start (reduced motion).
    pub clock: Option<Duration>,
}

/// The glyphs of the ten digits in one face and size, and the widest one's
/// advance.
#[derive(Clone)]
pub(crate) struct Digits {
    glyphs: Vec<LayoutGlyph>,
    advance: f32,
}

/// What identifies a face and size for [`Digits`].
pub(crate) type DigitKey = (String, u16, bool, u32, u32, bool);

/// A face, its size and its spacing.
#[derive(Clone)]
struct Face {
    family: String,
    weight: u16,
    italic: bool,
    size: f32,
    spacing: f32,
}

/// A glyph set on a line: `x` is the left of its slot, `slot` its width
/// (wider than the glyph for a spread digit).
struct Placed {
    glyph: LayoutGlyph,
    x: f32,
    slot: f32,
    digit: Option<usize>,
}

/// A piece on one line.
struct Frag {
    placed: Vec<Placed>,
    width: f32,
    /// Baseline below the top of its own line box.
    ascent: f32,
    height: f32,
    /// How far above the line's baseline it sits.
    raise: f32,
    paint: Option<Paint>,
    hidden: bool,
    wheels: Option<Vec<(u8, f32)>>,
    digits: Option<Digits>,
}

/// One line of fragments.
struct Line {
    frags: Vec<Frag>,
    width: f32,
    ascent: f32,
    height: f32,
}

impl Line {
    fn measure(&mut self, fallback: (f32, f32)) {
        self.width = self.frags.iter().map(|f| f.width).sum();
        if self.frags.is_empty() {
            (self.ascent, self.height) = fallback;
            return;
        }
        self.ascent = self
            .frags
            .iter()
            .map(|f| f.ascent + f.raise)
            .fold(0.0, f32::max);
        let below = self
            .frags
            .iter()
            .map(|f| f.height - f.ascent - f.raise)
            .fold(0.0, f32::max);
        self.height = self.ascent + below;
    }
}

fn is_digit(text: &str) -> bool {
    text.len() == 1 && text.as_bytes()[0].is_ascii_digit()
}

impl TextEngine {
    fn attrs<'b>(face: &'b Face, tabular: bool) -> Attrs<'b> {
        let mut attrs = Attrs::new()
            .family(Family::Name(&face.family))
            .weight(Weight(face.weight.clamp(1, 1000)))
            .style(if face.italic {
                Style::Italic
            } else {
                Style::Normal
            });
        if face.spacing.is_finite() && face.spacing != 0.0 {
            attrs = attrs.letter_spacing(face.spacing / face.size);
        }
        if tabular {
            let mut features = FontFeatures::new();
            features.enable(FeatureTag::new(b"tnum"));
            attrs = attrs.font_features(features);
        }
        attrs
    }

    /// One line of `text` shaped in `face`: its glyphs, width, baseline and
    /// height.
    fn shape_line(
        &mut self,
        text: &str,
        face: &Face,
        tabular: bool,
    ) -> (Vec<LayoutGlyph>, f32, f32, f32) {
        let metrics = Metrics::new(face.size, (face.size * LINE_HEIGHT).ceil());
        let mut buffer = Buffer::new(&mut self.fonts, metrics);
        buffer.set_wrap(Wrap::None);
        buffer.set_size(None, None);
        buffer.set_text(text, &Self::attrs(face, tabular), Shaping::Advanced, None);
        buffer.shape_until_scroll(&mut self.fonts, false);
        let fallback = (metrics.line_height * 0.8, metrics.line_height);
        match buffer.layout_runs().next() {
            Some(run) => (run.glyphs.to_vec(), run.line_w, run.line_y, run.line_height),
            None => (Vec::new(), 0.0, fallback.0, fallback.1),
        }
    }

    /// The ten digits of `face` and the widest one's advance (cached).
    fn digits(&mut self, face: &Face, tabular: bool) -> Digits {
        let key: DigitKey = (
            face.family.clone(),
            face.weight,
            face.italic,
            face.size.to_bits(),
            face.spacing.to_bits(),
            tabular,
        );
        if let Some(digits) = self.digits.get(&key) {
            return digits.clone();
        }
        let (glyphs, ..) = self.shape_line("0123456789", face, tabular);
        let advance = glyphs.iter().map(|g| g.w).fold(0.0, f32::max);
        let digits = Digits { glyphs, advance };
        if self.digits.len() >= DIGIT_CACHE {
            self.digits.clear();
        }
        self.digits.insert(key, digits.clone());
        digits
    }

    /// A fragment of `text` in `face`, digits of equal width when
    /// `tabular`. `digit` counts the piece's digits before it.
    fn frag(&mut self, text: &str, face: &Face, tabular: bool, digit: &mut usize) -> Frag {
        self.spread_frag(text, face, (tabular, tabular), digit)
    }

    /// [`Self::frag`] asking the font for `tnum` and spreading the digits
    /// to the widest one's advance separately (`(tnum, spread)`): without
    /// `tnum` in the font, the spreading alone makes them equal.
    fn spread_frag(
        &mut self,
        text: &str,
        face: &Face,
        (tnum, tabular): (bool, bool),
        digit: &mut usize,
    ) -> Frag {
        let (glyphs, width, ascent, height) = self.shape_line(text, face, tnum);
        let digits =
            (tabular || text.bytes().any(|b| b.is_ascii_digit())).then(|| self.digits(face, tnum));
        let mut shift = 0.0;
        let mut placed = Vec::with_capacity(glyphs.len());
        for glyph in glyphs {
            let numeral = text.get(glyph.start..glyph.end).is_some_and(is_digit);
            let slot = match &digits {
                Some(d) if numeral && tabular => d.advance.max(glyph.w),
                _ => glyph.w,
            };
            let x = glyph.x + shift;
            shift += slot - glyph.w;
            let index = numeral.then(|| {
                *digit += 1;
                *digit - 1
            });
            placed.push(Placed {
                glyph,
                x,
                slot,
                digit: index,
            });
        }
        Frag {
            placed,
            width: width + shift,
            ascent,
            height,
            raise: 0.0,
            paint: None,
            hidden: false,
            wheels: None,
            digits,
        }
    }

    /// The lines of `job` at `scale` times their size.
    fn lay(
        &mut self,
        job: &RichJob<'_>,
        scale: f32,
        assets: &BTreeMap<AssetRef, Vec<u8>>,
        diagnostics: &mut Diagnostics,
    ) -> Option<Vec<Line>> {
        let style = job.style;
        let size = (style.size * scale).min(MAX_FONT_SIZE);
        if !size.is_finite() || size < 1.0 {
            return None;
        }
        let family = self.family_for(&style.font, assets, diagnostics)?;
        let base = Face {
            family,
            weight: style.font.weight,
            italic: style.font.italic,
            size,
            spacing: style.letter_spacing * scale,
        };
        let fallback = {
            let height = (size * LINE_HEIGHT).ceil();
            (height * 0.8, height)
        };
        let mut lines = vec![Line {
            frags: Vec::new(),
            width: 0.0,
            ascent: 0.0,
            height: 0.0,
        }];
        for piece in &job.pieces {
            let face = match piece.part {
                None => base.clone(),
                Some(part) => {
                    let spec = FontSpec {
                        family: part
                            .family
                            .clone()
                            .unwrap_or_else(|| style.font.family.clone()),
                        weight: part.weight.unwrap_or(style.font.weight),
                        italic: style.font.italic,
                        asset: if part.family.is_some() {
                            part.asset.clone()
                        } else {
                            style.font.asset.clone()
                        },
                    };
                    let family = self
                        .family_for(&spec, assets, diagnostics)
                        .unwrap_or_else(|| base.family.clone());
                    Face {
                        family,
                        weight: spec.weight,
                        italic: spec.italic,
                        size: part.size(size).clamp(1.0, MAX_FONT_SIZE),
                        spacing: base.spacing,
                    }
                }
            };
            let raise = match piece.part {
                Some(part) if part.baseline == Baseline::Super => {
                    ((size - face.size) * CAP_HEIGHT).max(0.0)
                }
                _ => 0.0,
            };
            let text = style.casing.apply(&piece.text);
            let mut digit = 0;
            for (index, chunk) in text.split('\n').enumerate() {
                if index > 0 {
                    lines.push(Line {
                        frags: Vec::new(),
                        width: 0.0,
                        ascent: 0.0,
                        height: 0.0,
                    });
                }
                if chunk.is_empty() {
                    continue;
                }
                let mut frag = self.frag(chunk, &face, job.tabular, &mut digit);
                frag.raise = raise;
                frag.paint = piece.part.and_then(|p| p.color).map(Paint::Solid);
                frag.hidden = piece.hidden;
                frag.wheels = piece.wheels.clone();
                if let Some(line) = lines.last_mut() {
                    line.frags.push(frag);
                }
            }
        }
        for line in &mut lines {
            line.measure(fallback);
        }
        lines.iter().any(|l| !l.frags.is_empty()).then_some(lines)
    }

    /// Cuts `line` to `width`, ending it with an ellipsis in the style of
    /// its last piece.
    fn ellipsize(&mut self, line: &mut Line, width: f32, base: Option<Face>) {
        let Some(face) = base else { return };
        let mut none = 0;
        let ellipsis = self.frag(ELLIPSIS, &face, false, &mut none);
        let room = width - ellipsis.width;
        while let Some((last, rest)) = line.frags.split_last_mut() {
            let before: f32 = rest.iter().map(|f| f.width).sum();
            while last
                .placed
                .last()
                .is_some_and(|p| before + p.x + p.slot > room)
            {
                last.placed.pop();
            }
            last.width = last.placed.last().map_or(0.0, |p| p.x + p.slot);
            if !last.placed.is_empty() {
                break;
            }
            line.frags.pop();
        }
        let mut ellipsis = ellipsis;
        if let Some(last) = line.frags.last() {
            ellipsis.paint = last.paint.clone();
        }
        line.frags.push(ellipsis);
        line.width = line.frags.iter().map(|f| f.width).sum();
    }

    /// Natural size of `job`'s text: its widest line and its height.
    pub fn measure_rich(
        &mut self,
        job: &RichJob<'_>,
        assets: &BTreeMap<AssetRef, Vec<u8>>,
        diagnostics: &mut Diagnostics,
    ) -> Option<(f32, f32)> {
        let lines = self.lay(job, 1.0, assets, diagnostics)?;
        let width = lines.iter().map(|l| l.width).fold(0.0, f32::max);
        let height: f32 = lines.iter().map(|l| l.height).sum();
        Some((width.ceil().max(1.0), height.ceil().max(1.0)))
    }

    /// Draws `job` into `layer`; returns when a scrolling line wants its
    /// next frame, from now.
    pub fn draw_rich(
        &mut self,
        layer: &mut Layer<'_>,
        job: &RichJob<'_>,
        assets: &BTreeMap<AssetRef, Vec<u8>>,
        diagnostics: &mut Diagnostics,
    ) -> Option<Duration> {
        let area = job.area;
        let mut lines = self.lay(job, 1.0, assets, diagnostics)?;
        let widest = lines.iter().map(|l| l.width).fold(0.0, f32::max);
        let overflow = job
            .style
            .overflow
            .filter(|_| widest > area.width && area.width > 0.0);
        let mut scale = 1.0;
        if overflow == Some(TextOverflow::Shrink) {
            let mut widest = widest;
            for _ in 0..3 {
                if widest <= area.width {
                    break;
                }
                scale *= area.width / widest;
                lines = self.lay(job, scale, assets, diagnostics)?;
                widest = lines.iter().map(|l| l.width).fold(0.0, f32::max);
            }
        }
        if overflow == Some(TextOverflow::Ellipsis) {
            let face = self
                .family_for(&job.style.font, assets, diagnostics)
                .map(|family| Face {
                    family,
                    weight: job.style.font.weight,
                    italic: job.style.font.italic,
                    size: job.style.size.min(MAX_FONT_SIZE),
                    spacing: job.style.letter_spacing,
                });
            for line in lines.iter_mut().filter(|l| l.width > area.width) {
                self.ellipsize(line, area.width, face.clone());
            }
        }
        let scrolling = overflow == Some(TextOverflow::Scroll);
        let gap = job.style.size * SCROLL_GAP;
        let travelled = job.clock.map_or(0.0, |t| t.as_secs_f32() * SCROLL_SPEED);

        let (w, h) = (layer.pixmap.width(), layer.pixmap.height());
        let blank = || Coverage {
            data: vec![0; w as usize * h as usize],
            width: w as i32,
            height: h as i32,
            ink: None,
        };
        let mut paints: Vec<(Option<Paint>, Coverage)> = vec![(None, blank())];
        let mut color_glyphs = Vec::new();
        let total: f32 = lines.iter().map(|l| l.height).sum();
        let mut top = area.y
            + match job.style.valign {
                VAlign::Top => 0.0,
                VAlign::Middle => (area.height - total) / 2.0,
                VAlign::Bottom => area.height - total,
            };
        let mut moving = false;
        for line in &lines {
            let mut starts = Vec::with_capacity(2);
            if scrolling && line.width > area.width {
                let turn = line.width + gap;
                let left = area.x - travelled % turn;
                starts.extend([left, left + turn]);
                moving |= job.clock.is_some();
            } else {
                starts.push(
                    area.x
                        + match job.style.align {
                            HAlign::Left => 0.0,
                            HAlign::Center => (area.width - line.width) / 2.0,
                            HAlign::Right => area.width - line.width,
                        },
                );
            }
            let baseline = top + line.ascent;
            let band = (
                top.floor() as i32 - layer.y,
                (top + line.height).ceil() as i32 - layer.y,
            );
            for start in starts {
                let mut x = start;
                for frag in &line.frags {
                    if !frag.hidden {
                        let index = match paints.iter().position(|(p, _)| *p == frag.paint) {
                            Some(index) => index,
                            None => {
                                paints.push((frag.paint.clone(), blank()));
                                paints.len() - 1
                            }
                        };
                        let y = baseline - frag.raise;
                        for placed in &frag.placed {
                            let wheel = placed
                                .digit
                                .and_then(|d| frag.wheels.as_ref()?.get(d).copied());
                            let left = x + placed.x;
                            match (wheel, &frag.digits) {
                                (Some((digit, turn)), Some(digits)) => {
                                    for (n, dy) in [
                                        (digit, -turn * line.height),
                                        ((digit + 1) % 10, (1.0 - turn) * line.height),
                                    ] {
                                        if (n != digit && turn <= 0.0) || dy.abs() >= line.height {
                                            continue;
                                        }
                                        let Some(glyph) = digits.glyphs.get(usize::from(n)) else {
                                            continue;
                                        };
                                        let at = left + (placed.slot - glyph.w) / 2.0;
                                        self.put(
                                            layer,
                                            glyph,
                                            (at, y + dy),
                                            Some(band),
                                            &mut paints[index].1,
                                            &mut color_glyphs,
                                        );
                                    }
                                }
                                _ => {
                                    let at = left + (placed.slot - placed.glyph.w) / 2.0;
                                    self.put(
                                        layer,
                                        &placed.glyph,
                                        (at, y),
                                        None,
                                        &mut paints[index].1,
                                        &mut color_glyphs,
                                    );
                                }
                            }
                        }
                    }
                    x += frag.width;
                }
            }
            top += line.height;
        }
        if let Some(shadow) = job.style.shadow {
            shadow::draw(
                layer,
                paints.iter().map(|(_, c)| c.data.clone()),
                &color_glyphs,
                shadow,
            );
        }
        for (paint, coverage) in paints {
            let Some((l, t, r, b)) = coverage.ink else {
                continue;
            };
            let ink = Rect::from_xywh(
                (layer.x + l) as f32,
                (layer.y + t) as f32,
                (r - l) as f32,
                (b - t) as f32,
            );
            let paint = paint.as_ref().unwrap_or(&job.style.paint);
            let (Some(mask), Some(ink), Some(paint)) = (
                IntSize::from_wh(w, h).and_then(|size| Mask::from_vec(coverage.data, size)),
                ink,
                paint_for(paint, job.area),
            ) else {
                continue;
            };
            layer.fill_rect_masked(ink, &paint, &mask);
        }
        for (pixmap, x, y) in color_glyphs {
            layer.blit(pixmap.as_ref(), x, y);
        }
        moving.then_some(SCROLL_STEP)
    }

    /// Rasterizes `glyph` with its left at `x` and its baseline at `y`
    /// (canvas pixels), clipped to the layer `rows` when given.
    fn put(
        &mut self,
        layer: &Layer<'_>,
        glyph: &LayoutGlyph,
        (x, y): (f32, f32),
        rows: Option<(i32, i32)>,
        coverage: &mut Coverage,
        color_glyphs: &mut Vec<(Pixmap, i32, i32)>,
    ) {
        let physical = glyph.physical((x - glyph.x, y), 1.0);
        let Some(image) = self.glyphs.get_image(&mut self.fonts, physical.cache_key) else {
            return;
        };
        let gx = physical.x + image.placement.left;
        let gy = physical.y - image.placement.top;
        match image.content {
            SwashContent::Mask => coverage.add_rows(
                image,
                gx - layer.x,
                gy - layer.y,
                rows.unwrap_or((i32::MIN, i32::MAX)),
            ),
            SwashContent::Color => color_glyphs.extend(color_glyph(image).map(|p| (p, gx, gy))),
            SwashContent::SubpixelMask => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn inter() -> (TextEngine, Face) {
        let engine = TextEngine::new(
            vec![crate::fonts::SHIPPED_FONTS[0].to_vec()],
            SystemFonts::Skip,
        );
        let face = Face {
            family: "Inter".into(),
            weight: 400,
            italic: false,
            size: 64.0,
            spacing: 0.0,
        };
        (engine, face)
    }

    #[test]
    fn digits_are_as_wide_as_each_other_with_tnum_and_without() {
        let (mut engine, face) = inter();
        let width = |engine: &mut TextEngine, text: &str, mode| {
            engine.spread_frag(text, &face, mode, &mut 0).width
        };
        let (ones, eights) = (
            width(&mut engine, "11", (false, false)),
            width(&mut engine, "88", (false, false)),
        );
        assert!(
            ones + 4.0 < eights,
            "Inter's digits are proportional: {ones} {eights}"
        );
        for mode in [(true, true), (false, true)] {
            let (ones, eights) = (
                width(&mut engine, "11", mode),
                width(&mut engine, "88", mode),
            );
            assert!((ones - eights).abs() < 0.01, "{mode:?}: {ones} vs {eights}");
        }
        let spread = engine.spread_frag("1a1", &face, (false, true), &mut 0);
        assert_eq!(
            spread.placed.iter().filter(|p| p.digit.is_some()).count(),
            2
        );
        let one = &spread.placed[0];
        assert!(one.slot > one.glyph.w, "a narrow digit gets a wider slot");
        assert_eq!(engine.digits.len(), 2, "cached per face and tnum");
        engine.digits(&face, false);
        assert_eq!(engine.digits.len(), 2);
    }
}
