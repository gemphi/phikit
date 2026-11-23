use super::NSView;
use crate::render::RenderContext;

impl NSView {
    pub fn draw(&self, context: &mut RenderContext) {
        if self.is_hidden {
            return;
        }

        context.push_offset(self.frame.origin);

        // Before this view's clips, so glass can shadow what is around it.
        let glass_painted = self.paint_glass(context);

        let clips = self.clips_to_bounds;
        let has_rounded_clip = self.corner_radius > 0.0;

        if clips {
            context.push_clip(crate::graphics::NSRect::new(
                0.0,
                0.0,
                self.frame.width(),
                self.frame.height(),
            ));
        }
        if has_rounded_clip {
            // Push a rounded clip to prevent content from overflowing at rounded corners
            context.push_rounded_clip(
                crate::graphics::NSRect::sized(self.frame.width(), self.frame.height()),
                self.corner_radius,
            );
        }

        let bounds_offset = self.bounds.origin;
        let has_offset = bounds_offset.x != 0.0 || bounds_offset.y != 0.0;
        if has_offset {
            context.push_offset(crate::graphics::NSPoint::new(
                -bounds_offset.x,
                -bounds_offset.y,
            ));
        }

        if glass_painted {
            // The glass is this view's background.
        } else if let (Some(top), Some(bottom)) = (self.gradient_top, self.gradient_bottom) {
            // Gradient background takes priority over solid background
            context.fill_gradient_rect(self.bounds, top, bottom, self.corner_radius);
        } else if let Some(color) = if self.is_hovered {
            self.hover_color.or(self.background_color)
        } else {
            self.background_color
        } {
            context.set_color(color);
            if self.corner_radius > 0.0 {
                context.fill_rounded_rect(self.bounds, self.corner_radius);
            } else {
                context.fill_rect(self.bounds);
            }
        }

        // Render border if border_color is set and border_width > 0
        if let Some(border_clr) = self.border_color {
            if self.border_width > 0.0 {
                if self.corner_radius > 0.0 {
                    context.draw_rounded_rect_border(
                        self.bounds,
                        self.corner_radius,
                        self.border_width,
                        border_clr,
                    );
                } else {
                    context.draw_rect_border(self.bounds, border_clr);
                }
            }
        }

        if let Some(ref delegate) = self.delegate {
            if let Ok(d) = delegate.lock() {
                d.draw(self, context);
            }
        }

        let batches_glass = self.glass_container.is_some();
        context.glass_batched += u32::from(batches_glass);
        for subview in &self.subviews {
            subview.draw(context);
        }
        context.glass_batched -= u32::from(batches_glass);

        // Debug border rendering — draw colored 1px borders around every view
        if crate::render::debug_borders_enabled() {
            let border_color = crate::render::debug_border_color_for_name(&self.name);
            context.draw_rect_border(self.bounds, border_color);
        }

        if has_offset {
            context.pop_offset(crate::graphics::NSPoint::new(0.0, 0.0));
        }

        if has_rounded_clip {
            context.pop_rounded_clip();
        }
        if clips {
            context.pop_clip();
        }

        context.pop_offset(self.frame.origin);
    }
}
