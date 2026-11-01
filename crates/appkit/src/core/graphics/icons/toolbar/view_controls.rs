//! Toolbar icons — View controls: Pan, Zoom, Camera, Shading

use super::super::primitives::*;

/// Render the ViewPan (hand/grab) icon.
pub fn render_view_pan(pixels: &mut [u8], size: u32) {
    let s = size as f32;
    let center = s / 2.0;
    let margin = (s * 0.15).max(2.0);
    let t = (s * 0.07).max(1.5);
    // Palm
    draw_filled_rect(
        pixels,
        size,
        center - s * 0.2,
        center,
        center + s * 0.2,
        s - margin,
        &ICON_FG,
    );
    // Fingers
    for i in 0..4 {
        let fx = center - s * 0.15 + i as f32 * (s * 0.1);
        draw_filled_rect(pixels, size, fx, margin + s * 0.1, fx + t, center, &ICON_FG);
    }
}

/// Render the ViewZoom (magnifying glass) icon.
pub fn render_view_zoom(pixels: &mut [u8], size: u32) {
    let s = size as f32;
    let center = s / 2.0;
    let margin = (s * 0.15).max(2.0);
    let r = (s * 0.25).max(4.0);
    draw_circle(pixels, size, center - 2.0, center - 2.0, r, 2.0, &ICON_FG);
    draw_line(
        pixels,
        size,
        center + r * 0.5,
        center + r * 0.5,
        s - margin,
        s - margin,
        2.0,
        &ICON_FG,
    );
}

/// Render the ViewCamera / ZoomAll (camera/eye) icon.
pub fn render_view_camera(pixels: &mut [u8], size: u32) {
    let s = size as f32;
    let center = s / 2.0;
    let ew = s * 0.6;
    let eh = s * 0.35;
    let ex = (s - ew) / 2.0;
    let ey = (s - eh) / 2.0;
    draw_filled_rect(pixels, size, ex, ey, ex + ew, ey + eh, &ICON_FG);
    draw_filled_circle(pixels, size, center, center, eh * 0.3, &[40, 40, 45, 255]);
}

/// Render the ZoomIn (magnifying glass with plus) icon.
pub fn render_zoom_in(pixels: &mut [u8], size: u32) {
    let s = size as f32;
    let center = s / 2.0;
    let margin = (s * 0.15).max(2.0);
    let r = (s * 0.25).max(4.0);
    draw_circle(pixels, size, center - 2.0, center - 2.0, r, 2.0, &ICON_FG);
    draw_line(
        pixels,
        size,
        center + r * 0.5,
        center + r * 0.5,
        s - margin,
        s - margin,
        2.0,
        &ICON_FG,
    );
    // Plus inside
    let t = 1.5;
    draw_filled_rect(
        pixels,
        size,
        center - 2.0 - r * 0.4,
        center - 2.0 - t / 2.0,
        center - 2.0 + r * 0.4,
        center - 2.0 + t / 2.0,
        &ICON_FG,
    );
    draw_filled_rect(
        pixels,
        size,
        center - 2.0 - t / 2.0,
        center - 2.0 - r * 0.4,
        center - 2.0 + t / 2.0,
        center - 2.0 + r * 0.4,
        &ICON_FG,
    );
}

/// Render the ZoomOut (magnifying glass with minus) icon.
pub fn render_zoom_out(pixels: &mut [u8], size: u32) {
    let s = size as f32;
    let center = s / 2.0;
    let margin = (s * 0.15).max(2.0);
    let r = (s * 0.25).max(4.0);
    draw_circle(pixels, size, center - 2.0, center - 2.0, r, 2.0, &ICON_FG);
    draw_line(
        pixels,
        size,
        center + r * 0.5,
        center + r * 0.5,
        s - margin,
        s - margin,
        2.0,
        &ICON_FG,
    );
    // Minus inside
    let t = 1.5;
    draw_filled_rect(
        pixels,
        size,
        center - 2.0 - r * 0.4,
        center - 2.0 - t / 2.0,
        center - 2.0 + r * 0.4,
        center - 2.0 + t / 2.0,
        &ICON_FG,
    );
}

/// Render the ShadingSolid (half-filled circle) icon.
pub fn render_shading_solid(pixels: &mut [u8], size: u32) {
    let s = size as f32;
    let center = s / 2.0;
    let margin = (s * 0.15).max(2.0);
    let r = center - margin - 1.0;
    draw_circle(pixels, size, center, center, r, 1.5, &ICON_FG);
    // Fill right half
    for y in 0..size as i32 {
        for x in (center as i32)..size as i32 {
            let dx = x as f32 - center;
            let dy = y as f32 - center;
            if dx * dx + dy * dy <= r * r {
                set_pixel(pixels, size, x, y, &ICON_FG);
            }
        }
    }
}

/// Render the fallback (dot) icon for unrecognized toolbar icon IDs.
pub fn render_fallback(pixels: &mut [u8], size: u32) {
    let s = size as f32;
    let center = s / 2.0;
    draw_filled_circle(
        pixels,
        size,
        center,
        center,
        (s * 0.2).max(3.0),
        &ICON_FG_DIM,
    );
}
