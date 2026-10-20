//! `theme.json`, schema 1: serde shapes and their mapping to the core model.

use bezel_core::domain::clock::{
    AnalogClock, ClockCase, DialDate, DialNumbers, DialTicks, HandShape, Language,
};
use bezel_core::domain::frame::Rgba;
use bezel_core::domain::framing::{FramingPosition, Permille, VideoFit, VideoFraming, Zoom};
use bezel_core::domain::geometry::{Orientation, Size};
use bezel_core::domain::history::GRAPH_INTERVAL_MAX_SECONDS;
use bezel_core::domain::sensor::{ByteUnits, DisplayFormat, SensorKey, TemperatureUnit};
use bezel_core::domain::theme::{
    AssetRef, Background, Baseline, Binding, BoxF, Cap, Direction, Element, ElementId, ElementKind,
    Fit, FontSpec, GraphStyle, HAlign, ImageRepeat, PALETTE_MAX, Paint, PartStyle, SWATCH_NAME_MAX,
    Segments, ShapeImage, ShapeKind, Swatch, TextContent, TextOverflow, TextPart, TextParts,
    TextStyle, Theme, VAlign,
};
use bezel_core::domain::transition::{
    MAX_AVERAGE, MAX_TRANSITION_MS, MIN_TRANSITION_MS, TransitionMode, ValueTransition,
};
use bezel_core::domain::value_color::{ColorByValue, ColorThreshold};
use serde::{Deserialize, Serialize};

use crate::color::{from_hex, to_hex};

/// The schema this build writes and the newest it reads.
pub const SCHEMA: u32 = 1;

/// Why a `theme.json` could not become a theme.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DtoError(pub String);

type R<T> = std::result::Result<T, DtoError>;

fn err<T>(msg: impl Into<String>) -> R<T> {
    Err(DtoError(msg.into()))
}

fn color(s: &str) -> R<Rgba> {
    from_hex(s).ok_or_else(|| DtoError(format!("bad color {s:?}")))
}

fn key(s: &str) -> R<SensorKey> {
    SensorKey::new(s).ok_or_else(|| DtoError(format!("bad sensor key {s:?}")))
}

/// Root object.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThemeDto {
    /// Schema version.
    pub schema: u32,
    /// Display name.
    pub name: String,
    /// Canvas size.
    pub canvas: SizeDto,
    /// `portrait`, `reverse-portrait`, `landscape`, `reverse-landscape`.
    pub orientation: String,
    /// Seconds between refreshes.
    #[serde(default = "one")]
    pub refresh_seconds: f32,
    /// Background.
    pub background: BackgroundDto,
    /// Elements, bottom to top.
    #[serde(default)]
    pub elements: Vec<ElementDto>,
    /// The theme's saved colours (D-2026-10-16-studio-redesign-fidelity-29);
    /// written only when there is one, so older themes save unchanged.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub palette: Vec<SwatchDto>,
}

/// A saved colour: `{"name": "Brand", "color": "#rrggbbaa"}`, the name optional.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SwatchDto {
    /// A short name (left out when none).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// `#rrggbbaa`.
    pub color: String,
}

fn swatch(d: &SwatchDto) -> R<Swatch> {
    let name = d.name.as_deref().map(str::trim).filter(|n| !n.is_empty());
    if name.is_some_and(|n| n.chars().count() > SWATCH_NAME_MAX) {
        return err(format!(
            "swatch name longer than {SWATCH_NAME_MAX} characters"
        ));
    }
    Ok(Swatch {
        name: name.map(str::to_string),
        color: color(&d.color)?,
    })
}

fn palette(list: &[SwatchDto]) -> R<Vec<Swatch>> {
    if list.len() > PALETTE_MAX {
        return err(format!("a palette holds at most {PALETTE_MAX} swatches"));
    }
    list.iter().map(swatch).collect()
}

fn is_zero_f32(value: &f32) -> bool {
    *value == 0.0
}

fn one() -> f32 {
    1.0
}

fn yes() -> bool {
    true
}

fn one_f64() -> f64 {
    1.0
}

fn half() -> f64 {
    0.5
}

fn black() -> String {
    to_hex(Rgba::BLACK)
}

/// Width and height.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SizeDto {
    /// Width.
    pub width: u32,
    /// Height.
    pub height: u32,
}

/// Background.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum BackgroundDto {
    /// Video already stored on the screen.
    #[serde(rename_all = "camelCase")]
    DeviceVideo {
        /// Exact internal/SD video path.
        path: String,
        /// Replay at the end.
        #[serde(default = "yes")]
        looping: bool,
        /// Color outside video windows.
        #[serde(default = "black")]
        color: String,
    },
    /// A color.
    Color {
        /// `#rrggbbaa`.
        color: String,
    },
    /// An image.
    Image {
        /// Asset path.
        asset: String,
        /// Fit.
        #[serde(default)]
        fit: FitDto,
    },
    /// A video.
    Video {
        /// Asset path.
        asset: String,
        /// Poster asset.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        poster: Option<String>,
        /// How the video is framed; omitted when it is the default, so a
        /// theme without a framing is written as before.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        framing: Option<FramingDto>,
    },
}

/// A video background's framing (D-2026-10-01-video-background-framing-2).
/// A missing key takes its default; numbers out of range are clamped.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FramingDto {
    /// Clockwise degrees: 0, 90, 180 or 270; absent is Auto. Any number is
    /// read, so another one is refused by name rather than by type.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rotation: Option<serde_json::Number>,
    /// Fill or fit.
    #[serde(default)]
    pub fit: VideoFitDto,
    /// Zoom on top of the fit's scale, 1.0 to 4.0.
    #[serde(default = "one_f64")]
    pub zoom: f64,
    /// Where the picture sits, 0..=1 on each axis (CSS `object-position`).
    #[serde(default)]
    pub position: PositionDto,
    /// `#rrggbbaa` of the area a fitted picture leaves.
    #[serde(default = "black")]
    pub pad_color: String,
}

impl FramingDto {
    /// Clockwise quarter turns of its `rotation` (`None`: Auto). Only 0,
    /// 90, 180 and 270 degrees are rotations; another number is refused,
    /// for every reader of a theme (its file, the studio's commands).
    pub fn quarter_turns(&self) -> Result<Option<u8>, DtoError> {
        self.rotation.as_ref().map(quarter_turns).transpose()
    }
}

/// Video fit names.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum VideoFitDto {
    /// Fill: covers the canvas, the overflow cut off.
    #[default]
    Cover,
    /// Fit: inside the canvas, the rest painted with the pad color.
    Contain,
}

/// A place on each axis, 0 = start, 0.5 = middle, 1 = end.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PositionDto {
    /// Across.
    #[serde(default = "half")]
    pub x: f64,
    /// Down.
    #[serde(default = "half")]
    pub y: f64,
}

impl Default for PositionDto {
    fn default() -> Self {
        Self { x: 0.5, y: 0.5 }
    }
}

/// Fit names.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum FitDto {
    /// Stretch.
    #[default]
    Fill,
    /// Inside.
    Contain,
    /// Cover.
    Cover,
    /// Original size.
    None,
}

/// How a shape's image covers it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ImageRepeatDto {
    /// Cover, cropping.
    #[default]
    Cover,
    /// Inside.
    Contain,
    /// Stretch to the box.
    Stretch,
    /// One copy, centered.
    Center,
    /// Side by side both ways.
    Tile,
    /// Along one row.
    TileX,
    /// Along one column.
    TileY,
}

/// A picture inside a shape. Only `asset` is required.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShapeImageDto {
    /// Asset path inside the theme (empty: none chosen yet).
    pub asset: String,
    /// How it covers the shape.
    #[serde(default)]
    pub repeat: ImageRepeatDto,
    /// Size of one copy relative to the picture (0.05 to 10).
    #[serde(default = "one")]
    pub scale: f32,
    /// Horizontal shift in pixels.
    #[serde(default, skip_serializing_if = "is_zero_f32")]
    pub offset_x: f32,
    /// Vertical shift in pixels.
    #[serde(default, skip_serializing_if = "is_zero_f32")]
    pub offset_y: f32,
    /// Opacity of the picture, 0 to 1.
    #[serde(default = "one")]
    pub opacity: f32,
}

/// A paint: a color string or a gradient object.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PaintDto {
    /// `#rrggbbaa`.
    Solid(String),
    /// Gradient along a ring scale.
    Arc {
        /// Must be `arc`.
        scale: String,
        /// Color at zero.
        start: String,
        /// Color at full scale.
        end: String,
        /// Position of the half-color blend, 0..=100.
        transition: f32,
    },
    /// A linear gradient.
    Linear {
        /// Angle, degrees.
        angle: f32,
        /// `[position, color]` pairs.
        stops: Vec<(f32, String)>,
    },
}

/// A linear transparency mask in a shape's box.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FadeDto {
    /// Radial center X/Y and radius in normalized box coordinates.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub radial: Option<[f32; 3]>,
    /// Direction in degrees.
    pub angle: f32,
    /// Start opacity, 0..=1.
    pub start: f32,
    /// End opacity, 0..=1.
    pub end: f32,
    /// Where opacity is halfway; written only when moved from the middle.
    #[serde(default = "fade_midpoint", skip_serializing_if = "is_fade_midpoint")]
    pub midpoint: f32,
}
fn fade_midpoint() -> f32 {
    bezel_core::domain::gradient::Fade::MIDPOINT
}
fn is_fade_midpoint(v: &f32) -> bool {
    *v == bezel_core::domain::gradient::Fade::MIDPOINT
}

/// One element.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ElementDto {
    /// Optional native widget geometry; absent preserves older themes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub visualization: Option<VisualizationDto>,
    /// Optional sensor-driven visual rotation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sensor_motion: Option<SensorMotionDto>,
    /// Clockwise degrees; omitted for unchanged themes.
    #[serde(default, skip_serializing_if = "is_zero_f32")]
    pub rotation: f32,
    /// Ordinary group container; omitted in older themes.
    #[serde(default, skip_serializing_if = "is_false")]
    pub is_group: bool,
    /// Ordinary group owning this object.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub group_parent: Option<u32>,
    /// Optional card container, absent in existing themes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub card: Option<CardDto>,
    /// Optional membership of a shared base or face.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub card_member: Option<CardMemberDto>,
    /// Id.
    pub id: u32,
    /// Name.
    pub name: String,
    /// Box.
    pub frame: BoxDto,
    /// Opacity 0..=1.
    #[serde(default = "one")]
    pub opacity: f32,
    /// Visible.
    #[serde(default = "yes")]
    pub visible: bool,
    /// Locked in the editor.
    #[serde(default)]
    pub locked: bool,
    /// How a bar, ring, needle or sensor value moves to a new reading
    /// (D-2026-10-15-studio-redesign-fidelity-18); absent: it jumps, as in
    /// themes written before it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value_transition: Option<ValueTransitionDto>,
    /// Optional step colours; absent keeps the original fixed paint.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color_by_value: Option<ColorByValueDto>,
    /// Kind.
    pub kind: KindDto,
}

/// Portable optional visualization settings, with bounded defaults.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct VisualizationDto {
    /// matrix, pixel, dial, rounded or mirrored.
    pub style: String,
    /// Matrix columns.
    pub columns: u16,
    /// Matrix rows.
    pub rows: u16,
    /// Cell/bar spacing in pixels.
    pub gap: f32,
    /// Cell corners or marker radius.
    pub radius: f32,
    /// Dial tick count.
    pub ticks: u16,
    /// Show endpoint marker.
    pub marker: bool,
    /// Independent lower history source.
    pub secondary: Option<BindingDto>,
    /// Lower history color.
    pub secondary_color: String,
}
impl Default for VisualizationDto {
    fn default() -> Self {
        Self {
            style: "matrix".into(),
            columns: 16,
            rows: 6,
            gap: 3.,
            radius: 2.,
            ticks: 31,
            marker: true,
            secondary: None,
            secondary_color: "#e879a4ff".into(),
        }
    }
}
fn visualization(d: &VisualizationDto) -> R<bezel_core::domain::visualization::Visualization> {
    use bezel_core::domain::visualization::{VisualStyle, Visualization};
    let style = match d.style.as_str() {
        "matrix" => VisualStyle::Matrix,
        "pixel" => VisualStyle::Pixel,
        "dial" => VisualStyle::Dial,
        "rounded" => VisualStyle::Rounded,
        "mirrored" => VisualStyle::Mirrored,
        _ => return err(format!("unknown visualization {:?}", d.style)),
    };
    let v = Visualization {
        style,
        columns: d.columns,
        rows: d.rows,
        gap: d.gap,
        radius: d.radius,
        ticks: d.ticks,
        marker: d.marker,
        secondary: d.secondary.as_ref().map(binding).transpose()?,
        secondary_color: color(&d.secondary_color)?,
    };
    if !v.valid() {
        return err("visualization settings out of range");
    }
    Ok(v)
}
fn visualization_dto(v: &bezel_core::domain::visualization::Visualization) -> VisualizationDto {
    use bezel_core::domain::visualization::VisualStyle;
    VisualizationDto {
        style: match v.style {
            VisualStyle::Matrix => "matrix",
            VisualStyle::Pixel => "pixel",
            VisualStyle::Dial => "dial",
            VisualStyle::Rounded => "rounded",
            VisualStyle::Mirrored => "mirrored",
        }
        .into(),
        columns: v.columns,
        rows: v.rows,
        gap: v.gap,
        radius: v.radius,
        ticks: v.ticks,
        marker: v.marker,
        secondary: v.secondary.as_ref().map(binding_dto),
        secondary_color: to_hex(v.secondary_color),
    }
}

