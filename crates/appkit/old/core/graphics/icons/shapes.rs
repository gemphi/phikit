//! 3D primitive shape icons — Sphere, Cube, Cylinder, Cone, etc.

use super::primitives::*;
use super::types::IconId;

pub fn render(pixels: &mut [u8], size: u32, id: &IconId) {
    let s = size as f32;
    let center = s / 2.0;
    let margin = (s * 0.15).max(2.0);

    match id {
        IconId::Sphere => {
            let r = center - margin - 1.0;
            draw_circle(pixels, size, center, center, r, 1.5, &ICON_FG);
            // Equator line
            draw_line(
                pixels,
                size,
                center - r,
                center,
                center + r,
                center,
                1.0,
                &ICON_FG_DIM,
            );
            // Meridian (vertical ellipse approximation)
            let steps = 24;
            for i in 0..steps {
                let angle = (i as f32 / steps as f32) * std::f32::consts::TAU;
                let x = center + angle.cos() * r * 0.3;
                let y = center + angle.sin() * r;
                set_pixel(pixels, size, x as i32, y as i32, &ICON_FG_DIM);
            }
        }
        IconId::Cube => {
            let inset = margin + 2.0;
            let w = s - inset * 2.0;
            let offset = w * 0.25;
            // Front face
            draw_line(
                pixels,
                size,
                inset,
                inset + offset,
                inset + w - offset,
                inset + offset,
                1.5,
                &ICON_FG,
            );
            draw_line(
                pixels,
                size,
                inset,
                inset + offset,
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
                inset + w - offset,
                s - inset,
                1.5,
                &ICON_FG,
            );
            draw_line(
                pixels,
                size,
                inset + w - offset,
                inset + offset,
                inset + w - offset,
                s - inset,
                1.5,
                &ICON_FG,
            );
            // Top face
            draw_line(
                pixels,
                size,
                inset,
                inset + offset,
                inset + offset,
                inset,
                1.5,
                &ICON_FG,
            );
            draw_line(
                pixels,
                size,
                inset + offset,
                inset,
                s - inset,
                inset,
                1.5,
                &ICON_FG,
            );
            draw_line(
                pixels,
                size,
                s - inset,
                inset,
                inset + w - offset,
                inset + offset,
                1.5,
                &ICON_FG,
            );
            // Right face
            draw_line(
                pixels,
                size,
                inset + w - offset,
                s - inset,
                s - inset,
                s - inset - offset,
                1.5,
                &ICON_FG_DIM,
            );
            draw_line(
                pixels,
                size,
                s - inset,
                inset,
                s - inset,
                s - inset - offset,
                1.5,
                &ICON_FG_DIM,
            );
        }
        IconId::Cylinder => {
            let w = s * 0.5;
            let h = s * 0.6;
            let x = (s - w) / 2.0;
            let y = (s - h) / 2.0;
            // Side lines
            draw_line(pixels, size, x, y + h * 0.15, x, y + h, 1.5, &ICON_FG);
            draw_line(
                pixels,
                size,
                x + w,
                y + h * 0.15,
                x + w,
                y + h,
                1.5,
                &ICON_FG,
            );
            // Top ellipse
            let steps = 24;
            for i in 0..steps {
                let angle = (i as f32 / steps as f32) * std::f32::consts::TAU;
                let px = center + angle.cos() * w / 2.0;
                let py = y + h * 0.15 + angle.sin() * h * 0.1;
                set_pixel(pixels, size, px as i32, py as i32, &ICON_FG);
            }
            // Bottom ellipse
            for i in 0..steps {
                let angle = (i as f32 / steps as f32) * std::f32::consts::TAU;
                let px = center + angle.cos() * w / 2.0;
                let py = y + h + angle.sin() * h * 0.1;
                set_pixel(pixels, size, px as i32, py as i32, &ICON_FG_DIM);
            }
        }
        IconId::Cone => {
            let base_w = s * 0.6;
            let h = s * 0.7;
            let base_y = (s + h) / 2.0;
            let tip_y = (s - h) / 2.0;
            // Side lines
            draw_line(
                pixels,
                size,
                center,
                tip_y,
                center - base_w / 2.0,
                base_y,
                1.5,
                &ICON_FG,
            );
            draw_line(
                pixels,
                size,
                center,
                tip_y,
                center + base_w / 2.0,
                base_y,
                1.5,
                &ICON_FG,
            );
            // Base ellipse
            let steps = 24;
            for i in 0..steps {
                let angle = (i as f32 / steps as f32) * std::f32::consts::TAU;
                let px = center + angle.cos() * base_w / 2.0;
                let py = base_y + angle.sin() * (s * 0.06);
                set_pixel(pixels, size, px as i32, py as i32, &ICON_FG_DIM);
            }
        }
        IconId::Capsule => {
            let w = s * 0.35;
            let h = s * 0.6;
            let x = (s - w) / 2.0;
            let y = (s - h) / 2.0;
            let cap_r = w / 2.0;
            // Side lines
            draw_line(pixels, size, x, y + cap_r, x, y + h - cap_r, 1.5, &ICON_FG);
            draw_line(
                pixels,
                size,
                x + w,
                y + cap_r,
                x + w,
                y + h - cap_r,
                1.5,
                &ICON_FG,
            );
            // Top cap
            let steps = 12;
            for i in 0..=steps {
                let angle = std::f32::consts::PI + (i as f32 / steps as f32) * std::f32::consts::PI;
                let px = center + angle.cos() * cap_r;
                let py = y + cap_r + angle.sin() * cap_r;
                set_pixel(pixels, size, px as i32, py as i32, &ICON_FG);
            }
            // Bottom cap
            for i in 0..=steps {
                let angle = (i as f32 / steps as f32) * std::f32::consts::PI;
                let px = center + angle.cos() * cap_r;
                let py = y + h - cap_r + angle.sin() * cap_r;
                set_pixel(pixels, size, px as i32, py as i32, &ICON_FG);
            }
        }
        IconId::Plane => {
            // Flat quad in perspective
            let inset = margin + 3.0;
            draw_line(
                pixels,
                size,
                inset + s * 0.1,
                center - s * 0.1,
                s - inset - s * 0.1,
                center - s * 0.1,
                1.5,
                &ICON_FG,
            );
            draw_line(
                pixels,
                size,
                inset,
                center + s * 0.15,
                s - inset,
                center + s * 0.15,
                1.5,
                &ICON_FG,
            );
            draw_line(
                pixels,
                size,
                inset + s * 0.1,
                center - s * 0.1,
                inset,
                center + s * 0.15,
                1.5,
                &ICON_FG,
            );
            draw_line(
                pixels,
                size,
                s - inset - s * 0.1,
                center - s * 0.1,
                s - inset,
                center + s * 0.15,
                1.5,
                &ICON_FG,
            );
        }
        IconId::Torus => {
            // Donut shape (two concentric ellipses)
            let outer_r = center - margin - 1.0;
            let inner_r = outer_r * 0.5;
            draw_circle(pixels, size, center, center, outer_r, 1.5, &ICON_FG);
            draw_circle(pixels, size, center, center, inner_r, 1.0, &ICON_FG_DIM);
        }
        _ => {}
    }
}
