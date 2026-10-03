//! Each object's origin (`overlay_origin.hh`): a dot where the object
//! stands — the active object's in the active colour whether or not it is
//! selected, each selected one's in the selection colour, and with Origins
//! (All) every other one's in the wire colour. A dot is ringed by a pixel of
//! the overlay's light so it reads over any surface.

use super::set::{ORIGIN_DIAMETER, ORIGIN_RING};
use crate::v2::scene::BObject3D;
use crate::v2::scene::pick::extra::BPickExtra;
use crate::v2::scene::select::BSelection;
use crate::v2::scene::camera::{BViewSetup, SCREEN};
use crate::v2::scene::{MESH_SCALE, project3_full};
use gpui::{BorderStyle, Bounds, Hsla, Pixels, Window, point, px, quad, size};

/// The dots' colours.
#[derive(Clone, Copy)]
pub(super) struct BOriginInk {
    pub active: Hsla,
    pub selected: Hsla,
    pub other: Hsla,
}

/// One origin on the canvas.
pub(super) struct BOriginDot {
    pub(super) at: (f32, f32),
    pub(super) fill: Hsla,
}

impl BOriginDot {
    /// The dots of `scene` (meshes, rigs, the default primitives) and
    /// `extras` (cameras, lights, empties), each object once.
    pub(super) fn list(
        selection: &BSelection,
        active: &str,
        all: bool,
        scene: &[BObject3D],
        extras: &[BPickExtra],
        view: (f32, f32, (f32, f32), f32),
        setup: BViewSetup,
        ink: BOriginInk,
    ) -> Vec<Self> {
        let (yaw, pitch, pan, zoom) = view;
        // Behind a perspective eye, nothing is drawn.
        let near = 0.05 / (SCREEN * zoom * setup.fov) * 1.001;
        let world = |p: [f32; 3]| project3_full(p[0] * MESH_SCALE, p[2] * MESH_SCALE, p[1] * MESH_SCALE, yaw, pitch, pan, zoom);
        let placed = scene.iter().map(|object| match object {
            BObject3D::Primitive { name, x, y, .. } => (name.as_str(), project3_full(*x, 0.0, *y, yaw, pitch, pan, zoom)),
            BObject3D::Loaded { name, matrix_world: m, .. } => (name.as_str(), world([m[3][0], m[3][1], m[3][2]])),
        });
        let extras = extras.iter().map(|e| (e.name.as_str(), world(e.origin.to_array())));
        let mut seen = std::collections::HashSet::new();
        placed
            .chain(extras)
            .filter(|(name, at)| seen.insert(*name) && (!setup.perspective || at.2 > near))
            .filter_map(|(name, at)| {
                let fill = match () {
                    _ if name == active => ink.active,
                    _ if selection.contains(name) => ink.selected,
                    _ if all => ink.other,
                    _ => return None,
                };
                Some(Self { at: (at.0, at.1), fill })
            })
            .collect()
    }

    pub(super) fn paint(dots: &[Self], ring: Hsla, bounds: Bounds<Pixels>, window: &mut Window) {
        let (w, h) = (f32::from(bounds.size.width), f32::from(bounds.size.height));
        let r = ORIGIN_DIAMETER / 2.0;
        for dot in dots {
            let (x, y) = (f32::from(bounds.origin.x) + (0.5 + dot.at.0) * w, f32::from(bounds.origin.y) + (0.5 + dot.at.1) * h);
            let at = Bounds::new(point(px(x - r), px(y - r)), size(px(ORIGIN_DIAMETER), px(ORIGIN_DIAMETER)));
            window.paint_quad(quad(at, px(r), dot.fill, px(ORIGIN_RING), ring, BorderStyle::Solid));
        }
    }
}