/// Sensor-driven visual rotation; speed is deliberately distinct from fan RPM.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SensorMotionDto {
    /// Source key.
    pub sensor: String,
    /// Source reading at maximum visual speed.
    pub reference: f64,
    /// Maximum visual revolutions per second.
    pub turns_per_second: f64,
    /// Response time in milliseconds.
    pub inertia_ms: u32,
    /// Counterclockwise when true.
    pub reverse: bool,
}

fn sensor_motion(
    d: &Option<SensorMotionDto>,
) -> R<Option<bezel_core::domain::sensor_motion::SensorMotion>> {
    d.as_ref()
        .map(|d| {
            let motion = bezel_core::domain::sensor_motion::SensorMotion {
                sensor: SensorKey::new(&d.sensor)
                    .ok_or_else(|| DtoError("invalid motion sensor".into()))?,
                reference: d.reference,
                turns_per_second: d.turns_per_second,
                inertia_ms: d.inertia_ms,
                reverse: d.reverse,
            };
            if !motion.is_valid() {
                return err("invalid sensor motion settings");
            }
            Ok(motion)
        })
        .transpose()
}

/// Sensor-driven step colours, in the sensor's native units.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ColorByValueDto {
    /// Used only by otherwise unbound objects.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sensor: Option<String>,
    /// Colour below all thresholds and for unavailable readings.
    pub base: String,
    /// Inclusive lower bounds, in presentation order; at most 32.
    pub thresholds: Vec<ColorThresholdDto>,
}

/// One inclusive lower bound and its colour.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ColorThresholdDto {
    /// Native sensor value.
    pub value: f64,
    /// RGBA hex colour.
    pub color: String,
}

fn color_by_value_dto(rule: &ColorByValue) -> ColorByValueDto {
    ColorByValueDto {
        sensor: rule.sensor.as_ref().map(|s| s.as_str().to_owned()),
        base: to_hex(rule.base),
        thresholds: rule
            .thresholds
            .iter()
            .map(|t| ColorThresholdDto {
                value: t.value,
                color: to_hex(t.color),
            })
            .collect(),
    }
}
fn color_by_value(rule: &ColorByValueDto) -> R<ColorByValue> {
    if rule.thresholds.len() > 32 || rule.thresholds.iter().any(|t| !t.value.is_finite()) {
        return err("colorByValue needs at most 32 finite thresholds");
    }
    Ok(ColorByValue {
        sensor: rule
            .sensor
            .as_ref()
            .map(|s| {
                SensorKey::new(s).ok_or_else(|| DtoError("invalid colorByValue sensor".into()))
            })
            .transpose()?,
        base: color(&rule.base)?,
        thresholds: rule
            .thresholds
            .iter()
            .map(|t| {
                Ok(ColorThreshold {
                    value: t.value,
                    color: color(&t.color)?,
                })
            })
            .collect::<R<_>>()?,
    })
}

/// An element's value transition: `{"mode": "smooth", "durationMs": 400}`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ValueTransitionDto {
    /// Optional pursuit response; old readers fall back to ordinary smooth motion.
    #[serde(default, skip_serializing_if = "is_false")]
    pub inertia: bool,
    /// `none` or `smooth`.
    pub mode: TransitionModeDto,
    /// Milliseconds a smooth transition lasts (read within
    /// [`MIN_TRANSITION_MS`]..=[`MAX_TRANSITION_MS`]; 0 jumps).
    #[serde(default)]
    pub duration_ms: u32,
    /// Readings averaged (1..=30); 1, each reading as it is, is omitted.
    #[serde(default = "one_u8", skip_serializing_if = "is_one_u8")]
    pub average: u8,
}

fn one_u8() -> u8 {
    1
}

fn is_one_u8(v: &u8) -> bool {
    *v == 1
}

/// `none` or `smooth`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TransitionModeDto {
    /// Jumps.
    None,
    /// Eases.
    Smooth,
    /// Retains momentum while pursuing new readings.
    Inertia,
    /// Counts evenly through the values in between.
    Count,
    /// Turns each digit like a wheel.
    Roll,
}

fn value_transition_dto(t: Option<ValueTransition>) -> Option<ValueTransitionDto> {
    t.map(|t| ValueTransitionDto {
        mode: match t.mode {
            TransitionMode::None => TransitionModeDto::None,
            TransitionMode::Smooth => TransitionModeDto::Smooth,
            TransitionMode::Inertia => TransitionModeDto::Smooth,
            TransitionMode::Count => TransitionModeDto::Count,
            TransitionMode::Roll => TransitionModeDto::Roll,
        },
        duration_ms: t.duration_ms,
        inertia: t.mode == TransitionMode::Inertia,
        average: t.average,
    })
}

fn value_transition(t: Option<ValueTransitionDto>) -> Option<ValueTransition> {
    t.map(|t| ValueTransition {
        mode: match t.mode {
            TransitionModeDto::None => TransitionMode::None,
            TransitionModeDto::Smooth if t.inertia => TransitionMode::Inertia,
            TransitionModeDto::Smooth => TransitionMode::Smooth,
            TransitionModeDto::Inertia => TransitionMode::Inertia,
            TransitionModeDto::Count => TransitionMode::Count,
            TransitionModeDto::Roll => TransitionMode::Roll,
        },
        duration_ms: match t.duration_ms {
            0 => 0,
            ms => ms.clamp(MIN_TRANSITION_MS, MAX_TRANSITION_MS),
        },
        average: t.average.clamp(1, MAX_AVERAGE),
    })
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CardDto {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub triggers: Vec<CardTriggerDto>,
    pub faces: Vec<String>,
    #[serde(default)]
    pub active_face: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rotation_seconds: Option<u32>,
    /// The one transition for every face change. A `pairTransitions` list written by
    /// development builds of 2026-10-15 is not a field here, so it is ignored on load
    /// and never written back (D-24).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transition: Option<CardTransitionDto>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CardTriggerDto {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub interval_seconds: Option<u32>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub next_face: bool,
    pub source: String,
    #[serde(default)]
    pub app: String,
    #[serde(default)]
    pub face: usize,
    #[serde(default)]
    pub priority: u32,
    #[serde(default)]
    pub return_seconds: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub return_face: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CardTransitionDto {
    pub effect: String,
    pub direction: String,
    pub duration_ms: u32,
    #[serde(default)]
    pub include_base: bool,
}

fn card_transition_dto(t: bezel_core::domain::theme::CardTransition) -> CardTransitionDto {
    CardTransitionDto {
        effect: format!("{:?}", t.effect).to_lowercase(),
        direction: format!("{:?}", t.direction).to_lowercase(),
        duration_ms: t.duration_ms,
        include_base: t.include_base,
    }
}
fn card_transition(t: &CardTransitionDto) -> R<bezel_core::domain::theme::CardTransition> {
    use bezel_core::domain::theme::{CardDirection, CardEffect, CardTransition};
    let effect = match t.effect.as_str() {
        "none" => CardEffect::None,
        "fade" => CardEffect::Fade,
        "slide" => CardEffect::Slide,
        "flip" => CardEffect::Flip,
        _ => return err("unknown card transition effect"),
    };
    let direction = match t.direction.as_str() {
        "left" => CardDirection::Left,
        "right" => CardDirection::Right,
        "up" => CardDirection::Up,
        "down" => CardDirection::Down,
        _ => return err("unknown card transition direction"),
    };
    if !(150..=3000).contains(&t.duration_ms) {
        return err("card transition duration must be 150 to 3000 ms");
    }
    Ok(CardTransition {
        effect,
        direction,
        duration_ms: t.duration_ms,
        include_base: t.include_base,
    })
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CardMemberDto {
    pub parent: u32,
    pub face: Option<usize>,
}

/// A box.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct BoxDto {
    /// Left.
    pub x: f32,
    /// Top.
    pub y: f32,
    /// Width.
    pub width: f32,
    /// Height.
    pub height: f32,
}

/// Sensor binding.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BindingDto {
    /// Sensor key.
    pub key: String,
    /// Empty value.
    pub min: f64,
    /// Full value.
    pub max: f64,
}

/// Blocks.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SegmentsDto {
    /// Number of blocks.
    pub count: u16,
    /// Gap.
    pub gap: f32,
}

/// Number formatting.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FormatDto {
    /// Decimal places (none = automatic).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub decimals: Option<u8>,
    /// Append the unit.
    #[serde(default = "yes")]
    pub show_unit: bool,
    /// Show °F instead of °C.
    #[serde(default)]
    pub fahrenheit: bool,
    /// Use 1000-based byte units.
    #[serde(default)]
    pub decimal_bytes: bool,
}

fn is_true(v: &bool) -> bool {
    *v
}

fn is_default<T: Default + PartialEq>(v: &T) -> bool {
    *v == T::default()
}

/// The times a player prints.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TimeFormatDto {
    /// `1:24 / 3:30`.
    #[default]
    ElapsedTotal,
    /// `1:24 / -2:06`.
    ElapsedRemaining,
    /// `1:24`.
    Elapsed,
    /// `-2:06`.
    Remaining,
    /// `3:30`.
    Total,
}

/// Where a player's times sit while its bar is shown.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TimePlacementDto {
    /// A line of the text.
    #[default]
    Text,
    /// Left and right of the bar.
    Sides,
    /// Above the bar.
    Above,
}

/// How a player's bar is cut.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ProgressPatternDto {
    /// One bar.
    #[default]
    Continuous,
    /// Blocks.
    Segments,
    /// Dots.
    Dots,
}

/// The ends of a player's bar.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ProgressEndsDto {
    /// Rounded.
    #[default]
    Round,
    /// Square.
    Square,
}

fn default_track() -> String {
    to_hex(bezel_core::domain::playback::DEFAULT_PROGRESS_TRACK)
}

fn default_thickness() -> f32 {
    bezel_core::domain::playback::DEFAULT_PROGRESS_THICKNESS
}

fn default_segments() -> u16 {
    bezel_core::domain::playback::DEFAULT_PROGRESS_SEGMENTS
}

/// A player's progress bar; every field may be left out.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProgressStyleDto {
    /// Filled part; absent uses the text colour.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fill: Option<String>,
    /// Empty track.
    #[serde(default = "default_track")]
    pub track: String,
    /// Thickness, pixels.
    #[serde(default = "default_thickness")]
    pub thickness: f32,
    /// `round` or `square`.
    #[serde(default)]
    pub ends: ProgressEndsDto,
    /// A knob at the position.
    #[serde(default)]
    pub knob: bool,
    /// `continuous`, `segments` or `dots`.
    #[serde(default)]
    pub pattern: ProgressPatternDto,
    /// Blocks of `segments`.
    #[serde(default = "default_segments")]
    pub segments: u16,
    /// `text`, `sides` or `above`.
    #[serde(default)]
    pub times: TimePlacementDto,
}

fn progress_style_dto(s: &bezel_core::domain::playback::ProgressStyle) -> Option<ProgressStyleDto> {
    use bezel_core::domain::playback::{ProgressEnds, ProgressPattern, TimePlacement};
    (*s != Default::default()).then(|| ProgressStyleDto {
        fill: s.fill.map(to_hex),
        track: to_hex(s.track),
        thickness: s.thickness,
        ends: match s.ends {
            ProgressEnds::Round => ProgressEndsDto::Round,
            ProgressEnds::Square => ProgressEndsDto::Square,
        },
        knob: s.knob,
        pattern: match s.pattern {
            ProgressPattern::Continuous => ProgressPatternDto::Continuous,
            ProgressPattern::Segments => ProgressPatternDto::Segments,
            ProgressPattern::Dots => ProgressPatternDto::Dots,
        },
        segments: s.segments,
        times: match s.times {
            TimePlacement::Text => TimePlacementDto::Text,
            TimePlacement::Sides => TimePlacementDto::Sides,
            TimePlacement::Above => TimePlacementDto::Above,
        },
    })
}

