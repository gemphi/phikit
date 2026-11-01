//! Low-level pixel drawing primitives for icon rendering.

/// RGBA foreground color (brighter silver-white for premium HIG feel)
pub const ICON_FG: [u8; 4] = [245, 247, 250, 255];
/// Dimmed secondary color (sharper and more visible)
pub const ICON_FG_DIM: [u8; 4] = [195, 198, 204, 255];

/// Set a single pixel in an RGBA buffer (bounds-checked).
pub fn set_pixel(pixels: &mut [u8], size: u32, x: i32, y: i32, color: &[u8; 4]) {
    if x < 0 || y < 0 || x >= size as i32 || y >= size as i32 {
        return;
    }
    let idx = ((y as u32 * size + x as u32) * 4) as usize;
    if idx + 3 < pixels.len() {
        pixels[idx] = color[0];
        pixels[idx + 1] = color[1];
        pixels[idx + 2] = color[2];
        pixels[idx + 3] = color[3];
    }
}

/// Draw a filled rectangle.
pub fn draw_filled_rect(
    pixels: &mut [u8],
    size: u32,
    x0: f32,
    y0: f32,
    x1: f32,
    y1: f32,
    color: &[u8; 4],
) {
    let x_start = x0.min(x1).max(0.0) as i32;
    let x_end = x0.max(x1).min(size as f32) as i32;
    let y_start = y0.min(y1).max(0.0) as i32;
    let y_end = y0.max(y1).min(size as f32) as i32;
    for y in y_start..=y_end {
        for x in x_start..=x_end {
            set_pixel(pixels, size, x, y, color);
        }
    }
}

/// Draw a line using Bresenham's algorithm.
pub fn draw_line(
    pixels: &mut [u8],
    size: u32,
    x0: f32,
    y0: f32,
    x1: f32,
    y1: f32,
    thickness: f32,
    color: &[u8; 4],
) {
    let dx = (x1 - x0).abs();
    let dy = (y1 - y0).abs();
    let steps = dx.max(dy).max(1.0) as i32;
    let half_t = (thickness / 2.0).max(0.5) as i32;

    for i in 0..=steps {
        let t = i as f32 / steps as f32;
        let cx = (x0 + (x1 - x0) * t) as i32;
        let cy = (y0 + (y1 - y0) * t) as i32;
        for dy in -half_t..=half_t {
            for dx in -half_t..=half_t {
                set_pixel(pixels, size, cx + dx, cy + dy, color);
            }
        }
    }
}

/// Draw a circle outline.
pub fn draw_circle(
    pixels: &mut [u8],
    size: u32,
    cx: f32,
    cy: f32,
    radius: f32,
    thickness: f32,
    color: &[u8; 4],
) {
    let steps = (radius * 8.0).max(32.0) as i32;
    let half_t = (thickness / 2.0).max(0.5) as i32;
    for i in 0..steps {
        let angle = (i as f32 / steps as f32) * std::f32::consts::TAU;
        let x = (cx + angle.cos() * radius) as i32;
        let y = (cy + angle.sin() * radius) as i32;
        for dy in -half_t..=half_t {
            for dx in -half_t..=half_t {
                set_pixel(pixels, size, x + dx, y + dy, color);
            }
        }
    }
}

/// Draw a filled circle.
pub fn draw_filled_circle(
    pixels: &mut [u8],
    size: u32,
    cx: f32,
    cy: f32,
    radius: f32,
    color: &[u8; 4],
) {
    let r2 = radius * radius;
    let x_min = (cx - radius).max(0.0) as i32;
    let x_max = (cx + radius).min(size as f32) as i32;
    let y_min = (cy - radius).max(0.0) as i32;
    let y_max = (cy + radius).min(size as f32) as i32;
    for y in y_min..=y_max {
        for x in x_min..=x_max {
            let dx = x as f32 - cx;
            let dy = y as f32 - cy;
            if dx * dx + dy * dy <= r2 {
                set_pixel(pixels, size, x, y, color);
            }
        }
    }
}
