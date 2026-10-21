//! Themes: what a screen shows. A theme is a canvas (the panel in one
//! orientation) with a background and a z-ordered list of elements; each
//! element has a box and a kind (text, image, bar, ring, needle, graph, …)
//! and may be bound to a sensor.
//!
//! The model is plain data so the editor can change it freely and every
//! adapter (renderer, file formats, importers) reads the same thing.

use std::collections::{BTreeMap, BTreeSet};
use std::time::Duration;

use super::clock::{ClockCase, Language};
use super::frame::Rgba;
use super::framing::VideoFraming;
use super::geometry::{Orientation, Size};
use super::history::SeriesId;
use super::sensor::{DisplayFormat, SensorKey};

/// Identifies an element inside its theme (stable across edits).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ElementId(pub u32);

/// Identifies an asset (image, font, video) bundled with the theme, e.g.
/// `assets/background.png`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct AssetRef(pub String);

/// An element's box in canvas pixels. May extend past the canvas.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct BoxF {
    /// Left edge.
    pub x: f32,
    /// Top edge.
    pub y: f32,
    /// Width.
    pub width: f32,
    /// Height.
    pub height: f32,
}

impl BoxF {
    /// Builds a box.
    pub const fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    /// Center point.
    pub fn center(self) -> (f32, f32) {
        (self.x + self.width / 2.0, self.y + self.height / 2.0)
    }
}

/// A color or a linear gradient.
#[derive(Debug, Clone, PartialEq)]
pub enum Paint {
    /// One color.
    Solid(Rgba),
    /// Gradient along a ring scale, independent of its current value.
    Arc {
        /// Color at zero.
        start: Rgba,
        /// Color at full scale.
        end: Rgba,
        /// Position of the half-color blend, 0..=1.
        transition: f32,
    },
    /// A linear gradient across the element's box.
    Linear {
        /// Direction in degrees: 0 = left→right, 90 = top→bottom.
        angle: f32,
        /// Color stops, positions 0..=1 in order.
        stops: Vec<(f32, Rgba)>,
    },
}

impl Paint {
    /// A solid paint.
    pub const fn solid(color: Rgba) -> Self {
        Paint::Solid(color)
    }
}

/// How an image fills its box.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Fit {
    /// Stretch to the box.
    #[default]
    Fill,
    /// Scale to fit inside, keeping proportions.
    Contain,
    /// Scale to cover, keeping proportions, cropping the overflow.
    Cover,
    /// Original size, anchored top-left.
    None,
}

/// Which part of a picture stays in view when it covers its box: one of
/// nine points (D-2026-10-20-studio-ux-v2-04, Immagine).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Anchor {
    /// Top left corner.
    TopLeft,
    /// Middle of the top edge.
    Top,
    /// Top right corner.
    TopRight,
    /// Middle of the left edge.
    Left,
    /// The centre.
    #[default]
    Center,
    /// Middle of the right edge.
    Right,
    /// Bottom left corner.
    BottomLeft,
    /// Middle of the bottom edge.
    Bottom,
    /// Bottom right corner.
    BottomRight,
}

impl Anchor {
    /// Every anchor, row by row.
    pub const ALL: [Anchor; 9] = [
        Anchor::TopLeft,
        Anchor::Top,
        Anchor::TopRight,
        Anchor::Left,
        Anchor::Center,
        Anchor::Right,
        Anchor::BottomLeft,
        Anchor::Bottom,
        Anchor::BottomRight,
    ];

    /// The anchor as shares of the width and height (0, 0.5 or 1).
    pub fn shares(self) -> (f32, f32) {
        let index = Self::ALL.iter().position(|a| *a == self).unwrap_or(4);
        ((index % 3) as f32 / 2.0, (index / 3) as f32 / 2.0)
    }
}

/// How an animated picture (GIF) plays.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Playback {
    /// Plays once and stays on its last frame, instead of looping.
    pub once: bool,
    /// Speed: 1 as made, 0.5 half as fast, 2 twice as fast.
    pub speed: f32,
}

impl Default for Playback {
    fn default() -> Self {
        Self {
            once: false,
            speed: 1.0,
        }
    }
}

impl Playback {
    /// Allowed range of [`Playback::speed`].
    pub const SPEED: std::ops::RangeInclusive<f32> = 0.25..=4.0;

    fn speed(&self) -> f64 {
        f64::from(self.speed.clamp(*Self::SPEED.start(), *Self::SPEED.end()))
    }

    /// The moment of an animation lasting `total` shown `elapsed` after the
    /// theme started: sped up or slowed down, and held just before the end
    /// (its last frame) once a single play is over.
    pub fn moment(&self, elapsed: Duration, total: Duration) -> Duration {
        let local = elapsed.mul_f64(self.speed());
        if self.once && !total.is_zero() && local >= total {
            total - Duration::from_nanos(1)
        } else {
            local
        }
    }

    /// When the frame shown at `elapsed` changes, on the theme's clock, for
    /// a frame that changes at `change` on the animation's own clock
    /// (`None` once a single play has reached its last frame).
    pub fn next_change(
        &self,
        elapsed: Duration,
        total: Duration,
        change: impl FnOnce(Duration) -> Duration,
    ) -> Option<Duration> {
        let local = elapsed.mul_f64(self.speed());
        if self.once && local >= total {
            return None;
        }
        let next = change(local);
        if self.once && next >= total {
            return None;
        }
        Some(elapsed + (next.saturating_sub(local)).div_f64(self.speed()))
    }
}

/// How a shape's image covers the shape.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ImageRepeat {
    /// Scale to cover the shape, keeping proportions, cropping the overflow.
    #[default]
    Cover,
    /// Scale to fit inside the shape, keeping proportions; the fill shows around it.
    Contain,
    /// Stretch to the shape's box.
    Stretch,
    /// One copy at its own size times the scale, centered, then offset.
    Center,
    /// Copies side by side in both directions.
    Tile,
    /// Copies side by side along one row.
    TileX,
    /// Copies stacked along one column.
    TileY,
}

impl ImageRepeat {
    /// Whether the image keeps its own size (times the scale) instead of
    /// following the shape: scale and offset apply.
    pub const fn natural_size(self) -> bool {
        matches!(self, Self::Center | Self::Tile | Self::TileX | Self::TileY)
    }
}

