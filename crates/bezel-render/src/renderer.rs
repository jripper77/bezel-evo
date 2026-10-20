//! [`SkiaRenderer`]: the [`FrameRenderer`] adapter.

use std::collections::{BTreeMap, HashMap, HashSet};

use bezel_core::domain::animation::Timeline;
use bezel_core::domain::clock::{clock_case, clock_parts, format_clock};
use bezel_core::domain::frame::{Frame, Rgba};
use bezel_core::domain::geometry::Size;
use bezel_core::domain::history::SeriesId;
use bezel_core::domain::playback::PROGRESS_SPACING;
use bezel_core::domain::sensor::{Reading, format_reading, format_reading_parts, fraction};
use bezel_core::domain::theme::{
    AssetRef, Background, Binding, BoxF, Element, ElementId, ElementKind, Fit, ImageRepeat,
    ShapeImage, TextContent, TextOverflow, TextPart, TextStyle, Theme,
};
use bezel_core::ports::{Backdrop, FrameRenderer, RenderContext};
use bezel_core::{BezelError, Result};
use image::RgbaImage;
use image::imageops::{self, FilterType};
use tiny_skia::{Color, IntRect, Pixmap, PixmapMut, Rect, Transform};

use crate::catalog::FontCatalog;
use crate::composite::{self, Sprite, Target};
use crate::diagnostics::Diagnostics;
use crate::gauges::{self, BarStyle, NeedlePose, RingStyle};
use crate::graph::{self, GraphSpec};
use crate::images::{self, ImageCache};
use crate::layer::Layer;
use crate::paint;
use crate::shape;
use crate::text::{Piece, RichJob, SystemFonts, TextEngine, TextJob};

/// Half a blink of clock separators: shown for the first half of every
/// second, hidden for the second.
const BLINK_HALF: std::time::Duration = std::time::Duration::from_millis(500);

/// Decorations around a text run, shared by drawing and natural sizing.
#[derive(Default)]
struct TextFurniture {
    side: Option<f32>,
    fraction: f32,
    gap: f32,
}
impl TextFurniture {
    fn new(
        content: &TextContent,
        style: &bezel_core::domain::theme::TextStyle,
        context: &RenderContext<'_>,
    ) -> Self {
        match content {
            TextContent::Weather(w) if w.show_icon => Self {
                side: Some(w.icon_size.unwrap_or(style.size * 2.)),
                fraction: if w.icon_size.is_none() { 0.3 } else { 1. },
                gap: w.icon_gap.unwrap_or(style.size * 0.3).max(0.),
            },
            TextContent::Player(p) => {
                let Some(media) =
                    bezel_core::domain::playback::session(context.snapshot, &p.source)
                else {
                    return Self::default();
                };
                if !p.show_cover {
                    return Self::default();
                }
                let shown = p.text(media);
                // A cover with nothing beside it fills the box.
                let alone = shown.lines.is_empty() && shown.progress.is_none();
                Self {
                    side: Some(f32::INFINITY),
                    fraction: if alone { 1. } else { 0.4 },
                    gap: if alone {
                        0.
                    } else {
                        p.cover_gap.unwrap_or(style.size * 0.3).max(0.)
                    },
                }
            }
            _ => Self::default(),
        }
    }
    fn layout(&self, area: BoxF) -> (f32, f32) {
        let gap = self.gap.min(area.width.max(0.));
        let side = self
            .side
            .unwrap_or(0.)
            .min(area.height)
            .min(area.width * self.fraction)
            .min((area.width - gap).max(0.));
        (side, gap)
    }
    fn fit(&self, width: f32, height: f32) -> (f32, f32) {
        if let Some(side) = self.side {
            let side = if side.is_finite() { side } else { height };
            (
                (width + side + self.gap).max(side / self.fraction).ceil(),
                height.max(side).ceil(),
            )
        } else {
            (width.ceil(), height.ceil())
        }
    }
}

/// Largest canvas side rendered, pixels.
const MAX_CANVAS_SIDE: u32 = 8192;
/// Shown under a video background that has no poster.
const VIDEO_PLACEHOLDER: Rgba = Rgba::opaque(16, 17, 22);

/// Draws themes with tiny-skia (paths, gradients, compositing), cosmic-text
/// (shaping) and image (PNG, JPEG, GIF). Build it once and reuse it: fonts,
/// glyphs and decoded images stay cached between frames.
pub struct SkiaRenderer {
    text: TextEngine,
    images: ImageCache,
    diagnostics: Diagnostics,
    /// Layer pixels of the element being drawn (reused allocation).
    scratch: Vec<u8>,
    /// The previous canvas, reused when the size matches.
    canvas: Option<Pixmap>,
    /// Drawn layers of elements that look the same on every frame, by
    /// z-index and id (ids may repeat in imported themes).
    layers: HashMap<LayerKey, CachedLayer>,
    used_layers: HashSet<LayerKey>,
    cards: crate::cards::Cards,
    card_buffers: Vec<Pixmap>,
    face_cache: HashMap<FaceKey, CachedFace>,
    content_revision: u64,
    face_cache_enabled: bool,
    render_stats: bezel_core::ports::RenderStats,
    card_scene: u64,
    font_poll_due: Option<std::time::Duration>,
    card_scenes: HashMap<u64, crate::cards::Cards>,
    rotors: HashMap<(u64, ElementId), crate::sensor_motion::Rotor>,
    rotor_due: Option<std::time::Duration>,
    /// When a scrolling line, blinking separators, a clock or a falling
    /// peak mark want the next frame.
    text_due: Option<std::time::Duration>,
    /// Peak marks of needles, by scene and element.
    peaks: HashMap<(u64, ElementId), bezel_core::domain::needle::Peak>,
    /// Weather icons: the family shown and, while it fades in, the one it
    /// replaces and when the change began.
    icons: HashMap<(u64, ElementId), IconFade>,
}

/// The condition family a weather icon shows and the fade from the last one.
#[derive(Debug, Clone, Copy)]
struct IconFade {
    family: usize,
    from: Option<(u16, std::time::Duration)>,
    code: u16,
}

/// Where an element sits in its theme: z-index and id.
type LayerKey = (usize, ElementId);

/// The drawn layer of a static element.
struct CachedLayer {
    element: Element,
    bounds: IntRect,
    pixels: Vec<u8>,
    opaque: bool,
}

type FaceKey = (u64, ElementId, usize, bool, bool);

/// Face surfaces never contain the background; video cutouts have their own mask.
struct CachedFace {
    revision: u64,
    elements: Vec<(usize, Element)>,
    time: bezel_core::domain::clock::LocalTime,
    language: bezel_core::domain::clock::Language,
    snapshot: bezel_core::domain::sensor::Snapshot,
    histories: bezel_core::domain::history::Histories,
    quantities: bezel_core::domain::sensor::Quantities,
    shown: bezel_core::domain::transition::ShownValues,
    pixels: Pixmap,
    window: Option<Pixmap>,
}
const FACE_CACHE_BYTES: usize = 32 * 1024 * 1024;

/// Elements whose pixels depend on nothing but themselves (no sensor, no
/// clock, no asset), so their layer can be kept between frames.
fn is_static(element: &Element) -> bool {
    if element.color_by_value.is_some() || element.sensor_motion.is_some() {
        return false;
    }
    match &element.kind {
        // A picture may animate or be replaced under the same name.
        ElementKind::Shape {
            video_window,
            image,
            ..
        } => !video_window && image.is_none(),
        ElementKind::Text {
            content: TextContent::Static(_),
            style,
        } => style.font.asset.is_none() && style.overflow != Some(TextOverflow::Scroll),
        _ => false,
    }
}

impl std::fmt::Debug for SkiaRenderer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SkiaRenderer").finish_non_exhaustive()
    }
}

impl Default for SkiaRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl SkiaRenderer {
    /// A renderer that also knows the installed fonts. Scanning them takes a
    /// moment, so build one renderer and keep it.
    pub fn new() -> Self {
        Self::with_fonts(Vec::new(), SystemFonts::Load)
    }

    /// A renderer with `fonts` (TTF/OTF/TTC bytes) and, when `system` says
    /// so, the installed fonts. The first font found among the usual sans
    /// families (or else the first font) is the default.
    pub fn with_fonts(fonts: Vec<Vec<u8>>, system: SystemFonts) -> Self {
        Self {
            text: TextEngine::new(fonts, system),
            images: ImageCache::default(),
            diagnostics: Diagnostics::default(),
            scratch: Vec::new(),
            canvas: None,
            layers: HashMap::new(),
            used_layers: HashSet::new(),
            cards: crate::cards::Cards::new(),
            card_buffers: Vec::new(),
            face_cache: HashMap::new(),
            content_revision: 0,
            face_cache_enabled: std::env::var_os("BEZEL_CARD_FACE_CACHE").is_some_and(|v| v == "1"),
            render_stats: bezel_core::ports::RenderStats::default(),
            card_scene: 0,
            font_poll_due: None,
            card_scenes: HashMap::new(),
            rotors: HashMap::new(),
            rotor_due: None,
            text_due: None,
            peaks: HashMap::new(),
            icons: HashMap::new(),
        }
    }
}

impl SkiaRenderer {
    /// Natural size of a player: its visible parts. A side with nothing to
    /// measure (a lone cover, a bar without text) keeps the current frame.
    #[allow(clippy::too_many_arguments)]
    fn fit_player(
        &mut self,
        p: &bezel_core::domain::playback::Player,
        style: &bezel_core::domain::theme::TextStyle,
        old: BoxF,
        measured: Option<(f32, f32)>,
        furniture: &TextFurniture,
        assets: &BTreeMap<AssetRef, Vec<u8>>,
        context: &RenderContext<'_>,
    ) -> Option<(f32, f32)> {
        let session = bezel_core::domain::playback::session(context.snapshot, &p.source);
        let Some(media) = session.filter(|m| !p.hide_when_stopped || m.playing) else {
            return measured.map(|(w, h)| furniture.fit(w, h));
        };
        let shown = p.text(media);
        let mut fonts = crate::player::Fonts {
            engine: &mut self.text,
            assets,
            diagnostics: &mut self.diagnostics,
        };
        let block = crate::player::metrics(&mut fonts, p, media, &shown, style);
        let (text_w, text_h) = if shown.lines.is_empty() {
            (0., 0.)
        } else {
            measured?
        };
        let spacing = if shown.lines.is_empty() {
            0.
        } else {
            PROGRESS_SPACING
        };
        let reserve = block.map_or(0., |b| b.height + spacing);
        let width = text_w.max(block.map_or(0., |b| b.min_width));
        let height = text_h + reserve;
        Some(match (furniture.side, shown.lines.is_empty(), block) {
            (None, true, None) => return None,
            (None, _, _) => (
                if width > 0. { width.ceil() } else { old.width },
                height.ceil(),
            ),
            (Some(_), true, None) => {
                let side = old.width.min(old.height);
                (side, side)
            }
            (Some(_), true, Some(_)) => (old.width, old.height.max(height).ceil()),
            (Some(_), false, _) => furniture.fit(width, height),
        })
    }

    /// The families of the fonts a theme can use, with their faces, and the
    /// family drawn for a missing one: what a font picker lists
    /// (D-2026-10-16-studio-redesign-fidelity-28).
    pub fn font_catalog(&self) -> FontCatalog {
        self.text.catalog()
    }

    /// The distinct problems met while drawing so far (a missing asset, an
    /// unknown font family, an unreadable image), sorted. Each is also logged
    /// once with `tracing`.
    pub fn problems(&self) -> Vec<String> {
        self.diagnostics.messages()
    }
}

impl FrameRenderer for SkiaRenderer {
    fn measure_element(
        &mut self,
        theme: &Theme,
        id: ElementId,
        assets: &BTreeMap<AssetRef, Vec<u8>>,
        context: RenderContext<'_>,
    ) -> Result<Option<BoxF>> {
        let Some(element) = theme.elements.iter().find(|e| e.id == id) else {
            return Ok(None);
        };
        if theme.is_locked(element) || element.card.is_some() || element.is_group {
            return Ok(None);
        }
        if matches!(
            &element.kind,
            ElementKind::Text {
                content: TextContent::Spectrum(_),
                ..
            }
        ) {
            return Ok(None);
        }
        let old = element.frame;
        let (width, height, ax, ay) = match &element.kind {
            ElementKind::Text {
                content:
                    TextContent::Clock {
                        analog: Some(_), ..
                    },
                ..
            } => return Ok(None),
            ElementKind::Text { content, style } => {
                let measured = match rich_job(id, content, style, old, &context, false) {
                    Some((job, _)) => self.text.measure_rich(&job, assets, &mut self.diagnostics),
                    None => {
                        let text = text_of(id, content, &context);
                        let job = TextJob {
                            text: &text,
                            style,
                            area: old,
                        };
                        self.text.measure(&job, assets, &mut self.diagnostics)
                    }
                };
                let furniture = TextFurniture::new(content, style, &context);
                let fitted = match content {
                    TextContent::Player(p) => {
                        self.fit_player(p, style, old, measured, &furniture, assets, &context)
                    }
                    _ => measured.map(|(w, h)| furniture.fit(w, h)),
                };
                let Some((w, h)) = fitted else {
                    return Ok(None);
                };
                use bezel_core::domain::theme::{HAlign, VAlign};
                let ax = match style.align {
                    HAlign::Left => 0.,
                    HAlign::Center => 0.5,
                    HAlign::Right => 1.,
                };
                let ay = match style.valign {
                    VAlign::Top => 0.,
                    VAlign::Middle => 0.5,
                    VAlign::Bottom => 1.,
                };
                (w, h, ax, ay)
            }
            ElementKind::Image { asset, .. } => {
                let Some((w, h)) = self
                    .images
                    .natural_size(asset, assets, &mut self.diagnostics)
                else {
                    return Ok(None);
                };
                let scale = old.width.max(old.height) / w.max(h);
                (w * scale, h * scale, 0., 0.)
            }
            _ => return Ok(None),
        };
        Ok(Some(BoxF::new(
            old.x + (old.width - width) * ax,
            old.y + (old.height - height) * ay,
            width,
            height,
        )))
    }

