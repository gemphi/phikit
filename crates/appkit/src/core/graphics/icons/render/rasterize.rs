//! Direct-to-buffer icon rasterization for u32 ARGB compositing.

use super::cache::get_icon_pixels;
use super::super::IconId;

/// Free-function wrapper for buffer rasterization.
pub fn render_icon_to_buffer(
    id: &IconId,
    buffer: &mut [u32],
    buf_width: usize,
    x: usize,
    y: usize,
    icon_size: usize,
    tint: u32,
) {
    NSIconRasterizer::render_icon_to_buffer(id, buffer, buf_width, x, y, icon_size, tint);
}

pub struct NSIconRasterizer;

impl NSIconRasterizer {

/// Render icon pixels directly into a u32 ARGB buffer region.
/// The icon is rasterized at 2x internally and downsampled to fit the requested icon_size.
pub fn render_icon_to_buffer(
    id: &IconId,
    buffer: &mut [u32],
    buf_width: usize,
    x: usize,
    y: usize,
    icon_size: usize,
    tint: u32,
) {
    let (pixels, px_w, _px_h) = get_icon_pixels(id, icon_size as u32);
    let src_size = px_w as usize; // This is 2x icon_size due to retina rasterization
    let tint_r = ((tint >> 16) & 0xFF) as f32 / 255.0;
    let tint_g = ((tint >> 8) & 0xFF) as f32 / 255.0;
    let tint_b = (tint & 0xFF) as f32 / 255.0;

    for iy in 0..icon_size {
        for ix in 0..icon_size {
            // Sample from the 2x rasterized buffer (nearest-neighbor for speed)
            let src_x = ix * src_size / icon_size;
            let src_y = iy * src_size / icon_size;
            let src_idx = (src_y * src_size + src_x) * 4;
            if src_idx + 3 >= pixels.len() {
                continue;
            }
            let a = pixels[src_idx + 3] as f32 / 255.0;
            if a < 0.01 {
                continue;
            }

            let dst_x = x + ix;
            let dst_y = y + iy;
            if dst_x >= buf_width || dst_y * buf_width + dst_x >= buffer.len() {
                continue;
            }

            let r = (pixels[src_idx] as f32 / 255.0 * tint_r * 255.0) as u32;
            let g = (pixels[src_idx + 1] as f32 / 255.0 * tint_g * 255.0) as u32;
            let b = (pixels[src_idx + 2] as f32 / 255.0 * tint_b * 255.0) as u32;
            let alpha = (a * 255.0) as u32;

            let dst_idx = dst_y * buf_width + dst_x;
            buffer[dst_idx] = (alpha << 24) | (r << 16) | (g << 8) | b;
        }
    }
}

} // impl NSIconRasterizer