/// An image painted inside a shape, over its fill and under its outline.
#[derive(Debug, Clone, PartialEq)]
pub struct ShapeImage {
    /// The picture (PNG, JPEG, GIF or SVG); empty while none is chosen.
    pub asset: AssetRef,
    /// How it covers the shape.
    pub repeat: ImageRepeat,
    /// Size of one copy relative to the picture's own size
    /// ([`ShapeImage::SCALE`]); only when [`ImageRepeat::natural_size`].
    pub scale: f32,
    /// Shift of the copies in pixels, from the shape's center for
    /// [`ImageRepeat::Center`] and from its top-left corner for the tiles.
    pub offset: (f32, f32),
    /// Opacity of the picture, `0..=1`; the fill under it is unaffected.
    pub opacity: f32,
}

impl ShapeImage {
    /// Allowed range of [`ShapeImage::scale`].
    pub const SCALE: std::ops::RangeInclusive<f32> = 0.05..=10.0;

    /// A picture covering the shape at full opacity.
    pub fn new(asset: AssetRef) -> Self {
        Self {
            asset,
            repeat: ImageRepeat::default(),
            scale: 1.0,
            offset: (0.0, 0.0),
            opacity: 1.0,
        }
    }
}

/// What the canvas shows under the elements.
#[derive(Debug, Clone, PartialEq)]
pub enum Background {
    /// Video already stored on the screen, played without host decoding.
    DeviceVideo {
        /// Exact internal/SD video path.
        path: super::storage::RemotePath,
        /// Loop or play once.
        repeat: super::storage::Repeat,
        /// Color outside video windows (ignored when there are no windows).
        color: Rgba,
    },
    /// A solid color.
    Color(Rgba),
    /// A still image.
    Image {
        /// Image asset.
        asset: AssetRef,
        /// How it fills the canvas.
        fit: Fit,
    },
    /// A video: played by the screen itself when it can (the theme is then
    /// an overlay), otherwise decoded and streamed by the host.
    Video {
        /// Video asset.
        asset: AssetRef,
        /// Frame shown in previews and when playback is unavailable.
        poster: Option<AssetRef>,
        /// How the video is turned, fitted, zoomed and placed on the canvas;
        /// `None` is [`VideoFraming::default`] (Auto rotation, cover,
        /// centered; D-2026-10-01-video-background-framing-2).
        framing: Option<VideoFraming>,
    },
}

/// A font face by family name, as installed or bundled.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FontSpec {
    /// Family name, e.g. `Inter`, or a bundled asset.
    pub family: String,
    /// CSS-like weight, 100..=900.
    pub weight: u16,
    /// Italic.
    pub italic: bool,
    /// A bundled font file, preferred over the system family when present.
    pub asset: Option<AssetRef>,
}

impl Default for FontSpec {
    fn default() -> Self {
        Self {
            family: "Inter".to_string(),
            weight: 400,
            italic: false,
            asset: None,
        }
    }
}

/// Horizontal text alignment inside the element box.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum HAlign {
    /// Left.
    #[default]
    Left,
    /// Center.
    Center,
    /// Right.
    Right,
}

/// Vertical text alignment inside the element box.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum VAlign {
    /// Top.
    #[default]
    Top,
    /// Middle.
    Middle,
    /// Bottom.
    Bottom,
}

/// How text looks.
#[derive(Debug, Clone, PartialEq)]
pub struct TextStyle {
    /// Optional centred glow behind the glyphs, clipped to the text layer.
    pub shadow: Option<TextShadow>,
    /// Non-destructive letter case for every text source.
    pub casing: super::text_case::TextCase,
    /// Face.
    pub font: FontSpec,
    /// Size in pixels.
    pub size: f32,
    /// Fill.
    pub paint: Paint,
    /// Horizontal alignment.
    pub align: HAlign,
    /// Vertical alignment.
    pub valign: VAlign,
    /// Extra space between characters, pixels.
    pub letter_spacing: f32,
    /// Every digit as wide as the widest one, so a changing number does not
    /// move (D-2026-10-20-studio-ux-v2-05); on by default. Only texts that
    /// show changing numbers (sensor values, clocks, weather) use it.
    pub tabular_digits: bool,
    /// What a line wider than the box does; `None` draws it past the box
    /// edge (clipped), as before.
    pub overflow: Option<TextOverflow>,
    /// Parts of a value or a digital clock with a style of their own
    /// (D-2026-10-20-studio-ux-v2-06).
    pub parts: TextParts,
}

impl Default for TextStyle {
    fn default() -> Self {
        Self {
            shadow: None,
            casing: super::text_case::TextCase::Normal,
            font: FontSpec::default(),
            size: 24.0,
            paint: Paint::solid(Rgba::WHITE),
            align: HAlign::Left,
            valign: VAlign::Top,
            letter_spacing: 0.0,
            tabular_digits: true,
            overflow: None,
            parts: TextParts::default(),
        }
    }
}

/// A text shadow without an offset, as in the static-text inspector.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TextShadow {
    /// Shadow colour, including its opacity.
    pub color: Rgba,
    /// Radius in pixels of a separable box blur, from 0 to 64.
    pub blur: u8,
}

/// What a line wider than its box does.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextOverflow {
    /// The whole text gets smaller until it fits.
    Shrink,
    /// The end is cut and replaced by an ellipsis.
    Ellipsis,
    /// The line scrolls sideways, round and round.
    Scroll,
}

/// A piece of a value or of a digital clock that can have its own style.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum TextPart {
    /// Value: the text before the number.
    Prefix,
    /// Value: the number.
    Number,
    /// Value: the unit after the number.
    Unit,
    /// Value: the text after the unit.
    Suffix,
    /// Clock: the hours.
    Hours,
    /// Clock: the minutes.
    Minutes,
    /// Clock: the seconds.
    Seconds,
    /// Clock: what stands between hours, minutes and seconds.
    Separators,
}

/// Where a part sits on the line.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Baseline {
    /// On the baseline, like the rest of the text.
    #[default]
    Base,
    /// Raised, its top level with the top of the number (an exponent).
    Super,
}

