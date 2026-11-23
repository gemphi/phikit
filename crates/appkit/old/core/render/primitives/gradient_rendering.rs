//! Gradient rendering — vertical gradient fills and color interpolation.

use super::super::{blend_colors, RenderContext};
use crate::graphics::NSRect;

impl<'a> RenderContext<'a> {
    /// Fill a rounded rectangle with a vertical gradient (top_color at top, bottom_color at bottom).
    /// Provides subtle depth for glass/metal panel backgrounds.
    pub fn fill_gradient_rect(
        &mut self,
        rect: NSRect,
        top_color: crate::NSColor,
        bottom_color: crate::NSColor,
        radius: f32,
    ) {
        let orig_rect = rect.offset(self.offset.x, self.offset.y);

        let mut loop_rect = orig_rect;
        if let Some(clip) = self.clip_rect {
            loop_rect = loop_rect.intersection(&clip);
            if loop_rect.width() <= 0.0 || loop_rect.height() <= 0.0 {
                return;
            }
        }

        let dy_start = loop_rect.min_y().floor().max(0.0) as usize;
        let dy_end = loop_rect.max_y().ceil().min(self.height as f32) as usize;
        let dx_start = loop_rect.min_x().floor().max(0.0) as usize;
        let dx_end = loop_rect.max_x().ceil().min(self.width as f32) as usize;
        let height = orig_rect.height();

        if dx_start >= dx_end || dy_start >= dy_end {
            return;
        }

        let r = radius.max(0.0);
        let r_sq = r * r;
        let has_rounded_clip = self.rounded_clip.is_some();

        for y_usize in dy_start..dy_end {
            let y = y_usize as f32;
            let t = ((y - orig_rect.min_y()) / height.max(1.0)).clamp(0.0, 1.0);
            let cr = top_color.r + (bottom_color.r - top_color.r) * t;
            let cg = top_color.g + (bottom_color.g - top_color.g) * t;
            let cb = top_color.b + (bottom_color.b - top_color.b) * t;
            let ca = top_color.a + (bottom_color.a - top_color.a) * t;
            let row_color = crate::NSColor::new(cr, cg, cb, ca);
            let offset = y_usize * self.width;
            for x_usize in dx_start..dx_end {
                // Rounded clip check
                if has_rounded_clip && !self.pixel_in_rounded_clip(x_usize as f32, y_usize as f32) {
                    continue;
                }
                // Skip corners for rounded rect
                let _ = (r > 0.0).then(|| {
                    let x = x_usize as f32;
                    let cx = match x < orig_rect.min_x() + r {
                        true => orig_rect.min_x() + r,
                        false => match x > orig_rect.max_x() - r {
                            true => orig_rect.max_x() - r,
                            false => x,
                        },
                    };
                    let cy = match y < orig_rect.min_y() + r {
                        true => orig_rect.min_y() + r,
                        false => match y > orig_rect.max_y() - r {
                            true => orig_rect.max_y() - r,
                            false => y,
                        },
                    };
                    let d_sq = (x - cx).powi(2) + (y - cy).powi(2);
                    let _ = (d_sq <= r_sq).then(|| {
                        self.buffer[offset + x_usize] =
                            blend_colors(row_color, self.buffer[offset + x_usize], ca);
                    });
                });

                let _ = (r <= 0.0).then(|| {
                    self.buffer[offset + x_usize] =
                        blend_colors(row_color, self.buffer[offset + x_usize], ca);
                });
            }
        }
    }
}

/// Interpolate a single color channel between two values at parameter t.
/// This is the same formula used in `fill_gradient_rect`.
pub fn gradient_interpolate_channel(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

/// Interpolate between two NSColors at parameter t in [0,1].
/// Returns the interpolated color using the same linear interpolation
/// formula as `fill_gradient_rect`.
pub fn gradient_interpolate(top: crate::NSColor, bottom: crate::NSColor, t: f32) -> crate::NSColor {
    crate::NSColor::new(
        gradient_interpolate_channel(top.r, bottom.r, t),
        gradient_interpolate_channel(top.g, bottom.g, t),
        gradient_interpolate_channel(top.b, bottom.b, t),
        gradient_interpolate_channel(top.a, bottom.a, t),
    )
}
