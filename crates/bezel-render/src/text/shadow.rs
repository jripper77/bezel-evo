//! Centred text shadows. A separable box filter is also used by the demo.
use super::*;
use bezel_core::domain::theme::{Paint, TextShadow};

/// Blur a glyph mask with transparent pixels outside the layer. Integer
/// division after each pass keeps the native and demo filters identical.
fn blur(data: &[u8], width: usize, height: usize, radius: u8) -> Vec<u8> {
    let radius = usize::from(radius.min(64));
    let divisor = (radius * 2 + 1) as u32;
    let mut horizontal = vec![0; data.len()];
    let mut result = vec![0; data.len()];
    for y in 0..height {
        let mut sum: u32 = (0..=radius.min(width - 1))
            .map(|x| u32::from(data[y * width + x]))
            .sum();
        for x in 0..width {
            horizontal[y * width + x] = (sum / divisor) as u8;
            if x >= radius {
                sum -= u32::from(data[y * width + x - radius]);
            }
            if x + radius + 1 < width {
                sum += u32::from(data[y * width + x + radius + 1]);
            }
        }
    }
    for x in 0..width {
        let mut sum: u32 = (0..=radius.min(height - 1))
            .map(|y| u32::from(horizontal[y * width + x]))
            .sum();
        for y in 0..height {
            result[y * width + x] = (sum / divisor) as u8;
            if y >= radius {
                sum -= u32::from(horizontal[(y - radius) * width + x]);
            }
            if y + radius + 1 < height {
                sum += u32::from(horizontal[(y + radius + 1) * width + x]);
            }
        }
    }
    result
}

pub(super) fn draw(
    layer: &mut Layer<'_>,
    masks: impl Iterator<Item = Vec<u8>>,
    emoji: &[(Pixmap, i32, i32)],
    shadow: TextShadow,
) {
    if shadow.color.a == 0 {
        return;
    }
    let (w, h) = (layer.pixmap.width(), layer.pixmap.height());
    let mut coverage = vec![0; w as usize * h as usize];
    for mask in masks {
        for (a, b) in coverage.iter_mut().zip(mask) {
            *a = (*a).max(b);
        }
    }
    for (pixmap, x, y) in emoji {
        for row in 0..pixmap.height() as i32 {
            for col in 0..pixmap.width() as i32 {
                let (lx, ly) = (x - layer.x + col, y - layer.y + row);
                if lx >= 0 && ly >= 0 && lx < w as i32 && ly < h as i32 {
                    let at = ly as usize * w as usize + lx as usize;
                    coverage[at] = coverage[at].max(
                        pixmap.pixels()[(row as u32 * pixmap.width() + col as u32) as usize]
                            .alpha(),
                    );
                }
            }
        }
    }
    let (Some(size), Some(rect)) = (
        IntSize::from_wh(w, h),
        Rect::from_xywh(layer.x as f32, layer.y as f32, w as f32, h as f32),
    ) else {
        return;
    };
    let Some(mask) = Mask::from_vec(blur(&coverage, w as usize, h as usize, shadow.blur), size)
    else {
        return;
    };
    if let Some(paint) = paint_for(
        &Paint::Solid(shadow.color),
        BoxF {
            x: layer.x as f32,
            y: layer.y as f32,
            width: w as f32,
            height: h as f32,
        },
    ) {
        layer.fill_rect_masked(rect, &paint, &mask);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn blur_spreads_ink_and_treats_edges_as_transparent() {
        assert_eq!(blur(&[0, 0, 0, 0, 255, 0, 0, 0, 0], 3, 3, 1), vec![28; 9]);
        assert_eq!(blur(&[255], 1, 1, 1), vec![28]);
        assert_eq!(blur(&[12, 255, 0, 60], 2, 2, 0), vec![12, 255, 0, 60]);
    }
}
