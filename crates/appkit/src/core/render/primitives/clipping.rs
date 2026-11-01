//! Line drawing and shadow rendering operations with clip-aware rendering.

use super::super::RenderContext;
use crate::graphics::NSRect;

impl<'a> RenderContext<'a> {
    pub fn draw_line(
        &mut self,
        p1: crate::graphics::NSPoint,
        p2: crate::graphics::NSPoint,
        thickness: f32,
    ) {
        let p1 = p1.offset(self.offset.x, self.offset.y);
        let p2 = p2.offset(self.offset.x, self.offset.y);

        let clip = self.clip_rect;

        match (p1.x - p2.x).abs() < 0.1 {
            true => {
                // Vertical
                let x = p1.x.round() as usize;
                let _ = (x < self.width).then(|| {
                    let y_start = p1.y.min(p2.y).round().max(0.0) as usize;
                    let y_end = p1.y.max(p2.y).round().min(self.height as f32) as usize;
                    let color = self.current_color.to_bgra_u32();
                    for y in y_start..y_end {
                        let _ = match clip {
                            Some(c) => {
                                (x as f32) >= c.min_x()
                                    && (x as f32) < c.max_x()
                                    && (y as f32) >= c.min_y()
                                    && (y as f32) < c.max_y()
                            }
                            None => true,
                        }
                        .then(|| {
                            self.buffer[y * self.width + x] = color;
                        });
                    }
                });
                return;
            }
            false => {}
        }
        match (p1.y - p2.y).abs() < 0.1 {
            true => {
                // Horizontal
                let y = p1.y.round() as usize;
                let _ = (y < self.height).then(|| {
                    let x_start = p1.x.min(p2.x).round().max(0.0) as usize;
                    let x_end = p1.x.max(p2.x).round().min(self.width as f32) as usize;
                    let color = self.current_color.to_bgra_u32();
                    let offset = y * self.width;
                    for x in x_start..x_end {
                        let _ = match clip {
                            Some(c) => {
                                (x as f32) >= c.min_x()
                                    && (x as f32) < c.max_x()
                                    && (y as f32) >= c.min_y()
                                    && (y as f32) < c.max_y()
                            }
                            None => true,
                        }
                        .then(|| {
                            self.buffer[offset + x] = color;
                        });
                    }
                });
                return;
            }
            false => {}
        }

        // Fallback for diagonal lines
        let dx = p2.x - p1.x;
        let dy = p2.y - p1.y;
        let steps = dx.abs().max(dy.abs()).round() as usize;
        if steps == 0 {
            return;
        }
        let x_inc = dx / steps as f32;
        let y_inc = dy / steps as f32;
        let mut x = p1.x;
        let mut y = p1.y;
        let color = self.current_color.to_bgra_u32();
        for _ in 0..=steps {
            let xi = x.round() as i32;
            let yi = y.round() as i32;
            let _ = (xi >= 0 && xi < self.width as i32 && yi >= 0 && yi < self.height as i32).then(
                || {
                    let _ = match clip {
                        Some(c) => {
                            (xi as f32) >= c.min_x()
                                && (xi as f32) < c.max_x()
                                && (yi as f32) >= c.min_y()
                                && (yi as f32) < c.max_y()
                        }
                        None => true,
                    }
                    .then(|| {
                        self.buffer[yi as usize * self.width + xi as usize] = color;
                    });
                },
            );
            x += x_inc;
            y += y_inc;
        }
    }

    pub fn draw_shadow(&mut self, rect: NSRect, radius: f32, opacity: f32) {
        let shadow_rect = rect.offset(2.0, 2.0); // Simple offset
        let old_color = self.current_color;
        self.set_color(crate::theme::color(crate::theme::keys::CLR_SHADOW));
        self.fill_rounded_rect(shadow_rect, radius);
        self.set_color(old_color);
    }

    /// Draw a uniform shadow around a rect (all sides) with a given spread and opacity.
    /// Used for floating elements like dropdown menus and popovers.
    /// The shadow is drawn as a larger rounded rect behind the element, expanded by `spread`
    /// on all sides.
    pub fn draw_shadow_around(&mut self, rect: NSRect, radius: f32, spread: f32, opacity: f32) {
        let shadow_rect = NSRect::new(
            rect.min_x() - spread,
            rect.min_y() - spread,
            rect.width() + spread * 2.0,
            rect.height() + spread * 2.0,
        );
        let old_color = self.current_color;
        self.set_color(crate::theme::color(crate::theme::keys::CLR_SHADOW));
        self.fill_rounded_rect(shadow_rect, radius + spread);
        self.set_color(old_color);
    }

    /// Draw inner shadow at the top and left edges of a rect to create depth separation.
    /// Used at panel boundaries to simulate inset depth (macOS Sonoma style).
    pub fn draw_inner_shadow(&mut self, rect: NSRect, thickness: f32, opacity: f32) {
        let old_color = self.current_color;
        self.set_color(crate::theme::color(crate::theme::keys::CLR_SHADOW));
        // Top inner edge
        self.fill_rect(NSRect::new(
            rect.min_x(),
            rect.min_y(),
            rect.width(),
            thickness,
        ));
        // Left inner edge
        self.fill_rect(NSRect::new(
            rect.min_x(),
            rect.min_y(),
            thickness,
            rect.height(),
        ));
        self.set_color(old_color);
    }
}
