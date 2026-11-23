//! Shape primitive rendering — fill_rect, clear_rect, fill_rounded_rect, fill_circle,
//! and shadow drawing operations.

use super::super::{blend_colors, RenderContext};
use crate::graphics::NSRect;

impl<'a> RenderContext<'a> {
    pub fn fill_rect(&mut self, rect: NSRect) {
        let mut rect = rect.offset(self.offset.x, self.offset.y);

        if let Some(clip) = self.clip_rect {
            rect = rect.intersection(&clip);
            if rect.width() <= 0.0 || rect.height() <= 0.0 {
                return;
            }
        }

        let dy_start = rect.min_y().round().max(0.0) as usize;
        let dy_end = rect.max_y().round().min(self.height as f32) as usize;
        let dx_start = rect.min_x().round().max(0.0) as usize;
        let dx_end = rect.max_x().round().min(self.width as f32) as usize;

        if dx_start >= dx_end || dy_start >= dy_end {
            return;
        }

        let alpha = self.current_color.a;
        let has_rounded_clip = self.rounded_clip.is_some();

        match alpha >= 0.99 {
            true => {
                let color_u32 = self.current_color.to_bgra_u32();
                for y in dy_start..dy_end {
                    let offset = y * self.width;
                    match has_rounded_clip {
                        true => {
                            for x in dx_start..dx_end {
                                let _ = self.pixel_in_rounded_clip(x as f32, y as f32).then(|| {
                                    self.buffer[offset + x] = color_u32;
                                });
                            }
                        }
                        false => {
                            self.buffer[offset + dx_start..offset + dx_end].fill(color_u32);
                        }
                    }
                }
            }
            false => {
                let _ = (alpha > 0.0).then(|| {
                    for y in dy_start..dy_end {
                        let offset = y * self.width;
                        for x in dx_start..dx_end {
                            let _ = (!has_rounded_clip
                                || self.pixel_in_rounded_clip(x as f32, y as f32))
                            .then(|| {
                                let idx = offset + x;
                                self.buffer[idx] =
                                    blend_colors(self.current_color, self.buffer[idx], alpha);
                            });
                        }
                    }
                });
            }
        }
    }

    pub fn clear_rect(&mut self, rect: NSRect) {
        let mut rect = rect.offset(self.offset.x, self.offset.y);

        if let Some(clip) = self.clip_rect {
            rect = rect.intersection(&clip);
            if rect.width() <= 0.0 || rect.height() <= 0.0 {
                return;
            }
        }

        let dy_start = rect.min_y().round().max(0.0) as usize;
        let dy_end = rect.max_y().round().min(self.height as f32) as usize;
        let dx_start = rect.min_x().round().max(0.0) as usize;
        let dx_end = rect.max_x().round().min(self.width as f32) as usize;

        if dx_start >= dx_end || dy_start >= dy_end {
            return;
        }

        for y in dy_start..dy_end {
            let offset = y * self.width;
            self.buffer[offset + dx_start..offset + dx_end].fill(0);
        }
    }

    pub fn fill_rounded_rect(&mut self, rect: NSRect, radius: f32) {
        if radius <= 0.0 {
            self.fill_rect(rect);
            return;
        }
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
        let r = radius.max(0.0);
        let color = self.current_color;
        let has_rounded_clip = self.rounded_clip.is_some();

        for y_usize in dy_start..dy_end {
            let y = y_usize as f32;
            let offset = y_usize * self.width;
            for x_usize in dx_start..dx_end {
                let x = x_usize as f32;
                // Rounded clip check
                if has_rounded_clip && !self.pixel_in_rounded_clip(x, y) {
                    continue;
                }
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
                let _ = (d_sq < (r + 1.0).powi(2)).then(|| {
                    match d_sq <= (r - 1.0).max(0.0).powi(2) {
                        true => {
                            self.buffer[offset + x_usize] =
                                blend_colors(color, self.buffer[offset + x_usize], color.a);
                        }
                        false => {
                            // 1-pixel anti-aliased smooth edge
                            let d = d_sq.sqrt();
                            let coverage = (1.0 - (d - r + 1.0).clamp(0.0, 1.0)).clamp(0.0, 1.0);
                            let edge_alpha = color.a * coverage;
                            self.buffer[offset + x_usize] = blend_colors(
                                crate::NSColor::new(color.r, color.g, color.b, edge_alpha),
                                self.buffer[offset + x_usize],
                                edge_alpha,
                            );
                        }
                    }
                });
            }
        }
    }

    pub fn fill_circle(&mut self, center: crate::graphics::NSPoint, radius: f32) {
        let center = center.offset(self.offset.x, self.offset.y);
        let clip = self.clip_rect;

        let dy_start = (center.y - radius).floor().max(0.0) as usize;
        let dy_end = (center.y + radius).ceil().min(self.height as f32) as usize;
        let dx_start = (center.x - radius).floor().max(0.0) as usize;
        let dx_end = (center.x + radius).ceil().min(self.width as f32) as usize;

        let r_sq = radius * radius;
        let has_rounded_clip = self.rounded_clip.is_some();
        for y_usize in dy_start..dy_end {
            let y = y_usize as f32;
            let offset = y_usize * self.width;
            for x_usize in dx_start..dx_end {
                let x = x_usize as f32;
                let _ = match clip {
                    Some(c) => x >= c.min_x() && x < c.max_x() && y >= c.min_y() && y < c.max_y(),
                    None => true,
                }
                .then(|| {
                    let _ = (!has_rounded_clip || self.pixel_in_rounded_clip(x, y)).then(|| {
                        let d_sq = (x - center.x).powi(2) + (y - center.y).powi(2);
                        let _ = (d_sq <= r_sq).then(|| {
                            let idx = offset + x_usize;
                            self.buffer[idx] = blend_colors(
                                self.current_color,
                                self.buffer[idx],
                                self.current_color.a,
                            );
                        });
                    });
                });
            }
        }
    }
}
