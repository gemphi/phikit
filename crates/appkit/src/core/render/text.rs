use super::{blend_colors, RenderContext};
use crate::NSPoint;
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

#[derive(Clone)]
struct CachedGlyph {
    metrics: fontdue::Metrics,
    bitmap: Vec<u8>,
}
type GlyphCache = HashMap<(char, u32), CachedGlyph>;

fn global_glyph_cache() -> &'static Mutex<GlyphCache> {
    static CACHE: OnceLock<Mutex<GlyphCache>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

pub fn system_font() -> &'static fontdue::Font {
    static FONT: OnceLock<fontdue::Font> = OnceLock::new();
    FONT.get_or_init(|| {
        let font_data = std::fs::read("C:\\Windows\\Fonts\\segoeui.ttf").expect("Font not found");
        fontdue::Font::from_bytes(font_data, fontdue::FontSettings::default())
            .expect("Invalid font")
    })
}

/// Measure the pixel width of a text string at the given font size.
/// Uses actual per-glyph advance widths from the font — not a fixed char width.
pub fn measure_text(text: &str, font_size: f32) -> f32 {
    let font = system_font();
    let mut width = 0.0_f32;
    for c in text.chars() {
        let (metrics, _) = font.rasterize(c, font_size);
        width += metrics.advance_width;
    }
    width
}

impl<'a> RenderContext<'a> {
    pub fn draw_text(&mut self, text: &str, position: NSPoint, font_size: f32) {
        let scale = self.scale_factor;
        // Scale position from logical to physical coordinates
        let position = position.offset(self.offset.x, self.offset.y);
        let px = position.x * scale;
        let py = position.y * scale;
        // Rasterize at physical font size for sharp text on HiDPI
        let physical_font_size = font_size * scale;

        if let Some(font) = &self.font {
            let metrics = font.horizontal_line_metrics(physical_font_size);
            let ascent = metrics
                .map(|m| m.new_line_size * 0.75)
                .unwrap_or(physical_font_size * 0.75);
            let baseline = (py + ascent).round();
            let mut cursor_x = px;
            let mut cache = global_glyph_cache().lock().unwrap();
            let size_key = (physical_font_size * 10.0) as u32;

            let mut min_x = 0;
            let mut min_y = 0;
            let mut max_x = self.width as i32;
            let mut max_y = self.height as i32;

            if let Some(clip) = self.clip_rect {
                min_x = (clip.min_x() * scale).floor() as i32;
                min_y = (clip.min_y() * scale).floor() as i32;
                max_x = (clip.max_x() * scale).ceil() as i32;
                max_y = (clip.max_y() * scale).ceil() as i32;
            }

            for c in text.chars() {
                let glyph = cache.entry((c, size_key)).or_insert_with(|| {
                    let (metrics, bitmap) = font.rasterize(c, physical_font_size);
                    CachedGlyph { metrics, bitmap }
                });
                for r in 0..glyph.metrics.height {
                    for c_col in 0..glyph.metrics.width {
                        let alpha = glyph.bitmap[r * glyph.metrics.width + c_col] as f32 / 255.0;
                        if alpha > 0.0 {
                            let dx = cursor_x as i32 + glyph.metrics.xmin + c_col as i32;
                            let dy = (baseline
                                - glyph.metrics.height as f32
                                - glyph.metrics.ymin as f32)
                                as i32
                                + r as i32;
                            if dx >= min_x
                                && dx < max_x
                                && dy >= min_y
                                && dy < max_y
                                && dx >= 0
                                && dx < self.width as i32
                                && dy >= 0
                                && dy < self.height as i32
                            {
                                if self.pixel_in_rounded_clip(dx as f32 / scale, dy as f32 / scale) {
                                    let idx = (dy as usize) * self.width + (dx as usize);
                                    self.buffer[idx] =
                                        blend_colors(self.current_color, self.buffer[idx], alpha);
                                }
                            }
                        }
                    }
                }
                cursor_x += glyph.metrics.advance_width;
            }
        }
    }
}