fn progress_style(s: Option<&ProgressStyleDto>) -> R<bezel_core::domain::playback::ProgressStyle> {
    use bezel_core::domain::playback::{
        PROGRESS_SEGMENTS, PROGRESS_THICKNESS, ProgressEnds, ProgressPattern, ProgressStyle,
        TimePlacement,
    };
    let Some(s) = s else {
        return Ok(ProgressStyle::default());
    };
    let (thin, thick) = PROGRESS_THICKNESS;
    if !(thin..=thick).contains(&s.thickness) {
        return err(format!("progress thickness {} out of range", s.thickness));
    }
    if !(PROGRESS_SEGMENTS.0..=PROGRESS_SEGMENTS.1).contains(&s.segments) {
        return err(format!("progress segments {} out of range", s.segments));
    }
    Ok(ProgressStyle {
        fill: s.fill.as_deref().map(color).transpose()?,
        track: color(&s.track)?,
        thickness: s.thickness,
        ends: match s.ends {
            ProgressEndsDto::Round => ProgressEnds::Round,
            ProgressEndsDto::Square => ProgressEnds::Square,
        },
        knob: s.knob,
        pattern: match s.pattern {
            ProgressPatternDto::Continuous => ProgressPattern::Continuous,
            ProgressPatternDto::Segments => ProgressPattern::Segments,
            ProgressPatternDto::Dots => ProgressPattern::Dots,
        },
        segments: s.segments,
        times: match s.times {
            TimePlacementDto::Text => TimePlacement::Text,
            TimePlacementDto::Sides => TimePlacement::Sides,
            TimePlacementDto::Above => TimePlacement::Above,
        },
    })
}

fn time_format_dto(f: bezel_core::domain::playback::TimeFormat) -> TimeFormatDto {
    use bezel_core::domain::playback::TimeFormat;
    match f {
        TimeFormat::ElapsedTotal => TimeFormatDto::ElapsedTotal,
        TimeFormat::ElapsedRemaining => TimeFormatDto::ElapsedRemaining,
        TimeFormat::Elapsed => TimeFormatDto::Elapsed,
        TimeFormat::Remaining => TimeFormatDto::Remaining,
        TimeFormat::Total => TimeFormatDto::Total,
    }
}

fn time_format(f: TimeFormatDto) -> bezel_core::domain::playback::TimeFormat {
    use bezel_core::domain::playback::TimeFormat;
    match f {
        TimeFormatDto::ElapsedTotal => TimeFormat::ElapsedTotal,
        TimeFormatDto::ElapsedRemaining => TimeFormat::ElapsedRemaining,
        TimeFormatDto::Elapsed => TimeFormat::Elapsed,
        TimeFormatDto::Remaining => TimeFormat::Remaining,
        TimeFormatDto::Total => TimeFormat::Total,
    }
}

/// Optional spectrum configuration; defaults are applied only to this new kind.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct SpectrumDto {
    pub player: Option<u32>,
    pub bands: u16,
    pub low: f32,
    pub high: f32,
    pub motion: String,
    pub peaks: bool,
    pub demo: bool,
    pub thickness: f32,
    pub gap: f32,
    pub corners: [f32; 4],
    pub centre: bool,
    // D-AS-11 fields stay out of the file at their defaults, so a mirrored
    // spectrum still opens in versions that predate the other styles.
    #[serde(skip_serializing_if = "is_mirrored")]
    pub style: String,
    #[serde(skip_serializing_if = "is_default_block")]
    pub block: f32,
    #[serde(skip_serializing_if = "is_default_stroke")]
    pub stroke: f32,
    #[serde(skip_serializing_if = "is_default_fill")]
    pub fill: bool,
    #[serde(skip_serializing_if = "is_default_inner")]
    pub inner: f32,
    #[serde(skip_serializing_if = "is_default_gain")]
    pub gain: f32,
    #[serde(skip_serializing_if = "is_player_audio")]
    pub audio: String,
}
fn is_player_audio(audio: &str) -> bool {
    audio == "player"
}
fn is_mirrored(style: &str) -> bool {
    style == "mirrored"
}
fn spectrum_default() -> bezel_core::domain::spectrum::Spectrum {
    bezel_core::domain::spectrum::Spectrum::default()
}
fn is_default_block(v: &f32) -> bool {
    *v == spectrum_default().block
}
fn is_default_stroke(v: &f32) -> bool {
    *v == spectrum_default().stroke
}
fn is_default_fill(v: &bool) -> bool {
    *v == spectrum_default().fill
}
fn is_default_inner(v: &f32) -> bool {
    *v == spectrum_default().inner
}
fn is_default_gain(v: &f32) -> bool {
    *v == spectrum_default().gain
}
impl Default for SpectrumDto {
    fn default() -> Self {
        Self::from(&bezel_core::domain::spectrum::Spectrum::default())
    }
}
impl From<&bezel_core::domain::spectrum::Spectrum> for SpectrumDto {
    fn from(s: &bezel_core::domain::spectrum::Spectrum) -> Self {
        use bezel_core::domain::spectrum::{Motion, SpectrumAudio, SpectrumStyle};
        Self {
            player: s.player.map(|p| p.0),
            bands: s.bands,
            low: s.low,
            high: s.high,
            motion: match s.motion {
                Motion::Smooth => "smooth",
                Motion::Balanced => "balanced",
                Motion::Snappy => "snappy",
            }
            .into(),
            peaks: s.peaks,
            demo: s.demo,
            thickness: s.thickness,
            gap: s.gap,
            corners: s.corners,
            centre: s.centre,
            style: match s.style {
                SpectrumStyle::Mirrored => "mirrored",
                SpectrumStyle::Bars => "bars",
                SpectrumStyle::Blocks => "blocks",
                SpectrumStyle::Line => "line",
                SpectrumStyle::Radial => "radial",
            }
            .into(),
            block: s.block,
            stroke: s.stroke,
            fill: s.fill,
            inner: s.inner,
            gain: s.gain,
            audio: match s.audio {
                SpectrumAudio::Player => "player",
                SpectrumAudio::System => "system",
            }
            .into(),
        }
    }
}
impl SpectrumDto {
    fn domain(&self) -> R<bezel_core::domain::spectrum::Spectrum> {
        use bezel_core::domain::spectrum::{Motion, Spectrum, SpectrumAudio, SpectrumStyle};
        let motion = match self.motion.as_str() {
            "smooth" => Motion::Smooth,
            "balanced" => Motion::Balanced,
            "snappy" => Motion::Snappy,
            _ => return err("unknown spectrum motion"),
        };
        let style = match self.style.as_str() {
            "mirrored" => SpectrumStyle::Mirrored,
            "bars" => SpectrumStyle::Bars,
            "blocks" => SpectrumStyle::Blocks,
            "line" => SpectrumStyle::Line,
            "radial" => SpectrumStyle::Radial,
            _ => return err("unknown spectrum style"),
        };
        let audio = match self.audio.as_str() {
            "player" => SpectrumAudio::Player,
            "system" => SpectrumAudio::System,
            _ => return err("unknown spectrum audio"),
        };
        let s = Spectrum {
            player: self.player.map(ElementId),
            bands: self.bands,
            low: self.low,
            high: self.high,
            motion,
            peaks: self.peaks,
            demo: self.demo,
            thickness: self.thickness,
            gap: self.gap,
            corners: self.corners,
            centre: self.centre,
            style,
            block: self.block,
            stroke: self.stroke,
            fill: self.fill,
            inner: self.inner,
            gain: self.gain,
            audio,
        };
        if !s.valid() {
            return err("invalid spectrum bounds");
        }
        Ok(s)
    }
}

/// Text content.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum ContentDto {
    Spectrum {
        #[serde(flatten)]
        spectrum: SpectrumDto,
    },
    Player {
        #[serde(default)]
        source: String,
        #[serde(default = "yes", rename = "showCover")]
        show_cover: bool,
        #[serde(default, rename = "coverRadius")]
        cover_radius: f32,
        #[serde(
            default,
            rename = "coverCorners",
            skip_serializing_if = "Option::is_none"
        )]
        cover_corners: Option<[f32; 4]>,
        #[serde(default, rename = "coverGap", skip_serializing_if = "Option::is_none")]
        cover_gap: Option<f32>,
        #[serde(default = "yes", rename = "showProgress")]
        show_progress: bool,
        #[serde(default, rename = "showSource")]
        show_source: bool,
        #[serde(default, rename = "hideWhenStopped")]
        hide_when_stopped: bool,
        #[serde(default, rename = "emptyText")]
        empty_text: String,
        #[serde(default = "yes", rename = "showTitle", skip_serializing_if = "is_true")]
        show_title: bool,
        #[serde(
            default = "yes",
            rename = "showArtist",
            skip_serializing_if = "is_true"
        )]
        show_artist: bool,
        /// Absent follows `showProgress` (themes saved before D-26).
        #[serde(default, rename = "showTime", skip_serializing_if = "Option::is_none")]
        show_time: Option<bool>,
        #[serde(default, rename = "timeFormat", skip_serializing_if = "is_default")]
        time_format: TimeFormatDto,
        /// Absent draws the bar as before it had options.
        #[serde(
            default,
            rename = "progressStyle",
            skip_serializing_if = "Option::is_none"
        )]
        progress_style: Option<ProgressStyleDto>,
    },
    /// Current weather at a saved location.
    Weather {
        /// Displayed city.
        city: String,
        /// Latitude.
        latitude: f64,
        /// Longitude.
        longitude: f64,
        /// Condition language; absent follows the system.
        #[serde(default)]
        language: Option<String>,
        /// Fahrenheit instead of Celsius.
        #[serde(default)]
        fahrenheit: bool,
        /// Show the vector condition icon.
        #[serde(default = "yes", rename = "showIcon")]
        show_icon: bool,
        /// Outline (default) or filled condition icon.
        #[serde(default = "outline_icon", rename = "iconStyle")]
        icon_style: String,
        /// Distance from icon to text, pixels; absent is automatic.
        #[serde(default, rename = "iconGap", skip_serializing_if = "Option::is_none")]
        icon_gap: Option<f32>,
        /// Icon side, pixels; absent follows text size.
        #[serde(default, rename = "iconSize", skip_serializing_if = "Option::is_none")]
        icon_size: Option<f32>,
    },
    /// Fixed text.
    Static {
        /// The text.
        text: String,
    },
    /// Sensor value.
    Sensor {
        /// Key.
        key: String,
        /// Formatting.
        format: FormatDto,
        /// Before the value.
        #[serde(default)]
        prefix: String,
        /// After the value.
        #[serde(default)]
        suffix: String,
    },
    /// Clock.
    Clock {
        /// Pattern.
        pattern: String,
        /// Explicit clock language; absent follows the system.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        language: Option<String>,
        /// Normal (absent), upper or title.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        casing: Option<String>,
        /// A dial with hands instead of text; absent is digital.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        analog: Option<Box<AnalogDto>>,
    },
}

/// An analog clock (D-2026-10-20-studio-ux-v2-07); every absent field is
/// its default, so a default dial is `{}`.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalogDto {
    /// `none`, `hours` (absent) or `minutes`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ticks: Option<String>,
    /// `none` (absent), `quarters`, `all` or `roman`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub numbers: Option<String>,
    /// The hour hand shows (absent: yes).
    #[serde(default = "yes", skip_serializing_if = "is_true")]
    pub hour_hand: bool,
    /// The minute hand shows (absent: yes).
    #[serde(default = "yes", skip_serializing_if = "is_true")]
    pub minute_hand: bool,
    /// The second hand shows (absent: yes).
    #[serde(default = "yes", skip_serializing_if = "is_true")]
    pub second_hand: bool,
    /// `thin`, `rounded` (absent) or `arrow`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hand_shape: Option<String>,
    /// Outline width, px (absent: 2).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub border: Option<f32>,
    /// Dial colour.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub face: Option<String>,
    /// Marks, numbers, date and outline colour.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub marks: Option<String>,
    /// Hour hand colour.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hour_color: Option<String>,
    /// Minute hand colour.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub minute_color: Option<String>,
    /// Second hand colour.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub second_color: Option<String>,
    /// The second hand sweeps.
    #[serde(default, skip_serializing_if = "is_false")]
    pub sweep: bool,
    /// `none` (absent), `day` or `dayMonth`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub date: Option<String>,
}

fn analog_dto(a: &AnalogClock) -> Box<AnalogDto> {
    let d = AnalogClock::default();
    let differs = |v: Rgba, default: Rgba| (v != default).then(|| to_hex(v));
    let name = |changed: bool, n: &str| changed.then(|| n.to_string());
    Box::new(AnalogDto {
        ticks: name(
            a.ticks != d.ticks,
            match a.ticks {
                DialTicks::None => "none",
                DialTicks::Hours => "hours",
                DialTicks::Minutes => "minutes",
            },
        ),
        numbers: name(
            a.numbers != d.numbers,
            match a.numbers {
                DialNumbers::None => "none",
                DialNumbers::Quarters => "quarters",
                DialNumbers::All => "all",
                DialNumbers::Roman => "roman",
            },
        ),
        hour_hand: a.hour_hand,
        minute_hand: a.minute_hand,
        second_hand: a.second_hand,
        hand_shape: name(
            a.hand_shape != d.hand_shape,
            match a.hand_shape {
                HandShape::Thin => "thin",
                HandShape::Rounded => "rounded",
                HandShape::Arrow => "arrow",
            },
        ),
        border: (a.border != d.border).then_some(a.border),
        face: differs(a.face, d.face),
        marks: differs(a.marks, d.marks),
        hour_color: differs(a.hour_color, d.hour_color),
        minute_color: differs(a.minute_color, d.minute_color),
        second_color: differs(a.second_color, d.second_color),
        sweep: a.sweep,
        date: name(
            a.date != d.date,
            match a.date {
                DialDate::None => "none",
                DialDate::Day => "day",
                DialDate::DayMonth => "dayMonth",
            },
        ),
    })
}