/// The style of one part; every field left empty follows the text's style.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PartStyle {
    /// Font family.
    pub family: Option<String>,
    /// Bundled font file of that family.
    pub asset: Option<AssetRef>,
    /// Size in percent of the text's size.
    pub size_percent: Option<f32>,
    /// CSS-like weight, 100..=900.
    pub weight: Option<u16>,
    /// Colour, in place of the text's paint.
    pub color: Option<Rgba>,
    /// Where it sits on the line.
    pub baseline: Baseline,
}

impl PartStyle {
    /// Allowed range of [`PartStyle::size_percent`].
    pub const SIZE_PERCENT: std::ops::RangeInclusive<f32> = 10.0..=400.0;

    /// Whether it changes nothing.
    pub fn is_plain(&self) -> bool {
        *self == Self::default()
    }

    /// Its size for a text of `size` pixels.
    pub fn size(&self, size: f32) -> f32 {
        self.size_percent.map_or(size, |p| {
            size * p.clamp(*Self::SIZE_PERCENT.start(), *Self::SIZE_PERCENT.end()) / 100.0
        })
    }
}

/// The parts of a text with a style of their own.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct TextParts {
    /// Style of each part that has one.
    pub styles: BTreeMap<TextPart, PartStyle>,
    /// Clock separators blink: shown for the first half of every second.
    pub blink_separators: bool,
}

impl TextParts {
    /// Whether no part has a style and nothing blinks.
    pub fn is_empty(&self) -> bool {
        !self.blink_separators && self.styles.values().all(PartStyle::is_plain)
    }

    /// The style of `part`, if it has one.
    pub fn style(&self, part: TextPart) -> Option<&PartStyle> {
        self.styles.get(&part).filter(|s| !s.is_plain())
    }
}

/// A sensor binding with the range that maps it to 0..=1 for bars, rings,
/// needles and graphs.
#[derive(Debug, Clone, PartialEq)]
pub struct Binding {
    /// The sensor.
    pub key: SensorKey,
    /// Value shown as empty.
    pub min: f64,
    /// Value shown as full.
    pub max: f64,
}

/// What a text element prints.
#[derive(Debug, Clone, PartialEq)]
pub enum TextContent {
    /// Standalone mirrored audio spectrum.
    Spectrum(super::spectrum::Spectrum),
    /// Operating-system media session display.
    Player(super::playback::Player),
    /// Current weather at a saved location.
    Weather(super::weather::Weather),
    /// Fixed text.
    Static(String),
    /// A sensor value, formatted, with optional text around it.
    Sensor {
        /// The sensor.
        key: SensorKey,
        /// Number formatting.
        format: DisplayFormat,
        /// Text before the value.
        prefix: String,
        /// Text after the value.
        suffix: String,
    },
    /// The current date/time rendered with a pattern (see [`super::clock`]).
    Clock {
        /// Pattern, e.g. `%H:%M`.
        pattern: String,
        /// None follows the runtime system language.
        language: Option<Language>,
        /// Letter case of the resulting text.
        casing: ClockCase,
        /// Drawn as a dial with hands instead of text
        /// (D-2026-10-20-studio-ux-v2-07); the pattern is kept for when it
        /// turns digital again.
        analog: Option<super::clock::AnalogClock>,
    },
}

/// Line ends of rings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Cap {
    /// Flat.
    #[default]
    Butt,
    /// Rounded.
    Round,
}

/// Splits a bar or ring into blocks.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Segments {
    /// Number of blocks.
    pub count: u16,
    /// Gap between blocks, pixels for bars, degrees for rings.
    pub gap: f32,
}

/// Direction a bar fills.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Direction {
    /// From the left edge.
    #[default]
    LeftToRight,
    /// From the right edge.
    RightToLeft,
    /// From the bottom edge.
    BottomToTop,
    /// From the top edge.
    TopToBottom,
}

/// How a graph draws its history.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GraphStyle {
    /// A line.
    #[default]
    Line,
    /// A line with the area under it filled.
    Area,
    /// One bar per sample.
    Bars,
}

/// Simple shapes.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ShapeKind {
    /// Rectangle with rounded corners (radius 0 = square corners).
    Rect {
        /// Corner radius, pixels.
        radius: f32,
    },
    /// Independent radii: top-left, top-right, bottom-right, bottom-left.
    Corners([f32; 4]),
    /// Ellipse inscribed in the box.
    Ellipse,
}