    /// The frame times of a GIF, from its decoded frames (cached for
    /// drawing); delays of 10 ms or less play at 100 ms, as browsers do.
    fn animation(
        &mut self,
        asset: &AssetRef,
        assets: &BTreeMap<AssetRef, Vec<u8>>,
    ) -> Option<Timeline> {
        self.images.timeline(asset, assets, &mut self.diagnostics)
    }

    fn set_scene(&mut self, scene: u64) {
        if scene != self.card_scene {
            let next = match self.card_scenes.remove(&scene) {
                Some(cards) => cards,
                None => crate::cards::Cards::new(), // New scenes enable motion by default.
            };
            let previous = std::mem::replace(&mut self.cards, next);
            self.card_scenes.insert(self.card_scene, previous);
            self.card_scene = scene;
        }
    }
    fn set_content_revision(&mut self, revision: u64) {
        self.content_revision = revision;
    }
    fn render_stats(&self) -> bezel_core::ports::RenderStats {
        self.render_stats
    }
    fn next_change(&self) -> Option<std::time::Duration> {
        [
            self.cards.next,
            self.font_poll_due,
            self.rotor_due,
            self.text_due,
        ]
        .into_iter()
        .flatten()
        .min()
    }
    fn set_motion(&mut self, allowed: bool) {
        self.cards.motion = allowed;
    }

    fn render(
        &mut self,
        theme: &Theme,
        assets: &BTreeMap<AssetRef, Vec<u8>>,
        context: RenderContext<'_>,
    ) -> Result<Frame> {
        self.rotor_due = None;
        self.text_due = None;
        if self.icons.len() > 4096 {
            self.icons.retain(|(scene, _), _| *scene == self.card_scene);
        }
        if self.peaks.len() > 4096 {
            self.peaks.retain(|(scene, _), _| *scene == self.card_scene);
        }
        if self.rotors.len() > 4096 {
            self.rotors
                .retain(|(scene, _), _| *scene == self.card_scene);
        }
        let changed = self.text.refresh_user_fonts();
        if self.text.retain_assets(&theme.assets()) || changed {
            self.layers.clear();
            self.face_cache.clear();
        }
        self.font_poll_due = self
            .text
            .watches_user_fonts()
            .then_some(context.animation + std::time::Duration::from_secs(2));
        self.render_stats = bezel_core::ports::RenderStats {
            face_cache_enabled: self.face_cache_enabled,
            ..bezel_core::ports::RenderStats::default()
        };
        let size = theme.canvas;
        if size.area() == 0 {
            return Ok(Frame::filled(size, Rgba::default()));
        }
        let mut canvas = match self.canvas.take() {
            Some(c) if (c.width(), c.height()) == (size.width, size.height) => c,
            _ => new_canvas(size)?,
        };
        self.draw_background(&mut canvas, theme, assets, &context);
        let transitions = self.cards.update(theme, context.animation);
        for (index, element) in theme.elements.iter().enumerate() {
            if element.is_group {
                continue;
            }
            if let Some(transition) = transitions.get(&element.id) {
                self.draw_card(&mut canvas, theme, element, *transition, assets, &context)?;
                continue;
            }
            if element
                .card_member
                .as_ref()
                .is_some_and(|m| transitions.contains_key(&m.parent))
            {
                continue;
            }
            if theme.is_visible(element) {
                let key = (index, element.id);
                if element.card_member.is_some() || element.group_parent.is_some() {
                    let mut shown = element.clone();
                    shown.opacity = theme.rendered_opacity(element);
                    self.draw_element(&mut canvas, key, &shown, assets, &context);
                } else {
                    self.draw_element(&mut canvas, key, element, assets, &context);
                }
            }
        }
        self.images.sweep();
        let used = std::mem::take(&mut self.used_layers);
        self.layers.retain(|key, _| used.contains(key));
        let frame = composite::to_frame(&canvas, size);
        self.canvas = Some(canvas);
        Ok(frame)
    }
}

fn new_canvas(size: Size) -> Result<Pixmap> {
    let too_large = || {
        BezelError::Transport(format!(
            "cannot render a {}x{} canvas (the limit is {MAX_CANVAS_SIDE} per side)",
            size.width, size.height
        ))
    };
    if size.width > MAX_CANVAS_SIDE || size.height > MAX_CANVAS_SIDE {
        return Err(too_large());
    }
    Pixmap::new(size.width, size.height).ok_or_else(too_large)
}

/// Draws a frame of a host-decoded video over the whole canvas, cropped to
/// the canvas shape when its own differs (like [`Fit::Cover`]). False when
/// the frame is empty.
fn draw_video_frame(canvas: &mut Pixmap, frame: &Frame) -> bool {
    let size = frame.size();
    if (size.width, size.height) == (canvas.width(), canvas.height()) {
        let data = canvas.data_mut();
        data.copy_from_slice(frame.as_rgba());
        composite::premultiply(data);
        return true;
    }
    let area = BoxF::new(0.0, 0.0, canvas.width() as f32, canvas.height() as f32);
    let plan = images::plan(Fit::Cover, (size.width, size.height), area);
    let pixels = RgbaImage::from_raw(size.width, size.height, frame.as_rgba().to_vec());
    let (Some(plan), Some(mut pixels)) = (plan, pixels) else {
        return false;
    };
    composite::premultiply(&mut pixels);
    let (x, y, w, h) = plan.crop;
    let view = imageops::crop_imm(&pixels, x, y, w, h);
    let (width, height) = plan.size;
    let mut scaled = imageops::resize(&*view, width, height, FilterType::CatmullRom).into_raw();
    composite::clamp_premultiplied(&mut scaled);
    let data = canvas.data_mut();
    if scaled.len() != data.len() {
        return false;
    }
    data.copy_from_slice(&scaled);
    true
}

/// The fraction of a binding's range element `id` shows: the value of its
/// transition while one moves, else the current reading.
fn value_fraction(id: ElementId, binding: &Binding, context: &RenderContext<'_>) -> Option<f64> {
    let value = match context.shown.get(id) {
        Some(value) => value,
        None => context.snapshot.get(&binding.key).value()?,
    };
    fraction(value, binding.min, binding.max)
}

/// What a text element prints now.
fn text_of(id: ElementId, content: &TextContent, context: &RenderContext<'_>) -> String {
    match content {
        TextContent::Spectrum(_) => String::new(),
        TextContent::Player(p) => {
            let Some(m) = bezel_core::domain::playback::session(context.snapshot, &p.source) else {
                return p.empty_text.clone();
            };
            if p.hide_when_stopped && !m.playing {
                return String::new();
            }
            p.text(m).text()
        }
        TextContent::Static(text) => text.clone(),
        TextContent::Weather(w) => w.text(context.snapshot, context.language),
        TextContent::Sensor {
            key,
            format,
            prefix,
            suffix,
        } => {
            // A value moving to a new reading counts with the same format.
            let reading = match context.shown.get(id) {
                Some(value) => Reading::Value(value),
                None => context.snapshot.get(key),
            };
            let quantity = context.quantities.quantity(key);
            let value = format_reading(&reading, quantity, *format);
            format!("{prefix}{value}{suffix}")
        }
        TextContent::Clock {
            pattern,
            language,
            casing,
            ..
        } => clock_case(
            &format_clock(pattern, &context.time, language.unwrap_or(context.language)),
            *casing,
        ),
    }
}

/// What a text element draws in pieces (its parts, equal digits, its
/// overflow): a sensor value, a clock, the weather, or a fixed text that
/// shrinks, cuts or scrolls; `None` for the others, drawn as one run. With
/// the job comes when blinking separators next change (`motion` allowed).
fn rich_job<'a>(
    id: ElementId,
    content: &TextContent,
    style: &'a TextStyle,
    area: BoxF,
    context: &RenderContext<'_>,
    motion: bool,
) -> Option<(RichJob<'a>, Option<std::time::Duration>)> {
    let part = |part| style.parts.style(part);
    let mut blink = None;
    let pieces = match content {
        TextContent::Static(text) if style.overflow.is_some() => vec![Piece::plain(text.clone())],
        TextContent::Weather(w) => vec![Piece::plain(w.text(context.snapshot, context.language))],
        TextContent::Sensor {
            key,
            format,
            prefix,
            suffix,
        } => {
            let quantity = context.quantities.quantity(key);
            let roll = context.shown.roll(id);
            let reading = match (roll, context.shown.get(id)) {
                (Some(roll), _) => Reading::Value(roll.to),
                (None, Some(value)) => Reading::Value(value),
                (None, None) => context.snapshot.get(key),
            };
            let (number, unit) = format_reading_parts(&reading, quantity, *format);
            let wheels = roll.map(|roll| {
                let decimals = number.split_once('.').map_or(0, |(_, d)| d.len() as u32);
                let digits = number.bytes().filter(u8::is_ascii_digit).count();
                roll.wheels(decimals, digits)
            });
            vec![
                Piece {
                    part: part(TextPart::Prefix),
                    ..Piece::plain(prefix.clone())
                },
                Piece {
                    part: part(TextPart::Number),
                    wheels,
                    ..Piece::plain(number)
                },
                Piece {
                    part: part(TextPart::Unit),
                    ..Piece::plain(unit)
                },
                Piece {
                    part: part(TextPart::Suffix),
                    ..Piece::plain(suffix.clone())
                },
            ]
        }
        TextContent::Clock {
            pattern,
            language,
            casing,
            ..
        } => {
            let blinking = motion && style.parts.blink_separators;
            let off = blinking && context.animation.as_millis() % 1000 >= BLINK_HALF.as_millis();
            if blinking {
                let into = context.animation.as_millis() % BLINK_HALF.as_millis();
                let wait = BLINK_HALF.as_millis() - into;
                blink = Some(context.animation + std::time::Duration::from_millis(wait as u64));
            }
            let language = language.unwrap_or(context.language);
            clock_parts(pattern, &context.time, language)
                .into_iter()
                .map(|(name, text)| Piece {
                    part: name.and_then(part),
                    hidden: off && name == Some(TextPart::Separators),
                    ..Piece::plain(clock_case(&text, *casing))
                })
                .collect()
        }
        _ => return None,
    };
    let job = RichJob {
        pieces,
        style,
        area,
        tabular: style.tabular_digits && !matches!(content, TextContent::Static(_)),
        clock: motion.then_some(context.animation),
    };
    Some((job, blink))
}

/// Element opacity as 0..=255 (non-finite reads as opaque).
fn opacity_of(opacity: f32) -> u8 {
    if opacity.is_finite() {
        (opacity.clamp(0.0, 1.0) * 255.0).round() as u8
    } else {
        255
    }
}

/// Composites a layer covering `bounds` onto the canvas.
fn composite_layer(
    canvas: &mut Pixmap,
    pixels: &[u8],
    bounds: IntRect,
    opacity: u8,
    opaque: bool,
    element: &Element,
) {
    let angle = element.applied_rotation();
    if angle != 0.0 {
        let Some(source) =
            tiny_skia::PixmapRef::from_bytes(pixels, bounds.width(), bounds.height())
        else {
            return;
        };
        let (cx, cy) = element.frame.center();
        canvas.draw_pixmap(
            bounds.x(),
            bounds.y(),
            source,
            &tiny_skia::PixmapPaint {
                opacity: f32::from(opacity) / 255.0,
                quality: tiny_skia::FilterQuality::Bilinear,
                ..Default::default()
            },
            Transform::from_rotate_at(angle, cx, cy),
            None,
        );
        return;
    }
    let (width, height) = (canvas.width(), canvas.height());
    let mut target = Target {
        data: canvas.data_mut(),
        width,
        height,
    };
    let sprite = Sprite {
        data: pixels,
        width: bounds.width(),
        height: bounds.height(),
        x: bounds.x(),
        y: bounds.y(),
        opaque,
    };
    composite::blend(&mut target, &sprite, opacity);
}