fn analog(dto: &AnalogDto) -> R<AnalogClock> {
    let d = AnalogClock::default();
    let paint = |v: &Option<String>, default: Rgba| v.as_deref().map_or(Ok(default), color);
    Ok(AnalogClock {
        ticks: match dto.ticks.as_deref() {
            None => d.ticks,
            Some("none") => DialTicks::None,
            Some("hours") => DialTicks::Hours,
            Some("minutes") => DialTicks::Minutes,
            Some(other) => return err(format!("unknown dial ticks {other:?}")),
        },
        numbers: match dto.numbers.as_deref() {
            None => d.numbers,
            Some("none") => DialNumbers::None,
            Some("quarters") => DialNumbers::Quarters,
            Some("all") => DialNumbers::All,
            Some("roman") => DialNumbers::Roman,
            Some(other) => return err(format!("unknown dial numbers {other:?}")),
        },
        hour_hand: dto.hour_hand,
        minute_hand: dto.minute_hand,
        second_hand: dto.second_hand,
        hand_shape: match dto.hand_shape.as_deref() {
            None => d.hand_shape,
            Some("thin") => HandShape::Thin,
            Some("rounded") => HandShape::Rounded,
            Some("arrow") => HandShape::Arrow,
            Some(other) => return err(format!("unknown hand shape {other:?}")),
        },
        border: dto
            .border
            .filter(|b| b.is_finite())
            .map_or(d.border, |b| b.clamp(0.0, 64.0)),
        face: paint(&dto.face, d.face)?,
        marks: paint(&dto.marks, d.marks)?,
        hour_color: paint(&dto.hour_color, d.hour_color)?,
        minute_color: paint(&dto.minute_color, d.minute_color)?,
        second_color: paint(&dto.second_color, d.second_color)?,
        sweep: dto.sweep,
        date: match dto.date.as_deref() {
            None => d.date,
            Some("none") => DialDate::None,
            Some("day") => DialDate::Day,
            Some("dayMonth") => DialDate::DayMonth,
            Some(other) => return err(format!("unknown dial date {other:?}")),
        },
    })
}

/// Font face.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FontDto {
    /// Family.
    pub family: String,
    /// Weight.
    pub weight: u16,
    /// Italic.
    #[serde(default)]
    pub italic: bool,
    /// Bundled font asset.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub asset: Option<String>,
}

/// Text style.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextStyleDto {
    /// Non-destructive case; absent preserves the source.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub casing: Option<String>,
    /// Face.
    pub font: FontDto,
    /// Size, px.
    pub size: f32,
    /// Fill.
    pub paint: PaintDto,
    /// `left`, `center`, `right`.
    pub align: String,
    /// `top`, `middle`, `bottom`.
    pub valign: String,
    /// Extra letter spacing, px.
    #[serde(default)]
    pub letter_spacing: f32,
    /// Digits of equal width (D-2026-10-20-studio-ux-v2-05); absent is on.
    #[serde(default = "yes", skip_serializing_if = "is_true")]
    pub tabular_digits: bool,
    /// `shrink`, `ellipsis` or `scroll`; absent draws past the box.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub overflow: Option<String>,
    /// Parts with a style of their own (D-2026-10-20-studio-ux-v2-06).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parts: Option<Box<PartsDto>>,
}

/// The parts of a value or a digital clock with a style of their own;
/// absent parts follow the text.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PartsDto {
    /// Value: text before the number.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prefix: Option<PartDto>,
    /// Value: the number.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub number: Option<PartDto>,
    /// Value: the unit.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unit: Option<PartDto>,
    /// Value: text after the unit.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub suffix: Option<PartDto>,
    /// Clock: hours.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hours: Option<PartDto>,
    /// Clock: minutes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub minutes: Option<PartDto>,
    /// Clock: seconds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub seconds: Option<PartDto>,
    /// Clock: separators (with `blink`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub separators: Option<PartDto>,
}

impl PartsDto {
    fn slots(&self) -> [(TextPart, &Option<PartDto>); 8] {
        [
            (TextPart::Prefix, &self.prefix),
            (TextPart::Number, &self.number),
            (TextPart::Unit, &self.unit),
            (TextPart::Suffix, &self.suffix),
            (TextPart::Hours, &self.hours),
            (TextPart::Minutes, &self.minutes),
            (TextPart::Seconds, &self.seconds),
            (TextPart::Separators, &self.separators),
        ]
    }

    fn slot(&mut self, part: TextPart) -> &mut Option<PartDto> {
        match part {
            TextPart::Prefix => &mut self.prefix,
            TextPart::Number => &mut self.number,
            TextPart::Unit => &mut self.unit,
            TextPart::Suffix => &mut self.suffix,
            TextPart::Hours => &mut self.hours,
            TextPart::Minutes => &mut self.minutes,
            TextPart::Seconds => &mut self.seconds,
            TextPart::Separators => &mut self.separators,
        }
    }
}

/// One part's style; every absent field follows the text.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PartDto {
    /// Font family.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub font: Option<String>,
    /// Bundled font file of that family.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub font_asset: Option<String>,
    /// Size in percent of the text's size.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size_percent: Option<f32>,
    /// Weight, 100..=900.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub weight: Option<u16>,
    /// `#rrggbbaa`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    /// `base` (absent) or `super`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub baseline: Option<String>,
    /// Separators only: blink every second.
    #[serde(default, skip_serializing_if = "is_false")]
    pub blink: bool,
}

fn parts_dto(parts: &TextParts) -> Option<Box<PartsDto>> {
    let mut dto = PartsDto::default();
    for (part, style) in &parts.styles {
        if style.is_plain() {
            continue;
        }
        *dto.slot(*part) = Some(PartDto {
            font: style.family.clone(),
            font_asset: style.asset.as_ref().map(|a| a.0.clone()),
            size_percent: style.size_percent,
            weight: style.weight,
            color: style.color.map(to_hex),
            baseline: (style.baseline == Baseline::Super).then(|| "super".to_string()),
            blink: false,
        });
    }
    if parts.blink_separators {
        dto.separators.get_or_insert_with(PartDto::default).blink = true;
    }
    (dto != PartsDto::default()).then(|| Box::new(dto))
}

fn parts(dto: Option<&PartsDto>) -> R<TextParts> {
    let mut out = TextParts::default();
    let Some(dto) = dto else {
        return Ok(out);
    };
    for (part, slot) in dto.slots() {
        let Some(p) = slot else { continue };
        let style = PartStyle {
            family: p.font.clone().filter(|f| !f.is_empty()),
            asset: p.font_asset.clone().filter(|a| !a.is_empty()).map(AssetRef),
            size_percent: p.size_percent.filter(|v| v.is_finite()).map(|v| {
                v.clamp(
                    *PartStyle::SIZE_PERCENT.start(),
                    *PartStyle::SIZE_PERCENT.end(),
                )
            }),
            weight: p.weight.map(|w| w.clamp(100, 900)),
            color: p.color.as_deref().map(color).transpose()?,
            baseline: match p.baseline.as_deref() {
                None | Some("base") => Baseline::Base,
                Some("super") => Baseline::Super,
                Some(other) => return err(format!("unknown part baseline {other:?}")),
            },
        };
        if !style.is_plain() {
            out.styles.insert(part, style);
        }
        if part == TextPart::Separators && p.blink {
            out.blink_separators = true;
        }
    }
    Ok(out)
}

/// Element kinds.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum KindDto {
    /// Text.
    Text {
        /// Content.
        content: ContentDto,
        /// Style.
        style: TextStyleDto,
    },
    /// Image.
    Image {
        /// Optional image opacity gradient.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        fade: Option<FadeDto>,
        /// Asset.
        asset: String,
        /// Fit.
        #[serde(default)]
        fit: FitDto,
    },
    /// Shape.
    #[serde(rename_all = "camelCase")]
    Shape {
        /// Reveal the device video in this shape.
        #[serde(default, rename = "videoWindow")]
        video_window: bool,
        /// Linear transparency, including the outline.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        fade: Option<FadeDto>,
        /// `rect` or `ellipse`.
        shape: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        corners: Option<[f32; 4]>,
        /// Corner radius for rects.
        #[serde(default)]
        radius: f32,
        /// Fill.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        fill: Option<PaintDto>,
        /// Picture inside the shape, over the fill.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        image: Option<ShapeImageDto>,
        /// Outline color.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        stroke: Option<String>,
        /// Outline width.
        #[serde(default)]
        stroke_width: f32,
    },
    /// Progress bar.
    Bar {
        /// Binding.
        binding: BindingDto,
        /// `leftToRight`, `rightToLeft`, `bottomToTop`, `topToBottom`.
        direction: String,
        /// Filled part.
        fill: PaintDto,
        /// Track.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        track: Option<PaintDto>,
        /// Corner radius.
        #[serde(default)]
        radius: f32,
        /// Blocks.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        segments: Option<SegmentsDto>,
    },
    /// Ring gauge.
    #[serde(rename_all = "camelCase")]
    Ring {
        /// Show the complete ring for appearance testing.
        #[serde(default)]
        test_full: bool,
        /// Binding.
        binding: BindingDto,
        /// Start angle.
        start_angle: f32,
        /// Sweep.
        sweep: f32,
        /// Thickness.
        thickness: f32,
        /// Clockwise.
        #[serde(default = "yes")]
        clockwise: bool,
        /// Filled arc.
        fill: PaintDto,
        /// Track.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        track: Option<PaintDto>,
        /// `butt` or `round`.
        #[serde(default)]
        round_caps: bool,
        /// Blocks.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        segments: Option<SegmentsDto>,
    },
    /// Needle.
    #[serde(rename_all = "camelCase")]
    Needle {
        /// Binding.
        binding: BindingDto,
        /// Needle image.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        asset: Option<String>,
        /// Pivot, 0..=1.
        pivot: (f32, f32),
        /// Angle at min.
        start_angle: f32,
        /// Sweep.
        sweep: f32,
        /// Drawn line color.
        color: String,
        /// Drawn line width.
        width: f32,
    },
    /// Graph.
    #[serde(rename_all = "camelCase")]
    Graph {
        /// Binding.
        binding: BindingDto,
        /// Points across the width.
        history: u16,
        /// Seconds between two points (D-2026-10-15-studio-redesign-fidelity-16);
        /// absent: one point per refresh, as in themes written before it.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        interval_seconds: Option<u32>,
        /// `line`, `area`, `bars`.
        style: String,
        /// Line color.
        color: String,
        /// Area fill.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        fill: Option<PaintDto>,
        /// Line width.
        line_width: f32,
        /// Autoscale.
        #[serde(default)]
        autoscale: bool,
    },
}

fn orientation_name(o: Orientation) -> &'static str {
    match o {
        Orientation::Portrait => "portrait",
        Orientation::ReversePortrait => "reverse-portrait",
        Orientation::Landscape => "landscape",
        Orientation::ReverseLandscape => "reverse-landscape",
    }
}

fn orientation(s: &str) -> R<Orientation> {
    Ok(match s {
        "portrait" => Orientation::Portrait,
        "reverse-portrait" => Orientation::ReversePortrait,
        "landscape" => Orientation::Landscape,
        "reverse-landscape" => Orientation::ReverseLandscape,
        other => return err(format!("unknown orientation {other:?}")),
    })
}

fn fit_dto(f: Fit) -> FitDto {
    match f {
        Fit::Fill => FitDto::Fill,
        Fit::Contain => FitDto::Contain,
        Fit::Cover => FitDto::Cover,
        Fit::None => FitDto::None,
    }
}

fn shape_image_dto(i: &ShapeImage) -> ShapeImageDto {
    ShapeImageDto {
        asset: i.asset.0.clone(),
        repeat: match i.repeat {
            ImageRepeat::Cover => ImageRepeatDto::Cover,
            ImageRepeat::Contain => ImageRepeatDto::Contain,
            ImageRepeat::Stretch => ImageRepeatDto::Stretch,
            ImageRepeat::Center => ImageRepeatDto::Center,
            ImageRepeat::Tile => ImageRepeatDto::Tile,
            ImageRepeat::TileX => ImageRepeatDto::TileX,
            ImageRepeat::TileY => ImageRepeatDto::TileY,
        },
        scale: i.scale,
        offset_x: i.offset.0,
        offset_y: i.offset.1,
        opacity: i.opacity,
    }
}

