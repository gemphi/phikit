//! Procedural fallback renderer — kept for any icons not yet converted to SVG.

use super::super::primitives::{draw_filled_circle, ICON_FG_DIM};
use super::super::IconId;

/// Free-function wrapper for procedural icon rendering.
pub fn render_icon_procedural(id: &IconId, size: u32) -> Vec<u8> {
    NSIconProcedural::render_icon_procedural(id, size)
}

pub struct NSIconProcedural;

impl NSIconProcedural {

pub fn render_icon_procedural(id: &IconId, size: u32) -> Vec<u8> {
    let mut pixels = vec![0u8; (size * size * 4) as usize];
    let s = size as f32;
    let center = s / 2.0;
    let margin = (s * 0.15).max(2.0);

    match id {
        IconId::Select
        | IconId::Move
        | IconId::Rotate
        | IconId::Scale
        | IconId::ViewPan
        | IconId::ViewZoom
        | IconId::ViewCamera
        | IconId::ZoomAll
        | IconId::ZoomIn
        | IconId::ZoomOut => {
            super::super::toolbar::render(&mut pixels, size, id);
        }

        IconId::ArrowUp
        | IconId::ArrowDown
        | IconId::ArrowLeft
        | IconId::ArrowRight
        | IconId::ChevronRight
        | IconId::ChevronDown
        | IconId::TriaRight
        | IconId::TriaDown => {
            super::super::navigation::render(&mut pixels, size, id);
        }

        IconId::LayoutLeft
        | IconId::LayoutRight
        | IconId::LayoutBottom
        | IconId::LayoutTop
        | IconId::LayoutCenter => {
            super::super::layout::render(&mut pixels, size, id, s, margin);
        }

        IconId::Play
        | IconId::Pause
        | IconId::Stop
        | IconId::Image
        | IconId::Video
        | IconId::Audio
        | IconId::Mesh
        | IconId::Material
        | IconId::Shader
        | IconId::Object => {
            super::super::media::render(&mut pixels, size, id);
        }

        IconId::Sphere
        | IconId::Cube
        | IconId::Cylinder
        | IconId::Cone
        | IconId::Capsule
        | IconId::Plane
        | IconId::Torus => {
            super::super::shapes::render(&mut pixels, size, id);
        }

        _ => {
            super::super::simple::render(&mut pixels, size, id);
            if pixels.iter().all(|&p| p == 0) {
                draw_filled_circle(
                    &mut pixels,
                    size,
                    center,
                    center,
                    center - margin,
                    &ICON_FG_DIM,
                );
            }
        }
    }

    pixels
}

} // impl NSIconProcedural
