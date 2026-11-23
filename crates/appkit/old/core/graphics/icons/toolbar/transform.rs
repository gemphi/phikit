//! Toolbar icons — Transform tools: Select, Move, Rotate, Scale

use super::super::primitives::*;

/// Render the Select (arrow cursor) icon.
pub fn render_select(pixels: &mut [u8], size: u32) {
    let s = size as f32;
    let tip_x = s * 0.25;
    let tip_y = s * 0.12;
    let bottom_y = s * 0.85;
    let right_x = s * 0.6;
    let height = bottom_y - tip_y;
    for row in 0..(height as i32) {
        let t = row as f32 / height;
        let y = tip_y + row as f32;
        let left = tip_x;
        let right = if t < 0.5 {
            tip_x + (right_x - tip_x) * (t * 2.0)
        } else {
            let t2 = (t - 0.5) * 2.0;
            right_x - (right_x - tip_x * 1.5) * t2
        };
        if right > left {
            draw_filled_rect(pixels, size, left, y, right, y + 1.0, &ICON_FG);
        }
    }
}

/// Render the Move (four-way cross with arrowheads) icon.
pub fn render_move(pixels: &mut [u8], size: u32) {
    let s = size as f32;
    let center = s / 2.0;
    let margin = (s * 0.15).max(2.0);
    let bar_w = (s * 0.08).max(2.0);
    let half = bar_w / 2.0;
    let inner = margin + (s * 0.12);
    let outer = s - margin - (s * 0.12);
    let p = pixels;
    let sz = size;
    // Cross bars
    draw_filled_rect(p, sz, center - half, inner, center + half, outer, &ICON_FG);
    draw_filled_rect(p, sz, inner, center - half, outer, center + half, &ICON_FG);
    // Arrowheads (triangular tips)
    let aw = (s * 0.16).max(3.0);
    // Up
    draw_filled_rect(p, sz, center - aw / 2.0, margin, center + aw / 2.0, margin + 2.0, &ICON_FG);
    draw_filled_rect(p, sz, center - aw / 3.0, margin + 2.0, center + aw / 3.0, inner, &ICON_FG);
    // Down
    draw_filled_rect(p, sz, center - aw / 2.0, s - margin - 2.0, center + aw / 2.0, s - margin, &ICON_FG);
    draw_filled_rect(p, sz, center - aw / 3.0, outer, center + aw / 3.0, s - margin - 2.0, &ICON_FG);
    // Left
    draw_filled_rect(p, sz, margin, center - aw / 2.0, margin + 2.0, center + aw / 2.0, &ICON_FG);
    draw_filled_rect(p, sz, margin + 2.0, center - aw / 3.0, inner, center + aw / 3.0, &ICON_FG);
    // Right
    draw_filled_rect(p, sz, s - margin - 2.0, center - aw / 2.0, s - margin, center + aw / 2.0, &ICON_FG);
    draw_filled_rect(p, sz, outer, center - aw / 3.0, s - margin - 2.0, center + aw / 3.0, &ICON_FG);
}

/// Render the Rotate (circular arc with arrowhead) icon.
pub fn render_rotate(pixels: &mut [u8], size: u32) {
    let s = size as f32;
    let center = s / 2.0;
    let margin = (s * 0.15).max(2.0);
    let r = center - margin - 2.0;
    let thickness = (s * 0.09).max(2.0);
    let steps = (r * 10.0).max(48.0) as i32;
    for i in 0..((steps * 3) / 4) {
        let angle = (i as f32 / steps as f32) * std::f32::consts::TAU;
        for t in 0..(thickness as i32) {
            let tr = r - t as f32;
            let x = (center + angle.cos() * tr) as i32;
            let y = (center + angle.sin() * tr) as i32;
            set_pixel(pixels, size, x, y, &ICON_FG);
        }
    }
    // Arrowhead at end of arc
    let end_angle = (3.0 / 4.0) * std::f32::consts::TAU;
    let ax = center + end_angle.cos() * r;
    let ay = center + end_angle.sin() * r;
    draw_filled_circle(pixels, size, ax, ay, thickness + 1.0, &ICON_FG);
}

/// Render the Scale (diagonal resize with corner handles) icon.
pub fn render_scale(pixels: &mut [u8], size: u32) {
    let s = size as f32;
    let margin = (s * 0.15).max(2.0);
    let t = (s * 0.08).max(2.0);
    // Diagonal line
    draw_line(
        pixels,
        size,
        margin + 2.0,
        margin + 2.0,
        s - margin - 2.0,
        s - margin - 2.0,
        t,
        &ICON_FG,
    );
    // Corner squares
    let cs = (s * 0.12).max(3.0);
    draw_filled_rect(
        pixels,
        size,
        margin,
        margin,
        margin + cs,
        margin + cs,
        &ICON_FG,
    );
    draw_filled_rect(
        pixels,
        size,
        s - margin - cs,
        s - margin - cs,
        s - margin,
        s - margin,
        &ICON_FG,
    );
}
