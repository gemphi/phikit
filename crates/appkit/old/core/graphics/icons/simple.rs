//! Simple Icon Rendering
//!
//! this file implements the procedural rendering for basic geometric icons.
//! it handles common symbols like plus, minus, settings, and search.
//! mirrors Apple's minimalist aesthetic for secondary UI controls.

use super::primitives::*;
use super::types::IconId;

/// Free-function wrapper for simple icon rendering.
pub fn render(pixels: &mut [u8], size: u32, id: &IconId) {
    NSIconSimpleRenderer::render(pixels, size, id);
}

pub struct NSIconSimpleRenderer;

impl NSIconSimpleRenderer {

/// Renders a basic geometric icon based on the provided identifier.
/// uses procedural drawing commands to generate pixel-perfect symbols.
pub fn render(pixels: &mut [u8], size: u32, id: &IconId) {
    let s = size as f32;
    let center = s / 2.0;
    let margin = (s * 0.15).max(2.0);

    match id {
        IconId::Add | IconId::Plus => {
            let t = (s * 0.08).max(2.0);
            draw_filled_rect(
                pixels,
                size,
                center - t / 2.0,
                margin,
                center + t / 2.0,
                s - margin,
                &ICON_FG,
            );
            draw_filled_rect(
                pixels,
                size,
                margin,
                center - t / 2.0,
                s - margin,
                center + t / 2.0,
                &ICON_FG,
            );
        }
        IconId::Remove | IconId::Minus => {
            let t = (s * 0.08).max(2.0);
            draw_filled_rect(
                pixels,
                size,
                margin,
                center - t / 2.0,
                s - margin,
                center + t / 2.0,
                &ICON_FG,
            );
        }
        IconId::Settings => {
            draw_circle(
                pixels,
                size,
                center,
                center,
                center - margin - 1.0,
                2.0,
                &ICON_FG,
            );
            draw_filled_circle(pixels, size, center, center, (s * 0.15).max(2.0), &ICON_FG);
        }
        IconId::Search => {
            let r = center - margin - 2.0;
            draw_circle(
                pixels,
                size,
                center - 2.0,
                center - 2.0,
                r * 0.6,
                2.0,
                &ICON_FG,
            );
            draw_line(
                pixels,
                size,
                center + r * 0.3,
                center + r * 0.3,
                s - margin,
                s - margin,
                2.0,
                &ICON_FG,
            );
        }
        IconId::Grid => {
            let step = (s - margin * 2.0) / 3.0;
            let t = 1.5;
            for i in 0..4 {
                let pos = margin + i as f32 * step;
                draw_filled_rect(
                    pixels,
                    size,
                    pos - t / 2.0,
                    margin,
                    pos + t / 2.0,
                    s - margin,
                    &ICON_FG_DIM,
                );
                draw_filled_rect(
                    pixels,
                    size,
                    margin,
                    pos - t / 2.0,
                    s - margin,
                    pos + t / 2.0,
                    &ICON_FG_DIM,
                );
            }
        }
        IconId::Light | IconId::Sun => {
            draw_filled_circle(pixels, size, center, center, (s * 0.2).max(3.0), &ICON_FG);
            let ray_len = (s * 0.15).max(2.0);
            let ray_start = (s * 0.3).max(4.0);
            for i in 0..8 {
                let angle = (i as f32 / 8.0) * std::f32::consts::TAU;
                let x0 = center + angle.cos() * ray_start;
                let y0 = center + angle.sin() * ray_start;
                let x1 = center + angle.cos() * (ray_start + ray_len);
                let y1 = center + angle.sin() * (ray_start + ray_len);
                draw_line(pixels, size, x0, y0, x1, y1, 1.5, &ICON_FG);
            }
        }
        IconId::Camera => {
            let bw = s * 0.6;
            let bh = s * 0.4;
            let bx = (s - bw) / 2.0;
            let by = (s - bh) / 2.0;
            draw_filled_rect(pixels, size, bx, by, bx + bw, by + bh, &ICON_FG);
            draw_filled_circle(pixels, size, center, center, bh * 0.3, &[40, 40, 45, 255]);
        }
        _ => {}
    }
}

} // impl NSIconSimpleRenderer