fn shape_image(d: &ShapeImageDto) -> R<ShapeImage> {
    if !ShapeImage::SCALE.contains(&d.scale) {
        return err(format!(
            "shape image scale {} is outside 0.05 to 10",
            d.scale
        ));
    }
    if !(0.0..=1.0).contains(&d.opacity) {
        return err(format!(
            "shape image opacity {} is outside 0 to 1",
            d.opacity
        ));
    }
    if !(d.offset_x.is_finite() && d.offset_y.is_finite()) {
        return err("shape image offset must be finite");
    }
    Ok(ShapeImage {
        asset: AssetRef(d.asset.clone()),
        repeat: match d.repeat {
            ImageRepeatDto::Cover => ImageRepeat::Cover,
            ImageRepeatDto::Contain => ImageRepeat::Contain,
            ImageRepeatDto::Stretch => ImageRepeat::Stretch,
            ImageRepeatDto::Center => ImageRepeat::Center,
            ImageRepeatDto::Tile => ImageRepeat::Tile,
            ImageRepeatDto::TileX => ImageRepeat::TileX,
            ImageRepeatDto::TileY => ImageRepeat::TileY,
        },
        scale: d.scale,
        offset: (d.offset_x, d.offset_y),
        opacity: d.opacity,
    })
}

fn fit(f: FitDto) -> Fit {
    match f {
        FitDto::Fill => Fit::Fill,
        FitDto::Contain => Fit::Contain,
        FitDto::Cover => Fit::Cover,
        FitDto::None => Fit::None,
    }
}

/// The framing as written: `None` (no `framing` key) when it is the default.
fn framing_dto(f: Option<&VideoFraming>) -> Option<FramingDto> {
    let f = f.filter(|f| !f.is_default())?;
    Some(FramingDto {
        rotation: f
            .rotation
            .map(|turns| serde_json::Number::from(u16::from(turns % 4) * 90)),
        fit: match f.fit {
            VideoFit::Cover => VideoFitDto::Cover,
            VideoFit::Contain => VideoFitDto::Contain,
        },
        zoom: f.zoom.factor(),
        position: PositionDto {
            x: f.position.x.fraction(),
            y: f.position.y.fraction(),
        },
        pad_color: to_hex(f.pad),
    })
}

/// Clockwise quarter turns of a framing's `rotation` in degrees.
fn quarter_turns(degrees: &serde_json::Number) -> R<u8> {
    [(0.0, 0), (90.0, 1), (180.0, 2), (270.0, 3)]
        .into_iter()
        .find(|(whole, _)| degrees.as_f64() == Some(*whole))
        .map(|(_, turns)| turns)
        .ok_or_else(|| {
            DtoError(format!(
                "video framing rotation {degrees} is not 0, 90, 180 or 270"
            ))
        })
}

/// The framing read: `None` when it is the default. Numbers are clamped; a
/// rotation other than 0, 90, 180 or 270 is refused.
fn framing(d: Option<&FramingDto>) -> R<Option<VideoFraming>> {
    let Some(d) = d else {
        return Ok(None);
    };
    let framing = VideoFraming {
        rotation: d.quarter_turns()?,
        fit: match d.fit {
            VideoFitDto::Cover => VideoFit::Cover,
            VideoFitDto::Contain => VideoFit::Contain,
        },
        zoom: Zoom::from_factor(d.zoom),
        position: FramingPosition {
            x: Permille::from_fraction(d.position.x),
            y: Permille::from_fraction(d.position.y),
        },
        pad: color(&d.pad_color)
            .map_err(|e| DtoError(format!("video framing padColor: {}", e.0)))?,
    };
    Ok((!framing.is_default()).then_some(framing))
}

fn paint_dto(p: &Paint) -> PaintDto {
    match p {
        Paint::Solid(c) => PaintDto::Solid(to_hex(*c)),
        Paint::Arc {
            start,
            end,
            transition,
        } => PaintDto::Arc {
            scale: "arc".into(),
            start: to_hex(*start),
            end: to_hex(*end),
            transition: transition * 100.0,
        },
        Paint::Linear { angle, stops } => PaintDto::Linear {
            angle: *angle,
            stops: stops.iter().map(|(pos, c)| (*pos, to_hex(*c))).collect(),
        },
    }
}

fn paint(p: &PaintDto) -> R<Paint> {
    Ok(match p {
        PaintDto::Solid(c) => Paint::Solid(color(c)?),
        PaintDto::Arc {
            scale,
            start,
            end,
            transition,
        } => {
            if scale != "arc" || !transition.is_finite() || !(0.0..=100.0).contains(transition) {
                return Err(DtoError("invalid arc gradient scale or transition".into()));
            }
            Paint::Arc {
                start: color(start)?,
                end: color(end)?,
                transition: transition / 100.0,
            }
        }
        PaintDto::Linear { angle, stops } => Paint::Linear {
            angle: *angle,
            stops: stops
                .iter()
                .map(|(pos, c)| Ok((*pos, color(c)?)))
                .collect::<R<_>>()?,
        },
    })
}

fn opt_paint(p: &Option<PaintDto>) -> R<Option<Paint>> {
    p.as_ref().map(paint).transpose()
}

fn binding_dto(b: &Binding) -> BindingDto {
    BindingDto {
        key: b.key.as_str().to_string(),
        min: b.min,
        max: b.max,
    }
}

fn binding(b: &BindingDto) -> R<Binding> {
    Ok(Binding {
        key: key(&b.key)?,
        min: b.min,
        max: b.max,
    })
}

fn segments_dto(s: Option<Segments>) -> Option<SegmentsDto> {
    s.map(|s| SegmentsDto {
        count: s.count,
        gap: s.gap,
    })
}

fn segments(s: Option<SegmentsDto>) -> Option<Segments> {
    s.map(|s| Segments {
        count: s.count,
        gap: s.gap,
    })
}

fn format_dto(f: DisplayFormat) -> FormatDto {
    FormatDto {
        decimals: f.decimals,
        show_unit: f.show_unit,
        fahrenheit: f.temperature == TemperatureUnit::Fahrenheit,
        decimal_bytes: f.bytes == ByteUnits::Decimal,
    }
}

fn format(f: FormatDto) -> DisplayFormat {
    DisplayFormat {
        decimals: f.decimals,
        show_unit: f.show_unit,
        temperature: if f.fahrenheit {
            TemperatureUnit::Fahrenheit
        } else {
            TemperatureUnit::Celsius
        },
        bytes: if f.decimal_bytes {
            ByteUnits::Decimal
        } else {
            ByteUnits::Binary
        },
    }
}

fn style_dto(s: &TextStyle) -> TextStyleDto {
    TextStyleDto {
        casing: match s.casing {
            bezel_core::domain::text_case::TextCase::Normal => None,
            bezel_core::domain::text_case::TextCase::Upper => Some("upper".into()),
            bezel_core::domain::text_case::TextCase::Lower => Some("lower".into()),
            bezel_core::domain::text_case::TextCase::Sentence => Some("sentence".into()),
            bezel_core::domain::text_case::TextCase::Title => Some("title".into()),
        },
        font: FontDto {
            family: s.font.family.clone(),
            weight: s.font.weight,
            italic: s.font.italic,
            asset: s.font.asset.as_ref().map(|a| a.0.clone()),
        },
        size: s.size,
        paint: paint_dto(&s.paint),
        align: match s.align {
            HAlign::Left => "left",
            HAlign::Center => "center",
            HAlign::Right => "right",
        }
        .to_string(),
        valign: match s.valign {
            VAlign::Top => "top",
            VAlign::Middle => "middle",
            VAlign::Bottom => "bottom",
        }
        .to_string(),
        letter_spacing: s.letter_spacing,
        tabular_digits: s.tabular_digits,
        overflow: s.overflow.map(|o| {
            match o {
                TextOverflow::Shrink => "shrink",
                TextOverflow::Ellipsis => "ellipsis",
                TextOverflow::Scroll => "scroll",
            }
            .to_string()
        }),
        parts: parts_dto(&s.parts),
    }
}

fn style(s: &TextStyleDto) -> R<TextStyle> {
    Ok(TextStyle {
        casing: match s.casing.as_deref() {
            None | Some("normal") => bezel_core::domain::text_case::TextCase::Normal,
            Some("upper") => bezel_core::domain::text_case::TextCase::Upper,
            Some("lower") => bezel_core::domain::text_case::TextCase::Lower,
            Some("sentence") => bezel_core::domain::text_case::TextCase::Sentence,
            Some("title") => bezel_core::domain::text_case::TextCase::Title,
            Some(other) => return err(format!("unknown text casing {other:?}")),
        },
        font: FontSpec {
            family: s.font.family.clone(),
            weight: s.font.weight,
            italic: s.font.italic,
            asset: s.font.asset.clone().map(AssetRef),
        },
        size: s.size,
        paint: paint(&s.paint)?,
        align: match s.align.as_str() {
            "left" => HAlign::Left,
            "center" => HAlign::Center,
            "right" => HAlign::Right,
            other => return err(format!("unknown align {other:?}")),
        },
        valign: match s.valign.as_str() {
            "top" => VAlign::Top,
            "middle" => VAlign::Middle,
            "bottom" => VAlign::Bottom,
            other => return err(format!("unknown valign {other:?}")),
        },
        letter_spacing: s.letter_spacing,
        tabular_digits: s.tabular_digits,
        overflow: match s.overflow.as_deref() {
            None => None,
            Some("shrink") => Some(TextOverflow::Shrink),
            Some("ellipsis") => Some(TextOverflow::Ellipsis),
            Some("scroll") => Some(TextOverflow::Scroll),
            Some(other) => return err(format!("unknown text overflow {other:?}")),
        },
        parts: parts(s.parts.as_deref())?,
    })
}

#[cfg(test)]
mod text_case_tests {
    use super::*;
    #[test]
    fn text_case_round_trips_and_old_styles_keep_their_format() {
        let base = serde_json::json!({"font":{"family":"Inter","weight":400},"size":24,
            "paint":"#ffffffff","align":"left","valign":"top"});
        let old: TextStyleDto = serde_json::from_value(base.clone()).unwrap();
        let restored = style(&old).unwrap();
        assert_eq!(
            restored.casing,
            bezel_core::domain::text_case::TextCase::Normal
        );
        assert!(
            serde_json::to_value(style_dto(&restored))
                .unwrap()
                .get("casing")
                .is_none()
        );
        for mode in ["upper", "lower", "sentence", "title"] {
            let mut json = base.clone();
            json["casing"] = mode.into();
            let dto: TextStyleDto = serde_json::from_value(json).unwrap();
            assert_eq!(
                style_dto(&style(&dto).unwrap()).casing.as_deref(),
                Some(mode)
            );
        }
        let mut invalid = base;
        invalid["casing"] = "unknown".into();
        assert!(style(&serde_json::from_value(invalid).unwrap()).is_err());
    }

    fn base() -> serde_json::Value {
        serde_json::json!({"font":{"family":"Inter","weight":400},"size":24,
            "paint":"#ffffffff","align":"left","valign":"top"})
    }

    fn round_trip(json: serde_json::Value) -> (TextStyle, serde_json::Value) {
        let dto: TextStyleDto = serde_json::from_value(json).unwrap();
        let restored = style(&dto).unwrap();
        let written = serde_json::to_value(style_dto(&restored)).unwrap();
        (restored, written)
    }

    #[test]
    fn an_old_style_keeps_its_fields_and_gets_tabular_digits() {
        let (restored, written) = round_trip(base());
        assert!(restored.tabular_digits);
        assert_eq!(restored.overflow, None);
        assert!(restored.parts.is_empty());
        for field in ["tabularDigits", "overflow", "parts"] {
            assert!(written.get(field).is_none(), "{field} is not written");
        }
    }

    #[test]
    fn tabular_digits_and_overflow_round_trip() {
        let mut json = base();
        json["tabularDigits"] = false.into();
        let (restored, written) = round_trip(json);
        assert!(!restored.tabular_digits);
        assert_eq!(written["tabularDigits"], false);
        for (name, overflow) in [
            ("shrink", TextOverflow::Shrink),
            ("ellipsis", TextOverflow::Ellipsis),
            ("scroll", TextOverflow::Scroll),
        ] {
            let mut json = base();
            json["overflow"] = name.into();
            let (restored, written) = round_trip(json);
            assert_eq!(restored.overflow, Some(overflow));
            assert_eq!(written["overflow"], name);
        }
        let mut bad = base();
        bad["overflow"] = "wrap".into();
        assert!(style(&serde_json::from_value(bad).unwrap()).is_err());
    }

