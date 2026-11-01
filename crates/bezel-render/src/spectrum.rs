//! Spectrum styles (D-AS-4, D-AS-11) drawn within the element's clipped layer.
use crate::{
    layer::Layer,
    paint::{paint_for, solid},
    path::rounded_corners,
    shape,
};
use bezel_core::domain::{
    clock::Language,
    frame::Rgba,
    spectrum::{AudioState, Spectrum, SpectrumFrame, SpectrumStyle, dance_pose},
    theme::{BoxF, Paint, ShapeKind},
    value_color::ColorByValue,
};
use tiny_skia::{LineCap, LineJoin, PathBuilder, Stroke, Transform};

/// Bands of one frame scaled by the gain and clamped to 0..=1; missing values
/// read as silence.
struct Bands<'a>(&'a SpectrumFrame, f32);
impl Bands<'_> {
    fn level(&self, i: usize) -> f32 {
        (self.0.levels.get(i).copied().unwrap_or(0.) * self.1).clamp(0., 1.)
    }
    fn peak(&self, i: usize) -> f32 {
        (self.0.peaks.get(i).copied().unwrap_or(0.) * self.1).clamp(0., 1.)
    }
}

/// Draws `frame` in `area`; `seconds` moves the waves of a neon spectrum.
pub(crate) fn draw(
    layer: &mut Layer<'_>,
    area: BoxF,
    s: &Spectrum,
    frame: &SpectrumFrame,
    paint: &Paint,
    colors: Option<&ColorByValue>,
    seconds: f32,
) {
    if !s.valid() {
        return;
    }
    let bands = Bands(frame, s.gain);
    match s.style {
        SpectrumStyle::Mirrored => mirrored(layer, area, s, &bands, paint, colors),
        SpectrumStyle::Bars => bars(layer, area, s, &bands, paint, colors),
        SpectrumStyle::Blocks => blocks(layer, area, s, &bands, paint, colors),
        SpectrumStyle::Line => line(layer, area, s, &bands, paint, colors),
        SpectrumStyle::Radial => radial(layer, area, s, &bands, paint, colors),
        SpectrumStyle::Dancer => crate::dancer::draw(layer, area, s.dancer, dance_pose(frame)),
        SpectrumStyle::Neon => neon(layer, area, s, &bands, paint, colors, seconds),
    }
}

/// The paint of one band: its level's threshold color, else the element paint
/// spread over the whole widget, so a gradient runs across the bands.
fn band_paint(
    paint: &Paint,
    colors: Option<&ColorByValue>,
    level: f32,
    area: BoxF,
) -> Option<tiny_skia::Paint<'static>> {
    match colors {
        Some(c) => Some(solid(c.color(Some(f64::from(level) * 100.)))),
        None => paint_for(paint, area),
    }
}

/// Horizontal cell and bar width of each band.
fn columns(area: BoxF, s: &Spectrum) -> (f32, f32) {
    let cell = area.width / f32::from(s.bands);
    (cell, (cell - s.gap).min(s.thickness).max(0.))
}

fn mirrored(
    layer: &mut Layer<'_>,
    area: BoxF,
    s: &Spectrum,
    bands: &Bands<'_>,
    paint: &Paint,
    colors: Option<&ColorByValue>,
) {
    let (cell, width) = columns(area, s);
    let mid = area.y + area.height / 2.;
    if s.centre {
        shape::draw(
            layer,
            BoxF::new(area.x, mid - 0.5, area.width, 1.),
            ShapeKind::Rect { radius: 0. },
            Some(paint),
            None,
        );
    }
    for i in 0..usize::from(s.bands) {
        let value = bands.level(i);
        let peak = bands.peak(i);
        let color = colors.map(|c| Paint::solid(c.color(Some(f64::from(value) * 100.))));
        let paint = color.as_ref().unwrap_or(paint);
        let x = area.x + i as f32 * cell + (cell - width) / 2.;
        let height = value * (area.height / 2. - 2.).max(0.);
        if height > 0. {
            shape::draw(
                layer,
                BoxF::new(x, mid - height, width, height * 2.),
                ShapeKind::Corners(s.corners),
                Some(paint),
                None,
            );
        }
        if s.peaks && peak > 0. {
            let y = peak * (area.height / 2. - 2.).max(0.);
            for top in [mid - y - 1., mid + y] {
                shape::draw(
                    layer,
                    BoxF::new(x, top, width, 1.),
                    ShapeKind::Rect { radius: 0. },
                    Some(paint),
                    None,
                );
            }
        }
    }
}

