//! Border and stroke rendering operations.
//! These primitives handle outline/stroke-style rendering for rectangles.

use super::super::{blend_colors, RenderContext};
use crate::graphics::NSRect;

impl<'a> RenderContext<'a> {
    /// Draw a 1px border around a rect (for debug visualization).
    /// Draws top, bottom, left, and right edges as thin filled rects.
    pub fn draw_rect_border(&mut self, rect: NSRect, color: crate::NSColor) {
        let old_color = self.current_color;
        self.set_color(color);
        // Top edge
        self.fill_rect(NSRect::new(rect.min_x(), rect.min_y(), rect.width(), 1.0));
        // Bottom edge
        self.fill_rect(NSRect::new(
            rect.min_x(),
            rect.max_y() - 1.0,
            rect.width(),
            1.0,
        ));
        // Left edge
        self.fill_rect(NSRect::new(rect.min_x(), rect.min_y(), 1.0, rect.height()));
        // Right edge
        self.fill_rect(NSRect::new(
            rect.max_x() - 1.0,
            rect.min_y(),
            1.0,
            rect.height(),
        ));
        self.set_color(old_color);
    }

    /// Draw an anti-aliased rounded rectangle border of a specific thickness.
    pub fn draw_rounded_rect_border(
        &mut self,
        rect: NSRect,
        radius: f32,
        thickness: f32,
        color: crate::NSColor,
    ) {
        if radius <= 0.0 {
            self.draw_rect_border(rect, color);
            return;
        }
        let old_color = self.current_color;
        self.set_color(color);
        let orig_rect = rect.offset(self.offset.x, self.offset.y);

        let mut loop_rect = orig_rect;
        if let Some(clip) = self.clip_rect {
            loop_rect = loop_rect.intersection(&clip);
            if loop_rect.width() <= 0.0 || loop_rect.height() <= 0.0 {
                self.set_color(old_color);
                return;
            }
        }

        let dy_start = loop_rect.min_y().floor().max(0.0) as usize;
        let dy_end = loop_rect.max_y().ceil().min(self.height as f32) as usize;
        let dx_start = loop_rect.min_x().floor().max(0.0) as usize;
        let dx_end = loop_rect.max_x().ceil().min(self.width as f32) as usize;
        let r = radius.max(0.0);
        let has_rounded_clip = self.rounded_clip.is_some();

        for y_usize in dy_start..dy_end {
            let y = y_usize as f32;
            let offset = y_usize * self.width;
            for x_usize in dx_start..dx_end {
                let x = x_usize as f32;
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

                // Determine if pixel is on the border:
                // It should be within the outer rounded rect, but outside the inner rounded rect.
                let is_on_border = match x >= orig_rect.min_x() + r && x <= orig_rect.max_x() - r {
                    true => {
                        // Straight horizontal edges
                        y < orig_rect.min_y() + thickness || y > orig_rect.max_y() - thickness
                    }
                    false => match y >= orig_rect.min_y() + r && y <= orig_rect.max_y() - r {
                        true => {
                            // Straight vertical edges
                            x < orig_rect.min_x() + thickness
                                || x > orig_rect.max_x() - thickness
                        }
                        false => {
                            // Curved corners
                            d_sq < (r + 1.0).powi(2) && d_sq >= (r - thickness).max(0.0).powi(2)
                        }
                    },
                };

                let _ = is_on_border.then(|| {
                    match d_sq <= (r - 1.0).max(0.0).powi(2)
                        || (x >= orig_rect.min_x() + r && x <= orig_rect.max_x() - r)
                        || (y >= orig_rect.min_y() + r && y <= orig_rect.max_y() - r)
                    {
                        true => {
                            self.buffer[offset + x_usize] =
                                blend_colors(color, self.buffer[offset + x_usize], color.a);
                        }
                        false => {
                            // Anti-aliased corner edge
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
        self.set_color(old_color);
    }

    /// Draws a rounded rectangle border (stroke only, no fill).
    pub fn stroke_rounded_rect(&mut self, rect: NSRect, radius: f32, thickness: f32) {
        if thickness <= 0.0 {
            return;
        }
        let orig_rect = rect.offset(self.offset.x, self.offset.y);
        let r = radius.max(0.0);
        let color = self.current_color;

        let dy_start = orig_rect.min_y().floor().max(0.0) as usize;
        let dy_end = orig_rect.max_y().ceil().min(self.height as f32) as usize;
        let dx_start = orig_rect.min_x().floor().max(0.0) as usize;
        let dx_end = orig_rect.max_x().ceil().min(self.width as f32) as usize;

        for y_usize in dy_start..dy_end {
            let y = y_usize as f32;
            let offset = y_usize * self.width;
            for x_usize in dx_start..dx_end {
                let x = x_usize as f32;
                // Distance to nearest edge of rounded rect
                let cx = (x).clamp(orig_rect.min_x() + r, orig_rect.max_x() - r);
                let cy = (y).clamp(orig_rect.min_y() + r, orig_rect.max_y() - r);
                let d = ((x - cx).powi(2) + (y - cy).powi(2)).sqrt();
                // Only draw pixels near the edge (within thickness)
                let edge_dist = (d - r).abs();
                let inner_dist_x = (x - orig_rect.min_x()).min(orig_rect.max_x() - x);
                let inner_dist_y = (y - orig_rect.min_y()).min(orig_rect.max_y() - y);
                let min_inner = inner_dist_x.min(inner_dist_y);

                // For straight edges (not corners)
                let on_border = if d < 0.01 {
                    min_inner < thickness
                } else {
                    (r - d).abs() < thickness
                };

                if on_border {
                    let alpha = color.a;
                    self.buffer[offset + x_usize] =
                        blend_colors(color, self.buffer[offset + x_usize], alpha);
                }
            }
        }
    }


}
