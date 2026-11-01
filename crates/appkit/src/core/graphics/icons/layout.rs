//! Layout Panel Icon Rendering
//!
//! this file implements the procedural rendering for workspace layout toggles.
//! it draws schematic icons for left sidebar, right inspector, and bottom panels.
//! mirrors Apple's visual style for layout-aware interface controls.

use super::primitives::*;
use super::types::IconId;

/// Free-function wrapper for layout icon rendering.
pub fn render(pixels: &mut [u8], size: u32, id: &IconId, s: f32, margin: f32) {
    NSIconLayoutRenderer::render(pixels, size, id, s, margin);
}

pub struct NSIconLayoutRenderer;

impl NSIconLayoutRenderer {

/// Renders a schematic representation of a workspace layout panel.
/// uses highlights to indicate which panel (left, right, bottom, etc.) is active.
pub fn render(pixels: &mut [u8], size: u32, id: &IconId, s: f32, margin: f32) {
    let t = (s * 0.06).max(1.5);
    let inner = margin + t;
    let outer_r = s - margin;
    let outer_b = s - margin;

    // Draw outer rectangle frame
    draw_filled_rect(
        pixels,
        size,
        margin,
        margin,
        outer_r,
        margin + t,
        &ICON_FG_DIM,
    );
    draw_filled_rect(
        pixels,
        size,
        margin,
        outer_b - t,
        outer_r,
        outer_b,
        &ICON_FG_DIM,
    );
    draw_filled_rect(
        pixels,
        size,
        margin,
        margin,
        margin + t,
        outer_b,
        &ICON_FG_DIM,
    );
    draw_filled_rect(
        pixels,
        size,
        outer_r - t,
        margin,
        outer_r,
        outer_b,
        &ICON_FG_DIM,
    );

    match id {
        IconId::LayoutLeft => {
            let panel_w = (s - margin * 2.0) * 0.3;
            draw_filled_rect(
                pixels,
                size,
                inner,
                inner,
                margin + panel_w,
                outer_b - t,
                &ICON_FG,
            );
        }
        IconId::LayoutRight => {
            let panel_w = (s - margin * 2.0) * 0.3;
            draw_filled_rect(
                pixels,
                size,
                outer_r - t - panel_w,
                inner,
                outer_r - t,
                outer_b - t,
                &ICON_FG,
            );
        }
        IconId::LayoutBottom => {
            let panel_h = (s - margin * 2.0) * 0.3;
            draw_filled_rect(
                pixels,
                size,
                inner,
                outer_b - t - panel_h,
                outer_r - t,
                outer_b - t,
                &ICON_FG,
            );
        }
        IconId::LayoutTop => {
            let panel_h = (s - margin * 2.0) * 0.3;
            draw_filled_rect(
                pixels,
                size,
                inner,
                inner,
                outer_r - t,
                margin + panel_h,
                &ICON_FG,
            );
        }
        IconId::LayoutCenter => {
            let inset = (s - margin * 2.0) * 0.25;
            draw_filled_rect(
                pixels,
                size,
                margin + inset,
                margin + inset,
                outer_r - inset,
                outer_b - inset,
                &ICON_FG,
            );
        }
        _ => {}
    }
}

} // impl NSIconLayoutRenderer