/// A 1 px line along the bottom edge.
fn baseline(layer: &mut Layer<'_>, area: BoxF, s: &Spectrum, paint: &Paint) {
    if !s.centre {
        return;
    }
    let bottom = area.y + area.height - 1.;
    if let (Some(p), Some(path)) = (
        paint_for(paint, area),
        rounded_corners(area.x, bottom, area.width, 1., [0.; 4]),
    ) {
        layer.fill(&path, &p);
    }
}

fn bars(
    layer: &mut Layer<'_>,
    area: BoxF,
    s: &Spectrum,
    bands: &Bands<'_>,
    paint: &Paint,
    colors: Option<&ColorByValue>,
) {
    let (cell, width) = columns(area, s);
    let bottom = area.y + area.height;
    let full = (area.height - 3.).max(0.);
    baseline(layer, area, s, paint);
    for i in 0..usize::from(s.bands) {
        let value = bands.level(i);
        let x = area.x + i as f32 * cell + (cell - width) / 2.;
        let Some(p) = band_paint(paint, colors, value, area) else {
            continue;
        };
        let height = value * full;
        if height > 0.
            && let Some(path) = rounded_corners(x, bottom - height, width, height, s.corners)
        {
            layer.fill(&path, &p);
        }
        let peak = bands.peak(i);
        if s.peaks
            && peak > 0.
            && let Some(path) = rounded_corners(x, bottom - peak * full - 2., width, 1., [0.; 4])
        {
            layer.fill(&path, &p);
        }
    }
}

fn blocks(
    layer: &mut Layer<'_>,
    area: BoxF,
    s: &Spectrum,
    bands: &Bands<'_>,
    paint: &Paint,
    colors: Option<&ColorByValue>,
) {
    let (cell, width) = columns(area, s);
    let bottom = area.y + area.height;
    // Blocks keep at least one pixel between them, and never more than their own height.
    let space = s.gap.clamp(1., s.block);
    let pitch = s.block + space;
    let count = ((area.height + space) / pitch).floor().max(0.) as usize;
    if count == 0 {
        return;
    }
    // A block is colored by its own height, as on a hardware level meter;
    // an unlit one takes the unlit colour.
    let block = |layer: &mut Layer<'_>, x: f32, k: usize, unlit: Option<Rgba>| {
        let level = (k + 1) as f32 / count as f32;
        let top = bottom - k as f32 * pitch - s.block;
        let p = match unlit {
            Some(c) => Some(solid(c)),
            None => band_paint(paint, colors, level, area),
        };
        if let (Some(p), Some(path)) = (p, rounded_corners(x, top, width, s.block, s.corners)) {
            layer.fill(&path, &p);
        }
    };
    for i in 0..usize::from(s.bands) {
        let x = area.x + i as f32 * cell + (cell - width) / 2.;
        let lit = ((bands.level(i) * count as f32).round() as usize).min(count);
        let peak = ((bands.peak(i) * count as f32).round() as usize).min(count);
        let held = (s.peaks && peak > lit).then(|| peak - 1);
        for k in 0..count {
            if k < lit || held == Some(k) {
                block(layer, x, k, None);
            } else if s.unlit.is_some() {
                block(layer, x, k, s.unlit);
            }
        }
    }
}

/// Waves across a neon spectrum.
const WAVES: usize = 7;
/// Periods of a wave across the widget.
const WAVE_PERIODS: f32 = 3.;
/// How fast the waves travel, radians a second.
const WAVE_SPEED: f32 = 1.2;
/// Opacity of the tips of neon bars; the centre is opaque.
const NEON_TIPS: f32 = 0.1;
/// Halos around a neon bar: how much wider than the bar, and how opaque.
const NEON_HALOS: [(f32, f32); 3] = [(2.4, 0.07), (1.2, 0.1), (0.5, 0.16)];

