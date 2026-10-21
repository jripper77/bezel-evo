//! History graphs: a line, a filled area or one bar per sample. The newest
//! sample sits on the right edge; gaps (unavailable readings) break the line.

use bezel_core::domain::frame::Rgba;
use bezel_core::domain::sensor::fraction;
use bezel_core::domain::theme::{BoxF, GraphStyle, Paint};
use tiny_skia::{LineCap, LineJoin, PathBuilder, Stroke};

use crate::layer::Layer;
use crate::paint::{paint_for, solid};
use crate::path::{positive, rect};

/// How a graph looks and scales.
#[derive(Clone, Copy)]
pub(crate) struct GraphSpec<'a> {
    /// Explicit graph bar width, centered in each sample slot.
    pub bar_width: Option<f32>,
    /// Rounded bar geometry.
    pub rounded: bool,
    /// Explicit bar gap, or original proportional spacing.
    pub gap: Option<f32>,
    /// Draw from the top baseline downwards for a lower series.
    pub reverse: bool,
    /// Remaining fraction of a sample entering from the right.
    pub scroll: f32,
    /// Samples across the width.
    pub history: u16,
    /// Drawing style.
    pub style: GraphStyle,
    /// Line and bar color.
    pub color: Rgba,
    /// Area fill (default: the line color, faded).
    pub fill: Option<&'a Paint>,
    /// Line width.
    pub line_width: f32,
    /// Scale to the visible samples.
    pub autoscale: bool,
    /// Binding range.
    pub min: f64,
    /// Binding range.
    pub max: f64,
}

/// Alpha of the default area fill.
const AREA_ALPHA: u8 = 72;

/// The value range the graph maps to its height: the binding range, or the
/// visible samples with a 10% margin when autoscaling.
pub(crate) fn value_range(values: &[Option<f64>], spec: &GraphSpec<'_>) -> Option<(f64, f64)> {
    if !spec.autoscale {
        return (spec.max > spec.min).then_some((spec.min, spec.max));
    }
    let mut seen = values.iter().flatten().copied().filter(|v| v.is_finite());
    let first = seen.next()?;
    let (lo, hi) = seen.fold((first, first), |(lo, hi), v| (lo.min(v), hi.max(v)));
    let span = hi - lo;
    let margin = if span > 0.0 {
        span * 0.1
    } else {
        (hi.abs() * 0.1).max(1.0)
    };
    Some((lo - margin, hi + margin))
}

/// Canvas points of each unbroken run of samples.
fn runs(
    area: BoxF,
    values: &[Option<f64>],
    spec: &GraphSpec<'_>,
    range: (f64, f64),
) -> Vec<Vec<(f32, f32)>> {
    let slots = usize::from(spec.history).max(2);
    let step = area.width / (slots - 1) as f32;
    let inset = (spec.line_width / 2.0).clamp(0.0, area.height / 2.0);
    let right = area.x + area.width;
    let bottom = area.y + area.height - inset;
    let span = area.height - 2.0 * inset;
    let mut out: Vec<Vec<(f32, f32)>> = vec![Vec::new()];
    for (i, v) in values.iter().enumerate() {
        let f = v.and_then(|v| fraction(v, range.0, range.1));
        match (f, out.last_mut()) {
            (Some(f), Some(run)) => {
                let x = right - (values.len() - 1 - i) as f32 * step + spec.scroll * step;
                run.push((
                    x,
                    if spec.reverse {
                        area.y + inset + f as f32 * span
                    } else {
                        bottom - f as f32 * span
                    },
                ));
            }
            _ => out.push(Vec::new()),
        }
    }
    out.retain(|r| !r.is_empty());
    if spec.scroll > 0. {
        for run in &mut out {
            *run = clip_run(run, area.x, right);
        }
        out.retain(|r| !r.is_empty());
    }
    out
}

// Interpolate only within an unbroken run, including its moving edge. No
// synthetic segment may cross an unavailable reading or leave the graph box.
fn clip_run(run: &[(f32, f32)], left: f32, right: f32) -> Vec<(f32, f32)> {
    if run.len() == 1 {
        return run
            .iter()
            .copied()
            .filter(|p| p.0 >= left && p.0 <= right)
            .collect();
    }
    let mut clipped = Vec::new();
    for pair in run.windows(2) {
        let (a, b) = (pair[0], pair[1]);
        if b.0 < left || a.0 > right {
            continue;
        }
        let point = |x: f32| (x, a.1 + (b.1 - a.1) * ((x - a.0) / (b.0 - a.0)));
        let start = if a.0 < left { point(left) } else { a };
        let end = if b.0 > right { point(right) } else { b };
        if clipped.last() != Some(&start) {
            clipped.push(start);
        }
        clipped.push(end);
    }
    clipped
}

