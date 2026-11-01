//! # Image Rendering Implementation
//!
//! Drawing bitmapped and vector images into the software buffer.

use crate::graphics::image::NSImage;
use crate::graphics::NSRect;
use crate::render::RenderContext;

impl<'a> RenderContext<'a> {
    pub fn draw_image(&mut self, image: &NSImage, rect: NSRect) {
        let pixels = image.pixels();
        let img_size = image.size();

        let start_x = (rect.origin.x + self.offset.x) as i32;
        let start_y = (rect.origin.y + self.offset.y) as i32;
        let width = rect.size.width as i32;
        let height = rect.size.height as i32;

        let mut max_alpha = 0.0f32;
        for y in 0..height {
            for x in 0..width {
                let target_x = start_x + x;
                let target_y = start_y + y;

                if target_x < 0
                    || target_x >= self.width as i32
                    || target_y < 0
                    || target_y >= self.height as i32
                {
                    continue;
                }

                if !self.pixel_in_rounded_clip(target_x as f32, target_y as f32) {
                    continue;
                }

                let src_x_f = x as f32 * (img_size.width / rect.size.width);
                let src_y_f = y as f32 * (img_size.height / rect.size.height);

                let x0 = (src_x_f.floor() as usize).min((img_size.width - 1.0) as usize);
                let x1 = (x0 + 1).min((img_size.width - 1.0) as usize);
                let y0 = (src_y_f.floor() as usize).min((img_size.height - 1.0) as usize);
                let y1 = (y0 + 1).min((img_size.height - 1.0) as usize);

                let dx = src_x_f - x0 as f32;
                let dy = src_y_f - y0 as f32;

                let mut get_pixel = |px: usize, py: usize| -> (f32, f32, f32, f32) {
                    let idx = (py * img_size.width as usize + px) * 4;
                    if idx + 3 < pixels.len() {
                        (
                            pixels[idx] as f32,
                            pixels[idx + 1] as f32,
                            pixels[idx + 2] as f32,
                            pixels[idx + 3] as f32,
                        )
                    } else {
                        (0.0, 0.0, 0.0, 0.0)
                    }
                };

                let p00 = get_pixel(x0, y0);
                let p10 = get_pixel(x1, y0);
                let p01 = get_pixel(x0, y1);
                let p11 = get_pixel(x1, y1);

                let lerp = |a: f32, b: f32, t: f32| a + (b - a) * t;

                let r = lerp(lerp(p00.0, p10.0, dx), lerp(p01.0, p11.0, dx), dy);
                let g = lerp(lerp(p00.1, p10.1, dx), lerp(p01.1, p11.1, dx), dy);
                let b = lerp(lerp(p00.2, p10.2, dx), lerp(p01.2, p11.2, dx), dy);
                let a = lerp(lerp(p00.3, p10.3, dx), lerp(p01.3, p11.3, dx), dy);

                if a > max_alpha {
                    max_alpha = a;
                }
                if a <= 0.0 {
                    continue;
                }

                let target_idx = target_y as usize * self.width + target_x as usize;

                // Alpha blend
                if a >= 254.0 {
                    self.buffer[target_idx] =
                        (255 << 24) | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32);
                } else {
                    let bg = self.buffer[target_idx];
                    let alpha_f = a / 255.0;
                    let bg_r = (bg >> 16) & 0xFF;
                    let bg_g = (bg >> 8) & 0xFF;
                    let bg_b = bg & 0xFF;
                    let out_r = (r * alpha_f + bg_r as f32 * (1.0 - alpha_f)).min(255.0) as u32;
                    let out_g = (g * alpha_f + bg_g as f32 * (1.0 - alpha_f)).min(255.0) as u32;
                    let out_b = (b * alpha_f + bg_b as f32 * (1.0 - alpha_f)).min(255.0) as u32;
                    self.buffer[target_idx] = (255 << 24) | (out_r << 16) | (out_g << 8) | out_b;
                }
            }
        }
    }
}