/// The canvas pixels an element may touch.
fn element_bounds(
    element: &Element,
    context: &RenderContext<'_>,
    canvas: IntRect,
) -> Option<IntRect> {
    let area = element.frame;
    if element.applied_rotation() != 0.0 {
        let bounds = Rect::from_xywh(area.x, area.y, area.width, area.height)?.round_out()?;
        return IntRect::from_xywh(
            bounds.x(),
            bounds.y(),
            bounds.width().min(MAX_CANVAS_SIDE),
            bounds.height().min(MAX_CANVAS_SIDE),
        );
    }
    let rect = match &element.kind {
        ElementKind::Needle {
            binding,
            pivot,
            start_angle,
            sweep,
            width,
            look: None,
            ..
        } => {
            let pose = NeedlePose::new(
                area,
                *pivot,
                *start_angle,
                *sweep,
                value_fraction(element.id, binding, context),
            );
            pose.bounds(area, width.max(1.0) * 1.5)?
        }
        _ => Rect::from_xywh(area.x, area.y, area.width, area.height)?,
    };
    rect.round_out()?.intersect(&canvas)
}

impl SkiaRenderer {
    fn draw_background(
        &mut self,
        canvas: &mut Pixmap,
        theme: &Theme,
        assets: &BTreeMap<AssetRef, Vec<u8>>,
        context: &RenderContext<'_>,
    ) {
        let (asset, fit) = match (&theme.background, context.backdrop) {
            (Background::DeviceVideo { color, .. }, backdrop) => {
                let windows = theme.elements.iter().any(|e| {
                    theme.is_visible(e)
                        && opacity_of(e.opacity) > 0
                        && matches!(
                            e.kind,
                            ElementKind::Shape {
                                video_window: true,
                                ..
                            }
                        )
                });
                let color = if windows {
                    paint::color(*color)
                } else if matches!(backdrop, Backdrop::OnDevice) {
                    Color::TRANSPARENT
                } else {
                    paint::color(VIDEO_PLACEHOLDER)
                };
                canvas.fill(color);
                return;
            }
            (Background::Color(color), _) => {
                canvas.fill(paint::color(*color));
                return;
            }
            (Background::Image { asset, fit }, _) => (asset, *fit),
            (Background::Video { .. }, Backdrop::OnDevice) => {
                // The screen plays the video itself: the elements go on a
                // base of A = 0, which lets the video show through.
                canvas.fill(Color::TRANSPARENT);
                return;
            }
            (Background::Video { .. }, Backdrop::Frame(frame)) => {
                if !draw_video_frame(canvas, frame) {
                    canvas.fill(paint::color(VIDEO_PLACEHOLDER));
                }
                return;
            }
            (
                Background::Video {
                    poster: Some(poster),
                    ..
                },
                Backdrop::Poster,
            ) => (poster, Fit::Cover),
            (Background::Video { poster: None, .. }, Backdrop::Poster) => {
                canvas.fill(paint::color(VIDEO_PLACEHOLDER));
                return;
            }
        };
        canvas.fill(Color::TRANSPARENT);
        let area = BoxF::new(0.0, 0.0, canvas.width() as f32, canvas.height() as f32);
        let Some((image, plan)) =
            self.images
                .placed(asset, fit, area, assets, &mut self.diagnostics)
        else {
            return;
        };
        let Some(frame) = image.frame_at(context.animation) else {
            return;
        };
        let (width, height) = (canvas.width(), canvas.height());
        let mut target = Target {
            data: canvas.data_mut(),
            width,
            height,
        };
        let sprite = Sprite {
            data: frame.data(),
            width: frame.width(),
            height: frame.height(),
            x: plan.at.0.round() as i32,
            y: plan.at.1.round() as i32,
            opaque: false,
        };
        composite::blend(&mut target, &sprite, 255);
    }

    fn draw_card(
        &mut self,
        canvas: &mut Pixmap,
        theme: &Theme,
        parent: &Element,
        transition: crate::cards::Transition,
        assets: &BTreeMap<AssetRef, Vec<u8>>,
        context: &RenderContext<'_>,
    ) -> Result<()> {
        use bezel_core::domain::theme::CardEffect;
        let started = std::time::Instant::now();
        let draw_before = self.render_stats.face_draw_ms;
        let include = transition.settings.include_base;
        let mut buffers = std::mem::take(&mut self.card_buffers);
        let mut acquire = || -> Result<Pixmap> {
            let mut pixmap = match buffers
                .pop()
                .filter(|p| p.width() == theme.canvas.width && p.height() == theme.canvas.height)
            {
                Some(p) => p,
                None => new_canvas(theme.canvas)?,
            };
            pixmap.fill(Color::TRANSPARENT);
            Ok(pixmap)
        };
        let mut group = acquire()?;
        let mut combined = acquire()?;
        let has_windows = matches!(context.backdrop, Backdrop::OnDevice)
            && theme.elements.iter().any(|e| {
                (e.id == parent.id
                    || e.card_member
                        .as_ref()
                        .is_some_and(|m| m.parent == parent.id))
                    && matches!(
                        e.kind,
                        ElementKind::Shape {
                            video_window: true,
                            ..
                        }
                    )
            });
        let mut window_group = has_windows.then(&mut acquire).transpose()?;
        let mut window_combined = has_windows.then(&mut acquire).transpose()?;
        if !include {
            for (index, e) in theme.elements.iter().enumerate().filter(|(_, e)| {
                e.id == parent.id
                    || e.card_member
                        .as_ref()
                        .is_some_and(|m| m.parent == parent.id && m.face.is_none())
            }) {
                if theme.visible_without_face(e) {
                    let mut shown = e.clone();
                    shown.opacity = theme.rendered_opacity(e);
                    self.draw_element(canvas, (index, e.id), &shown, assets, context);
                }
            }
        }
        for pose in transition.poses(parent.frame) {
            group.fill(Color::TRANSPARENT);
            if let Some(window) = &mut window_group {
                window.fill(Color::TRANSPARENT);
            }
            if !self.face_cache_enabled {
                // Original face drawing path: no key construction, image probing,
                // cache insertion or surface copies. Keep timers for the A/B test.
                self.render_stats.face_misses += 1;
                let drawing = std::time::Instant::now();
                for (index, e) in theme.elements.iter().enumerate() {
                    let selected = e.card_member.as_ref().is_some_and(|m| {
                        m.parent == parent.id
                            && (m.face == Some(pose.face) || include && m.face.is_none())
                    }) || include && e.id == parent.id;
                    if selected && theme.visible_without_face(e) {
                        let mut shown = e.clone();
                        shown.opacity = theme.rendered_opacity(e);
                        self.draw_element(&mut group, (index, e.id), &shown, assets, context);
                        if let Some(window) = &mut window_group
                            && let ElementKind::Shape {
                                video_window: true,
                                shape,
                                fade,
                                ..
                            } = shown.kind
                        {
                            shown.kind = ElementKind::Shape {
                                image: None,
                                video_window: false,
                                shape,
                                fade,
                                fill: Some(bezel_core::domain::theme::Paint::solid(Rgba::WHITE)),
                                stroke: None,
                            };
                            self.draw_element(window, (index, e.id), &shown, assets, context);
                        }
                    }
                }
                self.render_stats.face_draw_ms += drawing.elapsed().as_secs_f64() * 1000.0;
            } else {
                let selected: Vec<_> = theme
                    .elements
                    .iter()
                    .enumerate()
                    .filter(|(_, e)| {
                        theme.visible_without_face(e)
                            && (e.card_member.as_ref().is_some_and(|m| {
                                m.parent == parent.id
                                    && (m.face == Some(pose.face) || include && m.face.is_none())
                            }) || include && e.id == parent.id)
                    })
                    .map(|(index, e)| {
                        let mut shown = e.clone();
                        shown.opacity = theme.rendered_opacity(e);
                        (index, shown)
                    })
                    .collect();
                // Animated image faces take the original drawing path. Static icons
                // and all sensor/text widgets may reuse surfaces until inputs change.
                let animated = selected.iter().any(|(_, e)| {
                    e.sensor_motion.is_some()
                        || match &e.kind {
                            ElementKind::Graph {
                                binding,
                                interval_seconds,
                                ..
                            } => {
                                self.cards.motion
                                    && context.histories.scroll_offset(
                                        &SeriesId::graph(&binding.key, *interval_seconds),
                                        context.animation,
                                    ) > 0.
                            }
                            kind => kind.picture().is_some_and(|asset| {
                                self.images
                                    .timeline(asset, assets, &mut self.diagnostics)
                                    .is_some()
                            }),
                        }
                });
                let key = (self.card_scene, parent.id, pose.face, include, has_windows);
                let cached = self.face_cache.get(&key).filter(|face| {
                    self.content_revision != 0
                        && !animated
                        && face.revision == self.content_revision
                        && face.pixels.width() == theme.canvas.width
                        && face.pixels.height() == theme.canvas.height
                        && face.elements == selected
                        && face.time == context.time
                        && face.language == context.language
                        && face.snapshot == *context.snapshot
                        && face.histories == *context.histories
                        && face.quantities == *context.quantities
                        && face.shown == *context.shown
                });
                if let Some(face) = cached {
                    group.data_mut().copy_from_slice(face.pixels.data());
                    if let (Some(window), Some(kept)) = (&mut window_group, &face.window) {
                        window.data_mut().copy_from_slice(kept.data());
                    }
                    self.render_stats.face_hits += 1;
                } else {
                    self.render_stats.face_misses += 1;
                    let drawing = std::time::Instant::now();
                    for (index, shown) in &selected {
                        self.draw_element(&mut group, (*index, shown.id), shown, assets, context);
                        if let Some(window) = &mut window_group
                            && let ElementKind::Shape {
                                video_window: true,
                                shape,
                                fade,
                                ..
                            } = shown.kind
                        {
                            let mut mask = shown.clone();
                            mask.kind = ElementKind::Shape {
                                image: None,
                                video_window: false,
                                shape,
                                fade,
                                fill: Some(bezel_core::domain::theme::Paint::solid(Rgba::WHITE)),
                                stroke: None,
                            };
                            self.draw_element(window, (*index, shown.id), &mask, assets, context);
                        }
                    }
                    self.render_stats.face_draw_ms += drawing.elapsed().as_secs_f64() * 1000.0;
                    let bytes = group.data().len() * if has_windows { 2 } else { 1 };
                    if self.content_revision != 0 && !animated && bytes <= FACE_CACHE_BYTES {
                        self.face_cache.remove(&key);
                        let used: usize = self
                            .face_cache
                            .values()
                            .map(|f| {
                                f.pixels.data().len()
                                    + f.window.as_ref().map_or(0, |w| w.data().len())
                            })
                            .sum();
                        if used + bytes > FACE_CACHE_BYTES {
                            self.face_cache.clear();
                        }
                        self.face_cache.insert(
                            key,
                            CachedFace {
                                revision: self.content_revision,
                                elements: selected,
                                time: context.time,
                                language: context.language,
                                snapshot: context.snapshot.clone(),
                                histories: context.histories.clone(),
                                quantities: context.quantities.clone(),
                                shown: context.shown.clone(),
                                pixels: group.clone(),
                                window: window_group.clone(),
                            },
                        );
                    }
                }
            }
            if let Some(projection) = pose.projection {
                projection.draw(&group, &mut combined, pose.shade, pose.alpha);
                if let (Some(window), Some(combined)) = (&window_group, &mut window_combined) {
                    projection.draw(window, combined, 1.0, pose.alpha);
                }
                continue;
            }
            if pose.shade < 1.0 {
                // Premultiplied RGB shading preserves transparency.
                for pixel in group.data_mut().as_chunks_mut::<4>().0 {
                    for channel in &mut pixel[..3] {
                        *channel = (f32::from(*channel) * pose.shade).round() as u8;
                    }
                }
            }
            let paint = tiny_skia::PixmapPaint {
                opacity: pose.alpha,
                blend_mode: if transition.settings.effect == CardEffect::Fade {
                    tiny_skia::BlendMode::Plus
                } else {
                    tiny_skia::BlendMode::SourceOver
                },
                quality: tiny_skia::FilterQuality::Bilinear,
            };
            combined.draw_pixmap(0, 0, group.as_ref(), &paint, pose.transform, None);
            if let (Some(window), Some(combined)) = (&window_group, &mut window_combined) {
                combined.draw_pixmap(0, 0, window.as_ref(), &paint, pose.transform, None);
            }
        }
        let mask = if transition.settings.effect == CardEffect::Slide {
            let b = parent.frame;
            let mut mask = tiny_skia::Mask::new(canvas.width(), canvas.height());
            if let (Some(mask), Some(rect)) =
                (&mut mask, Rect::from_xywh(b.x, b.y, b.width, b.height))
            {
                mask.fill_path(
                    &tiny_skia::PathBuilder::from_rect(rect),
                    tiny_skia::FillRule::Winding,
                    true,
                    Transform::identity(),
                );
            }
            mask
        } else {
            None
        };
        if let Some(window) = &window_combined {
            canvas.draw_pixmap(
                0,
                0,
                window.as_ref(),
                &tiny_skia::PixmapPaint {
                    blend_mode: tiny_skia::BlendMode::DestinationOut,
                    ..tiny_skia::PixmapPaint::default()
                },
                Transform::identity(),
                mask.as_ref(),
            );
        }
        canvas.draw_pixmap(
            0,
            0,
            combined.as_ref(),
            &tiny_skia::PixmapPaint::default(),
            Transform::identity(),
            mask.as_ref(),
        );
        self.card_buffers = vec![group, combined];
        self.card_buffers.extend(window_group);
        self.card_buffers.extend(window_combined);
        self.render_stats.face_compose_ms += started.elapsed().as_secs_f64() * 1000.0
            - (self.render_stats.face_draw_ms - draw_before);
        Ok(())
    }

