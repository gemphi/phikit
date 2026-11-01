//! Media icons — Play, Pause, Stop, Audio, Video, etc.

use super::primitives::*;
use super::types::IconId;

/// Free-function wrapper for media icon rendering.
pub fn render(pixels: &mut [u8], size: u32, id: &IconId) {
    NSIconMediaRenderer::render(pixels, size, id);
}

pub struct NSIconMediaRenderer;

impl NSIconMediaRenderer {

pub fn render(pixels: &mut [u8], size: u32, id: &IconId) {
    let s = size as f32;
    let center = s / 2.0;
    let margin = (s * 0.15).max(2.0);

    match id {
        IconId::Play => {
            // Filled right-pointing triangle
            let h = s - margin * 2.0;
            for row in 0..(h as i32) {
                let frac = row as f32 / h;
                let y = margin + row as f32;
                let w = (s * 0.6) * (1.0 - (frac - 0.5).abs() * 2.0);
                draw_filled_rect(pixels, size, s * 0.3, y, s * 0.3 + w, y + 1.0, &ICON_FG);
            }
        }
        IconId::Pause => {
            let bar_w = (s * 0.15).max(3.0);
            let gap = (s * 0.1).max(2.0);
            draw_filled_rect(
                pixels,
                size,
                center - gap - bar_w,
                margin,
                center - gap,
                s - margin,
                &ICON_FG,
            );
            draw_filled_rect(
                pixels,
                size,
                center + gap,
                margin,
                center + gap + bar_w,
                s - margin,
                &ICON_FG,
            );
        }
        IconId::Stop => {
            let inset = margin + (s * 0.1);
            draw_filled_rect(pixels, size, inset, inset, s - inset, s - inset, &ICON_FG);
        }
        IconId::Image => {
            // Rectangle with mountain/sun
            let inset = margin + 1.0;
            draw_filled_rect(
                pixels,
                size,
                inset,
                inset + 2.0,
                s - inset,
                s - inset - 2.0,
                &ICON_FG_DIM,
            );
            // Sun circle
            draw_filled_circle(
                pixels,
                size,
                s * 0.35,
                s * 0.35,
                (s * 0.08).max(2.0),
                &ICON_FG,
            );
        }
        IconId::Video => {
            // Film frame
            let bw = s * 0.55;
            let bh = s * 0.4;
            let bx = margin;
            let by = (s - bh) / 2.0;
            draw_filled_rect(pixels, size, bx, by, bx + bw, by + bh, &ICON_FG_DIM);
            // Play triangle on right
            draw_line(
                pixels,
                size,
                bx + bw + 2.0,
                center - bh * 0.3,
                s - margin,
                center,
                2.0,
                &ICON_FG,
            );
            draw_line(
                pixels,
                size,
                s - margin,
                center,
                bx + bw + 2.0,
                center + bh * 0.3,
                2.0,
                &ICON_FG,
            );
        }
        IconId::Audio => {
            // Speaker cone
            let bw = s * 0.25;
            let bh = s * 0.3;
            let bx = margin + 2.0;
            let by = (s - bh) / 2.0;
            draw_filled_rect(pixels, size, bx, by, bx + bw, by + bh, &ICON_FG);
            // Sound waves
            for i in 1..=3 {
                let r = (s * 0.12) * i as f32;
                let cx = bx + bw;
                let steps = 16;
                for j in 0..steps {
                    let angle = -0.4 + (j as f32 / steps as f32) * 0.8;
                    let x = (cx + angle.cos() * r) as i32;
                    let y = (center + angle.sin() * r) as i32;
                    set_pixel(pixels, size, x, y, &ICON_FG);
                }
            }
        }
        IconId::Mesh => {
            // Wireframe cube
            let inset = margin + 2.0;
            let w = s - inset * 2.0;
            // Front face
            draw_line(
                pixels,
                size,
                inset,
                inset + w * 0.3,
                inset + w * 0.6,
                inset + w * 0.3,
                1.5,
                &ICON_FG,
            );
            draw_line(
                pixels,
                size,
                inset,
                inset + w * 0.3,
                inset,
                s - inset,
                1.5,
                &ICON_FG,
            );
            draw_line(
                pixels,
                size,
                inset,
                s - inset,
                inset + w * 0.6,
                s - inset,
                1.5,
                &ICON_FG,
            );
            draw_line(
                pixels,
                size,
                inset + w * 0.6,
                inset + w * 0.3,
                inset + w * 0.6,
                s - inset,
                1.5,
                &ICON_FG,
            );
            // Back edges
            draw_line(
                pixels,
                size,
                inset + w * 0.4,
                inset,
                s - inset,
                inset,
                1.5,
                &ICON_FG_DIM,
            );
            draw_line(
                pixels,
                size,
                s - inset,
                inset,
                s - inset,
                s - inset - w * 0.3,
                1.5,
                &ICON_FG_DIM,
            );
        }
        IconId::Material => {
            // Sphere with gradient (material preview)
            let r = center - margin - 1.0;
            draw_filled_circle(pixels, size, center, center, r, &ICON_FG_DIM);
            // Highlight
            draw_filled_circle(
                pixels,
                size,
                center - r * 0.3,
                center - r * 0.3,
                r * 0.25,
                &ICON_FG,
            );
        }
        IconId::Shader => {
            // Node graph icon (two connected boxes)
            let bw = s * 0.25;
            let bh = s * 0.2;
            draw_filled_rect(
                pixels,
                size,
                margin,
                margin + 2.0,
                margin + bw,
                margin + 2.0 + bh,
                &ICON_FG_DIM,
            );
            draw_filled_rect(
                pixels,
                size,
                s - margin - bw,
                s - margin - bh - 2.0,
                s - margin,
                s - margin - 2.0,
                &ICON_FG_DIM,
            );
            // Connection line
            draw_line(
                pixels,
                size,
                margin + bw,
                margin + 2.0 + bh / 2.0,
                s - margin - bw,
                s - margin - bh / 2.0 - 2.0,
                1.5,
                &ICON_FG,
            );
        }
        IconId::Object => {
            // Generic 3D object (cube outline)
            let r = center - margin;
            draw_circle(pixels, size, center, center, r, 1.5, &ICON_FG_DIM);
            draw_filled_circle(pixels, size, center, center, r * 0.3, &ICON_FG);
        }
        _ => {}
    }
}

} // impl NSIconMediaRenderer