/// The different element kinds.
#[derive(Debug, Clone, PartialEq)]
pub enum ElementKind {
    /// Text: static, sensor value or clock.
    Text {
        /// What to print.
        content: TextContent,
        /// How it looks.
        style: TextStyle,
    },
    /// A still or animated (GIF) image.
    Image {
        /// Optional opacity mask, applied after fitting the image.
        fade: Option<super::gradient::Fade>,
        /// Image asset.
        asset: AssetRef,
        /// How it fills the box.
        fit: Fit,
        /// What stays in view when it covers the box.
        anchor: Anchor,
        /// How a GIF plays.
        playback: Playback,
    },
    /// A filled shape.
    Shape {
        /// Clear this shape to reveal the device video underneath.
        video_window: bool,
        /// Linear opacity across the whole shape, including its outline.
        fade: Option<super::gradient::Fade>,
        /// Geometry.
        shape: ShapeKind,
        /// Fill (none = outline only).
        fill: Option<Paint>,
        /// Picture inside the shape, over the fill and under the outline.
        image: Option<ShapeImage>,
        /// Outline color and width.
        stroke: Option<(Rgba, f32)>,
    },
    /// A progress bar.
    Bar {
        /// Value and range.
        binding: Binding,
        /// Fill direction.
        direction: Direction,
        /// Filled part.
        fill: Paint,
        /// Empty track (none = transparent).
        track: Option<Paint>,
        /// Corner radius.
        radius: f32,
        /// Optional blocks.
        segments: Option<Segments>,
    },
    /// A ring/arc gauge inscribed in the box.
    Ring {
        /// Temporarily show the full scale to inspect its appearance.
        test_full: bool,
        /// Value and range.
        binding: Binding,
        /// Start angle in degrees, 0 = 12 o'clock, clockwise positive.
        start_angle: f32,
        /// Total sweep in degrees (360 = full circle).
        sweep: f32,
        /// Ring thickness in pixels.
        thickness: f32,
        /// Fill clockwise from the start angle.
        clockwise: bool,
        /// Filled arc.
        fill: Paint,
        /// Track under it (none = transparent).
        track: Option<Paint>,
        /// Line ends.
        cap: Cap,
        /// Optional blocks.
        segments: Option<Segments>,
    },
    /// A rotating needle (image or drawn line) for analog gauges.
    Needle {
        /// Value and range.
        binding: Binding,
        /// Needle image pointing to 12 o'clock at rest; none = a drawn line.
        asset: Option<AssetRef>,
        /// Pivot inside the box, 0..=1 of width/height.
        pivot: (f32, f32),
        /// Angle at `min`, degrees, 0 = 12 o'clock, clockwise positive.
        start_angle: f32,
        /// Angle travelled from `min` to `max`.
        sweep: f32,
        /// Drawn-line color and width when there is no image.
        color: Rgba,
        /// Drawn-line width.
        width: f32,
        /// Arc, zones, scale, shape, cap, centre value and peak
        /// (D-2026-10-20-studio-ux-v2-08); `None` keeps the original line.
        look: Option<Box<super::needle::NeedleLook>>,
    },
    /// A history graph.
    Graph {
        /// Value and range.
        binding: Binding,
        /// Points drawn across the width: one per refresh of the theme, or
        /// one per `interval_seconds` when the graph has its own interval
        /// (the window is then `history` x `interval_seconds`).
        history: u16,
        /// Seconds between two points, whatever the theme's refresh
        /// (D-2026-10-15-studio-redesign-fidelity-16); `None`: one point
        /// per refresh, as themes without it always had.
        interval_seconds: Option<u32>,
        /// Drawing style.
        style: GraphStyle,
        /// Line and bar color.
        color: Rgba,
        /// Area fill.
        fill: Option<Paint>,
        /// Line width.
        line_width: f32,
        /// Scale to the visible samples instead of the binding range.
        autoscale: bool,
    },
}

impl ElementKind {
    /// The picture this element draws as a whole or inside it: an image
    /// element's file, or a shape's image. Empty placeholders count too.
    pub fn picture(&self) -> Option<&AssetRef> {
        match self {
            ElementKind::Image { asset, .. } => Some(asset),
            ElementKind::Shape {
                image: Some(image), ..
            } => Some(&image.asset),
            _ => None,
        }
    }
}

/// One thing drawn on the canvas.
#[derive(Debug, Clone, PartialEq)]
pub struct Element {
    /// Optional native preset; absent keeps the original widget appearance.
    pub visualization: Option<super::visualization::Visualization>,
    /// Optional sensor-driven visual rotation with inertia.
    pub sensor_motion: Option<super::sensor_motion::SensorMotion>,
    /// Clockwise degrees around the frame center, normalized to (-180, 180].
    pub rotation: f32,
    /// Invisible ordinary group container.
    pub is_group: bool,
    /// Ordinary group owning this object.
    pub group_parent: Option<ElementId>,
    /// Optional multi-face container configuration.
    pub card: Option<Card>,
    /// Container and face owning this object; none means an independent object.
    pub card_member: Option<CardMember>,
    /// Stable id.
    pub id: ElementId,
    /// Name shown in the layer list.
    pub name: String,
    /// Box in canvas pixels.
    pub frame: BoxF,
    /// 0..=1.
    pub opacity: f32,
    /// Hidden elements are neither drawn nor hit-tested.
    pub visible: bool,
    /// Locked elements cannot be moved in the editor.
    pub locked: bool,
    /// How a bar, ring, needle or sensor text moves to a new reading
    /// (D-2026-10-15-studio-redesign-fidelity-18); `None`: it jumps, as in
    /// themes written before it. Other elements ignore it.
    pub value_transition: Option<super::transition::ValueTransition>,
    /// Optional sensor-driven step colour; absent preserves the fixed paint.
    pub color_by_value: Option<super::value_color::ColorByValue>,
    /// What it is.
    pub kind: ElementKind,
}

/// A card base is drawn as a normal shape; face objects keep stable theme IDs.
#[derive(Debug, Clone, PartialEq)]
pub struct Card {
    /// State-driven face overrides.
    pub triggers: Vec<super::playback::CardTrigger>,
    /// Face names, in presentation order.
    pub faces: Vec<String>,
    /// Face shown by the renderer.
    pub active_face: usize,
    /// Legacy rotation input, normalized to a timed trigger at persistence/runtime boundaries.
    pub rotation_seconds: Option<u32>,
    /// The one transition used for every face change; absent means an immediate switch.
    pub transition: Option<CardTransition>,
}

impl Card {
    /// Read legacy rotation as a timed trigger, without losing existing app rules.
    pub fn normalized(&self) -> std::borrow::Cow<'_, Self> {
        let Some(seconds) = self.rotation_seconds else {
            return std::borrow::Cow::Borrowed(self);
        };
        let mut card = self.clone();
        card.rotation_seconds = None;
        card.triggers.push(super::playback::CardTrigger {
            source: super::playback::TriggerSource::Timer,
            interval_seconds: Some(seconds),
            next_face: true,
            app: String::new(),
            face: 0,
            priority: 0,
            return_seconds: 0,
            return_face: None,
        });
        std::borrow::Cow::Owned(card)
    }
}

/// Face transition style.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CardEffect {
    /// Immediate switch.
    None,
    /// Crossfade.
    Fade,
    /// Sliding faces.
    Slide,
    /// Turn around the center axis.
    Flip,
}
/// Direction of a slide or flip.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CardDirection {
    /// Left.
    Left,
    /// Right.
    Right,
    /// Up.
    Up,
    /// Down.
    Down,
}
/// Saved card animation settings (animation progress is never saved).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CardTransition {
    /// Effect.
    pub effect: CardEffect,
    /// Movement direction.
    pub direction: CardDirection,
    /// Duration in milliseconds, 150 to 3000.
    pub duration_ms: u32,
    /// Move the common base with the face.
    pub include_base: bool,
}

