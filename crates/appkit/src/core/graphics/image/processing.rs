//! Image processing: spritesheet extraction and supersampled rasterization.

use crate::graphics::NSSize;
use super::format_types::NSImage;

/// Render a spritesheet SVG (viewBox > 200) by finding content bounds and scaling.
pub(super) fn render_spritesheet(
    tree: &usvg::Tree,
    physical_w: f32,
    physical_h: f32,
    name: &str,
    target_size: NSSize,
) -> Option<NSImage> {
    let (tree_w, tree_h) = (tree.size.width(), tree.size.height());
    let render_w = (tree_w as u32).min(2000);
    let render_h = (tree_h as u32).min(2000);
    let mut full_pixmap = tiny_skia::Pixmap::new(render_w, render_h)?;
    resvg::render(tree, tiny_skia::Transform::identity(), &mut full_pixmap.as_mut());

    // Find bounding box of non-transparent pixels
    let data = full_pixmap.data();
    let (mut min_x, mut min_y) = (render_w as i32, render_h as i32);
    let (mut max_x, mut max_y) = (0i32, 0i32);
    for y in 0..render_h {
        for x in 0..render_w {
            let idx = ((y * render_w + x) * 4 + 3) as usize;
            if idx < data.len() && data[idx] > 0 {
                min_x = std::cmp::min(min_x, x as i32);
                min_y = std::cmp::min(min_y, y as i32);
                max_x = std::cmp::max(max_x, x as i32);
                max_y = std::cmp::max(max_y, y as i32);
            }
        }
    }

    if max_x <= min_x || max_y <= min_y {
        println!("[NSImage] SVG '{}' rendered blank at 1:1 ({}x{})", name, render_w, render_h);
        return render_fallback_rect(physical_w, physical_h, name, target_size);
    }

    // Add 1px padding
    min_x = (min_x - 1).max(0);
    min_y = (min_y - 1).max(0);
    max_x = (max_x + 1).min(render_w as i32 - 1);
    max_y = (max_y + 1).min(render_h as i32 - 1);
    let crop_w = (max_x - min_x + 1) as u32;
    let crop_h = (max_y - min_y + 1) as u32;

    // Crop and scale to target (at physical resolution)
    let tw = physical_w as u32;
    let th = physical_h as u32;
    let mut target_pixmap = tiny_skia::Pixmap::new(tw, th)?;
    let scale = (tw as f32 / crop_w as f32).min(th as f32 / crop_h as f32);
    let scaled_w = (crop_w as f32 * scale) as u32;
    let scaled_h = (crop_h as f32 * scale) as u32;
    let offset_x = (tw - scaled_w) / 2;
    let offset_y = (th - scaled_h) / 2;

    for ty in 0..scaled_h.min(th) {
        for tx in 0..scaled_w.min(tw) {
            let src_x = (tx as f32 / scale) as u32 + min_x as u32;
            let src_y = (ty as f32 / scale) as u32 + min_y as u32;
            if src_x < render_w && src_y < render_h {
                let si = ((src_y * render_w + src_x) * 4) as usize;
                let di = (((ty + offset_y) * tw + (tx + offset_x)) * 4) as usize;
                if si + 3 < data.len() && di + 3 < target_pixmap.data().len() {
                    target_pixmap.data_mut()[di] = data[si];
                    target_pixmap.data_mut()[di + 1] = data[si + 1];
                    target_pixmap.data_mut()[di + 2] = data[si + 2];
                    target_pixmap.data_mut()[di + 3] = data[si + 3];
                }
            }
        }
    }

    Some(NSImage {
        name: name.to_string(),
        size: target_size,
        physical_size: NSSize::new(tw as f32, th as f32),
        pixels: target_pixmap.data().to_vec(),
    })
}

/// Render a normal SVG at 4× supersample then box-filter downsample.
pub(super) fn render_normal(
    tree: &usvg::Tree,
    physical_w: f32,
    physical_h: f32,
    tree_w: f32,
    tree_h: f32,
    name: &str,
    target_size: NSSize,
) -> Option<NSImage> {
    let supersample = 4u32;
    let ss_w = (physical_w as u32) * supersample;
    let ss_h = (physical_h as u32) * supersample;
    let mut hi_res_pixmap = tiny_skia::Pixmap::new(ss_w, ss_h)?;
    let sx = ss_w as f32 / tree_w;
    let sy = ss_h as f32 / tree_h;
    resvg::render(tree, tiny_skia::Transform::from_scale(sx, sy), &mut hi_res_pixmap.as_mut());

    // Box-filter downsample
    let final_w = physical_w as u32;
    let final_h = physical_h as u32;
    let mut pixmap = tiny_skia::Pixmap::new(final_w, final_h)?;
    let hi_data = hi_res_pixmap.data();
    let count = supersample * supersample;

    for y in 0..final_h {
        for x in 0..final_w {
            let (mut r_sum, mut g_sum, mut b_sum, mut a_sum) = (0u32, 0u32, 0u32, 0u32);
            for sy_off in 0..supersample {
                for sx_off in 0..supersample {
                    let src_x = x * supersample + sx_off;
                    let src_y = y * supersample + sy_off;
                    let idx = ((src_y * ss_w + src_x) * 4) as usize;
                    if idx + 3 < hi_data.len() {
                        r_sum += hi_data[idx] as u32;
                        g_sum += hi_data[idx + 1] as u32;
                        b_sum += hi_data[idx + 2] as u32;
                        a_sum += hi_data[idx + 3] as u32;
                    }
                }
            }
            let dst_idx = ((y * final_w + x) * 4) as usize;
            pixmap.data_mut()[dst_idx] = (r_sum / count) as u8;
            pixmap.data_mut()[dst_idx + 1] = (g_sum / count) as u8;
            pixmap.data_mut()[dst_idx + 2] = (b_sum / count) as u8;
            pixmap.data_mut()[dst_idx + 3] = (a_sum / count) as u8;
        }
    }

    // If blank, inject fallback
    if !pixmap.data().iter().any(|&p| p != 0) {
        println!("[NSImage] SVG '{}' rendered blank ({}x{}), drawing fallback", name, physical_w, physical_h);
        return render_fallback_rect(physical_w, physical_h, name, target_size);
    }

    Some(NSImage {
        name: name.to_string(),
        size: target_size,
        physical_size: NSSize::new(physical_w, physical_h),
        pixels: pixmap.data().to_vec(),
    })
}

/// Draw a tinted fallback rectangle when SVG rendering produces no content.
fn render_fallback_rect(
    physical_w: f32,
    physical_h: f32,
    name: &str,
    target_size: NSSize,
) -> Option<NSImage> {
    let tint = crate::theme::color(crate::theme::keys::CLR_TEXT_LABEL);
    let mut pixmap = tiny_skia::Pixmap::new(physical_w as u32, physical_h as u32)?;
    let mut paint = tiny_skia::Paint::default();
    paint.set_color_rgba8(
        (tint.r * 255.0) as u8,
        (tint.g * 255.0) as u8,
        (tint.b * 255.0) as u8,
        180,
    );
    let r = tiny_skia::Rect::from_xywh(2.0, 2.0, physical_w - 4.0, physical_h - 4.0)?;
    pixmap.fill_rect(r, &paint, tiny_skia::Transform::default(), None);
    Some(NSImage {
        name: name.to_string(),
        size: target_size,
        physical_size: NSSize::new(physical_w, physical_h),
        pixels: pixmap.data().to_vec(),
    })
}
