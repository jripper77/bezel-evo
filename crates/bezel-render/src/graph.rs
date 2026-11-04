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
    /// Bars of square cells, and the colour of their unlit cells.
    pub pixel: Option<PixelCells>,
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

/// Square cells of pixel bars.
#[derive(Clone, Copy)]
pub(crate) struct PixelCells {
    /// Unlit cells; absent leaves them out.
    pub unlit: Option<Rgba>,
    /// Cell corner radius.
    pub radius: f32,
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

/// While a sample scrolls in, the visible window slides from `values[..n-1]`
/// to `values[1..]`: the range glides between the two windows' ranges with
/// the scroll, so autoscaling never snaps. Linear on purpose: the entering
/// and leaving edges move linearly too, so every visible point stays inside.
pub(crate) fn scrolling_range(
    values: &[Option<f64>],
    spec: &GraphSpec<'_>,
    range: impl Fn(&[Option<f64>]) -> Option<(f64, f64)>,
) -> Option<(f64, f64)> {
    if spec.scroll <= 0. || values.len() < 2 {
        return range(values);
    }
    let t = f64::from(1. - spec.scroll.min(1.));
    match (range(&values[..values.len() - 1]), range(&values[1..])) {
        (Some(a), Some(b)) => Some((a.0 + (b.0 - a.0) * t, a.1 + (b.1 - a.1) * t)),
        (a, b) => a.or(b),
    }
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
    let Some(range) = scrolling_range(values, spec, |v| value_range(v, spec)) else {
        return;
    };
    match spec.style {
        GraphStyle::Bars => match spec.pixel {
            Some(cells) => draw_pixel_bars(layer, area, values, spec, range, cells),
            None => draw_bars(layer, area, values, spec, range),
        },
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

/// While scrolling, the entering bar grows from its neighbour's height and
/// the leaving one settles towards its neighbour's, just as a line's clipped
/// edges do: the gliding range then always holds every visible bar, so none
/// is drawn against the old scale and cut off.
fn gliding_edges(values: &[Option<f64>], scroll: f32) -> Vec<Option<f64>> {
    let mut out = values.to_vec();
    let n = values.len();
    if scroll <= 0. || n < 2 {
        return out;
    }
    let t = f64::from(1. - scroll.min(1.));
    let towards = |from: Option<f64>, to: Option<f64>, t: f64| match (from, to) {
        (Some(a), Some(b)) => Some(a + (b - a) * t),
        (_, to) => to,
    };
    out[n - 1] = towards(values[n - 2], values[n - 1], t);
    if n > 2 {
        out[0] = towards(values[1], values[0], 1. - t);
    }
    out
}

/// Horizontal gap between bars of `slot` width.
fn bar_gap(spec: &GraphSpec<'_>, slot: f32) -> f32 {
    spec.bar_width
        .map(|width| slot - width.clamp(0., slot))
        .unwrap_or_else(|| {
            spec.gap
                .unwrap_or(if slot >= 4.0 { slot * 0.2 } else { 0.0 })
                .clamp(0., slot * 0.8)
        })
}

/// Bars as stacks of square cells: a cell is one bar wide and the cells are
/// as far apart as the bars. Every slot shows its unlit cells, so the graph
/// reads as a grid even before its history fills up.
fn draw_pixel_bars(
    layer: &mut Layer<'_>,
    area: BoxF,
    values: &[Option<f64>],
    spec: &GraphSpec<'_>,
    range: (f64, f64),
    cells: PixelCells,
) {
    let slots = usize::from(spec.history).max(1);
    let slot = area.width / slots as f32;
    let gap = bar_gap(spec, slot);
    let side = slot - gap;
    if !positive(side) {
        return;
    }
    let count = ((area.height + gap) / (side + gap)).floor() as usize;
    let radius = cells.radius.clamp(0., side / 2.);
    let right = area.x + area.width;
    let values = gliding_edges(values, spec.scroll);
    let (mut lit, mut unlit) = (PathBuilder::new(), PathBuilder::new());
    for k in 0..=slots {
        let x = right - (k + 1) as f32 * slot + gap / 2.0 + spec.scroll * slot;
        let left = x.max(area.x);
        let width = (x + side).min(right) - left;
        if !positive(width) {
            continue;
        }
        let on = values
            .len()
            .checked_sub(k + 1)
            .and_then(|i| values[i])
            .and_then(|v| fraction(v, range.0, range.1))
            .map_or(0, |f| (f * count as f64).round() as usize);
        for c in 0..count {
            let offset = c as f32 * (side + gap);
            let y = if spec.reverse {
                area.y + offset
            } else {
                area.y + area.height - offset - side
            };
            let target = if c < on {
                &mut lit
            } else if cells.unlit.is_some() {
                &mut unlit
            } else {
                break;
            };
            if let Some(cell) = crate::path::rounded_rect(left, y, width, side, radius) {
                target.push_path(&cell);
            }
        }
    }
    if let (Some(path), Some(color)) = (unlit.finish(), cells.unlit) {
        layer.fill(&path, &solid(color));
    }
    if let Some(path) = lit.finish() {
        layer.fill(&path, &solid(spec.color));
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
    let gap = bar_gap(spec, slot);
    let right = area.x + area.width;
    let bottom = area.y + area.height;
    let mut pb = PathBuilder::new();
    let values = gliding_edges(values, spec.scroll);
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
            pixel: None,
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
    fn autoscale_glides_with_the_scroll_and_never_snaps_between_samples() {
        let at = |values: &[Option<f64>], scroll: f32| {
            let spec = GraphSpec {
                scroll,
                ..spec(true)
            };
            scrolling_range(values, &spec, |v| value_range(v, &spec)).unwrap()
        };
        // A peak of 100 arrives: the range starts where the old window was.
        let before = [Some(5.), Some(10.), Some(20.), Some(30.)];
        let arriving = [Some(10.), Some(20.), Some(30.), Some(100.)];
        // Last frame of the previous sample vs first frame of the new one.
        let settled = at(&before, f32::MIN_POSITIVE);
        let start = at(&arriving, 1.);
        assert!((settled.0 - start.0).abs() < 1e-6 && (settled.1 - start.1).abs() < 1e-6);
        let half = at(&arriving, 0.5);
        assert!(half.1 > settled.1 && half.1 < value_range(&arriving[1..], &spec(true)).unwrap().1);
        assert_eq!(
            at(&arriving[1..], 0.),
            value_range(&arriving[1..], &spec(true)).unwrap()
        );
        // Without motion the range is still the plain window's.
        assert_eq!(at(&before, 0.), value_range(&before, &spec(true)).unwrap());
    }

    #[test]
    fn scrolling_bars_grow_in_and_settle_out_inside_the_gliding_range() {
        let values = [Some(5.), Some(10.), Some(20.), Some(1000.)];
        assert_eq!(
            gliding_edges(&values, 1.),
            vec![Some(5.), Some(10.), Some(20.), Some(20.)]
        );
        assert_eq!(
            gliding_edges(&values, 0.5),
            vec![Some(7.5), Some(10.), Some(20.), Some(510.)]
        );
        assert_eq!(gliding_edges(&values, 0.), values.to_vec());
        for scroll in [1., 0.75, 0.5, 0.25, 0.01] {
            let spec = GraphSpec {
                scroll,
                ..spec(true)
            };
            let (lo, hi) = scrolling_range(&values, &spec, |v| value_range(v, &spec)).unwrap();
            assert!(
                gliding_edges(&values, scroll)
                    .iter()
                    .flatten()
                    .all(|v| (lo..=hi).contains(v)),
                "every visible bar fits at scroll {scroll}"
            );
        }
    }

    #[test]
    fn pixel_bars_light_whole_square_cells_and_show_unlit_ones_only_with_a_colour() {
        let grey = Rgba::opaque(40, 40, 40);
        let render = |unlit: Option<Rgba>| {
            let mut pixels = tiny_skia::Pixmap::new(40, 40).unwrap();
            let mut layer = Layer {
                pixmap: pixels.as_mut(),
                x: 0,
                y: 0,
            };
            let spec = GraphSpec {
                bar_width: Some(10.),
                history: 2,
                style: GraphStyle::Bars,
                pixel: Some(PixelCells { unlit, radius: 0. }),
                ..spec(false)
            };
            // Two 20px slots of 10px cells, 10px apart: two cells per bar.
            draw(
                &mut layer,
                BoxF::new(0., 0., 40., 40.),
                &[Some(100.), Some(50.)],
                &spec,
            );
            pixels
        };
        let rgb = |p: &tiny_skia::Pixmap, x, y| {
            let c = p.pixel(x, y).unwrap();
            (c.red(), c.green(), c.blue(), c.alpha())
        };
        let pixels = render(Some(grey));
        assert_eq!(
            rgb(&pixels, 10, 35),
            (255, 255, 255, 255),
            "full bar, low cell"
        );
        assert_eq!(
            rgb(&pixels, 10, 15),
            (255, 255, 255, 255),
            "full bar, high cell"
        );
        assert_eq!(
            rgb(&pixels, 30, 35),
            (255, 255, 255, 255),
            "half bar, low cell"
        );
        assert_eq!(
            rgb(&pixels, 30, 15),
            (40, 40, 40, 255),
            "half bar, unlit cell"
        );
        assert_eq!(
            rgb(&pixels, 30, 25).3,
            0,
            "the gap between cells stays empty"
        );
        assert_eq!(rgb(&render(None), 30, 15).3, 0, "no colour, no unlit cells");
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