fn neon(
    layer: &mut Layer<'_>,
    area: BoxF,
    s: &Spectrum,
    bands: &Bands<'_>,
    paint: &Paint,
    colors: Option<&ColorByValue>,
    seconds: f32,
) {
    let (cell, width) = columns(area, s);
    let mid = area.y + area.height / 2.;
    let half = (area.height / 2. - 2.).max(0.);
    for i in 0..usize::from(s.bands) {
        let value = bands.level(i);
        let height = value * half;
        let Some(p) = band_paint(paint, colors, value, area).filter(|_| height > 0.) else {
            continue;
        };
        let x = area.x + i as f32 * cell + (cell - width) / 2.;
        for (grow, opacity) in NEON_HALOS {
            let extra = width * grow;
            let mut halo = p.clone();
            halo.shader.apply_opacity(opacity);
            if let Some(path) = rounded_corners(
                x - extra / 2.,
                mid - height - extra / 2.,
                width + extra,
                height * 2. + extra,
                [(width + extra) / 2.; 4],
            ) {
                layer.fill(&path, &halo);
            }
        }
        if let Some(path) = rounded_corners(x, mid - height, width, height * 2., s.corners) {
            layer.fill(&path, &p);
        }
    }
    if s.centre {
        shape::draw(
            layer,
            BoxF::new(area.x, mid - 0.5, area.width, 1.),
            ShapeKind::Rect { radius: 0. },
            Some(paint),
            None,
        );
    }
    fade_from(layer, mid, half);
    if !s.waves || half <= 0. {
        return;
    }
    let count = usize::from(s.bands);
    // The bands' levels between their centres, so a wave swells over loud ones.
    let swell = |x: f32| {
        let at = ((x - area.x) / cell - 0.5).clamp(0., (count - 1) as f32);
        let (i, t) = (at.floor() as usize, at.fract());
        bands.level(i) * (1. - t) + bands.level((i + 1).min(count - 1)) * t
    };
    let loudest = (0..count).map(|i| bands.level(i)).fold(0., f32::max);
    let Some(mut p) = band_paint(paint, colors, loudest, area) else {
        return;
    };
    p.shader.apply_opacity(0.8);
    let stroke = Stroke {
        width: 1.,
        line_cap: LineCap::Round,
        line_join: LineJoin::Round,
        ..Stroke::default()
    };
    let length = std::f32::consts::TAU * WAVE_PERIODS / area.width.max(1.);
    let steps = (area.width / 2.).ceil().max(1.) as usize;
    for k in 0..WAVES {
        let phase = seconds * WAVE_SPEED + k as f32 * 0.5;
        let reach = half * 0.8 * (1. - k as f32 * 0.07);
        let mut wave = PathBuilder::new();
        for j in 0..=steps {
            let x = area.x + area.width * j as f32 / steps as f32;
            let y = mid + swell(x) * reach * ((x - area.x) * length + phase).sin();
            if j == 0 {
                wave.move_to(x, y);
            } else {
                wave.line_to(x, y);
            }
        }
        if let Some(path) = wave.finish() {
            layer.stroke(&path, &p, &stroke);
        }
    }
}

/// Fades the layer from opaque on the line at `mid` to [`NEON_TIPS`] at
/// `half` above and below it.
fn fade_from(layer: &mut Layer<'_>, mid: f32, half: f32) {
    let stride = layer.pixmap.width() as usize * 4;
    let top = layer.y as f32;
    for (row, pixels) in layer.pixmap.data_mut().chunks_mut(stride).enumerate() {
        let away = ((top + row as f32 + 0.5 - mid).abs() / half.max(1.)).min(1.);
        let keep = 1. - (1. - NEON_TIPS) * away.powf(0.8);
        for channel in pixels {
            *channel = (f32::from(*channel) * keep).round() as u8;
        }
    }
}

