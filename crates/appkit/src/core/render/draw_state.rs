//! Debug border rendering, color blending, and blit operations.

use crate::{NSColor, NSRect};
use super::render_context::RenderContext;

// ─── Debug Border Rendering ─────────────────────────────────────────────────

/// Global flag to enable/disable debug border rendering around every view.
/// When enabled, colored 1px borders are drawn around each view after its
/// normal content, making layout issues immediately visible.
pub static DEBUG_BORDERS: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Enable or disable debug border rendering.
pub fn set_debug_borders(enabled: bool) {
    DEBUG_BORDERS.store(enabled, std::sync::atomic::Ordering::Relaxed);
}

/// Check if debug border rendering is currently enabled.
pub fn debug_borders_enabled() -> bool {
    DEBUG_BORDERS.load(std::sync::atomic::Ordering::Relaxed)
}

/// Determine the debug border color based on the view's name/role.
/// - Red: Panel containers (Panel, Left, Right, Bottom, Inspector, Hierarchy)
/// - Yellow: Interactive controls (Button, Btn, Tab)
/// - Green: Content areas (Viewport, Content, Canvas)
/// - Cyan: Separators (Separator, Sep)
/// - Magenta: Everything else
pub fn debug_border_color_for_name(name: &str) -> NSColor {
    match name {
        n if n.contains("Panel")
            || n.contains("Left")
            || n.contains("Right")
            || n.contains("Bottom")
            || n.contains("Inspector")
            || n.contains("Hierarchy")
            || n.contains("Container")
            || n.contains("Split") =>
        {
            crate::theme::color(crate::theme::keys::CLR_RED)
        }
        n if n.contains("Button") || n.contains("Btn") || n.contains("Tab") => {
            crate::theme::color(crate::theme::keys::CLR_YELLOW)
        }
        n if n.contains("Viewport") || n.contains("Content") || n.contains("Canvas") => {
            crate::theme::color(crate::theme::keys::CLR_GREEN)
        }
        n if n.contains("Separator") || n.contains("Sep") => {
            crate::theme::color(crate::theme::keys::CLR_ACCENT)
        }
        _ => crate::theme::color(crate::theme::keys::CLR_SHADOW),
    }
}

// ─── Color Blending ─────────────────────────────────────────────────────────

/// Helper for alpha blending colors using fast integer math
pub(crate) fn blend_colors(src: NSColor, bg_u32: u32, alpha: f32) -> u32 {
    let alpha_int = (alpha * 255.0) as u32;
    match alpha_int >= 250 {
        true => src.to_bgra_u32(),
        false => match alpha_int <= 5 {
            true => bg_u32,
            false => {
                let src_r = (src.r * 255.0) as u32;
                let src_g = (src.g * 255.0) as u32;
                let src_b = (src.b * 255.0) as u32;

                let bg_r = (bg_u32 >> 16) & 0xFF;
                let bg_g = (bg_u32 >> 8) & 0xFF;
                let bg_b = bg_u32 & 0xFF;

                let r = ((src_r * alpha_int) + (bg_r * (255 - alpha_int))) >> 8;
                let g = ((src_g * alpha_int) + (bg_g * (255 - alpha_int))) >> 8;
                let b = ((src_b * alpha_int) + (bg_b * (255 - alpha_int))) >> 8;

                (255 << 24) | (r << 16) | (g << 8) | b
            }
        },
    }
}

// ─── Blit Operations ────────────────────────────────────────────────────────

impl<'a> RenderContext<'a> {
    /// Blit RGBA pixel data from GPU render into the software framebuffer.
    /// Converts from RGBA (GPU) to BGRA (software buffer) with alpha compositing.
    pub fn blit_rgba(
        &mut self,
        pixels: &[u8],
        src_width: u32,
        src_height: u32,
        dst_x: f32,
        dst_y: f32,
    ) {
        let dx = (dst_x + self.offset.x) as i32;
        let dy = (dst_y + self.offset.y) as i32;
        let sw = src_width as usize;
        let sh = src_height as usize;
        let dw = self.width;
        let dh = self.height;

        // Fast path: no clip, no rounded clip, fully within bounds
        let no_clip = self.clip_rect.is_none() && self.rounded_clip.is_none();
        let fully_inside = dx >= 0 && dy >= 0
            && (dx as usize + sw) <= dw
            && (dy as usize + sh) <= dh;

        if no_clip && fully_inside {
            let dx = dx as usize;
            let dy = dy as usize;
            for sy in 0..sh {
                let src_row_start = sy * sw * 4;
                let dst_row_start = (dy + sy) * dw + dx;
                for sx in 0..sw {
                    let pi = src_row_start + sx * 4;
                    if pi + 3 >= pixels.len() { break; }
                    let r = pixels[pi] as u32;
                    let g = pixels[pi + 1] as u32;
                    let b = pixels[pi + 2] as u32;
                    self.buffer[dst_row_start + sx] = (255 << 24) | (r << 16) | (g << 8) | b;
                }
            }
            return;
        }

        // Slow path with clipping and alpha blending
        let (sw, sh, dw, dh) = (sw as i32, sh as i32, dw as i32, dh as i32);
        let (mut min_x, mut min_y, mut max_x, mut max_y) = (0, 0, dw, dh);
        let _ = self.clip_rect.as_ref().map(|clip| {
            min_x = clip.min_x().floor() as i32;
            min_y = clip.min_y().floor() as i32;
            max_x = clip.max_x().ceil() as i32;
            max_y = clip.max_y().ceil() as i32;
        });

        for sy in 0..sh {
            let fy = dy + sy;
            if fy < min_y || fy >= max_y || fy < 0 || fy >= dh { continue; }
            let src_row = (sy * sw * 4) as usize;
            let dst_row = (fy * dw) as usize;
            for sx in 0..sw {
                let fx = dx + sx;
                if fx < min_x || fx >= max_x || fx < 0 || fx >= dw { continue; }
                if !self.pixel_in_rounded_clip(fx as f32, fy as f32) { continue; }
                let pi = src_row + (sx as usize * 4);
                if pi + 3 >= pixels.len() { continue; }
                let r = pixels[pi] as u32;
                let g = pixels[pi + 1] as u32;
                let b = pixels[pi + 2] as u32;
                let a = pixels[pi + 3] as u32;
                let di = dst_row + fx as usize;
                if di >= self.buffer.len() { continue; }
                if a >= 250 {
                    self.buffer[di] = (255 << 24) | (r << 16) | (g << 8) | b;
                } else if a > 5 {
                    let bg = self.buffer[di];
                    let bg_r = (bg >> 16) & 0xFF;
                    let bg_g = (bg >> 8) & 0xFF;
                    let bg_b = bg & 0xFF;
                    let out_r = ((r * a) + (bg_r * (255 - a))) >> 8;
                    let out_g = ((g * a) + (bg_g * (255 - a))) >> 8;
                    let out_b = ((b * a) + (bg_b * (255 - a))) >> 8;
                    self.buffer[di] = (255 << 24) | (out_r << 16) | (out_g << 8) | out_b;
                }
            }
        }
    }
}
