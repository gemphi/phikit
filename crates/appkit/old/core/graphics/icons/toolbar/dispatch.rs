//! Toolbar icon dispatch — routes IconId to the appropriate renderer

use super::super::primitives::*;
use super::super::types::IconId;
use super::transform;
use super::view_controls;

/// Free-function wrapper for toolbar icon rendering.
pub fn render(pixels: &mut [u8], size: u32, id: &IconId) {
    NSIconToolbarRenderer::render(pixels, size, id);
}

pub struct NSIconToolbarRenderer;

impl NSIconToolbarRenderer {
    pub fn render(pixels: &mut [u8], size: u32, id: &IconId) {
        match id {
            IconId::Select => transform::render_select(pixels, size),
            IconId::Move => transform::render_move(pixels, size),
            IconId::Rotate => transform::render_rotate(pixels, size),
            IconId::Scale => transform::render_scale(pixels, size),
            IconId::ViewPan => view_controls::render_view_pan(pixels, size),
            IconId::ViewZoom => view_controls::render_view_zoom(pixels, size),
            IconId::ViewCamera | IconId::ZoomAll => {
                view_controls::render_view_camera(pixels, size)
            }
            IconId::ZoomIn => view_controls::render_zoom_in(pixels, size),
            IconId::ZoomOut => view_controls::render_zoom_out(pixels, size),
            IconId::ShadingSolid => view_controls::render_shading_solid(pixels, size),
            _ => view_controls::render_fallback(pixels, size),
        }
    }
}