    /// Draws one element into a scratch layer the size of its bounds (or
    /// reuses the layer kept from an earlier frame), then composites the
    /// layer with the element's opacity.
    fn draw_element(
        &mut self,
        canvas: &mut Pixmap,
        key: LayerKey,
        element: &Element,
        assets: &BTreeMap<AssetRef, Vec<u8>>,
        context: &RenderContext<'_>,
    ) {
        let mut moving;
        let element = if self.cards.motion
            && element.supports_sensor_motion()
            && let Some(settings) = element.sensor_motion.as_ref().filter(|s| s.is_valid())
        {
            let rotor = self
                .rotors
                .entry((self.card_scene, element.id))
                .or_insert_with(|| crate::sensor_motion::Rotor::new(settings, context.animation));
            let (phase, active) = rotor.step(
                settings,
                context.animation,
                context.snapshot.get(&settings.sensor).value(),
            );
            if active {
                self.rotor_due = Some(context.animation + std::time::Duration::from_millis(20));
            }
            moving = element.clone();
            moving.rotation =
                bezel_core::domain::rotation::normalize_degrees(element.applied_rotation() + phase);
            &moving
        } else {
            element
        };
        let opacity = opacity_of(element.opacity);
        let Some(canvas_rect) = IntRect::from_xywh(0, 0, canvas.width(), canvas.height()) else {
            return;
        };
        let Some(bounds) = element_bounds(element, context, canvas_rect).filter(|_| opacity > 0)
        else {
            return;
        };
        if let ElementKind::Shape {
            video_window: true,
            shape,
            fade,
            ..
        } = &element.kind
            && matches!(context.backdrop, Backdrop::OnDevice)
            && let Some(path) = shape::outline_path(element.frame, *shape, 0.0)
            && let Some(mut mask) = tiny_skia::Mask::new(bounds.width(), bounds.height())
        {
            mask.fill_path(
                &path,
                tiny_skia::FillRule::Winding,
                true,
                Transform::from_translate(-(bounds.x() as f32), -(bounds.y() as f32)),
            );
            let width = canvas.width() as usize;
            let frame = element.frame;
            let ramp = fade.map(|f| f.ramp());
            for (index, coverage) in mask.data().iter().enumerate() {
                if *coverage == 0 {
                    continue;
                }
                let x = bounds.x() as usize + index % bounds.width() as usize;
                let y = bounds.y() as usize + index / bounds.width() as usize;
                let strength = ramp.as_ref().map_or(1.0, |f| {
                    f(
                        (x as f32 + 0.5 - frame.x) / frame.width,
                        (y as f32 + 0.5 - frame.y) / frame.height,
                    )
                });
                let retain =
                    1.0 - f32::from(*coverage) / 255.0 * f32::from(opacity) / 255.0 * strength;
                let pixel = &mut canvas.data_mut()[(y * width + x) * 4..][..4];
                for channel in pixel {
                    *channel = (f32::from(*channel) * retain).round() as u8;
                }
            }
        }
        let cacheable = is_static(element);
        if cacheable {
            self.used_layers.insert(key);
            let kept = self.layers.get(&key);
            if let Some(kept) = kept.filter(|k| k.bounds == bounds && k.element == *element) {
                composite_layer(canvas, &kept.pixels, bounds, opacity, kept.opaque, element);
                return;
            }
        }
        let (w, h) = (bounds.width(), bounds.height());
        let mut scratch = std::mem::take(&mut self.scratch);
        scratch.clear();
        scratch.resize(w as usize * h as usize * 4, 0);
        if let Some(pixmap) = PixmapMut::from_bytes(&mut scratch, w, h) {
            let mut layer = Layer {
                pixmap,
                x: bounds.x(),
                y: bounds.y(),
            };
            self.draw_kind(&mut layer, element, assets, context);
        }
        if let ElementKind::Shape {
            fade: Some(fade), ..
        }
        | ElementKind::Image {
            fade: Some(fade), ..
        } = &element.kind
        {
            let frame = element.frame;
            let ramp = fade.ramp();
            for (index, pixel) in scratch.as_chunks_mut::<4>().0.iter_mut().enumerate() {
                let x = bounds.x() as f32 + (index % w as usize) as f32 + 0.5;
                let y = bounds.y() as f32 + (index / w as usize) as f32 + 0.5;
                let strength = ramp((x - frame.x) / frame.width, (y - frame.y) / frame.height);
                for channel in pixel {
                    *channel = (f32::from(*channel) * strength).round() as u8;
                }
            }
        }
        let opaque = cacheable && composite::is_opaque(&scratch);
        composite_layer(canvas, &scratch, bounds, opacity, opaque, element);
        if cacheable {
            let kept = CachedLayer {
                element: element.clone(),
                bounds,
                pixels: scratch.clone(),
                opaque,
            };
            self.layers.insert(key, kept);
        }
        self.scratch = scratch;
    }

    fn draw_kind(
        &mut self,
        layer: &mut Layer<'_>,
        element: &Element,
        assets: &BTreeMap<AssetRef, Vec<u8>>,
        context: &RenderContext<'_>,
    ) {
        if let ElementKind::Text {
            content: TextContent::Spectrum(s),
            style,
        } = &element.kind
        {
            let mut frame = context
                .snapshot
                .spectra
                .get(&element.id)
                .cloned()
                // No result yet is silence, not a disabled capture.
                .unwrap_or(bezel_core::domain::spectrum::SpectrumFrame {
                    state: bezel_core::domain::spectrum::AudioState::Live,
                    ..Default::default()
                });
            if s.demo {
                frame.state = bezel_core::domain::spectrum::AudioState::Demo;
                frame.levels = (0..s.bands)
                    .map(|i| {
                        0.45 + 0.25
                            * (context.animation.as_secs_f32() * 2. + f32::from(i) * 0.37).sin()
                    })
                    .collect();
                frame.peaks = frame.levels.iter().map(|v| (v + 0.08).min(1.)).collect();
            }
            crate::spectrum::draw(
                layer,
                element.frame,
                s,
                &frame,
                &style.paint,
                element.color_by_value.as_ref(),
            );
            let label = crate::spectrum::label(frame.state, context.language);
            if !label.is_empty() {
                self.text.draw(
                    layer,
                    &TextJob {
                        text: label,
                        style,
                        area: element.frame,
                    },
                    assets,
                    &mut self.diagnostics,
                );
            }
            return;
        }
        let color = element.value_color(context.snapshot, context.shown);
        let mut resolved;
        let element = if let Some(color) = color {
            resolved = element.clone();
            use bezel_core::domain::theme::Paint;
            match &mut resolved.kind {
                ElementKind::Text { style, .. } => style.paint = Paint::solid(color),
                ElementKind::Bar { fill, .. } | ElementKind::Ring { fill, .. } => {
                    *fill = Paint::solid(color)
                }
                ElementKind::Shape { fill, .. } => *fill = Some(Paint::solid(color)),
                ElementKind::Needle { color: c, .. } | ElementKind::Graph { color: c, .. } => {
                    *c = color
                }
                _ => {}
            }
            &resolved
        } else {
            element
        };
        let area = element.frame;
        match &element.kind {
            ElementKind::Text { content, style } => {
                let furniture = TextFurniture::new(content, style, context);
                let mut area = area;
                if let TextContent::Player(p) = content {
                    let media = bezel_core::domain::playback::session(context.snapshot, &p.source);
                    if p.hide_when_stopped && media.is_none_or(|m| !m.playing) {
                        return;
                    }
                    if let Some(m) = media {
                        if p.show_cover {
                            let (side, gap) = furniture.layout(area);
                            if !m.cover.is_empty() {
                                use std::hash::{Hash, Hasher};
                                let mut hash = std::collections::hash_map::DefaultHasher::new();
                                m.cover.hash(&mut hash);
                                let asset =
                                    AssetRef(format!("runtime/player/{:x}.png", hash.finish()));
                                let covers = BTreeMap::from([(asset.clone(), m.cover.clone())]);
                                self.draw_image(
                                    layer,
                                    &asset,
                                    Fit::Contain,
                                    BoxF::new(area.x, area.y, side, side),
                                    &covers,
                                    context,
                                );
                            }
                            let cover_path = if let Some(r) = p.cover_corners {
                                crate::path::rounded_corners(area.x, area.y, side, side, r)
                            } else if p.cover_radius > 0. {
                                crate::path::rounded_rect(
                                    area.x,
                                    area.y,
                                    side,
                                    side,
                                    p.cover_radius,
                                )
                            } else {
                                None
                            };
                            if let Some(path) = cover_path
                                && let Some(mask) = layer.mask_of(&path)
                            {
                                for (pixel, alpha) in layer
                                    .pixmap
                                    .data_mut()
                                    .as_chunks_mut::<4>()
                                    .0
                                    .iter_mut()
                                    .zip(mask.data())
                                {
                                    for channel in pixel {
                                        *channel = ((u16::from(*channel) * u16::from(*alpha) + 127)
                                            / 255)
                                            as u8;
                                    }
                                }
                            }
                            area.x += side + gap;
                            area.width = (area.width - side - gap).max(0.);
                        }
                        let shown = p.text(m);
                        let mut fonts = crate::player::Fonts {
                            engine: &mut self.text,
                            assets,
                            diagnostics: &mut self.diagnostics,
                        };
                        if let Some(block) = crate::player::metrics(&mut fonts, p, m, &shown, style)
                        {
                            let alone = shown.lines.is_empty();
                            let place =
                                crate::player::place(area, block.height, alone, style.valign);
                            crate::player::draw(layer, &mut fonts, place, &block, &shown, p, style);
                            let spacing = if alone { 0. } else { PROGRESS_SPACING };
                            area.height = (area.height - block.height - spacing).max(0.);
                        }
                    }
                }
                if let TextContent::Weather(w) = content
                    && w.show_icon
                {
                    let (side, gap) = furniture.layout(area);
                    let code = match context.snapshot.get(&w.keys()[1]) {
                        bezel_core::domain::sensor::Reading::Value(v) => v as u16,
                        _ => 999,
                    };
                    let icon = BoxF::new(area.x, area.y + (area.height - side) / 2.0, side, side);
                    self.draw_weather_icon(layer, element.id, icon, code, w, &style.paint, context);
                    area.x += side + gap;
                    area.width = (area.width - side - gap).max(0.0);
                }
                let motion = self.cards.motion;
                if let TextContent::Clock {
                    analog: Some(dial),
                    language,
                    ..
                } = content
                {
                    let words = crate::analog_clock::Words {
                        engine: &mut self.text,
                        assets,
                        diagnostics: &mut self.diagnostics,
                        style,
                    };
                    let language = language.unwrap_or(context.language);
                    crate::analog_clock::draw(
                        layer,
                        area,
                        dial,
                        &context.time,
                        language,
                        motion,
                        words,
                    );
                    let due = dial
                        .next_change(&context.time, motion)
                        .map(|wait| context.animation + wait);
                    self.text_due = due.into_iter().chain(self.text_due).min();
                } else if let Some((job, blink)) =
                    rich_job(element.id, content, style, area, context, motion)
                {
                    let scroll = self
                        .text
                        .draw_rich(layer, &job, assets, &mut self.diagnostics);
                    let due = [blink, scroll.map(|step| context.animation + step)]
                        .into_iter()
                        .flatten()
                        .chain(self.text_due)
                        .min();
                    self.text_due = due;
                } else {
                    let text = text_of(element.id, content, context);
                    let job = TextJob {
                        text: &text,
                        style,
                        area,
                    };
                    self.text.draw(layer, &job, assets, &mut self.diagnostics);
                }
            }
            ElementKind::Image { asset, fit, .. } => {
                self.draw_image(layer, asset, *fit, area, assets, context);
                if let Some(color) = color {
                    tint(layer, color);
                }
            }
            ElementKind::Shape {
                video_window,
                shape,
                fill,
                image,
                stroke,
                ..
            } => {
                let placeholder = bezel_core::domain::theme::Paint::solid(VIDEO_PLACEHOLDER);
                let fill = if *video_window {
                    if matches!(context.backdrop, Backdrop::OnDevice) {
                        None
                    } else {
                        Some(&placeholder)
                    }
                } else {
                    fill.as_ref()
                };
                // A video window shows the screen's video, never a picture.
                let picture = image.as_ref().filter(|_| !*video_window);
                shape::draw_with(layer, area, *shape, fill, *stroke, |layer, path| {
                    if let Some(picture) = picture {
                        self.draw_shape_image(layer, area, path, picture, assets, context);
                    }
                });
            }
            ElementKind::Bar {
                binding,
                direction,
                fill,
                track,
                radius,
                segments,
            } => {
                let style = BarStyle {
                    direction: *direction,
                    fill,
                    track: track.as_ref(),
                    radius: *radius,
                    segments: *segments,
                };
                if let Some(v) = element
                    .visualization
                    .as_ref()
                    .filter(|v| v.style == bezel_core::domain::visualization::VisualStyle::Matrix)
                {
                    crate::visualization::matrix(
                        layer,
                        area,
                        value_fraction(element.id, binding, context),
                        &style,
                        v,
                    );
                    return;
                }
                gauges::draw_bar(
                    layer,
                    area,
                    value_fraction(element.id, binding, context),
                    &style,
                );
            }
            ElementKind::Ring { .. } => draw_ring(layer, element, context),
            ElementKind::Needle { .. } => self.draw_needle(layer, element, assets, context),
            ElementKind::Graph { .. } => draw_graph(layer, element, context, self.cards.motion),
        }
    }