/// Draws the graph of `values` (oldest first) in `area`.
pub(crate) fn draw(
    layer: &mut Layer<'_>,
    area: BoxF,
    values: &[Option<f64>],
    spec: &GraphSpec<'_>,
) {
    let Some(range) = value_range(values, spec) else {
        return;
    };
    match spec.style {
        GraphStyle::Bars => draw_bars(layer, area, values, spec, range),
        GraphStyle::Line => draw_lines(layer, &runs(area, values, spec, range), spec),
        GraphStyle::Area => {
            let runs = runs(area, values, spec, range);
            draw_area(layer, area, &runs, spec);
            draw_lines(layer, &runs, spec);
        }
    }
}

fn draw_lines(layer: &mut Layer<'_>, runs: &[Vec<(f32, f32)>], spec: &GraphSpec<'_>) {
    let width = spec.line_width;
    if !positive(width) {
        return;
    }
    let paint = solid(spec.color);
    let mut pb = PathBuilder::new();
    for run in runs {
        match run.as_slice() {
            [(x, y)] => pb.push_circle(*x, *y, width / 2.0),
            [(x, y), rest @ ..] => {
                pb.move_to(*x, *y);
                for (x, y) in rest {
                    pb.line_to(*x, *y);
                }
            }
            [] => {}
        }
    }
    let Some(path) = pb.finish() else {
        return;
    };
    let stroke = Stroke {
        width,
        line_cap: LineCap::Round,
        line_join: LineJoin::Round,
        ..Stroke::default()
    };
    layer.stroke(&path, &paint, &stroke);
}

fn draw_area(layer: &mut Layer<'_>, area: BoxF, runs: &[Vec<(f32, f32)>], spec: &GraphSpec<'_>) {
    let faded = Paint::Solid(Rgba {
        a: (u16::from(spec.color.a) * u16::from(AREA_ALPHA) / 255) as u8,
        ..spec.color
    });
    let Some(paint) = paint_for(spec.fill.unwrap_or(&faded), area) else {
        return;
    };
    let base = if spec.reverse {
        area.y
    } else {
        area.y + area.height
    };
    let mut pb = PathBuilder::new();
    for run in runs.iter().filter(|r| r.len() >= 2) {
        let (Some(first), Some(last)) = (run.first(), run.last()) else {
            continue;
        };
        pb.move_to(first.0, base);
        for (x, y) in run {
            pb.line_to(*x, *y);
        }
        pb.line_to(last.0, base);
        pb.close();
    }
    if let Some(path) = pb.finish() {
        layer.fill(&path, &paint);
    }
}