    #[test]
    fn parts_round_trip_and_empty_ones_are_omitted() {
        let mut json = base();
        json["parts"] = serde_json::json!({
            "prefix": {"font": "IBM Plex Mono", "fontAsset": "assets/plex.ttf"},
            "number": {"weight": 700},
            "unit": {"sizePercent": 55.0, "color": "#ff9248ff", "baseline": "super"},
            "suffix": {"baseline": "base"},
            "hours": {"weight": 300},
            "minutes": {"color": "#ff9248ff"},
            "seconds": {"sizePercent": 60.0},
            "separators": {"color": "#6b747cff", "blink": true}
        });
        let (restored, written) = round_trip(json.clone());
        let unit = restored.parts.style(TextPart::Unit).expect("unit");
        assert_eq!(unit.baseline, Baseline::Super);
        assert_eq!(unit.size_percent, Some(55.0));
        assert_eq!(
            restored
                .parts
                .style(TextPart::Prefix)
                .and_then(|p| p.asset.clone()),
            Some(AssetRef("assets/plex.ttf".into()))
        );
        assert!(restored.parts.blink_separators);
        assert_eq!(restored.parts.style(TextPart::Suffix), None, "plain");
        let mut expected = json["parts"].clone();
        expected.as_object_mut().unwrap().remove("suffix");
        assert_eq!(written["parts"], expected);

        let mut blink_only = base();
        blink_only["parts"] = serde_json::json!({"separators": {"blink": true}});
        let (restored, written) = round_trip(blink_only);
        assert!(restored.parts.blink_separators && restored.parts.styles.is_empty());
        assert_eq!(
            written["parts"],
            serde_json::json!({"separators": {"blink": true}})
        );

        let mut odd = base();
        odd["parts"] = serde_json::json!({"unit": {"sizePercent": 9000.0, "weight": 5}});
        let (restored, _) = round_trip(odd);
        let unit = restored.parts.style(TextPart::Unit).expect("unit");
        assert_eq!((unit.size_percent, unit.weight), (Some(400.0), Some(100)));
        for bad in [
            serde_json::json!({"unit": {"baseline": "sub"}}),
            serde_json::json!({"unit": {"color": "orange"}}),
        ] {
            let mut json = base();
            json["parts"] = bad;
            assert!(style(&serde_json::from_value(json).unwrap()).is_err());
        }
    }
}

fn direction_name(d: Direction) -> &'static str {
    match d {
        Direction::LeftToRight => "leftToRight",
        Direction::RightToLeft => "rightToLeft",
        Direction::BottomToTop => "bottomToTop",
        Direction::TopToBottom => "topToBottom",
    }
}

fn direction(s: &str) -> R<Direction> {
    Ok(match s {
        "leftToRight" => Direction::LeftToRight,
        "rightToLeft" => Direction::RightToLeft,
        "bottomToTop" => Direction::BottomToTop,
        "topToBottom" => Direction::TopToBottom,
        other => return err(format!("unknown direction {other:?}")),
    })
}

fn graph_style_name(s: GraphStyle) -> &'static str {
    match s {
        GraphStyle::Line => "line",
        GraphStyle::Area => "area",
        GraphStyle::Bars => "bars",
    }
}

fn graph_style(s: &str) -> R<GraphStyle> {
    Ok(match s {
        "line" => GraphStyle::Line,
        "area" => GraphStyle::Area,
        "bars" => GraphStyle::Bars,
        other => return err(format!("unknown graph style {other:?}")),
    })
}

fn kind_dto(k: &ElementKind) -> KindDto {
    match k {
        ElementKind::Text { content, style } => KindDto::Text {
            content: match content {
                TextContent::Spectrum(s) => ContentDto::Spectrum { spectrum: s.into() },
                TextContent::Player(p) => ContentDto::Player {
                    source: p.source.clone(),
                    show_cover: p.show_cover,
                    cover_radius: p.cover_radius,
                    cover_corners: p.cover_corners,
                    cover_gap: p.cover_gap,
                    show_progress: p.show_progress,
                    show_source: p.show_source,
                    hide_when_stopped: p.hide_when_stopped,
                    empty_text: p.empty_text.clone(),
                    show_title: p.show_title,
                    show_artist: p.show_artist,
                    show_time: p.show_time,
                    time_format: time_format_dto(p.time_format),
                    progress_style: progress_style_dto(&p.progress),
                },
                TextContent::Static(text) => ContentDto::Static { text: text.clone() },
                TextContent::Sensor {
                    key,
                    format,
                    prefix,
                    suffix,
                } => ContentDto::Sensor {
                    key: key.as_str().to_string(),
                    format: format_dto(*format),
                    prefix: prefix.clone(),
                    suffix: suffix.clone(),
                },
                TextContent::Weather(w) => ContentDto::Weather {
                    city: w.city.clone(),
                    latitude: w.latitude,
                    longitude: w.longitude,
                    language: w.language.map(|l| {
                        match l {
                            Language::Italian => "it",
                            Language::English => "en",
                            Language::PortugueseBr => "pt-BR",
                        }
                        .to_string()
                    }),
                    fahrenheit: w.fahrenheit,
                    show_icon: w.show_icon,
                    icon_style: match w.icon_style {
                        bezel_core::domain::weather::IconStyle::Outline => "outline",
                        bezel_core::domain::weather::IconStyle::Filled => "filled",
                        bezel_core::domain::weather::IconStyle::Colored => "colored",
                        bezel_core::domain::weather::IconStyle::Dimensional => "dimensional",
                    }
                    .into(),
                    icon_gap: w.icon_gap,
                    icon_size: w.icon_size,
                },
                TextContent::Clock {
                    pattern,
                    language,
                    casing,
                    analog,
                } => ContentDto::Clock {
                    analog: analog.as_ref().map(analog_dto),
                    pattern: pattern.clone(),
                    language: language.map(|l| {
                        match l {
                            Language::English => "en",
                            Language::PortugueseBr => "pt-BR",
                            Language::Italian => "it",
                        }
                        .to_string()
                    }),
                    casing: match casing {
                        ClockCase::Normal => None,
                        ClockCase::Upper => Some("upper".into()),
                        ClockCase::Title => Some("title".into()),
                    },
                },
            },
            style: style_dto(style),
        },
        ElementKind::Image { asset, fit, fade } => KindDto::Image {
            fade: fade.map(fade_dto),
            asset: asset.0.clone(),
            fit: fit_dto(*fit),
        },
        ElementKind::Shape {
            video_window,
            fade,
            shape,
            fill,
            image,
            stroke,
        } => {
            let (name, radius, corners) = match shape {
                ShapeKind::Rect { radius } => ("rect", *radius, None),
                ShapeKind::Ellipse => ("ellipse", 0.0, None),
                ShapeKind::Corners(r) => ("rect", 0., Some(*r)),
            };
            KindDto::Shape {
                video_window: *video_window,
                fade: fade.map(|f| FadeDto {
                    radial: f.radial,
                    angle: f.angle,
                    start: f.start,
                    end: f.end,
                    midpoint: f.midpoint,
                }),
                corners,
                shape: name.to_string(),
                radius,
                fill: fill.as_ref().map(paint_dto),
                image: image.as_ref().map(shape_image_dto),
                stroke: stroke.map(|(c, _)| to_hex(c)),
                stroke_width: stroke.map_or(0.0, |(_, w)| w),
            }
        }
        ElementKind::Bar {
            binding,
            direction,
            fill,
            track,
            radius,
            segments,
        } => KindDto::Bar {
            binding: binding_dto(binding),
            direction: direction_name(*direction).to_string(),
            fill: paint_dto(fill),
            track: track.as_ref().map(paint_dto),
            radius: *radius,
            segments: segments_dto(*segments),
        },
        ElementKind::Ring {
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
        } => KindDto::Ring {
            test_full: *test_full,
            binding: binding_dto(binding),
            start_angle: *start_angle,
            sweep: *sweep,
            thickness: *thickness,
            clockwise: *clockwise,
            fill: paint_dto(fill),
            track: track.as_ref().map(paint_dto),
            round_caps: *cap == Cap::Round,
            segments: segments_dto(*segments),
        },
        ElementKind::Needle {
            binding,
            asset,
            pivot,
            start_angle,
            sweep,
            color,
            width,
        } => KindDto::Needle {
            binding: binding_dto(binding),
            asset: asset.as_ref().map(|a| a.0.clone()),
            pivot: *pivot,
            start_angle: *start_angle,
            sweep: *sweep,
            color: to_hex(*color),
            width: *width,
        },
        ElementKind::Graph {
            binding,
            history,
            interval_seconds,
            style,
            color,
            fill,
            line_width,
            autoscale,
        } => KindDto::Graph {
            binding: binding_dto(binding),
            history: *history,
            interval_seconds: *interval_seconds,
            style: graph_style_name(*style).to_string(),
            color: to_hex(*color),
            fill: fill.as_ref().map(paint_dto),
            line_width: *line_width,
            autoscale: *autoscale,
        },
    }
}

/// A graph's interval as read: 0 means none (one point per refresh), more
/// than [`GRAPH_INTERVAL_MAX_SECONDS`] is that.
fn graph_interval(seconds: Option<u32>) -> Option<u32> {
    seconds
        .filter(|s| *s > 0)
        .map(|s| s.min(GRAPH_INTERVAL_MAX_SECONDS))
}

fn kind(k: &KindDto) -> R<ElementKind> {
    Ok(match k {
        KindDto::Text { content, style: s } => ElementKind::Text {
            content: match content {
                ContentDto::Spectrum { spectrum } => TextContent::Spectrum(spectrum.domain()?),
                ContentDto::Player {
                    source,
                    show_cover,
                    cover_radius,
                    cover_corners,
                    cover_gap,
                    show_progress,
                    show_source,
                    hide_when_stopped,
                    empty_text,
                    show_title,
                    show_artist,
                    show_time,
                    time_format: format_of_time,
                    progress_style: style_of_progress,
                } => {
                    if cover_corners.is_some_and(|r| r.iter().any(|v| !v.is_finite() || *v < 0.))
                        || !cover_radius.is_finite()
                        || *cover_radius < 0.
                        || cover_gap.is_some_and(|v| !v.is_finite() || v < 0.)
                    {
                        return err("invalid cover geometry");
                    }
                    if source.chars().count() > 160 || empty_text.chars().count() > 512 {
                        return err("player text is too long");
                    }
                    TextContent::Player(bezel_core::domain::playback::Player {
                        source: source.clone(),
                        show_cover: *show_cover,
                        cover_radius: *cover_radius,
                        cover_corners: *cover_corners,
                        cover_gap: *cover_gap,
                        show_progress: *show_progress,
                        show_source: *show_source,
                        hide_when_stopped: *hide_when_stopped,
                        empty_text: empty_text.clone(),
                        show_title: *show_title,
                        show_artist: *show_artist,
                        show_time: *show_time,
                        time_format: time_format(*format_of_time),
                        progress: progress_style(style_of_progress.as_ref())?,
                    })
                }
                ContentDto::Static { text } => TextContent::Static(text.clone()),
                ContentDto::Sensor {
                    key: k,
                    format: f,
                    prefix,
                    suffix,
                } => TextContent::Sensor {
                    key: key(k)?,
                    format: format(*f),
                    prefix: prefix.clone(),
                    suffix: suffix.clone(),
                },
                ContentDto::Weather {
                    city,
                    latitude,
                    longitude,
                    language,
                    fahrenheit,
                    show_icon,
                    icon_style,
                    icon_gap,
                    icon_size,
                } => {
                    if !bezel_core::domain::weather::valid(city, *latitude, *longitude) {
                        return err("invalid weather location");
                    }
                    if icon_gap.is_some_and(|v| !v.is_finite() || !(0.0..=256.0).contains(&v))
                        || icon_size.is_some_and(|v| !v.is_finite() || !(1.0..=512.0).contains(&v))
                    {
                        return err("invalid weather icon dimensions");
                    }
                    TextContent::Weather(bezel_core::domain::weather::Weather {
                        city: city.clone(),
                        latitude: *latitude,
                        longitude: *longitude,
                        language: match language.as_deref() {
                            None | Some("") => None,
                            Some("it") => Some(Language::Italian),
                            Some("en") => Some(Language::English),
                            Some("pt-BR") => Some(Language::PortugueseBr),
                            _ => return err("invalid weather language"),
                        },
                        fahrenheit: *fahrenheit,
                        show_icon: *show_icon,
                        icon_style: match icon_style.as_str() {
                            "outline" => bezel_core::domain::weather::IconStyle::Outline,
                            "filled" => bezel_core::domain::weather::IconStyle::Filled,
                            "colored" => bezel_core::domain::weather::IconStyle::Colored,
                            "dimensional" => bezel_core::domain::weather::IconStyle::Dimensional,
                            _ => return err("invalid weather icon style"),
                        },
                        icon_gap: *icon_gap,
                        icon_size: *icon_size,
                    })
                }
                ContentDto::Clock {
                    pattern,
                    language,
                    casing,
                    analog: dial,
                } => TextContent::Clock {
                    analog: dial.as_deref().map(analog).transpose()?,
                    pattern: pattern.clone(),
                    language: match language.as_deref() {
                        None | Some("") => None,
                        Some("en") => Some(Language::English),
                        Some("pt-BR") => Some(Language::PortugueseBr),
                        Some("it") => Some(Language::Italian),
                        Some(other) => return err(format!("bad clock language {other:?}")),
                    },
                    casing: match casing.as_deref() {
                        None | Some("normal") => ClockCase::Normal,
                        Some("upper") => ClockCase::Upper,
                        Some("title") => ClockCase::Title,
                        Some(other) => return err(format!("bad clock casing {other:?}")),
                    },
                },
            },
            style: style(s)?,
        },
        KindDto::Image {
            asset,
            fit: f,
            fade,
        } => ElementKind::Image {
            fade: fade.as_ref().map(parse_fade).transpose()?,
            asset: AssetRef(asset.clone()),
            fit: fit(*f),
        },
        KindDto::Shape {
            video_window,
            fade,
            shape,
            corners,
            radius,
            fill,
            image,
            stroke,
            stroke_width,
        } => ElementKind::Shape {
            video_window: *video_window,
            fade: fade.as_ref().map(parse_fade).transpose()?,
            shape: match shape.as_str() {
                "rect" => {
                    if let Some(r) = corners {
                        if r.iter().any(|v| !v.is_finite() || *v < 0.) {
                            return err("invalid corner radius");
                        }
                        ShapeKind::Corners(*r)
                    } else {
                        ShapeKind::Rect { radius: *radius }
                    }
                }
                "ellipse" => ShapeKind::Ellipse,
                other => return err(format!("unknown shape {other:?}")),
            },
            fill: opt_paint(fill)?,
            image: image.as_ref().map(shape_image).transpose()?,
            stroke: stroke
                .as_deref()
                .map(color)
                .transpose()?
                .map(|c| (c, *stroke_width)),
        },
        KindDto::Bar {
            binding: b,
            direction: d,
            fill,
            track,
            radius,
            segments: s,
        } => ElementKind::Bar {
            binding: binding(b)?,
            direction: direction(d)?,
            fill: paint(fill)?,
            track: opt_paint(track)?,
            radius: *radius,
            segments: segments(*s),
        },
        KindDto::Ring {
            test_full,
            binding: b,
            start_angle,
            sweep,
            thickness,
            clockwise,
            fill,
            track,
            round_caps,
            segments: s,
        } => ElementKind::Ring {
            test_full: *test_full,
            binding: binding(b)?,
            start_angle: *start_angle,
            sweep: *sweep,
            thickness: *thickness,
            clockwise: *clockwise,
            fill: paint(fill)?,
            track: opt_paint(track)?,
            cap: if *round_caps { Cap::Round } else { Cap::Butt },
            segments: segments(*s),
        },
        KindDto::Needle {
            binding: b,
            asset,
            pivot,
            start_angle,
            sweep,
            color: c,
            width,
        } => ElementKind::Needle {
            binding: binding(b)?,
            asset: asset.clone().map(AssetRef),
            pivot: *pivot,
            start_angle: *start_angle,
            sweep: *sweep,
            color: color(c)?,
            width: *width,
        },
        KindDto::Graph {
            binding: b,
            history,
            interval_seconds,
            style: s,
            color: c,
            fill,
            line_width,
            autoscale,
        } => ElementKind::Graph {
            binding: binding(b)?,
            history: *history,
            interval_seconds: graph_interval(*interval_seconds),
            style: graph_style(s)?,
            color: color(c)?,
            fill: opt_paint(fill)?,
            line_width: *line_width,
            autoscale: *autoscale,
        },
    })
}

