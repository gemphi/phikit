//! Navigation icons — Arrows, chevrons, disclosure triangles

use super::primitives::*;
use super::types::IconId;

pub fn render(pixels: &mut [u8], size: u32, id: &IconId) {
    let s = size as f32;
    let center = s / 2.0;
    let margin = (s * 0.15).max(2.0);
    let t = (s * 0.08).max(2.0);

    match id {
        IconId::ArrowUp => {
            draw_filled_rect(
                pixels,
                size,
                center - t / 2.0,
                margin + s * 0.2,
                center + t / 2.0,
                s - margin,
                &ICON_FG,
            );
            // Arrowhead
            draw_line(
                pixels,
                size,
                center,
                margin,
                margin + 2.0,
                margin + s * 0.25,
                t,
                &ICON_FG,
            );
            draw_line(
                pixels,
                size,
                center,
                margin,
                s - margin - 2.0,
                margin + s * 0.25,
                t,
                &ICON_FG,
            );
        }
        IconId::ArrowDown => {
            draw_filled_rect(
                pixels,
                size,
                center - t / 2.0,
                margin,
                center + t / 2.0,
                s - margin - s * 0.2,
                &ICON_FG,
            );
            draw_line(
                pixels,
                size,
                center,
                s - margin,
                margin + 2.0,
                s - margin - s * 0.25,
                t,
                &ICON_FG,
            );
            draw_line(
                pixels,
                size,
                center,
                s - margin,
                s - margin - 2.0,
                s - margin - s * 0.25,
                t,
                &ICON_FG,
            );
        }
        IconId::ArrowLeft => {
            draw_filled_rect(
                pixels,
                size,
                margin + s * 0.2,
                center - t / 2.0,
                s - margin,
                center + t / 2.0,
                &ICON_FG,
            );
            draw_line(
                pixels,
                size,
                margin,
                center,
                margin + s * 0.25,
                margin + 2.0,
                t,
                &ICON_FG,
            );
            draw_line(
                pixels,
                size,
                margin,
                center,
                margin + s * 0.25,
                s - margin - 2.0,
                t,
                &ICON_FG,
            );
        }
        IconId::ArrowRight => {
            draw_filled_rect(
                pixels,
                size,
                margin,
                center - t / 2.0,
                s - margin - s * 0.2,
                center + t / 2.0,
                &ICON_FG,
            );
            draw_line(
                pixels,
                size,
                s - margin,
                center,
                s - margin - s * 0.25,
                margin + 2.0,
                t,
                &ICON_FG,
            );
            draw_line(
                pixels,
                size,
                s - margin,
                center,
                s - margin - s * 0.25,
                s - margin - 2.0,
                t,
                &ICON_FG,
            );
        }
        IconId::ChevronRight => {
            draw_line(
                pixels,
                size,
                s * 0.35,
                margin + 1.0,
                s * 0.65,
                center,
                t,
                &ICON_FG,
            );
            draw_line(
                pixels,
                size,
                s * 0.65,
                center,
                s * 0.35,
                s - margin - 1.0,
                t,
                &ICON_FG,
            );
        }
        IconId::ChevronDown => {
            draw_line(
                pixels,
                size,
                margin + 1.0,
                s * 0.35,
                center,
                s * 0.65,
                t,
                &ICON_FG,
            );
            draw_line(
                pixels,
                size,
                center,
                s * 0.65,
                s - margin - 1.0,
                s * 0.35,
                t,
                &ICON_FG,
            );
        }
        IconId::TriaRight => {
            // Filled right-pointing triangle
            let h = s - margin * 2.0;
            for row in 0..(h as i32) {
                let frac = row as f32 / h;
                let y = margin + row as f32;
                let w = (s * 0.5) * (1.0 - (frac - 0.5).abs() * 2.0);
                draw_filled_rect(pixels, size, s * 0.3, y, s * 0.3 + w, y + 1.0, &ICON_FG);
            }
        }
        IconId::TriaDown => {
            // Filled down-pointing triangle
            let w = s - margin * 2.0;
            for col in 0..(w as i32) {
                let frac = col as f32 / w;
                let x = margin + col as f32;
                let h = (s * 0.5) * (1.0 - (frac - 0.5).abs() * 2.0);
                draw_filled_rect(pixels, size, x, s * 0.3, x + 1.0, s * 0.3 + h, &ICON_FG);
            }
        }
        _ => {}
    }
}