fn draw_bars(
    layer: &mut Layer<'_>,
    area: BoxF,
    values: &[Option<f64>],
    spec: &GraphSpec<'_>,
    range: (f64, f64),
) {
    let slots = usize::from(spec.history).max(1);
    let slot = area.width / slots as f32;
    let gap = spec
        .bar_width
        .map(|width| slot - width.clamp(0., slot))
        .unwrap_or_else(|| {
            spec.gap
                .unwrap_or(if slot >= 4.0 { slot * 0.2 } else { 0.0 })
                .clamp(0., slot * 0.8)
        });
    let right = area.x + area.width;
    let bottom = area.y + area.height;
    let mut pb = PathBuilder::new();
    for (i, v) in values.iter().enumerate() {
        let Some(f) = v.and_then(|v| fraction(v, range.0, range.1)) else {
            continue;
        };
        let height = f as f32 * area.height;
        let x = right - (values.len() - i) as f32 * slot + gap / 2.0 + spec.scroll * slot;
        let left = x.max(area.x);
        let width = (x + slot - gap).min(right) - left;
        let y = if spec.reverse {
            area.y
        } else {
            bottom - height
        };
        let shape = if spec.rounded {
            crate::path::rounded_rect(left, y, width, height, width / 2.)
        } else {
            rect(left, y, width, height)
        };
        if let Some(bar) = shape {
            pb.push_path(&bar);
        }
    }
    if let Some(path) = pb.finish() {
        layer.fill(&path, &solid(spec.color));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn motion_clips_the_interpolated_edge_and_never_bridges_a_gap() {
        let area = BoxF::new(10., 20., 120., 80.);
        let spec = GraphSpec {
            scroll: 0.5,
            ..spec(false)
        };
        let r = runs(
            area,
            &[Some(0.), Some(20.), None, Some(60.), Some(100.)],
            &spec,
            (0., 100.),
        );
        assert_eq!(r.len(), 2);
        assert!(r.iter().flatten().all(|p| p.0 >= 10. && p.0 <= 130.));
        let last = r.last().unwrap().last().unwrap();
        assert_eq!(last.0, 130.);
        assert!(
            (last.1 - 36.6).abs() < 0.01,
            "right edge interpolates 80%, not the future endpoint"
        );
    }

    fn spec(autoscale: bool) -> GraphSpec<'static> {
        GraphSpec {
            bar_width: None,
            rounded: false,
            gap: None,
            reverse: false,
            scroll: 0.,
            history: 4,
            style: GraphStyle::Line,
            color: Rgba::WHITE,
            fill: None,
            line_width: 2.0,
            autoscale,
            min: 0.0,
            max: 100.0,
        }
    }

    fn bars_pixels_with_width(bar_width: Option<f32>) -> tiny_skia::Pixmap {
        let mut pixels = tiny_skia::Pixmap::new(100, 80).unwrap();
        let mut layer = Layer {
            pixmap: pixels.as_mut(),
            x: 0,
            y: 0,
        };
        let spec = GraphSpec {
            bar_width,
            style: GraphStyle::Bars,
            ..spec(false)
        };
        draw(
            &mut layer,
            BoxF::new(0., 0., 100., 80.),
            &[Some(100.), Some(50.), None, Some(75.)],
            &spec,
        );
        pixels
    }

    #[test]
    fn explicit_bar_width_changes_exact_pixel_runs_and_stays_centered() {
        for width in [3, 11] {
            let pixels = bars_pixels_with_width(Some(width as f32));
            let row: Vec<u32> = (0..100)
                .filter(|x| pixels.pixel(*x, 60).unwrap().alpha() == 255)
                .collect();
            // Samples 0, 1 and 3 reach this row; each has exactly the requested width.
            let expected: Vec<u32> = [0, 1, 3]
                .into_iter()
                .flat_map(|slot| {
                    let left = slot * 25 + (25 - width) / 2;
                    left..left + width
                })
                .collect();
            assert_eq!(row, expected);
        }
    }

    #[test]
    fn legacy_bar_pixels_are_unchanged() {
        let pixels = bars_pixels_with_width(None);
        let checksum = pixels
            .data()
            .iter()
            .fold(0xcbf29ce484222325_u64, |hash, byte| {
                (hash ^ u64::from(*byte)).wrapping_mul(0x100000001b3)
            });
        // Captured from the pre-barWidth renderer (defd7b6); all RGBA bytes.
        assert_eq!(checksum, 1239134809160493365);
    }

    #[test]
    fn ranges_come_from_the_binding_or_the_samples() {
        let values = [Some(10.0), None, Some(30.0)];
        assert_eq!(value_range(&values, &spec(false)), Some((0.0, 100.0)));
        assert_eq!(value_range(&values, &spec(true)), Some((8.0, 32.0)));
        assert_eq!(value_range(&[Some(5.0)], &spec(true)), Some((4.0, 6.0)));
        assert_eq!(value_range(&[None], &spec(true)), None);
        let empty = GraphSpec {
            max: 0.0,
            ..spec(false)
        };
        assert_eq!(value_range(&values, &empty), None);
    }

    #[test]
    fn runs_break_at_gaps_and_end_on_the_right() {
        let area = BoxF::new(0.0, 0.0, 30.0, 12.0);
        let values = [Some(0.0), Some(100.0), None, Some(50.0)];
        let runs = runs(area, &values, &spec(false), (0.0, 100.0));
        assert_eq!(
            runs,
            vec![vec![(0.0, 11.0), (10.0, 1.0)], vec![(30.0, 6.0)]]
        );
    }
}