    fn draw_image(
        &mut self,
        layer: &mut Layer<'_>,
        asset: &AssetRef,
        fit: Fit,
        area: BoxF,
        assets: &BTreeMap<AssetRef, Vec<u8>>,
        context: &RenderContext<'_>,
    ) {
        let Some((image, plan)) =
            self.images
                .placed(asset, fit, area, assets, &mut self.diagnostics)
        else {
            return;
        };
        if let Some(frame) = image.frame_at(context.animation) {
            layer.blit(
                frame.as_ref(),
                plan.at.0.round() as i32,
                plan.at.1.round() as i32,
            );
        }
    }

    /// Paints a shape's picture inside `path` (the shape's fill outline):
    /// fitted to the shape's box, or at its own size times the scale, once
    /// or repeated, then offset. Pictures are resampled once per size by the
    /// image cache (SVG rasterised at that size) and blitted pixel-aligned.
    fn draw_shape_image(
        &mut self,
        layer: &mut Layer<'_>,
        area: BoxF,
        path: &tiny_skia::Path,
        image: &ShapeImage,
        assets: &BTreeMap<AssetRef, Vec<u8>>,
        context: &RenderContext<'_>,
    ) {
        if image.asset.0.is_empty() || !image.opacity.is_finite() || image.opacity <= 0.0 {
            return;
        }
        let Some(mask) = layer.mask_of(path) else {
            return;
        };
        let opacity = image.opacity.min(1.0);
        let natural = image.repeat.natural_size();
        let (fit, target) = match image.repeat {
            ImageRepeat::Cover => (Fit::Cover, area),
            ImageRepeat::Contain => (Fit::Contain, area),
            ImageRepeat::Stretch => (Fit::Fill, area),
            _ => {
                let Some((w, h)) =
                    self.images
                        .natural_size(&image.asset, assets, &mut self.diagnostics)
                else {
                    return;
                };
                let scale = image
                    .scale
                    .clamp(*ShapeImage::SCALE.start(), *ShapeImage::SCALE.end());
                (
                    Fit::Fill,
                    BoxF::new(0.0, 0.0, (w * scale).max(1.0), (h * scale).max(1.0)),
                )
            }
        };
        let Some((scaled, plan)) =
            self.images
                .placed(&image.asset, fit, target, assets, &mut self.diagnostics)
        else {
            return;
        };
        let Some(frame) = scaled.frame_at(context.animation) else {
            return;
        };
        let (tw, th) = (frame.width() as f32, frame.height() as f32);
        let (ox, oy) = image.offset;
        let origin = match image.repeat {
            _ if !natural => plan.at,
            ImageRepeat::Center => (
                area.x + (area.width - tw) / 2.0 + ox,
                area.y + (area.height - th) / 2.0 + oy,
            ),
            _ => (area.x + ox, area.y + oy),
        };
        let origin = (origin.0.round(), origin.1.round());
        let (spread, rect) = match image.repeat {
            ImageRepeat::Tile => (
                tiny_skia::SpreadMode::Repeat,
                tiny_skia::Rect::from_xywh(area.x, area.y, area.width, area.height),
            ),
            ImageRepeat::TileX => (
                tiny_skia::SpreadMode::Repeat,
                tiny_skia::Rect::from_xywh(area.x, origin.1, area.width, th),
            ),
            ImageRepeat::TileY => (
                tiny_skia::SpreadMode::Repeat,
                tiny_skia::Rect::from_xywh(origin.0, area.y, tw, area.height),
            ),
            _ => (
                tiny_skia::SpreadMode::Pad,
                tiny_skia::Rect::from_xywh(origin.0, origin.1, tw, th),
            ),
        };
        let Some(rect) = rect else {
            return;
        };
        let paint = tiny_skia::Paint {
            shader: tiny_skia::Pattern::new(
                frame.as_ref(),
                spread,
                tiny_skia::FilterQuality::Nearest,
                opacity,
                Transform::from_translate(origin.0, origin.1),
            ),
            anti_alias: false,
            ..tiny_skia::Paint::default()
        };
        layer.fill_rect_masked(rect, &paint, &mask);
    }

    fn draw_needle(
        &mut self,
        layer: &mut Layer<'_>,
        element: &Element,
        assets: &BTreeMap<AssetRef, Vec<u8>>,
        context: &RenderContext<'_>,
    ) {
        let ElementKind::Needle {
            binding,
            asset,
            pivot,
            start_angle,
            sweep,
            color,
            width,
            look,
        } = &element.kind
        else {
            return;
        };
        let area = element.frame;
        if let Some(look) = look {
            self.draw_needle_look(layer, element, look, assets, context);
            if asset.is_none() {
                return;
            }
        }
        let pose = NeedlePose::new(
            area,
            *pivot,
            *start_angle,
            *sweep,
            value_fraction(element.id, binding, context),
        );
        let image = asset.as_ref().and_then(|asset| {
            self.images
                .placed(asset, Fit::Fill, area, assets, &mut self.diagnostics)
        });
        let Some((image, plan)) = image else {
            if look.is_none() {
                gauges::draw_line_needle(layer, area, pose, *color, *width);
            }
            return;
        };
        if let Some(frame) = image.frame_at(context.animation) {
            let (px, py) = pose.pivot;
            let placement =
                Transform::from_translate(plan.at.0, plan.at.1).post_rotate_at(pose.angle, px, py);
            layer.draw_transformed(frame.as_ref(), placement);
            if let Some(color) = element.value_color(context.snapshot, context.shown) {
                tint(layer, color);
            }
        }
    }
}

impl SkiaRenderer {
    /// The extended needle: arc, scale, peak mark, hand (unless an image
    /// replaces it), cap and the value under the pivot.
    fn draw_needle_look(
        &mut self,
        layer: &mut Layer<'_>,
        element: &Element,
        look: &bezel_core::domain::needle::NeedleLook,
        assets: &BTreeMap<AssetRef, Vec<u8>>,
        context: &RenderContext<'_>,
    ) {
        let ElementKind::Needle {
            binding,
            asset,
            pivot,
            start_angle,
            sweep,
            color,
            width,
            ..
        } = &element.kind
        else {
            return;
        };
        let value = match context.shown.get(element.id) {
            Some(value) => Some(value),
            None => context.snapshot.get(&binding.key).value(),
        };
        let mut peak = None;
        if let (Some(seconds), Some(v)) = (look.peak_seconds, value) {
            let hold = std::time::Duration::from_secs(u64::from(seconds));
            let mark = self
                .peaks
                .entry((self.card_scene, element.id))
                .or_insert_with(|| bezel_core::domain::needle::Peak::new(v, context.animation));
            let high = mark.update(v, context.animation, hold);
            if high > v {
                peak = Some(high);
            }
            self.text_due = mark
                .falls_at(v, hold)
                .into_iter()
                .chain(self.text_due)
                .min();
        }
        let margin = look.arc_width / 2.0 + 1.0;
        let gauge =
            crate::needle_gauge::Gauge::fit(element.frame, *pivot, *start_angle, *sweep, margin);
        let fallback = bezel_core::domain::theme::TextStyle::default();
        let style = look.center.as_ref().map_or(&fallback, |c| &c.style);
        let mut words = crate::analog_clock::Words {
            engine: &mut self.text,
            assets,
            diagnostics: &mut self.diagnostics,
            style,
        };
        let reading = crate::needle_gauge::Reading {
            binding,
            value,
            peak,
            color: *color,
            width: *width,
            language: context.language,
        };
        crate::needle_gauge::draw(layer, gauge, look, &reading, asset.is_none(), &mut words);
        if let Some(center) = &look.center {
            let content = TextContent::Sensor {
                key: binding.key.clone(),
                format: center.format,
                prefix: center.prefix.clone(),
                suffix: center.suffix.clone(),
            };
            let size = center.style.size.max(1.0);
            let (px, py) = gauge.pivot;
            let top = py + look.cap_diameter / 2.0 + size * 0.2;
            let half = gauge.radius.max(size * 2.0);
            let area = BoxF::new(px - half, top, half * 2.0, size * LINE_BOX);
            let motion = self.cards.motion;
            if let Some((job, _)) =
                rich_job(element.id, &content, &center.style, area, context, motion)
            {
                let scroll = self
                    .text
                    .draw_rich(layer, &job, assets, &mut self.diagnostics);
                let due = scroll.map(|step| context.animation + step);
                self.text_due = due.into_iter().chain(self.text_due).min();
            }
        }
    }
}

impl SkiaRenderer {
    /// A weather icon: at rest, or moving with its family's movement and
    /// cross-fading from the last family for [`ICON_FADE`]; asks for the next
    /// frame while it moves (D-2026-10-20-studio-ux-v2-09).
    #[allow(clippy::too_many_arguments)]
    fn draw_weather_icon(
        &mut self,
        layer: &mut Layer<'_>,
        id: ElementId,
        icon: BoxF,
        code: u16,
        weather: &bezel_core::domain::weather::Weather,
        paint: &bezel_core::domain::theme::Paint,
        context: &RenderContext<'_>,
    ) {
        use bezel_core::domain::weather::{ICON_FADE, IconPose, family, pose};
        let step = weather.icon_motion.step().filter(|_| self.cards.motion);
        let Some(step) = step else {
            self.icons.remove(&(self.card_scene, id));
            crate::weather::draw(
                layer,
                icon,
                code,
                paint,
                weather.icon_style,
                &IconPose::REST,
            );
            return;
        };
        let now = context.animation;
        let current = family(code);
        let state = self.icons.entry((self.card_scene, id)).or_insert(IconFade {
            family: current,
            from: None,
            code,
        });
        if state.family != current {
            state.from = Some((state.code, now));
            state.family = current;
        }
        state.code = code;
        let fading = state
            .from
            .filter(|(_, since)| now.saturating_sub(*since) < ICON_FADE);
        if fading.is_none() {
            state.from = None;
        }
        let at = now.as_secs_f32();
        let amplitude = weather.icon_motion.amplitude();
        let speed = weather.icon_speed;
        let progress = fading.map_or(1.0, |(_, since)| {
            now.saturating_sub(since).as_secs_f32() / ICON_FADE.as_secs_f32()
        });
        if let Some((old, _)) = fading {
            let before = pose(family(old), at, amplitude, speed);
            crate::weather::faded(layer, icon, 1.0 - progress, |layer| {
                crate::weather::draw(layer, icon, old, paint, weather.icon_style, &before);
            });
        }
        let shown = pose(current, at, amplitude, speed);
        crate::weather::faded(layer, icon, progress, |layer| {
            crate::weather::draw(layer, icon, code, paint, weather.icon_style, &shown);
        });
        self.text_due = Some(now + step).into_iter().chain(self.text_due).min();
    }
}

/// Height of the box of the value under a needle's pivot, in text sizes.
const LINE_BOX: f32 = 1.3;