impl From<&Theme> for ThemeDto {
    fn from(t: &Theme) -> Self {
        ThemeDto {
            schema: SCHEMA,
            name: t.name.clone(),
            canvas: SizeDto {
                width: t.canvas.width,
                height: t.canvas.height,
            },
            orientation: orientation_name(t.orientation).to_string(),
            refresh_seconds: t.refresh_seconds,
            palette: t
                .palette
                .iter()
                .map(|s| SwatchDto {
                    name: s.name.clone(),
                    color: to_hex(s.color),
                })
                .collect(),
            background: match &t.background {
                Background::DeviceVideo {
                    path,
                    repeat,
                    color,
                } => BackgroundDto::DeviceVideo {
                    path: path.to_string(),
                    looping: *repeat == bezel_core::domain::storage::Repeat::Loop,
                    color: to_hex(*color),
                },
                Background::Color(c) => BackgroundDto::Color { color: to_hex(*c) },
                Background::Image { asset, fit } => BackgroundDto::Image {
                    asset: asset.0.clone(),
                    fit: fit_dto(*fit),
                },
                Background::Video {
                    asset,
                    poster,
                    framing,
                } => BackgroundDto::Video {
                    asset: asset.0.clone(),
                    poster: poster.as_ref().map(|p| p.0.clone()),
                    framing: framing_dto(framing.as_ref()),
                },
            },
            elements: t
                .elements
                .iter()
                .map(|e| ElementDto {
                    card: e.card.as_ref().map(|c| {
                        let c = c.normalized();
                        CardDto {
                            triggers: c
                                .triggers
                                .iter()
                                .map(|r| {
                                    CardTriggerDto {
                                interval_seconds: r.interval_seconds, next_face: r.next_face,
                                source: match r.source {
                                    bezel_core::domain::playback::TriggerSource::Timer => "timer",
                                    bezel_core::domain::playback::TriggerSource::Process => {
                                        "process"
                                    }
                                    bezel_core::domain::playback::TriggerSource::ProcessClosed => {
                                        "processClosed"
                                    }
                                    bezel_core::domain::playback::TriggerSource::Foreground => {
                                        "foreground"
                                    }
                                    bezel_core::domain::playback::TriggerSource::MediaPlaying => {
                                        "mediaPlaying"
                                    }
                                }
                                .into(),
                                app: r.app.clone(),
                                face: r.face,
                                priority: r.priority,
                                return_seconds: r.return_seconds,
                                return_face: r.return_face,
                            }
                                })
                                .collect(),
                            faces: c.faces.clone(),
                            active_face: c.active_face,
                            rotation_seconds: c.rotation_seconds,
                            transition: c.transition.map(card_transition_dto),
                        }
                    }),
                    rotation: e.applied_rotation(),
                    is_group: e.is_group,
                    group_parent: e.group_parent.map(|id| id.0),
                    card_member: e.card_member.as_ref().map(|m| CardMemberDto {
                        parent: m.parent.0,
                        face: m.face,
                    }),
                    id: e.id.0,
                    name: e.name.clone(),
                    frame: BoxDto {
                        x: e.frame.x,
                        y: e.frame.y,
                        width: e.frame.width,
                        height: e.frame.height,
                    },
                    opacity: e.opacity,
                    visible: e.visible,
                    locked: e.locked,
                    color_by_value: e.color_by_value.as_ref().map(color_by_value_dto),
                    visualization: e.visualization.as_ref().map(visualization_dto),
                    value_transition: value_transition_dto(e.value_transition),
                    sensor_motion: e.sensor_motion.as_ref().map(|m| SensorMotionDto {
                        sensor: m.sensor.to_string(),
                        reference: m.reference,
                        turns_per_second: m.turns_per_second,
                        inertia_ms: m.inertia_ms,
                        reverse: m.reverse,
                    }),
                    kind: kind_dto(&e.kind),
                })
                .collect(),
        }
    }
}

impl TryFrom<&ThemeDto> for Theme {
    type Error = DtoError;

    fn try_from(d: &ThemeDto) -> R<Theme> {
        if d.schema == 0 || d.schema > SCHEMA {
            return err(format!(
                "theme schema {} is not supported (this build reads 1..={SCHEMA})",
                d.schema
            ));
        }
        if d.canvas.width == 0 || d.canvas.height == 0 {
            return err("empty canvas");
        }
        let background = match &d.background {
            BackgroundDto::DeviceVideo {
                path,
                looping,
                color: c,
            } => {
                let path = bezel_core::domain::storage::RemotePath::parse(path)
                    .map_err(|e| DtoError(e.to_string()))?;
                if path.location.kind != bezel_core::domain::media::MediaKind::Video {
                    return err("device background must reference a video");
                }
                Background::DeviceVideo {
                    path,
                    repeat: if *looping {
                        bezel_core::domain::storage::Repeat::Loop
                    } else {
                        bezel_core::domain::storage::Repeat::Once
                    },
                    color: color(c)?,
                }
            }
            BackgroundDto::Color { color: c } => Background::Color(color(c)?),
            BackgroundDto::Image { asset, fit: f } => Background::Image {
                asset: AssetRef(asset.clone()),
                fit: fit(*f),
            },
            BackgroundDto::Video {
                asset,
                poster,
                framing: f,
            } => Background::Video {
                asset: AssetRef(asset.clone()),
                poster: poster.clone().map(AssetRef),
                framing: framing(f.as_ref())?,
            },
        };
        let elements = d
            .elements
            .iter()
            .map(|e| {
                if !e.rotation.is_finite() {
                    return err("invalid rotation");
                }
                let mut element = Element {
                    card: e
                        .card
                        .as_ref()
                        .map(|c| -> R<bezel_core::domain::theme::Card> {
                            if c.rotation_seconds
                                .is_some_and(|seconds| !(5..=3600).contains(&seconds))
                            {
                                return err("card rotation interval must be 5 to 3600 seconds");
                            }
                            if c.triggers.len() > if c.rotation_seconds.is_some() { 32 } else { 33 }
                            {
                                return err("too many card triggers");
                            }
                            let triggers = c
                                .triggers
                                .iter()
                                .map(|r| {
                                    use bezel_core::domain::playback::{
                                        CardTrigger, TriggerSource,
                                    };
                                    let source = match r.source.as_str() {
                                        "timer" => TriggerSource::Timer,
                                        "process" => TriggerSource::Process,
                                        "processClosed" => TriggerSource::ProcessClosed,
                                        "foreground" => TriggerSource::Foreground,
                                        "mediaPlaying" => TriggerSource::MediaPlaying,
                                        _ => return err("unknown card trigger"),
                                    };
                                    if r.app.chars().count() > 160
                                        || (r.app.trim().is_empty()
                                            && !matches!(
                                                source,
                                                TriggerSource::MediaPlaying | TriggerSource::Timer
                                            ))
                                        || (!r.next_face && r.face >= c.faces.len())
                                        || (source == TriggerSource::Timer
                                            && !r
                                                .interval_seconds
                                                .is_some_and(|s| (5..=3600).contains(&s)))
                                        || (source != TriggerSource::Timer
                                            && (r.interval_seconds.is_some() || r.next_face))
                                        || r.priority > 100
                                        || r.return_seconds > 300
                                        || r.return_face.is_some_and(|f| f >= c.faces.len())
                                    {
                                        return err("invalid card trigger");
                                    }
                                    Ok(CardTrigger {
                                        interval_seconds: r.interval_seconds,
                                        next_face: r.next_face,
                                        source,
                                        app: r.app.clone(),
                                        face: r.face,
                                        priority: r.priority,
                                        return_seconds: r.return_seconds,
                                        return_face: r.return_face,
                                    })
                                })
                                .collect::<R<Vec<_>>>()?;
                            let transition =
                                c.transition.as_ref().map(card_transition).transpose()?;
                            Ok(bezel_core::domain::theme::Card {
                                triggers,
                                faces: c.faces.clone(),
                                active_face: c.active_face,
                                rotation_seconds: c.rotation_seconds,
                                transition,
                            }
                            .normalized()
                            .into_owned())
                        })
                        .transpose()?,
                    rotation: bezel_core::domain::rotation::normalize_degrees(e.rotation),
                    is_group: e.is_group,
                    group_parent: e.group_parent.map(ElementId),
                    card_member: e.card_member.as_ref().map(|m| {
                        bezel_core::domain::theme::CardMember {
                            parent: ElementId(m.parent),
                            face: m.face,
                        }
                    }),
                    id: ElementId(e.id),
                    name: e.name.clone(),
                    frame: BoxF::new(e.frame.x, e.frame.y, e.frame.width, e.frame.height),
                    opacity: e.opacity.clamp(0.0, 1.0),
                    visible: e.visible,
                    locked: e.locked,
                    color_by_value: e.color_by_value.as_ref().map(color_by_value).transpose()?,
                    visualization: e.visualization.as_ref().map(visualization).transpose()?,
                    value_transition: value_transition(e.value_transition),
                    sensor_motion: sensor_motion(&e.sensor_motion)?,
                    kind: kind(&e.kind)
                        .map_err(|x| DtoError(format!("element {} ({}): {}", e.id, e.name, x.0)))?,
                };
                element.rotation = element.applied_rotation();
                if !element.supports_sensor_motion() {
                    element.sensor_motion = None;
                }
                Ok(element)
            })
            .collect::<R<Vec<_>>>()?;
        if elements.iter().any(|e| {
            e.card.is_some() || e.card_member.is_some() || e.is_group || e.group_parent.is_some()
        }) {
            let ids: std::collections::BTreeSet<_> = elements.iter().map(|e| e.id.0).collect();
            if ids.len() != elements.len() {
                return err("duplicate element IDs in card theme");
            }
        }
        for e in &elements {
            if let Some(card) = &e.card
                && (card.faces.is_empty()
                    || card.faces.len() > 16
                    || card.active_face >= card.faces.len()
                    || card
                        .faces
                        .iter()
                        .any(|name| name.trim().is_empty() || name.len() > 128)
                    || e.card_member.is_some()
                    || !matches!(e.kind, ElementKind::Shape { .. }))
            {
                return err("invalid card configuration");
            }
            if let Some(member) = &e.card_member {
                let parent = elements
                    .iter()
                    .find(|parent| parent.id == member.parent && parent.id != e.id);
                if !parent
                    .and_then(|p| p.card.as_ref())
                    .is_some_and(|c| member.face.is_none_or(|f| f < c.faces.len()))
                {
                    return err("invalid card membership");
                }
            }
        }
        for e in &elements {
            if e.is_group
                && (e.card.is_some()
                    || !matches!(
                        e.kind,
                        ElementKind::Shape {
                            fill: None,
                            stroke: None,
                            fade: None,
                            video_window: false,
                            ..
                        }
                    ))
            {
                return err("invalid ordinary group container");
            }
            let mut current = e;
            let mut seen = std::collections::BTreeSet::from([e.id]);
            while let Some(id) = current.group_parent {
                let Some(parent) = elements.iter().find(|p| p.id == id && p.is_group) else {
                    return err("invalid group membership");
                };
                if !seen.insert(id) || seen.len() > 64 || parent.card_member != current.card_member
                {
                    return err("invalid group hierarchy");
                }
                current = parent;
            }
        }
        Ok(Theme {
            name: d.name.clone(),
            canvas: Size::new(d.canvas.width, d.canvas.height),
            orientation: orientation(&d.orientation)?,
            background,
            elements,
            refresh_seconds: if d.refresh_seconds > 0.0 {
                d.refresh_seconds
            } else {
                1.0
            },
            palette: palette(&d.palette)?,
        })
    }
}