/// Membership in a card shared base or one face.
#[derive(Debug, Clone, PartialEq)]
pub struct CardMember {
    /// Card element ID.
    pub parent: ElementId,
    /// None belongs to the shared base, otherwise a face index.
    pub face: Option<usize>,
}

/// A complete theme.
#[derive(Debug, Clone, PartialEq)]
pub struct Theme {
    /// Display name.
    pub name: String,
    /// Canvas size = panel size in `orientation`.
    pub canvas: Size,
    /// How the user looks at the screen.
    pub orientation: Orientation,
    /// Under everything.
    pub background: Background,
    /// Bottom to top.
    pub elements: Vec<Element>,
    /// Seconds between refreshes (sensor sampling and frames).
    pub refresh_seconds: f32,
    /// Colours saved with the theme for reuse, in the user's order
    /// (D-2026-10-16-studio-redesign-fidelity-29). Picking one copies its
    /// colour: nothing drawn refers to a swatch, so changing the palette
    /// never repaints an element. At most [`PALETTE_MAX`].
    pub palette: Vec<Swatch>,
}

/// Most swatches a theme keeps.
pub const PALETTE_MAX: usize = 64;
/// Longest swatch name, in characters.
pub const SWATCH_NAME_MAX: usize = 40;

/// A colour of a theme's palette, optionally named ("Brand", "Alert").
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Swatch {
    /// A short name; `None` shows the colour's hex.
    pub name: Option<String>,
    /// The colour, with its opacity.
    pub color: Rgba,
}

/// Fastest refresh, in seconds: a theme (or a watch of the sensors) never
/// samples and draws more often than this.
pub const MIN_REFRESH_SECONDS: f32 = 0.25;

/// Time between two refreshes of a theme asking for `refresh_seconds`: at
/// least [`MIN_REFRESH_SECONDS`], at most `slowest_seconds` (the driving
/// adapter's limit), one second when it asks for no number.
pub fn refresh_interval(refresh_seconds: f32, slowest_seconds: f32) -> Duration {
    let seconds = if refresh_seconds.is_finite() {
        refresh_seconds
            .min(slowest_seconds)
            .max(MIN_REFRESH_SECONDS)
    } else {
        1.0
    };
    Duration::from_secs_f32(seconds)
}

impl Theme {
    /// An empty theme for a panel whose portrait size is `panel`.
    pub fn blank(name: &str, panel: Size, orientation: Orientation) -> Self {
        Self {
            name: name.to_string(),
            canvas: panel.portrait().in_orientation(orientation),
            orientation,
            background: Background::Color(Rgba::opaque(12, 14, 22)),
            elements: Vec::new(),
            refresh_seconds: 1.0,
            palette: Vec::new(),
        }
    }

    /// Whether the theme fits a panel whose portrait size is `panel`: `None`
    /// when its canvas is that panel turned the theme's way up, else that
    /// size (the canvas a theme for this panel would have).
    pub fn misfit(&self, panel: Size) -> Option<Size> {
        let canvas = panel.in_orientation(self.orientation);
        (canvas != self.canvas).then_some(canvas)
    }

    /// The element with `id`.
    pub fn element(&self, id: ElementId) -> Option<&Element> {
        self.elements.iter().find(|e| e.id == id)
    }

    /// Visibility includes group ancestors and the owning card's active face.
    pub fn is_visible(&self, element: &Element) -> bool {
        if element.group_parent.is_none() {
            if !element.visible {
                return false;
            }
            if let Some(member) = &element.card_member {
                if let Some(parent) = self.element(member.parent)
                    && parent.group_parent.is_none()
                {
                    return parent.visible
                        && parent
                            .card
                            .as_ref()
                            .is_some_and(|c| member.face.is_none_or(|f| f == c.active_face));
                }
            } else {
                return true;
            }
        }
        self.owners(element).is_some_and(|owners| {
            owners.iter().all(|e| {
                e.visible
                    && e.card_member.as_ref().is_none_or(|m| {
                        self.element(m.parent)
                            .and_then(|p| p.card.as_ref())
                            .is_some_and(|c| m.face.is_none_or(|f| f == c.active_face))
                    })
            })
        })
    }

    /// Visibility for drawing either face during a card transition.
    pub fn visible_without_face(&self, element: &Element) -> bool {
        if element.group_parent.is_none() {
            if let Some(m) = &element.card_member {
                if let Some(p) = self.element(m.parent)
                    && p.group_parent.is_none()
                {
                    return element.visible && p.visible;
                }
            } else {
                return element.visible;
            }
        }
        self.owners(element)
            .is_some_and(|owners| owners.iter().all(|e| e.visible))
    }

    /// Opacity inherited once from each distinct container.
    pub fn rendered_opacity(&self, element: &Element) -> f32 {
        if element.group_parent.is_none() {
            if let Some(m) = &element.card_member {
                if let Some(p) = self.element(m.parent)
                    && p.group_parent.is_none()
                {
                    return element.opacity * p.opacity;
                }
            } else {
                return element.opacity;
            }
        }
        self.owners(element)
            .map_or(0.0, |owners| owners.iter().map(|e| e.opacity).product())
    }

    /// Whether an object or any of its owning containers is locked.
    /// Broken ownership references are not editable.
    pub fn is_locked(&self, element: &Element) -> bool {
        self.owners(element)
            .is_none_or(|owners| owners.iter().any(|e| e.locked))
    }

    fn owners<'a>(&'a self, element: &'a Element) -> Option<Vec<&'a Element>> {
        let mut owners = vec![element];
        let mut index = 0;
        while index < owners.len() {
            let e = owners[index];
            for id in [e.group_parent, e.card_member.as_ref().map(|m| m.parent)]
                .into_iter()
                .flatten()
            {
                if !owners.iter().any(|p| p.id == id) {
                    owners.push(self.element(id)?);
                }
            }
            index += 1;
        }
        Some(owners)
    }