/// Recolour existing image coverage without changing cached source pixels.
fn tint(layer: &mut Layer<'_>, color: Rgba) {
    for pixel in layer.pixmap.data_mut().as_chunks_mut::<4>().0 {
        let alpha = (u16::from(pixel[3]) * u16::from(color.a) + 127) / 255;
        *pixel =
            [color.r, color.g, color.b, 255].map(|c| ((u16::from(c) * alpha + 127) / 255) as u8);
    }
}

fn draw_ring(layer: &mut Layer<'_>, element: &Element, context: &RenderContext<'_>) {
    let ElementKind::Ring {
        test_full,
        binding,
        start_angle,
        sweep,
        thickness,
        clockwise,
        fill,
        track,
        cap,
        segments,
    } = &element.kind
    else {
        return;
    };
    let style = RingStyle {
        start_angle: *start_angle,
        sweep: *sweep,
        thickness: *thickness,
        clockwise: *clockwise,
        fill,
        track: track.as_ref(),
        cap: *cap,
        segments: *segments,
    };
    let value = if *test_full {
        Some(1.)
    } else {
        value_fraction(element.id, binding, context)
    };
    let visual = element.visualization.as_ref();
    if let Some(v) =
        visual.filter(|v| v.style == bezel_core::domain::visualization::VisualStyle::Pixel)
    {
        crate::visualization::pixel_arc(layer, element.frame, value, &style, v);
        return;
    }
    let dial = visual.filter(|v| v.style == bezel_core::domain::visualization::VisualStyle::Dial);
    gauges::draw_ring(
        layer,
        if dial.is_some() {
            crate::visualization::dial_area(element.frame)
        } else {
            element.frame
        },
        value,
        &style,
    );
    if let Some(v) = dial {
        crate::visualization::dial(layer, element.frame, value, &style, v);
    }
}