fn outline_icon() -> String {
    "outline".into()
}

fn is_false(value: &bool) -> bool {
    !value
}

fn fade_dto(f: bezel_core::domain::gradient::Fade) -> FadeDto {
    FadeDto {
        angle: f.angle,
        start: f.start,
        end: f.end,
        radial: f.radial,
        midpoint: f.midpoint,
    }
}
fn parse_fade(f: &FadeDto) -> R<bezel_core::domain::gradient::Fade> {
    if !f.angle.is_finite()
        || !f.start.is_finite()
        || !f.end.is_finite()
        || !(0.0..=1.0).contains(&f.start)
        || !(0.0..=1.0).contains(&f.end)
        || !bezel_core::domain::gradient::Fade::MIDPOINTS.contains(&f.midpoint)
        || f.radial.is_some_and(|[x, y, r]| {
            !x.is_finite()
                || !y.is_finite()
                || !r.is_finite()
                || !(0.0..=1.0).contains(&x)
                || !(0.0..=1.0).contains(&y)
                || !(0.01..=2.0).contains(&r)
        })
    {
        return err("invalid opacity gradient");
    }
    Ok(bezel_core::domain::gradient::Fade {
        angle: f.angle,
        start: f.start,
        end: f.end,
        radial: f.radial,
        midpoint: f.midpoint,
    })
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::panic)]
mod spectrum_tests {
    use super::*;
    #[test]
    fn spectrum_defaults_round_trip_without_changing_the_schema() {
        let dto: ContentDto = serde_json::from_str(r#"{"type":"spectrum"}"#).unwrap();
        let ContentDto::Spectrum { spectrum } = dto else {
            panic!("spectrum expected")
        };
        assert_eq!(
            spectrum.domain().unwrap(),
            bezel_core::domain::spectrum::Spectrum::default()
        );
        assert_eq!(SCHEMA, 1);
        let value = ContentDto::Spectrum { spectrum };
        assert_eq!(
            serde_json::from_slice::<ContentDto>(&serde_json::to_vec(&value).unwrap()).unwrap(),
            value
        );
    }
    #[test]
    fn spectrum_rejects_invalid_ranges_and_motion() {
        let mut s = SpectrumDto {
            bands: 1000,
            ..Default::default()
        };
        assert!(s.domain().is_err());
        s.bands = 32;
        s.low = 18000.;
        s.high = 60.;
        assert!(s.domain().is_err());
        s.low = 60.;
        s.high = 16000.;
        s.motion = "unknown".into();
        assert!(s.domain().is_err());
        s.motion = "balanced".into();
        s.style = "spiral".into();
        assert!(s.domain().is_err());
        s.style = "blocks".into();
        s.block = 0.;
        assert!(s.domain().is_err());
        s.block = 6.;
        s.inner = 95.;
        assert!(s.domain().is_err());
        s.inner = 40.;
        s.gain = 5.;
        assert!(s.domain().is_err());
        s.gain = 1.;
        s.audio = "microphone".into();
        assert!(s.domain().is_err());
    }
    #[test]
    fn every_style_round_trips_and_the_default_keeps_the_old_shape() {
        use bezel_core::domain::spectrum::{Spectrum, SpectrumStyle};
        let mirrored = serde_json::to_value(SpectrumDto::default()).unwrap();
        for key in ["style", "block", "stroke", "fill", "inner", "gain", "audio"] {
            assert!(mirrored.get(key).is_none(), "{key}");
        }
        for style in [
            SpectrumStyle::Bars,
            SpectrumStyle::Blocks,
            SpectrumStyle::Line,
            SpectrumStyle::Radial,
        ] {
            let s = Spectrum {
                style,
                block: 4.,
                stroke: 3.,
                fill: false,
                inner: 25.,
                gain: 0.5,
                audio: bezel_core::domain::spectrum::SpectrumAudio::System,
                ..Spectrum::default()
            };
            let json = serde_json::to_vec(&SpectrumDto::from(&s)).unwrap();
            let back: SpectrumDto = serde_json::from_slice(&json).unwrap();
            assert_eq!(back.domain().unwrap(), s);
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod fade_tests {
    use super::*;
    #[test]
    fn a_centred_fade_keeps_the_old_shape_and_a_moved_midpoint_round_trips() {
        let old: FadeDto = serde_json::from_str(r#"{"angle":0,"start":1,"end":0}"#).unwrap();
        let fade = parse_fade(&old).unwrap();
        assert_eq!(fade.midpoint, bezel_core::domain::gradient::Fade::MIDPOINT);
        assert!(
            serde_json::to_value(fade_dto(fade))
                .unwrap()
                .get("midpoint")
                .is_none()
        );
        let moved = bezel_core::domain::gradient::Fade {
            midpoint: 0.3,
            ..fade
        };
        let json = serde_json::to_vec(&fade_dto(moved)).unwrap();
        let back: FadeDto = serde_json::from_slice(&json).unwrap();
        assert_eq!(parse_fade(&back).unwrap(), moved);
        assert!(
            parse_fade(&FadeDto {
                midpoint: 0.99,
                ..back
            })
            .is_err()
        );
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod shape_image_tests {
    use super::*;

    fn shape(json: &str) -> R<ElementKind> {
        kind(&serde_json::from_str::<KindDto>(json).unwrap())
    }

    #[test]
    fn a_shape_without_image_keeps_the_old_shape() {
        let k =
            shape(r##"{"type":"shape","shape":"rect","radius":4,"fill":"#ff0000ff"}"##).unwrap();
        let ElementKind::Shape { image, .. } = &k else {
            panic!("{k:?}")
        };
        assert!(image.is_none());
        assert!(
            serde_json::to_value(kind_dto(&k))
                .unwrap()
                .get("image")
                .is_none()
        );
    }

    #[test]
    fn a_shape_image_fills_its_defaults_and_round_trips() {
        let k = shape(r#"{"type":"shape","shape":"ellipse","image":{"asset":"assets/dots.svg"}}"#)
            .unwrap();
        let ElementKind::Shape { image: Some(i), .. } = &k else {
            panic!("{k:?}")
        };
        assert_eq!(i, &ShapeImage::new(AssetRef("assets/dots.svg".into())));
        let tiled = ShapeImage {
            repeat: ImageRepeat::TileX,
            scale: 0.5,
            offset: (-3.0, 12.0),
            opacity: 0.4,
            ..i.clone()
        };
        let json = serde_json::to_value(shape_image_dto(&tiled)).unwrap();
        assert_eq!(json["repeat"], "tileX");
        assert_eq!(json["offsetY"], 12.0);
        let back: ShapeImageDto = serde_json::from_value(json).unwrap();
        assert_eq!(shape_image(&back).unwrap(), tiled);
        let centred = serde_json::to_value(shape_image_dto(i)).unwrap();
        assert!(centred.get("offsetX").is_none() && centred.get("offsetY").is_none());
    }

    #[test]
    fn out_of_range_shape_images_are_refused() {
        for bad in [
            r#"{"asset":"a.png","scale":0}"#,
            r#"{"asset":"a.png","scale":11}"#,
            r#"{"asset":"a.png","opacity":1.5}"#,
            r#"{"asset":"a.png","repeat":"mirror"}"#,
        ] {
            let parsed = serde_json::from_str::<ShapeImageDto>(bad);
            assert!(
                parsed.is_err() || shape_image(&parsed.unwrap()).is_err(),
                "{bad}"
            );
        }
    }
}

#[cfg(test)]
mod analog_clock_tests {
    use super::*;

    fn clock(analog: serde_json::Value) -> (TextContent, serde_json::Value) {
        let mut json = serde_json::json!({"type": "clock", "pattern": "%H:%M"});
        if !analog.is_null() {
            json["analog"] = analog;
        }
        let dto: KindDto = serde_json::from_value(serde_json::json!({
            "type": "text",
            "content": json,
            "style": {"font": {"family": "Inter", "weight": 400}, "size": 24,
                "paint": "#ffffffff", "align": "left", "valign": "top"}
        }))
        .unwrap();
        let kind = super::kind(&dto).unwrap();
        let ElementKind::Text { content, .. } = &kind else {
            unreachable!()
        };
        let written = serde_json::to_value(kind_dto(&kind)).unwrap();
        (content.clone(), written["content"].clone())
    }

    #[test]
    fn a_digital_clock_writes_no_dial_and_a_default_dial_is_empty() {
        let (content, written) = clock(serde_json::Value::Null);
        assert!(matches!(content, TextContent::Clock { analog: None, .. }));
        assert!(written.get("analog").is_none());
        let (content, written) = clock(serde_json::json!({}));
        assert!(matches!(
            content,
            TextContent::Clock { analog: Some(a), .. } if a == AnalogClock::default()
        ));
        assert_eq!(written["analog"], serde_json::json!({}));
    }

    #[test]
    fn every_dial_setting_round_trips() {
        let full = serde_json::json!({
            "ticks": "minutes", "numbers": "roman", "hourHand": false, "minuteHand": false,
            "secondHand": false, "handShape": "arrow", "border": 3.5, "face": "#101010ff",
            "marks": "#202020ff", "hourColor": "#303030ff", "minuteColor": "#404040ff",
            "secondColor": "#505050ff", "sweep": true, "date": "dayMonth"
        });
        let (content, written) = clock(full.clone());
        let TextContent::Clock {
            analog: Some(a), ..
        } = content
        else {
            panic!("a dial")
        };
        assert_eq!(a.ticks, DialTicks::Minutes);
        assert_eq!(a.numbers, DialNumbers::Roman);
        assert!(!a.hour_hand && !a.minute_hand && !a.second_hand && a.sweep);
        assert_eq!(a.hand_shape, HandShape::Arrow);
        assert_eq!(a.date, DialDate::DayMonth);
        assert_eq!(written["analog"], full);
        for (field, value) in [
            ("ticks", "none"),
            ("numbers", "quarters"),
            ("numbers", "all"),
            ("handShape", "thin"),
            ("date", "day"),
        ] {
            let (_, written) = clock(serde_json::json!({ field: value }));
            assert_eq!(written["analog"][field], value);
        }
        let (content, written) =
            clock(serde_json::json!({"ticks": "hours", "handShape": "rounded",
            "numbers": "none", "date": "none", "border": 2.0}));
        assert!(
            matches!(content, TextContent::Clock { analog: Some(a), .. } if a == AnalogClock::default())
        );
        assert_eq!(
            written["analog"],
            serde_json::json!({}),
            "defaults are omitted"
        );
        let (content, _) = clock(serde_json::json!({"border": 1000.0}));
        assert!(matches!(content, TextContent::Clock { analog: Some(a), .. } if a.border == 64.0));
    }

    #[test]
    fn unknown_dial_settings_are_refused() {
        for bad in [
            serde_json::json!({"ticks": "seconds"}),
            serde_json::json!({"numbers": "arabic"}),
            serde_json::json!({"handShape": "sword"}),
            serde_json::json!({"date": "year"}),
            serde_json::json!({"face": "black"}),
        ] {
            let dto: KindDto = serde_json::from_value(serde_json::json!({
                "type": "text",
                "content": {"type": "clock", "pattern": "%H", "analog": bad},
                "style": {"font": {"family": "Inter", "weight": 400}, "size": 24,
                    "paint": "#ffffffff", "align": "left", "valign": "top"}
            }))
            .unwrap();
            assert!(super::kind(&dto).is_err());
        }
    }
}