fn line(
    layer: &mut Layer<'_>,
    area: BoxF,
    s: &Spectrum,
    bands: &Bands<'_>,
    paint: &Paint,
    colors: Option<&ColorByValue>,
) {
    let count = usize::from(s.bands);
    let cell = area.width / count as f32;
    let bottom = area.y + area.height;
    let room = (area.height - s.stroke - 1.).max(0.);
    let points: Vec<(f32, f32)> = (0..count)
        .map(|i| {
            (
                area.x + (i as f32 + 0.5) * cell,
                bottom - s.stroke / 2. - bands.level(i) * room,
            )
        })
        .collect();
    let loudest = (0..count).map(|i| bands.level(i)).fold(0., f32::max);
    let Some(p) = band_paint(paint, colors, loudest, area) else {
        return;
    };
    // A quadratic through the midpoints: smooth, and never overshooting a band.
    let mut curve = PathBuilder::new();
    let (first, last) = (points[0], points[count - 1]);
    curve.move_to(area.x, first.1);
    curve.line_to(first.0, first.1);
    for pair in points.windows(2) {
        let (a, b) = (pair[0], pair[1]);
        curve.quad_to(a.0, a.1, (a.0 + b.0) / 2., (a.1 + b.1) / 2.);
    }
    curve.line_to(last.0, last.1);
    curve.line_to(area.x + area.width, last.1);
    let Some(stroke_path) = curve.clone().finish() else {
        return;
    };
    if s.fill {
        curve.line_to(area.x + area.width, bottom);
        curve.line_to(area.x, bottom);
        curve.close();
        if let Some(fill) = curve.finish() {
            let mut soft = p.clone();
            soft.shader.apply_opacity(FILL_OPACITY);
            layer.fill(&fill, &soft);
        }
    }
    baseline(layer, area, s, paint);
    let stroke = Stroke {
        width: s.stroke,
        line_cap: LineCap::Round,
        line_join: LineJoin::Round,
        ..Stroke::default()
    };
    layer.stroke(&stroke_path, &p, &stroke);
}
/// The area below a line spectrum, under its curve.
const FILL_OPACITY: f32 = 0.35;

fn radial(
    layer: &mut Layer<'_>,
    area: BoxF,
    s: &Spectrum,
    bands: &Bands<'_>,
    paint: &Paint,
    colors: Option<&ColorByValue>,
) {
    let count = usize::from(s.bands);
    let (cx, cy) = (area.x + area.width / 2., area.y + area.height / 2.);
    let outer = (area.width.min(area.height) / 2. - 3.).max(0.);
    let inner = outer * s.inner / 100.;
    let room = outer - inner;
    if room <= 0. {
        return;
    }
    let step = 360. / count as f32;
    // Bars are closest together on the inner ring, or halfway out around a tiny ring.
    let radius = inner.max(room / 2.);
    let width = (std::f32::consts::TAU * radius / count as f32 - s.gap)
        .min(s.thickness)
        .max(0.5);
    if s.centre
        && inner >= 1.
        && let (Some(p), Some(ring)) = (
            paint_for(paint, area),
            PathBuilder::from_circle(cx, cy, inner),
        )
    {
        let stroke = Stroke {
            width: 1.,
            ..Stroke::default()
        };
        layer.stroke(&ring, &p, &stroke);
    }
    for i in 0..count {
        let value = bands.level(i);
        let Some(p) = band_paint(paint, colors, value, area) else {
            continue;
        };
        // Band 0 points up; the others follow clockwise.
        let turn = Transform::from_rotate(i as f32 * step).post_translate(cx, cy);
        let length = value * room;
        if length > 0.
            && let Some(path) =
                rounded_corners(-width / 2., -inner - length, width, length, s.corners)
                    .and_then(|path| path.transform(turn))
        {
            layer.fill(&path, &p);
        }
        let peak = bands.peak(i);
        if s.peaks
            && peak > 0.
            && let Some(path) =
                rounded_corners(-width / 2., -inner - peak * room - 2., width, 1., [0.; 4])
                    .and_then(|path| path.transform(turn))
        {
            layer.fill(&path, &p);
        }
    }
}

