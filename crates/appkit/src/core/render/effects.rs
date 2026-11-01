//! # Gradient Effects
//!
//! Vertical/horizontal linear gradients and glass blur effects for
//! Apple-style Quartz rendering. Used by themed panel backgrounds.

use crate::graphics::{NSColor, NSRect};
use crate::render::RenderContext;

/// Direction of a linear gradient fill.
pub enum GradientDirection {
    /// Top to bottom
    Vertical,
    /// Left to right
    Horizontal,
}

impl<'a> RenderContext<'a> {
    /// Fill a rectangle with a linear gradient between two colors.
    ///
    /// ```ignore
    /// ctx.fill_gradient(rect, top_color, bottom_color, GradientDirection::Vertical);
    /// ```
    pub fn fill_gradient(
        &mut self,
        rect: NSRect,
        from: NSColor,
        to: NSColor,
        direction: GradientDirection,
    ) {
        let x0 = (rect.origin.x + self.offset.x) as i32;
        let y0 = (rect.origin.y + self.offset.y) as i32;
        let w = rect.size.width as i32;
        let h = rect.size.height as i32;

        for y in 0..h {
            for x in 0..w {
                let px = x0 + x;
                let py = y0 + y;
                if px < 0 || px >= self.width as i32 || py < 0 || py >= self.height as i32 {
                    continue;
                }

                let t = match direction {
                    GradientDirection::Vertical => y as f32 / h.max(1) as f32,
                    GradientDirection::Horizontal => x as f32 / w.max(1) as f32,
                };

                let r = (from.r * (1.0 - t) + to.r * t).clamp(0.0, 1.0);
                let g = (from.g * (1.0 - t) + to.g * t).clamp(0.0, 1.0);
                let b = (from.b * (1.0 - t) + to.b * t).clamp(0.0, 1.0);
                let a = (from.a * (1.0 - t) + to.a * t).clamp(0.0, 1.0);

                let idx = py as usize * self.width + px as usize;
                if a >= 0.99 {
                    self.buffer[idx] = (255 << 24)
                        | ((r * 255.0) as u32) << 16
                        | ((g * 255.0) as u32) << 8
                        | (b * 255.0) as u32;
                } else if a > 0.0 {
                    let bg = self.buffer[idx];
                    let bg_r = ((bg >> 16) & 0xFF) as f32 / 255.0;
                    let bg_g = ((bg >> 8) & 0xFF) as f32 / 255.0;
                    let bg_b = (bg & 0xFF) as f32 / 255.0;
                    let out_r = (r * a + bg_r * (1.0 - a)).clamp(0.0, 1.0);
                    let out_g = (g * a + bg_g * (1.0 - a)).clamp(0.0, 1.0);
                    let out_b = (b * a + bg_b * (1.0 - a)).clamp(0.0, 1.0);
                    self.buffer[idx] = (255 << 24)
                        | ((out_r * 255.0) as u32) << 16
                        | ((out_g * 255.0) as u32) << 8
                        | (out_b * 255.0) as u32;
                }
            }
        }
    }
}