    /// Assigned assets the theme references (for packaging and loading).
    /// Empty editor placeholders retain their objects but reference no file.
    pub fn assets(&self) -> Vec<AssetRef> {
        let mut out = Vec::new();
        match &self.background {
            Background::Image { asset, .. } => out.push(asset.clone()),
            Background::Video { asset, poster, .. } => {
                out.push(asset.clone());
                out.extend(poster.clone());
            }
            Background::Color(_) | Background::DeviceVideo { .. } => {}
        }
        for e in &self.elements {
            match &e.kind {
                ElementKind::Image { asset, .. } => out.push(asset.clone()),
                ElementKind::Shape {
                    image: Some(image), ..
                } => out.push(image.asset.clone()),
                ElementKind::Needle { asset, look, .. } => {
                    out.extend(asset.clone());
                    out.extend(
                        look.as_ref()
                            .and_then(|l| l.center.as_ref())
                            .and_then(|c| c.style.font.asset.clone()),
                    );
                }
                ElementKind::Text { style, .. } => {
                    out.extend(style.font.asset.clone());
                    out.extend(style.parts.styles.values().filter_map(|p| p.asset.clone()));
                }
                _ => {}
            }
        }
        out.retain(|asset| !asset.0.is_empty());
        out.sort();
        out.dedup();
        out
    }

    fn is_sampled(&self, element: &Element) -> bool {
        self.is_visible(element)
            || (self.visible_without_face(element)
                && element.card_member.as_ref().is_some_and(|m| {
                    self.element(m.parent).is_some_and(|parent| {
                        parent.visible
                            && parent.card.as_ref().is_some_and(|card| {
                                !card.triggers.is_empty()
                                    || card.rotation_seconds.is_some()
                                    || card
                                        .transition
                                        .is_some_and(|t| t.effect != CardEffect::None)
                            })
                    })
                }))
    }

    /// The sensors the visible elements show: what running this theme
    /// wants measured (hidden elements are not drawn).
    pub fn sensor_keys(&self) -> BTreeSet<SensorKey> {
        let mut keys: BTreeSet<SensorKey> = self
            .elements
            .iter()
            .filter(|e| self.is_sampled(e))
            .filter_map(|e| e.bound_sensor().cloned())
            .collect();
        for e in self.elements.iter().filter(|e| self.is_sampled(e)) {
            if matches!(e.kind, ElementKind::Graph { .. }) {
                keys.extend(
                    e.visualization
                        .as_ref()
                        .and_then(|v| v.active_secondary())
                        .map(|b| b.key.clone()),
                );
            }
            keys.extend(e.sensor_motion.as_ref().map(|motion| motion.sensor.clone()));
            keys.extend(e.color_sensor().cloned());
            if let ElementKind::Text {
                content: TextContent::Weather(weather),
                ..
            } = &e.kind
            {
                keys.extend(weather.keys());
            }
        }
        for e in self.elements.iter().filter(|e| self.is_sampled(e)) {
            if matches!(
                e.kind,
                ElementKind::Text {
                    content: TextContent::Player(_) | TextContent::Spectrum(_),
                    ..
                }
            ) {
                keys.extend(SensorKey::new("media.sessions"));
            }
            if let Some(card) = &e.card {
                for rule in &card.triggers {
                    let key = match rule.source {
                        super::playback::TriggerSource::Timer => continue,
                        super::playback::TriggerSource::MediaPlaying => "media.sessions",
                        _ => "activity.processes",
                    };
                    keys.extend(SensorKey::new(key));
                }
            }
        }
        keys
    }

    /// The pictures a frame shows (as [`Theme::shown_images`]) with how
    /// each plays: an image element's own playback, the default for a
    /// shape's picture.
    pub fn shown_pictures(&self) -> impl Iterator<Item = (&AssetRef, Playback)> {
        let shown: Vec<&AssetRef> = self.shown_images().collect();
        self.elements.iter().filter_map(move |e| {
            let asset = e.kind.picture()?;
            shown.iter().any(|s| std::ptr::eq(*s, asset)).then(|| {
                let playback = match &e.kind {
                    ElementKind::Image { playback, .. } => *playback,
                    _ => Playback::default(),
                };
                (asset, playback)
            })
        })
    }

    /// The pictures of the image elements and shape images a frame shows:
    /// visible, not fully transparent, with a box that reaches into the
    /// canvas. An animated one among them sets the pace of the frames
    /// ([`super::animation`]); hidden ones do not animate.
    pub fn shown_images(&self) -> impl Iterator<Item = &AssetRef> {
        let (width, height) = (self.canvas.width as f32, self.canvas.height as f32);
        self.elements.iter().filter_map(move |e| {
            let asset = e.kind.picture()?;
            let b = e.frame;
            let on_canvas = b.width > 0.0
                && b.height > 0.0
                && b.x < width
                && b.y < height
                && b.x + b.width > 0.0
                && b.y + b.height > 0.0;
            (self.is_visible(e) && e.opacity > 0.0 && on_canvas).then_some(asset)
        })
    }

    /// The history series the theme's graphs draw: their sensor at their
    /// cadence ([`SeriesId::graph`]).
    pub fn graph_series(&self) -> Vec<SeriesId> {
        self.elements
            .iter()
            .flat_map(|e| match &e.kind {
                ElementKind::Graph {
                    binding,
                    interval_seconds,
                    ..
                } => {
                    let mut series = vec![SeriesId::graph(&binding.key, *interval_seconds)];
                    if let Some(b) = e.visualization.as_ref().and_then(|v| v.active_secondary()) {
                        series.push(SeriesId::graph(&b.key, *interval_seconds));
                    }
                    series
                }
                _ => Vec::new(),
            })
            .collect()
    }