pub(crate) fn label(state: AudioState, language: Language) -> &'static str {
    use AudioState::*;
    use Language::*;
    match (state, language) {
        (Live, _) => "",
        (Demo, Italian) => "Demo",
        (Demo, PortugueseBr) => "Demonstração",
        (Demo, English) => "Demo",
        (Disabled, Italian) => "Cattura audio disabilitata",
        (Disabled, PortugueseBr) => "Captura de áudio desativada",
        (Disabled, English) => "Audio capture disabled",
        (PlayerMissing, Italian) => "Player mancante",
        (PlayerMissing, PortugueseBr) => "Player ausente",
        (PlayerMissing, English) => "Player missing",
        (Unavailable, Italian) => "Sorgente non disponibile",
        (Unavailable, PortugueseBr) => "Fonte indisponível",
        (Unavailable, English) => "Source unavailable",
        (Unsupported, Italian) => "Non supportato su questa versione di Windows",
        (Unsupported, PortugueseBr) => "Não compatível com esta versão do Windows",
        (Unsupported, English) => "Unsupported on this version of Windows",
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::testkit::{self, Scene};
    use bezel_core::domain::{
        frame::{Frame, Rgba},
        theme::{Background, ElementId, ElementKind, TextContent, TextStyle},
        value_color::ColorThreshold,
    };
    #[test]
    fn mirrored_bands_use_existing_thresholds_and_change_only_inside_the_widget() {
        let config = Spectrum {
            bands: 8,
            peaks: false,
            centre: false,
            corners: [0.; 4],
            ..Default::default()
        };
        let mut element = testkit::element(
            BoxF::new(40., 20., 80., 60.),
            ElementKind::Text {
                content: TextContent::Spectrum(config),
                style: TextStyle::default(),
            },
        );
        let green = Rgba::opaque(0, 255, 0);
        let red = Rgba::opaque(255, 0, 0);
        element.color_by_value = Some(ColorByValue {
            sensor: None,
            base: green,
            thresholds: vec![ColorThreshold {
                value: 50.,
                color: red,
            }],
        });
        let theme = testkit::theme(160, 100, Background::Color(Rgba::BLACK), vec![element]);
        let mut scene = Scene::empty();
        let mut levels = vec![0.2; 8];
        levels[1] = 0.8;
        scene.snapshot.spectra.insert(
            ElementId(1),
            SpectrumFrame {
                state: AudioState::Live,
                levels,
                peaks: vec![],
                beat: None,
            },
        );
        let mut renderer = testkit::renderer();
        let before = testkit::render(&mut renderer, &theme, &scene);
        assert_eq!(before.pixel(45, 49), Some(green));
        assert_eq!(before.pixel(55, 39), Some(red));
        assert_eq!(before.pixel(55, 60), Some(red));
        scene
            .snapshot
            .spectra
            .get_mut(&ElementId(1))
            .unwrap()
            .levels
            .fill(0.);
        let after = testkit::render(&mut renderer, &theme, &scene);
        for y in 0..100 {
            for x in 0..160 {
                if before.pixel(x, y) != after.pixel(x, y) {
                    assert!((40..120).contains(&x) && (20..80).contains(&y));
                }
            }
        }
        assert_eq!(after.pixel(55, 39), Some(Rgba::BLACK));
    }
    fn green() -> Rgba {
        Rgba::opaque(0, 255, 0)
    }
    fn red() -> Rgba {
        Rgba::opaque(255, 0, 0)
    }
    /// One style in a 160x100 black canvas; colors by level when `thresholds`.
    fn styled(
        config: Spectrum,
        frame: BoxF,
        levels: Vec<f32>,
        thresholds: bool,
    ) -> bezel_core::domain::frame::Frame {
        let mut element = testkit::element(
            frame,
            ElementKind::Text {
                content: TextContent::Spectrum(config),
                style: TextStyle::default(),
            },
        );
        element.color_by_value = thresholds.then(|| ColorByValue {
            sensor: None,
            base: green(),
            thresholds: vec![ColorThreshold {
                value: 50.,
                color: red(),
            }],
        });
        let theme = testkit::theme(160, 100, Background::Color(Rgba::BLACK), vec![element]);
        let mut scene = Scene::empty();
        scene.snapshot.spectra.insert(
            ElementId(1),
            SpectrumFrame {
                state: AudioState::Live,
                levels,
                peaks: vec![],
                beat: None,
            },
        );
        testkit::render(&mut testkit::renderer(), &theme, &scene)
    }
    fn plain(style: SpectrumStyle) -> Spectrum {
        Spectrum {
            style,
            bands: 8,
            peaks: false,
            centre: false,
            corners: [0.; 4],
            ..Default::default()
        }
    }
    const WIDGET: BoxF = BoxF {
        x: 40.,
        y: 20.,
        width: 80.,
        height: 60.,
    };
    fn two_bands() -> Vec<f32> {
        let mut levels = vec![0.2; 8];
        levels[1] = 0.8;
        levels
    }
    #[test]
    fn bars_rise_from_the_bottom_edge_with_band_thresholds() {
        let f = styled(plain(SpectrumStyle::Bars), WIDGET, two_bands(), true);
        assert_eq!(f.pixel(45, 75), Some(green()));
        assert_eq!(f.pixel(45, 60), Some(Rgba::BLACK));
        assert_eq!(f.pixel(55, 40), Some(red()));
        assert_eq!(f.pixel(55, 30), Some(Rgba::BLACK));
    }
    #[test]
    fn blocks_are_separated_and_colored_by_their_own_height() {
        let f = styled(plain(SpectrumStyle::Blocks), WIDGET, two_bands(), true);
        // 7 blocks of 6 px every 9 px; 0.8 lights 6, 0.2 lights 1.
        assert_eq!(f.pixel(55, 77), Some(green()));
        assert_eq!(f.pixel(55, 72), Some(Rgba::BLACK));
        assert_eq!(f.pixel(55, 32), Some(red()));
        assert_eq!(f.pixel(55, 25), Some(Rgba::BLACK));
        assert_eq!(f.pixel(45, 77), Some(green()));
        assert_eq!(f.pixel(45, 68), Some(Rgba::BLACK));
    }
    #[test]
    fn line_draws_a_curve_with_a_softer_fill_below() {
        let f = styled(plain(SpectrumStyle::Line), WIDGET, vec![0.4; 8], true);
        // Curve at 80 - 1 - 0.4 * 57 = 56.2; the fill below is the same color, fainter.
        assert_eq!(f.pixel(80, 56), Some(green()));
        let below = f.pixel(80, 70).unwrap();
        assert!(below.g > 0 && below.g < 255 && below.r == 0, "{below:?}");
        assert_eq!(f.pixel(80, 40), Some(Rgba::BLACK));
        let open = Spectrum {
            fill: false,
            ..plain(SpectrumStyle::Line)
        };
        let f = styled(open, WIDGET, vec![0.4; 8], true);
        assert_eq!(f.pixel(80, 70), Some(Rgba::BLACK));
    }
    #[test]
    fn radial_bars_start_at_the_top_and_turn_clockwise() {
        let square = BoxF::new(50., 20., 60., 60.);
        let mut levels = vec![0.; 8];
        levels[0] = 1.;
        levels[4] = 1.;
        let f = styled(plain(SpectrumStyle::Radial), square, levels, true);
        // Centre (80, 50), outer radius 27, inner ring 40% of it.
        assert_eq!(f.pixel(80, 30), Some(red()));
        assert_eq!(f.pixel(80, 70), Some(red()));
        assert_eq!(f.pixel(100, 50), Some(Rgba::BLACK));
        assert_eq!(f.pixel(80, 50), Some(Rgba::BLACK));
    }
    #[test]
    fn every_style_stays_inside_the_widget() {
        for style in [
            SpectrumStyle::Bars,
            SpectrumStyle::Blocks,
            SpectrumStyle::Line,
            SpectrumStyle::Radial,
            SpectrumStyle::Neon,
        ] {
            let config = Spectrum {
                style,
                peaks: true,
                centre: true,
                ..Default::default()
            };
            let f = styled(config, WIDGET, vec![1.; 32], false);
            for y in 0..100 {
                for x in 0..160 {
                    if f.pixel(x, y) != Some(Rgba::BLACK) {
                        assert!(
                            (40..120).contains(&x) && (20..80).contains(&y),
                            "{style:?} at {x},{y}"
                        );
                    }
                }
            }
        }
    }
    /// `config` in [`WIDGET`] drawing `frame` at `seconds`, colored by level.
    fn framed(config: Spectrum, frame: SpectrumFrame, seconds: f32) -> Frame {
        let mut element = testkit::element(
            WIDGET,
            ElementKind::Text {
                content: TextContent::Spectrum(config),
                style: TextStyle::default(),
            },
        );
        element.color_by_value = Some(ColorByValue {
            sensor: None,
            base: green(),
            thresholds: vec![ColorThreshold {
                value: 50.,
                color: red(),
            }],
        });
        let theme = testkit::theme(160, 100, Background::Color(Rgba::BLACK), vec![element]);
        let mut scene = Scene::empty();
        scene.animation = std::time::Duration::from_secs_f32(seconds);
        scene.snapshot.spectra.insert(ElementId(1), frame);
        testkit::render(&mut testkit::renderer(), &theme, &scene)
    }
    fn live(levels: Vec<f32>, peaks: Vec<f32>) -> SpectrumFrame {
        SpectrumFrame {
            state: AudioState::Live,
            levels,
            peaks,
            beat: None,
        }
    }
    #[test]
    fn unlit_blocks_show_their_colour_and_a_held_peak_lights_its_own() {
        let blue = Rgba::opaque(0, 0, 255);
        let config = Spectrum {
            unlit: Some(blue),
            peaks: true,
            ..plain(SpectrumStyle::Blocks)
        };
        let mut peaks = vec![0.; 8];
        peaks[0] = 0.8;
        let f = framed(config.clone(), live(vec![0.2; 8], peaks.clone()), 0.);
        // Band 0: block 0 lit, blocks 1..=4 unlit, block 5 the held peak, block 6 unlit.
        assert_eq!(f.pixel(45, 77), Some(green()));
        assert_eq!(f.pixel(45, 68), Some(blue));
        assert_eq!(
            f.pixel(45, 72),
            Some(Rgba::BLACK),
            "the space between blocks"
        );
        assert_eq!(f.pixel(45, 32), Some(red()));
        assert_eq!(f.pixel(45, 23), Some(blue));
        let dark = Spectrum {
            unlit: None,
            ..config
        };
        let f = framed(dark, live(vec![0.2; 8], peaks), 0.);
        assert_eq!(f.pixel(45, 68), Some(Rgba::BLACK));
        assert_eq!(f.pixel(45, 32), Some(red()));
    }
    #[test]
    fn neon_bars_glow_and_fade_towards_their_tips() {
        let config = Spectrum {
            waves: false,
            ..plain(SpectrumStyle::Neon)
        };
        let mut levels = vec![0.; 8];
        levels[1] = 0.8;
        let f = framed(config, live(levels, vec![]), 0.);
        // Band 1 spans x 51.5..58.5 and y 27.6..72.4 around the centre line at 50.
        let core = f.pixel(55, 50).unwrap();
        assert!(core.r > 230 && core.g == 0, "bright core {core:?}");
        let tip = f.pixel(55, 29).unwrap();
        assert!(tip.r > 60 && tip.r < 180 && tip.g == 0, "faded tip {tip:?}");
        assert_eq!(f.pixel(55, 71), f.pixel(55, 28), "mirrored");
        let halo = f.pixel(50, 50).unwrap();
        assert!(halo.r > 20 && halo.r < 120 && halo.g == 0, "halo {halo:?}");
        assert_eq!(f.pixel(41, 50), Some(Rgba::BLACK), "beyond the widest halo");
    }
    #[test]
    fn neon_waves_swell_with_the_music_and_travel() {
        let config = plain(SpectrumStyle::Neon);
        let loud = |seconds| framed(config.clone(), live(vec![0.5; 8], vec![]), seconds);
        assert_ne!(loud(0.), loud(1.), "the waves travel");
        let silent = |seconds| framed(config.clone(), live(vec![0.; 8], vec![]), seconds);
        assert_eq!(silent(0.), silent(1.), "flat in silence");
        let still = Spectrum {
            waves: false,
            ..plain(SpectrumStyle::Neon)
        };
        assert_eq!(
            framed(still.clone(), live(vec![0.5; 8], vec![]), 0.),
            framed(still, live(vec![0.5; 8], vec![]), 1.),
            "no waves, nothing moves"
        );
    }
    #[test]
    fn the_gain_scales_every_band() {
        let half = Spectrum {
            gain: 0.5,
            ..plain(SpectrumStyle::Bars)
        };
        let f = styled(half, WIDGET, two_bands(), true);
        // 0.8 at half gain reads 0.4: below the 50 threshold and 22.8 px tall.
        assert_eq!(f.pixel(55, 70), Some(green()));
        assert_eq!(f.pixel(55, 50), Some(Rgba::BLACK));
        let double = Spectrum {
            gain: 4.,
            ..plain(SpectrumStyle::Bars)
        };
        let f = styled(double, WIDGET, two_bands(), true);
        // 0.2 at four times reads 0.8.
        assert_eq!(f.pixel(45, 40), Some(red()));
    }
    #[test]
    fn a_spectrum_without_a_result_is_silent_not_disabled() {
        let element = testkit::element(
            WIDGET,
            ElementKind::Text {
                content: TextContent::Spectrum(plain(SpectrumStyle::Mirrored)),
                style: TextStyle::default(),
            },
        );
        let theme = testkit::theme(160, 100, Background::Color(Rgba::BLACK), vec![element]);
        let f = testkit::render(&mut testkit::renderer(), &theme, &Scene::empty());
        for y in 0..100 {
            for x in 0..160 {
                assert_eq!(f.pixel(x, y), Some(Rgba::BLACK), "{x},{y}");
            }
        }
    }
}