fn draw_graph(layer: &mut Layer<'_>, element: &Element, context: &RenderContext<'_>, motion: bool) {
    let ElementKind::Graph {
        binding,
        history,
        interval_seconds,
        style,
        color,
        fill,
        line_width,
        autoscale,
    } = &element.kind
    else {
        return;
    };
    let series = SeriesId::graph(&binding.key, *interval_seconds);
    let scroll = if motion {
        context.histories.scroll_offset(&series, context.animation)
    } else {
        0.
    };
    let spec = GraphSpec {
        rounded: element.visualization.as_ref().is_some_and(|v| {
            matches!(
                v.style,
                bezel_core::domain::visualization::VisualStyle::Rounded
                    | bezel_core::domain::visualization::VisualStyle::Mirrored
            )
        }),
        gap: element.visualization.as_ref().map(|v| v.gap),
        reverse: false,
        scroll,
        history: *history,
        style: *style,
        color: *color,
        fill: fill.as_ref(),
        line_width: *line_width,
        autoscale: *autoscale,
        min: binding.min,
        max: binding.max,
    };
    let values = context
        .histories
        .points(&series, usize::from(*history) + usize::from(scroll > 0.));
    if let Some(v) = element
        .visualization
        .as_ref()
        .filter(|v| v.style == bezel_core::domain::visualization::VisualStyle::Mirrored)
    {
        let half = (element.frame.height - 2.).max(0.) / 2.;
        let top = BoxF::new(element.frame.x, element.frame.y, element.frame.width, half);
        let bottom = BoxF::new(
            element.frame.x,
            element.frame.y + half + 2.,
            element.frame.width,
            half,
        );
        let lower = v
            .secondary
            .as_ref()
            .map(|b| {
                context.histories.points(
                    &SeriesId::graph(&b.key, *interval_seconds),
                    usize::from(*history) + usize::from(scroll > 0.),
                )
            })
            .unwrap_or_default();
        let mut upper_spec = spec;
        let mut lower_spec = GraphSpec {
            reverse: true,
            color: v.secondary_color,
            fill: None,
            ..spec
        };
        if let Some(b) = &v.secondary {
            lower_spec.min = b.min;
            lower_spec.max = b.max;
        }
        if spec.autoscale {
            // Both series share the same zero baseline and peak; preserve their relative magnitudes.
            let peak = values
                .iter()
                .chain(lower.iter())
                .flatten()
                .copied()
                .filter(|v| v.is_finite())
                .fold(0_f64, f64::max)
                .max(1.)
                * 1.1;
            upper_spec.autoscale = false;
            upper_spec.min = 0.;
            upper_spec.max = peak;
            lower_spec.autoscale = false;
            lower_spec.min = 0.;
            lower_spec.max = peak;
        }
        graph::draw(layer, top, &values, &upper_spec);
        graph::draw(layer, bottom, &lower, &lower_spec);
    } else {
        graph::draw(layer, element.frame, &values, &spec);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testkit::{self, Scene, count, element, png, px, render_over};
    use bezel_core::domain::theme::{GraphStyle, Paint, ShapeKind};

    #[test]
    fn timed_graphs_move_between_samples_in_every_style_and_respect_reduced_motion() {
        use bezel_core::domain::{
            history::{Histories, Tick},
            sensor::{Reading, Snapshot},
        };
        use std::time::Duration;
        let key = bezel_core::domain::sensor::SensorKey::new("cpu.usage").unwrap();
        for style in [GraphStyle::Line, GraphStyle::Area, GraphStyle::Bars] {
            let e = element(
                BoxF::new(8., 8., 112., 64.),
                ElementKind::Graph {
                    binding: bezel_core::domain::theme::Binding {
                        key: key.clone(),
                        min: 0.,
                        max: 100.,
                    },
                    history: 4,
                    interval_seconds: None,
                    style,
                    color: Rgba::WHITE,
                    fill: None,
                    line_width: 2.,
                    autoscale: false,
                },
            );
            let theme = testkit::theme(128, 80, Background::Color(Rgba::BLACK), vec![e]);
            let mut scene = Scene::empty();
            let id = SeriesId::graph(&key, None);
            scene.histories = Histories::new(std::slice::from_ref(&id));
            for (n, v) in [10., 80., 30., 90.].into_iter().enumerate() {
                let mut snapshot = Snapshot::default();
                snapshot.insert(key.clone(), Reading::Value(v));
                scene.histories.push(
                    &snapshot,
                    Tick::At {
                        now: Duration::from_secs(n as u64),
                        refresh: true,
                    },
                );
            }
            let mut renderer = testkit::renderer();
            scene.animation = Duration::from_millis(3000);
            let a = testkit::render(&mut renderer, &theme, &scene);
            scene.animation = Duration::from_millis(3500);
            let b = testkit::render(&mut renderer, &theme, &scene);
            assert_ne!(a, b, "{style:?} must move without sampling again");
            assert_eq!(scene.histories.points(&id, 10).len(), 4);
            renderer.set_motion(false);
            let still = testkit::render(&mut renderer, &theme, &scene);
            scene.animation = Duration::from_secs(4);
            renderer.set_motion(true);
            assert_eq!(still, testkit::render(&mut renderer, &theme, &scene));
        }
    }

    const CLEAR: Rgba = Rgba {
        r: 0,
        g: 0,
        b: 0,
        a: 0,
    };

    fn rect(frame: BoxF, color: Rgba, opacity: f32) -> Element {
        let mut e = element(
            frame,
            ElementKind::Shape {
                image: None,
                video_window: false,
                fade: None,
                shape: ShapeKind::Rect { radius: 0.0 },
                fill: Some(Paint::solid(color)),
                stroke: None,
            },
        );
        e.opacity = opacity;
        e
    }

    #[test]
    fn value_colour_reaches_every_supported_renderer_paint() {
        use bezel_core::domain::theme::{Cap, Direction, GraphStyle, TextStyle};
        use bezel_core::domain::value_color::{ColorByValue, ColorThreshold};
        let green = Rgba::opaque(0, 200, 0);
        let red = Rgba::opaque(255, 0, 0);
        let binding = Binding {
            key: testkit::key("cpu.usage"),
            min: 0.,
            max: 100.,
        };
        let kinds = vec![
            ElementKind::Shape {
                image: None,
                shape: ShapeKind::Rect { radius: 0. },
                fill: Some(Paint::solid(Rgba::WHITE)),
                stroke: None,
                fade: None,
                video_window: false,
            },
            ElementKind::Text {
                content: TextContent::Static("X".into()),
                style: TextStyle {
                    size: 20.,
                    ..Default::default()
                },
            },
            ElementKind::Text {
                content: TextContent::Sensor {
                    key: binding.key.clone(),
                    format: Default::default(),
                    prefix: String::new(),
                    suffix: String::new(),
                },
                style: TextStyle {
                    size: 16.,
                    ..Default::default()
                },
            },
            ElementKind::Bar {
                binding: binding.clone(),
                direction: Direction::LeftToRight,
                fill: Paint::solid(Rgba::WHITE),
                track: None,
                radius: 0.,
                segments: None,
            },
            ElementKind::Ring {
                binding: binding.clone(),
                start_angle: 0.,
                sweep: 360.,
                thickness: 4.,
                clockwise: true,
                fill: Paint::solid(Rgba::WHITE),
                track: None,
                cap: Cap::Butt,
                segments: None,
                test_full: false,
            },
            ElementKind::Needle {
                binding: binding.clone(),
                asset: None,
                pivot: (0.5, 0.5),
                start_angle: 0.,
                sweep: 180.,
                color: Rgba::WHITE,
                width: 4.,
                look: None,
            },
            ElementKind::Needle {
                binding: binding.clone(),
                asset: Some(AssetRef("icon.svg".into())),
                pivot: (0.5, 0.5),
                start_angle: 0.,
                sweep: 180.,
                color: Rgba::WHITE,
                width: 4.,
                look: None,
            },
            ElementKind::Graph {
                binding: binding.clone(),
                history: 4,
                interval_seconds: None,
                style: GraphStyle::Line,
                color: Rgba::WHITE,
                fill: None,
                line_width: 4.,
                autoscale: false,
            },
            ElementKind::Image {
                asset: AssetRef("icon.svg".into()),
                fit: Fit::Contain,
                fade: None,
            },
        ];
        let scene = Scene::empty().with("cpu.usage",90.).history("cpu.usage", &[Some(10.),Some(30.),Some(70.),Some(90.)])
            .asset("icon.svg", br#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24"><rect width="24" height="24" fill="white"/></svg>"#.to_vec());
        let mut renderer = testkit::renderer();
        for kind in kinds {
            let mut e = element(BoxF::new(0., 0., 32., 32.), kind);
            e.color_by_value = Some(ColorByValue {
                sensor: Some(binding.key.clone()),
                base: green,
                thresholds: vec![ColorThreshold {
                    value: 60.,
                    color: red,
                }],
            });
            let mut theme = testkit::theme(32, 32, Background::Color(Rgba::BLACK), vec![e]);
            let frame = render_over(&mut renderer, &theme, &scene, Backdrop::Poster);
            assert!(
                count(&frame, (0, 0, 32, 32), |p| p.r > p.g && p.r > 100) > 0,
                "red for {:?}",
                theme.elements[0].kind
            );
            let empty =
                Scene::empty().history("cpu.usage", &[Some(10.), Some(30.), Some(70.), Some(90.)]);
            // Keep the icon asset while removing the sample.
            let mut empty = empty;
            empty.assets = scene.assets.clone();
            let frame = render_over(&mut renderer, &theme, &empty, Backdrop::Poster);
            if !matches!(
                theme.elements[0].kind,
                ElementKind::Bar { .. } | ElementKind::Ring { .. } | ElementKind::Needle { .. }
            ) {
                assert!(
                    count(&frame, (0, 0, 32, 32), |p| p.g > p.r && p.g > 100) > 0,
                    "base for {:?}",
                    theme.elements[0].kind
                );
            }
            let e = &mut theme.elements[0];
            if e.bound_sensor().is_some() {
                let eased = Scene::empty().with("cpu.usage", 90.).easing(1, 59.);
                assert_eq!(e.value_color(&eased.snapshot, &eased.shown), Some(green));
                let eased = eased.easing(1, 60.);
                assert_eq!(e.value_color(&eased.snapshot, &eased.shown), Some(red));
                let missing = Scene::empty().easing(1, 90.);
                assert_eq!(
                    e.value_color(&missing.snapshot, &missing.shown),
                    Some(green)
                );
                e.color_by_value.as_mut().unwrap().sensor = Some(testkit::key("ignored.sensor"));
                assert_eq!(e.color_sensor(), Some(&binding.key));
            }
        }
    }

    #[test]
    fn rule_sensors_are_wanted_only_while_sampled() {
        use bezel_core::domain::value_color::ColorByValue;
        let mut e = rect(BoxF::new(0., 0., 32., 32.), Rgba::WHITE, 1.);
        e.color_by_value = Some(ColorByValue {
            sensor: Some(testkit::key("cpu.temperature")),
            base: Rgba::WHITE,
            thresholds: Vec::new(),
        });
        let mut theme = testkit::theme(32, 32, Background::Color(Rgba::BLACK), vec![e]);
        assert_eq!(
            theme.sensor_keys(),
            [testkit::key("cpu.temperature")].into_iter().collect()
        );
        theme.elements[0].visible = false;
        assert!(theme.sensor_keys().is_empty());
    }

    fn measure(r: &mut SkiaRenderer, theme: &Theme, scene: &Scene) -> Option<BoxF> {
        let context = RenderContext {
            snapshot: &scene.snapshot,
            histories: &scene.histories,
            quantities: &bezel_core::domain::sensor::Quantities::new(),
            time: scene.time,
            animation: scene.animation,
            language: bezel_core::domain::clock::Language::English,
            backdrop: Backdrop::Poster,
            shown: &scene.shown,
        };
        r.measure_element(theme, ElementId(1), &scene.assets, context)
            .unwrap()
    }

    #[test]
    fn natural_text_frames_use_current_font_format_value_and_alignment() {
        use bezel_core::domain::theme::{HAlign, TextStyle, VAlign};
        let style = TextStyle {
            size: 20.,
            align: HAlign::Right,
            valign: VAlign::Bottom,
            ..TextStyle::default()
        };
        let kinds = [
            TextContent::Static("Hello".into()),
            TextContent::Clock {
                pattern: "%H:%M".into(),
                language: None,
                casing: bezel_core::domain::clock::ClockCase::Normal,
                analog: None,
            },
            TextContent::Sensor {
                key: testkit::key("cpu.usage"),
                format: Default::default(),
                prefix: "CPU ".into(),
                suffix: "!".into(),
            },
            TextContent::Weather(bezel_core::domain::weather::Weather {
                city: "Rome".into(),
                latitude: 42.,
                longitude: 12.,
                language: None,
                fahrenheit: false,
                show_icon: true,
                icon_style: Default::default(),
                icon_gap: Some(12.),
                icon_size: Some(50.),
                icon_motion: Default::default(),
                icon_speed: 1.0,
            }),
            TextContent::Player(bezel_core::domain::playback::Player {
                source: String::new(),
                show_cover: true,
                cover_radius: 0.,
                cover_corners: None,
                cover_gap: None,
                show_progress: true,
                show_source: true,
                hide_when_stopped: false,
                empty_text: "No media".into(),
                ..Default::default()
            }),
        ];
        let mut scene = Scene::empty().with("cpu.usage", 35.);
        scene
            .snapshot
            .media
            .push(bezel_core::domain::playback::MediaSession {
                title: "Track".into(),
                artist: "Artist".into(),
                source: "Player".into(),
                duration: 240.,
                playing: true,
                ..Default::default()
            });
        let mut renderer = testkit::renderer();
        for content in kinds {
            let e = element(
                BoxF::new(10., 20., 400., 300.),
                ElementKind::Text {
                    content,
                    style: style.clone(),
                },
            );
            let mut theme = testkit::theme(600, 600, Background::Color(Rgba::BLACK), vec![e]);
            let b = measure(&mut renderer, &theme, &scene).expect("text measurement");
            assert!(b.width > 0. && b.height > 0. && b.height < 300.);
            assert_eq!(b.x + b.width, 410.);
            assert_eq!(b.y + b.height, 320.);
            theme.elements[0].frame = b;
            assert_eq!(
                measure(&mut renderer, &theme, &scene),
                Some(b),
                "fit is stable"
            );
            theme.elements[0].locked = true;
            assert_eq!(measure(&mut renderer, &theme, &scene), None);
        }
    }

    #[test]
    fn measurement_follows_shown_reading_format_size_and_spacing() {
        use bezel_core::domain::theme::TextStyle;
        let e = element(
            BoxF::new(0., 0., 300., 100.),
            ElementKind::Text {
                content: TextContent::Sensor {
                    key: testkit::key("cpu.usage"),
                    format: Default::default(),
                    prefix: String::new(),
                    suffix: String::new(),
                },
                style: TextStyle {
                    size: 20.,
                    ..Default::default()
                },
            },
        );
        let mut theme = testkit::theme(400, 400, Background::Color(Rgba::BLACK), vec![e]);
        let mut renderer = testkit::renderer();
        let scene = Scene::empty().with("cpu.usage", 9.);
        let small = measure(&mut renderer, &theme, &scene).unwrap();
        let shown = Scene::empty().with("cpu.usage", 9.).easing(1, 100.);
        assert!(measure(&mut renderer, &theme, &shown).unwrap().width > small.width);
        if let ElementKind::Text {
            content: TextContent::Sensor { format, .. },
            ..
        } = &mut theme.elements[0].kind
        {
            format.decimals = Some(3);
        }
        let formatted = measure(&mut renderer, &theme, &scene).unwrap();
        assert!(formatted.width > small.width);
        if let ElementKind::Text { style, .. } = &mut theme.elements[0].kind {
            style.size = 40.;
        }
        let large = measure(&mut renderer, &theme, &scene).unwrap();
        assert!(large.width > formatted.width * 1.8 && large.height > formatted.height);
        if let ElementKind::Text { style, .. } = &mut theme.elements[0].kind {
            style.letter_spacing = 8.;
        }
        assert!(measure(&mut renderer, &theme, &scene).unwrap().width > large.width);
    }

    #[test]
    fn natural_images_and_svg_icons_keep_the_long_side() {
        let scene = Scene::empty().asset("wide.png", png(80, 20, |_, _| Rgba::WHITE))
            .asset("tall.svg", br#"<svg xmlns="http://www.w3.org/2000/svg" width="20" height="80"><rect width="20" height="80"/></svg>"#.to_vec());
        let mut renderer = testkit::renderer();
        for (asset, expected) in [("wide.png", (100., 25.)), ("tall.svg", (25., 100.))] {
            let e = element(
                BoxF::new(10., 20., 100., 100.),
                ElementKind::Image {
                    asset: AssetRef(asset.into()),
                    fit: Fit::Contain,
                    fade: None,
                },
            );
            let theme = testkit::theme(200, 200, Background::Color(Rgba::BLACK), vec![e]);
            let b = measure(&mut renderer, &theme, &scene).unwrap();
            assert_eq!(
                (b.x, b.y, b.width, b.height),
                (10., 20., expected.0, expected.1)
            );
        }
    }

    #[test]
    fn device_video_background_renders_a_transparent_base() {
        let mut r = testkit::renderer();
        let green = Rgba::opaque(0, 255, 0);
        let red = Rgba::opaque(255, 0, 0);
        let white = Rgba::WHITE;
        let scene = Scene::empty().asset("poster.png", png(2, 2, |_, _| green));
        let disc = element(
            BoxF::new(20.0, 0.0, 12.0, 12.0),
            ElementKind::Shape {
                image: None,
                video_window: false,
                fade: None,
                shape: ShapeKind::Ellipse,
                fill: Some(Paint::solid(white)),
                stroke: None,
            },
        );
        let elements = vec![
            rect(BoxF::new(0.0, 0.0, 8.0, 8.0), red, 1.0),
            rect(BoxF::new(10.0, 0.0, 8.0, 8.0), white, 0.5),
            disc,
        ];
        let video = Background::Video {
            asset: AssetRef("assets/clip.mp4".into()),
            poster: Some(AssetRef("poster.png".into())),
            framing: None,
        };
        let theme = testkit::theme(32, 16, video, elements.clone());

        let overlay = render_over(&mut r, &theme, &scene, Backdrop::OnDevice);
        assert_eq!(px(&overlay, 0, 12), CLEAR, "A = 0 lets the video show");
        assert_eq!(count(&overlay, (0, 8, 20, 16), |p| p == CLEAR), 20 * 8);
        assert_eq!(px(&overlay, 4, 4), red, "opaque elements hide the video");
        assert_eq!(
            px(&overlay, 14, 4),
            Rgba { a: 128, ..white },
            "straight alpha"
        );
        let edge = count(&overlay, (20, 0, 32, 12), |p| p.a > 0 && p.a < 255);
        assert!(edge > 0, "anti-aliased edges keep their partial alpha");
        let tinted = count(&overlay, (20, 0, 32, 12), |p| {
            p.a > 0 && p.r.min(p.g).min(p.b) < 250
        });
        assert_eq!(tinted, 0, "partial pixels keep the element's color");

        let poster = render_over(&mut r, &theme, &scene, Backdrop::Poster);
        assert_eq!(px(&poster, 0, 12), green, "previews show the poster");
        assert_eq!(px(&poster, 4, 4), red);

        for background in [
            Background::Color(Rgba::opaque(1, 2, 3)),
            Background::Image {
                asset: AssetRef("poster.png".into()),
                fit: Fit::Fill,
            },
        ] {
            let still = testkit::theme(32, 16, background, elements.clone());
            let a = render_over(&mut r, &still, &scene, Backdrop::Poster);
            let b = render_over(&mut r, &still, &scene, Backdrop::OnDevice);
            assert_eq!(
                a.as_rgba(),
                b.as_rgba(),
                "other backgrounds ignore the backdrop"
            );
        }
    }

    #[test]
    fn opacity_maps_to_bytes() {
        assert_eq!(opacity_of(1.0), 255);
        assert_eq!(opacity_of(0.5), 128);
        assert_eq!(opacity_of(-3.0), 0);
        assert_eq!(opacity_of(f32::NAN), 255);
    }

    #[test]
    fn oversized_canvases_are_refused() {
        assert!(new_canvas(Size::new(MAX_CANVAS_SIDE + 1, 1)).is_err());
        assert!(new_canvas(Size::new(4, 4)).is_ok());
    }
    #[test]
    fn groups_render_nested_opacity_visibility_and_card_ownership_once() {
        use bezel_core::domain::theme::{Card, CardMember};
        let black = Rgba::opaque(0, 0, 0);
        let red = Rgba::opaque(255, 0, 0);
        let mut group = rect(BoxF::new(0.0, 0.0, 16.0, 16.0), CLEAR, 0.5);
        group.id = ElementId(1);
        group.is_group = true;
        let mut child = rect(group.frame, red, 1.0);
        child.id = ElementId(2);
        child.group_parent = Some(group.id);
        let mut theme = testkit::theme(16, 16, Background::Color(black), vec![group, child]);
        let mut r = testkit::renderer();
        let scene = Scene::empty();
        let frame = render_over(&mut r, &theme, &scene, Backdrop::Poster);
        assert!((126..=129).contains(&px(&frame, 4, 4).r));
        theme.elements[0].visible = false;
        assert_eq!(
            px(&render_over(&mut r, &theme, &scene, Backdrop::Poster), 4, 4),
            black
        );
        theme.elements[0].visible = true;
        let mut parent = rect(theme.elements[0].frame, CLEAR, 0.5);
        parent.id = ElementId(3);
        parent.card = Some(Card {
            triggers: Vec::new(),
            faces: vec!["A".into(), "B".into()],
            active_face: 0,
            rotation_seconds: None,
            transition: None,
        });
        for e in &mut theme.elements {
            e.card_member = Some(CardMember {
                parent: parent.id,
                face: Some(0),
            });
        }
        theme.elements.insert(0, parent);
        let frame = render_over(&mut r, &theme, &scene, Backdrop::Poster);
        assert!((62..=66).contains(&px(&frame, 4, 4).r));
        theme.elements[0].card.as_mut().unwrap().active_face = 1;
        assert_eq!(
            px(&render_over(&mut r, &theme, &scene, Backdrop::Poster), 4, 4),
            black
        );
    }

    #[test]
    fn card_faces_render_shared_base_and_inherit_parent_opacity() {
        use bezel_core::domain::theme::{Card, CardMember};
        let black = Rgba::opaque(0, 0, 0);
        let red = Rgba::opaque(255, 0, 0);
        let green = Rgba::opaque(0, 255, 0);
        let mut parent = rect(BoxF::new(0.0, 0.0, 32.0, 16.0), black, 1.0);
        parent.card = Some(Card {
            triggers: Vec::new(),
            faces: vec!["A".into(), "B".into()],
            active_face: 0,
            rotation_seconds: None,
            transition: None,
        });
        let mut base = rect(BoxF::new(0.0, 0.0, 8.0, 8.0), Rgba::WHITE, 1.0);
        base.id = ElementId(2);
        base.card_member = Some(CardMember {
            parent: parent.id,
            face: None,
        });
        let mut a = rect(BoxF::new(10.0, 0.0, 8.0, 8.0), red, 1.0);
        a.id = ElementId(3);
        a.card_member = Some(CardMember {
            parent: parent.id,
            face: Some(0),
        });
        let mut b = a.clone();
        b.id = ElementId(4);
        b.kind = rect(b.frame, green, 1.0).kind;
        b.card_member.as_mut().unwrap().face = Some(1);
        let mut theme = testkit::theme(32, 16, Background::Color(black), vec![parent, base, a, b]);
        let mut r = testkit::renderer();
        let scene = Scene::empty();
        let frame = render_over(&mut r, &theme, &scene, Backdrop::Poster);
        assert_eq!(px(&frame, 4, 4), Rgba::WHITE);
        assert_eq!(px(&frame, 14, 4), red);
        theme.elements[0].card.as_mut().unwrap().active_face = 1;
        let frame = render_over(&mut r, &theme, &scene, Backdrop::Poster);
        assert_eq!(px(&frame, 14, 4), green);
        theme.elements[0].opacity = 0.5;
        let frame = render_over(&mut r, &theme, &scene, Backdrop::Poster);
        assert!((126..=129).contains(&px(&frame, 14, 4).g));
        theme.elements[0].visible = false;
        let frame = render_over(&mut r, &theme, &scene, Backdrop::Poster);
        assert_eq!(px(&frame, 4, 4), black);
        assert_eq!(px(&frame, 14, 4), black);
    }
    #[test]
    fn card_transitions_schedule_intermediate_frames_and_finish_exactly() {
        use bezel_core::app::ThemeRuntime;
        use bezel_core::domain::clock::Language;
        use bezel_core::domain::theme::{
            Card, CardDirection, CardEffect, CardMember, CardTransition,
        };
        use std::time::Duration;
        let black = Rgba::opaque(0, 0, 0);
        let red = Rgba::opaque(255, 0, 0);
        let green = Rgba::opaque(0, 255, 0);
        for effect in [CardEffect::Fade, CardEffect::Slide, CardEffect::Flip] {
            let mut parent = rect(BoxF::new(0.0, 0.0, 32.0, 16.0), black, 1.0);
            parent.card = Some(Card {
                triggers: Vec::new(),
                faces: vec!["A".into(), "B".into()],
                active_face: 0,
                rotation_seconds: None,
                transition: Some(CardTransition {
                    effect,
                    direction: CardDirection::Left,
                    duration_ms: 600,
                    include_base: false,
                }),
            });
            let mut a = rect(BoxF::new(10.0, 0.0, 8.0, 8.0), red, 1.0);
            a.id = ElementId(2);
            a.card_member = Some(CardMember {
                parent: parent.id,
                face: Some(0),
            });
            let mut b = a.clone();
            b.id = ElementId(3);
            b.kind = rect(b.frame, green, 1.0).kind;
            b.card_member.as_mut().unwrap().face = Some(1);
            let mut theme = testkit::theme(32, 16, Background::Color(black), vec![parent, a, b]);
            let mut runtime = ThemeRuntime::new(theme.clone(), BTreeMap::new(), Language::English);
            let mut renderer = testkit::renderer();
            assert!(
                runtime
                    .preview(&mut renderer, testkit::TIME, Duration::ZERO)
                    .unwrap()
                    .1
                    .is_none()
            );
            theme.elements[0].card.as_mut().unwrap().active_face = 1;
            runtime.replace_theme(theme.clone());
            let (first, due) = runtime
                .preview(&mut renderer, testkit::TIME, Duration::from_millis(100))
                .unwrap();
            assert_eq!(px(&first, 14, 4), red);
            assert_eq!(due, Some(Duration::from_millis(133)));
            let (middle, due) = runtime
                .preview(&mut renderer, testkit::TIME, Duration::from_millis(400))
                .unwrap();
            assert!(due.is_some());
            if effect == CardEffect::Fade {
                let p = px(&middle, 14, 4);
                assert!((126..=129).contains(&p.r) && (126..=129).contains(&p.g));
            } else {
                assert_eq!(px(&middle, 14, 4), black);
            }
            let (last, due) = runtime
                .preview(&mut renderer, testkit::TIME, Duration::from_millis(700))
                .unwrap();
            assert_eq!(px(&last, 14, 4), green);
            assert_eq!(due, None);
            renderer.set_motion(false);
            theme.elements[0].card.as_mut().unwrap().active_face = 0;
            runtime.replace_theme(theme);
            let (still, due) = runtime
                .preview(&mut renderer, testkit::TIME, Duration::from_millis(800))
                .unwrap();
            assert_eq!(px(&still, 14, 4), red);
            assert_eq!(due, None);
            if effect == CardEffect::Flip {
                let mut window_theme = runtime.theme().clone();
                if let ElementKind::Shape { video_window, .. } = &mut window_theme.elements[0].kind
                {
                    *video_window = true;
                }
                window_theme.elements[0]
                    .card
                    .as_mut()
                    .unwrap()
                    .transition
                    .as_mut()
                    .unwrap()
                    .include_base = true;
                renderer.set_motion(true);
                let mut scene = Scene::empty();
                scene.animation = Duration::from_millis(900);
                let first = render_over(&mut renderer, &window_theme, &scene, Backdrop::OnDevice);
                assert_eq!(px(&first, 16, 12).a, 0);
                window_theme.elements[0].card.as_mut().unwrap().active_face = 1;
                scene.animation = Duration::from_millis(1000);
                let first = render_over(&mut renderer, &window_theme, &scene, Backdrop::OnDevice);
                assert_eq!(
                    px(&first, 16, 12).a,
                    0,
                    "device video remains visible when motion starts"
                );
                scene.animation = Duration::from_millis(1150);
                let middle = render_over(&mut renderer, &window_theme, &scene, Backdrop::OnDevice);
                assert_eq!(
                    px(&middle, 16, 12).a,
                    0,
                    "transformed video window remains a cutout"
                );
            }
        }
    }
    #[test]
    fn cached_card_faces_match_fresh_pixels_and_invalidate_inputs() {
        use bezel_core::domain::theme::{
            Card, CardDirection, CardEffect, CardMember, CardTransition,
        };
        use std::time::Duration;
        let mut parent = rect(BoxF::new(0., 0., 32., 16.), Rgba::opaque(20, 30, 40), 1.);
        parent.card = Some(Card {
            triggers: Vec::new(),
            faces: vec!["A".into(), "B".into()],
            active_face: 0,
            rotation_seconds: None,
            transition: Some(CardTransition {
                effect: CardEffect::Flip,
                direction: CardDirection::Left,
                duration_ms: 750,
                include_base: true,
            }),
        });
        let mut member = rect(BoxF::new(4., 4., 12., 8.), Rgba::WHITE, 0.8);
        member.id = ElementId(2);
        member.card_member = Some(CardMember {
            parent: parent.id,
            face: Some(0),
        });
        let mut theme = testkit::theme(
            32,
            16,
            Background::DeviceVideo {
                path: bezel_core::domain::storage::RemotePath::parse("internal/video/a.mp4")
                    .unwrap(),
                repeat: bezel_core::domain::storage::Repeat::Loop,
                color: Rgba::BLACK,
            },
            vec![parent, member],
        );
        let mut scene = Scene::empty();
        let mut cached = testkit::renderer();
        let mut fresh = testkit::renderer();
        cached.face_cache_enabled = true;
        cached.set_content_revision(1);
        for r in [&mut cached, &mut fresh] {
            render_over(r, &theme, &scene, Backdrop::Poster);
        }
        theme.elements[0].card.as_mut().unwrap().active_face = 1;
        for ms in [100, 150, 200, 250, 500, 550, 600] {
            scene.animation = Duration::from_millis(ms);
            let a = render_over(&mut cached, &theme, &scene, Backdrop::Poster);
            let b = render_over(&mut fresh, &theme, &scene, Backdrop::Poster);
            assert_eq!(a, b, "cached pose {ms}");
            if ms == 150 {
                assert_eq!(cached.render_stats().face_hits, 1);
            }
            let a = render_over(&mut cached, &theme, &scene, Backdrop::OnDevice);
            let b = render_over(&mut fresh, &theme, &scene, Backdrop::OnDevice);
            assert_eq!(a, b, "overlay pose {ms}");
            assert_eq!(
                cached.render_stats().face_hits,
                1,
                "same face across backgrounds"
            );
        }
        for change in 0..4 {
            match change {
                0 => {
                    scene.snapshot.insert(
                        testkit::key("cpu.usage"),
                        bezel_core::domain::sensor::Reading::Value(20.),
                    );
                }
                1 => {
                    theme.elements[0].opacity = 0.3;
                }
                2 => {
                    scene.time.second += 1;
                }
                _ => cached.set_content_revision(2),
            }
            let a = render_over(&mut cached, &theme, &scene, Backdrop::OnDevice);
            let b = render_over(&mut fresh, &theme, &scene, Backdrop::OnDevice);
            assert_eq!(a, b);
            assert_eq!(
                cached.render_stats().face_misses,
                1,
                "invalidation {change}"
            );
        }
        // Window masks are reused only with their matching backdrop variant.
        if let ElementKind::Shape { video_window, .. } = &mut theme.elements[0].kind {
            *video_window = true;
        }
        for backdrop in [
            Backdrop::Poster,
            Backdrop::OnDevice,
            Backdrop::Poster,
            Backdrop::OnDevice,
        ] {
            let a = render_over(&mut cached, &theme, &scene, backdrop);
            let b = render_over(&mut fresh, &theme, &scene, backdrop);
            assert_eq!(a, b, "cached window mask");
        }
        // A GIF never reuses a face across animation times.
        theme.elements[1].card_member.as_mut().unwrap().face = Some(1);
        theme.elements[1].kind = ElementKind::Image {
            fade: None,
            asset: AssetRef("a.gif".into()),
            fit: Fit::Fill,
        };
        scene.assets.insert(
            AssetRef("a.gif".into()),
            testkit::gif(2, 2, &[(Rgba::WHITE, 100), (Rgba::BLACK, 100)]),
        );
        cached.set_content_revision(3);
        for ms in [610, 630] {
            scene.animation = Duration::from_millis(ms);
            let a = render_over(&mut cached, &theme, &scene, Backdrop::OnDevice);
            let b = render_over(&mut fresh, &theme, &scene, Backdrop::OnDevice);
            assert_eq!(a, b, "animated face");
            assert_eq!(cached.render_stats().face_hits, 0);
        }
        assert!(
            cached
                .face_cache
                .values()
                .map(|f| f.pixels.data().len() + f.window.as_ref().map_or(0, |w| w.data().len()))
                .sum::<usize>()
                <= FACE_CACHE_BYTES
        );
    }

    /// Run explicitly in release mode; records CPU render cost, excluding serial/IPC.
    #[test]
    #[ignore]
    fn profile_card_flip_480x1920() {
        use bezel_core::domain::theme::{
            Card, CardDirection, CardEffect, CardMember, CardTransition,
        };
        use std::time::{Duration, Instant};
        let mut parent = rect(
            BoxF::new(36.0, 650.0, 408.0, 408.0),
            Rgba::opaque(30, 41, 59),
            1.0,
        );
        parent.card = Some(Card {
            triggers: Vec::new(),
            faces: vec!["A".into(), "B".into()],
            active_face: 0,
            rotation_seconds: None,
            transition: Some(CardTransition {
                effect: CardEffect::Flip,
                direction: CardDirection::Left,
                duration_ms: 1000,
                include_base: true,
            }),
        });
        let mut members = vec![parent];
        for face in 0..2 {
            for i in 0..8 {
                let mut e = rect(
                    BoxF::new(65.0, 690.0 + i as f32 * 38.0, 330.0, 20.0),
                    if face == 0 {
                        Rgba::opaque(34, 211, 238)
                    } else {
                        Rgba::opaque(192, 132, 252)
                    },
                    1.0,
                );
                e.id = ElementId(2 + face as u32 * 8 + i as u32);
                e.card_member = Some(CardMember {
                    parent: ElementId(1),
                    face: Some(face),
                });
                members.push(e);
            }
        }
        let mut theme = testkit::theme(480, 1920, Background::Color(Rgba::BLACK), members);
        let mut renderer = testkit::renderer();
        let mut scene = Scene::empty();
        render_over(&mut renderer, &theme, &scene, Backdrop::Poster);
        renderer.set_content_revision(1);
        theme.elements[0].card.as_mut().unwrap().active_face = 1;
        let mut costs = Vec::new();
        let mut film = RgbaImage::new(480 * 5, 600);
        for i in 0..60 {
            scene.animation = Duration::from_millis(i * 16);
            let started = Instant::now();
            let frame = render_over(&mut renderer, &theme, &scene, Backdrop::Poster);
            costs.push(started.elapsed().as_secs_f64() * 1000.0);
            if [0, 15, 30, 45, 59].contains(&i) {
                let image = RgbaImage::from_raw(480, 1920, frame.as_rgba().to_vec()).unwrap();
                let crop = image::imageops::crop_imm(&image, 0, 560, 480, 600).to_image();
                image::imageops::replace(
                    &mut film,
                    &crop,
                    ([0, 15, 30, 45, 59].iter().position(|v| *v == i).unwrap() * 480) as i64,
                    0,
                );
            }
        }
        costs.sort_by(f64::total_cmp);
        eprintln!(
            "480x1920 Flip CPU: median {:.2} ms, p95 {:.2} ms, max {:.2} ms",
            costs[30], costs[57], costs[59]
        );
        film.save("../../target/card-flip-perspective.png").unwrap();
    }
}