    /// The graph history length wanted for each bound sensor.
    pub fn history_lengths(&self) -> Vec<(SensorKey, usize)> {
        self.elements
            .iter()
            .flat_map(|e| match &e.kind {
                ElementKind::Graph {
                    binding, history, ..
                } => {
                    let mut lengths = vec![(binding.key.clone(), usize::from(*history))];
                    if let Some(b) = e.visualization.as_ref().and_then(|v| v.active_secondary()) {
                        lengths.push((b.key.clone(), usize::from(*history)));
                    }
                    lengths
                }
                _ => Vec::new(),
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::sensor::keys;

    fn key(k: &str) -> SensorKey {
        SensorKey::new(k).expect("valid key")
    }

    fn element(id: u32, kind: ElementKind) -> Element {
        Element {
            color_by_value: None,
            visualization: None,
            value_transition: None,
            sensor_motion: None,
            card: None,
            rotation: 0.0,
            is_group: false,
            group_parent: None,
            card_member: None,
            id: ElementId(id),
            name: format!("e{id}"),
            frame: BoxF::new(0.0, 0.0, 100.0, 40.0),
            opacity: 1.0,
            visible: true,
            locked: false,
            kind,
        }
    }

    fn binding(k: &str) -> Binding {
        Binding {
            key: key(k),
            min: 0.0,
            max: 100.0,
        }
    }

    #[test]
    fn blank_theme_takes_the_orientation() {
        let t = Theme::blank("x", Size::new(480, 1920), Orientation::Landscape);
        assert_eq!(t.canvas, Size::new(1920, 480));
    }

    #[test]
    fn keys_assets_and_histories_are_collected() {
        let mut t = Theme::blank("x", Size::new(480, 1920), Orientation::Portrait);
        t.background = Background::Video {
            asset: AssetRef("assets/bg.mp4".into()),
            poster: Some(AssetRef("assets/bg.png".into())),
            framing: None,
        };
        let mut style = TextStyle::default();
        style.font.asset = Some(AssetRef("assets/font.ttf".into()));
        t.elements = vec![
            element(
                1,
                ElementKind::Text {
                    content: TextContent::Sensor {
                        key: key(keys::CPU_TEMPERATURE),
                        format: DisplayFormat::default(),
                        prefix: String::new(),
                        suffix: String::new(),
                    },
                    style,
                },
            ),
            element(
                2,
                ElementKind::Graph {
                    binding: binding(keys::CPU_USAGE),
                    history: 60,
                    interval_seconds: None,
                    style: GraphStyle::Area,
                    color: Rgba::WHITE,
                    fill: None,
                    line_width: 2.0,
                    autoscale: false,
                },
            ),
            element(
                3,
                ElementKind::Image {
                    fade: None,
                    asset: AssetRef("assets/logo.png".into()),
                    fit: Fit::Contain,
                    anchor: Default::default(),
                    playback: Default::default(),
                },
            ),
            element(
                4,
                ElementKind::Needle {
                    binding: binding(keys::CPU_USAGE),
                    asset: Some(AssetRef("assets/needle.png".into())),
                    pivot: (0.5, 0.9),
                    start_angle: -120.0,
                    sweep: 240.0,
                    color: Rgba::WHITE,
                    width: 3.0,
                    look: None,
                },
            ),
        ];
        let mut hidden = element(
            5,
            ElementKind::Bar {
                binding: binding(keys::GPU_USAGE),
                direction: Direction::LeftToRight,
                fill: Paint::solid(Rgba::WHITE),
                track: None,
                radius: 4.0,
                segments: None,
            },
        );
        hidden.visible = false;
        t.elements.push(hidden);

        let assets: Vec<String> = t.assets().into_iter().map(|a| a.0).collect();
        assert_eq!(
            assets,
            vec![
                "assets/bg.mp4",
                "assets/bg.png",
                "assets/font.ttf",
                "assets/logo.png",
                "assets/needle.png"
            ]
        );
        assert_eq!(t.history_lengths(), vec![(key(keys::CPU_USAGE), 60)]);
        assert_eq!(
            t.graph_series(),
            vec![SeriesId::graph(&key(keys::CPU_USAGE), None)]
        );
        assert_eq!(
            t.sensor_keys(),
            BTreeSet::from([key(keys::CPU_TEMPERATURE), key(keys::CPU_USAGE)]),
            "the hidden bar's sensor is not wanted"
        );
        assert!(t.element(ElementId(3)).is_some());
    }

    #[test]
    fn a_shape_image_is_a_theme_asset_and_a_shown_picture() {
        let mut t = Theme::blank("x", Size::new(480, 1920), Orientation::Portrait);
        let shape = |id, asset: &str| {
            let mut e = element(
                id,
                ElementKind::Shape {
                    video_window: false,
                    fade: None,
                    shape: ShapeKind::Rect { radius: 8.0 },
                    fill: Some(Paint::solid(Rgba::WHITE)),
                    image: Some(ShapeImage::new(AssetRef(asset.into()))),
                    stroke: None,
                },
            );
            e.frame = BoxF::new(0.0, 0.0, 100.0, 100.0);
            e
        };
        t.elements = vec![shape(1, "assets/tile.svg"), shape(2, "")];
        assert_eq!(t.assets(), vec![AssetRef("assets/tile.svg".into())]);
        let shown: Vec<_> = t.shown_images().map(|a| a.0.as_str()).collect();
        assert_eq!(shown, ["assets/tile.svg", ""]);
        assert_eq!(
            t.elements[0].kind.picture(),
            Some(&AssetRef("assets/tile.svg".into()))
        );
        assert!(ImageRepeat::Tile.natural_size() && !ImageRepeat::Cover.natural_size());
    }

    #[test]
    fn only_images_a_frame_shows_are_shown_images() {
        let mut t = Theme::blank("x", Size::new(480, 1920), Orientation::Portrait);
        let image = |id, name: &str, frame| Element {
            frame,
            ..element(
                id,
                ElementKind::Image {
                    fade: None,
                    asset: AssetRef(name.into()),
                    fit: Fit::Fill,
                    anchor: Default::default(),
                    playback: Default::default(),
                },
            )
        };
        let mut hidden = image(2, "hidden.gif", BoxF::new(0.0, 0.0, 10.0, 10.0));
        hidden.visible = false;
        let mut clear = image(3, "clear.gif", BoxF::new(0.0, 0.0, 10.0, 10.0));
        clear.opacity = 0.0;
        t.elements = vec![
            image(1, "shown.gif", BoxF::new(470.0, 1910.0, 64.0, 64.0)),
            hidden,
            clear,
            image(4, "outside.gif", BoxF::new(480.0, 0.0, 10.0, 10.0)),
            image(5, "above.gif", BoxF::new(0.0, -10.0, 10.0, 10.0)),
            image(6, "empty.gif", BoxF::new(5.0, 5.0, 0.0, 10.0)),
            element(
                7,
                ElementKind::Text {
                    content: TextContent::Static("x".into()),
                    style: TextStyle::default(),
                },
            ),
        ];
        let shown: Vec<&str> = t.shown_images().map(|a| a.0.as_str()).collect();
        assert_eq!(shown, ["shown.gif"]);
    }

    #[test]
    fn parts_with_a_style_and_their_fonts() {
        let mut parts = TextParts::default();
        assert!(parts.is_empty());
        parts.styles.insert(TextPart::Number, PartStyle::default());
        assert!(parts.is_empty(), "an empty style changes nothing");
        assert_eq!(parts.style(TextPart::Number), None);
        let unit = PartStyle {
            size_percent: Some(55.0),
            asset: Some(AssetRef("assets/unit.ttf".into())),
            ..PartStyle::default()
        };
        parts.styles.insert(TextPart::Unit, unit.clone());
        assert_eq!(parts.style(TextPart::Unit), Some(&unit));
        assert!((unit.size(64.0) - 35.2).abs() < 1e-4);
        assert_eq!(PartStyle::default().size(64.0), 64.0);
        let huge = PartStyle {
            size_percent: Some(9_000.0),
            ..PartStyle::default()
        };
        assert_eq!(huge.size(10.0), 40.0, "at most 400 %");
        let blinking = TextParts {
            blink_separators: true,
            ..TextParts::default()
        };
        assert!(!blinking.is_empty());
        assert!(TextStyle::default().tabular_digits, "on by default");

        let mut t = Theme::blank("x", Size::new(480, 1920), Orientation::Portrait);
        let style = TextStyle {
            parts,
            ..TextStyle::default()
        };
        t.elements = vec![element(
            1,
            ElementKind::Text {
                content: TextContent::Static("1".into()),
                style,
            },
        )];
        assert_eq!(t.assets(), vec![AssetRef("assets/unit.ttf".into())]);
    }

    #[test]
    fn anchors_and_playback() {
        assert_eq!(Anchor::default().shares(), (0.5, 0.5));
        assert_eq!(Anchor::TopLeft.shares(), (0.0, 0.0));
        assert_eq!(Anchor::Right.shares(), (1.0, 0.5));
        assert_eq!(Anchor::BottomRight.shares(), (1.0, 1.0));
        let ms = Duration::from_millis;
        let total = ms(1_000);
        let looping = Playback::default();
        assert_eq!(looping.moment(ms(2_500), total), ms(2_500));
        let fast = Playback {
            speed: 2.0,
            ..looping
        };
        assert_eq!(fast.moment(ms(300), total), ms(600));
        let once = Playback {
            once: true,
            ..looping
        };
        assert_eq!(once.moment(ms(400), total), ms(400));
        assert_eq!(
            once.moment(ms(5_000), total),
            total - Duration::from_nanos(1)
        );
        let step = |at: Duration| ms((at.as_millis() as u64 / 100 + 1) * 100);
        assert_eq!(looping.next_change(ms(250), total, step), Some(ms(300)));
        assert_eq!(
            fast.next_change(ms(250), total, step),
            Some(ms(300)),
            "500 → 600 at 2×"
        );
        let slow = Playback {
            speed: 0.5,
            ..looping
        };
        assert_eq!(slow.next_change(ms(250), total, step), Some(ms(400)));
        assert_eq!(
            once.next_change(ms(950), total, step),
            None,
            "the last frame stays"
        );
        assert_eq!(once.next_change(ms(1_500), total, step), None);
        assert_eq!(once.next_change(ms(850), total, step), Some(ms(900)));
        let wild = Playback {
            speed: 99.0,
            ..looping
        };
        assert_eq!(wild.moment(ms(100), total), ms(400), "at most 4×");
    }

    #[test]
    fn shown_pictures_carry_their_playback() {
        let mut t = Theme::blank("x", Size::new(480, 1920), Orientation::Portrait);
        let once = Playback {
            once: true,
            speed: 2.0,
        };
        t.elements = vec![
            element(
                1,
                ElementKind::Image {
                    fade: None,
                    asset: AssetRef("a.gif".into()),
                    fit: Fit::Fill,
                    anchor: Anchor::Center,
                    playback: once,
                },
            ),
            element(
                2,
                ElementKind::Shape {
                    video_window: false,
                    fade: None,
                    shape: ShapeKind::Ellipse,
                    fill: None,
                    image: Some(ShapeImage::new(AssetRef("b.gif".into()))),
                    stroke: None,
                },
            ),
        ];
        let shown: Vec<_> = t.shown_pictures().map(|(a, p)| (a.0.as_str(), p)).collect();
        assert_eq!(shown, [("a.gif", once), ("b.gif", Playback::default())]);
    }

    #[test]
    fn box_math() {
        let b = BoxF::new(15.0, 15.0, 100.0, 50.0);
        assert_eq!(b.center(), (65.0, 40.0));
        let bg = Theme::blank("x", Size::new(80, 160), Orientation::Portrait);
        assert!(bg.assets().is_empty());
    }

    #[test]
    fn a_theme_fits_its_panel_turned_its_way_up() {
        let panel = Size::new(480, 1920);
        let landscape = Theme::blank("x", panel, Orientation::Landscape);
        assert_eq!(landscape.misfit(panel), None);
        let upside_down = Theme::blank("x", panel, Orientation::ReversePortrait);
        assert_eq!(upside_down.misfit(panel), None);
        assert_eq!(
            landscape.misfit(Size::new(320, 480)),
            Some(Size::new(480, 320))
        );
        let small = Theme::blank("x", Size::new(320, 480), Orientation::Portrait);
        assert_eq!(small.misfit(panel), Some(panel));
    }

    #[test]
    fn refreshes_stay_between_the_fastest_and_the_slowest() {
        assert_eq!(refresh_interval(1.0, 60.0), Duration::from_secs(1));
        assert_eq!(refresh_interval(0.01, 60.0), Duration::from_millis(250));
        assert_eq!(refresh_interval(3600.0, 60.0), Duration::from_secs(60));
        assert_eq!(refresh_interval(5.0, 2.0), Duration::from_secs(2));
        assert_eq!(refresh_interval(f32::NAN, 2.0), Duration::from_secs(1));
        assert_eq!(
            refresh_interval(f32::INFINITY, 60.0),
            Duration::from_secs(1)
        );
        assert_eq!(refresh_interval(1.0, 0.1), Duration::from_millis(250));
    }
}
