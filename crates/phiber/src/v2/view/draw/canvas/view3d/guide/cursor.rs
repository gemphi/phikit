//! The 3D cursor (`overlay_cursor.hh`): a ring a widget unit across on the
//! screen, alternately white and red, and a crosshair along the cursor's own
//! axes drawn in 3D — so it turns and foreshortens with the view — at the
//! size that makes a widget unit at the cursor's depth. Each axis is its
//! colour at the tip, the overlay's white toward the middle, the negative
//! half a little paler and darker, and nothing at the very centre.

use super::set::*;
use crate::v2::scene::camera::{BViewSetup, SCREEN};
use crate::v2::scene::{MESH_SCALE, project3_full};
use gpui::{Bounds, Hsla, PathBuilder, Pixels, Rgba, Window, point, px};

/// The cursor's colours: the ring's red and white, the crosshair's axes.
#[derive(Clone, Copy)]
pub(super) struct BCursorInk {
    pub red: Hsla,
    pub white: Hsla,
    pub axis: [Hsla; 3],
}

/// The cursor as this frame draws it: its centre on the canvas and the
/// crosshair's segments as pixel offsets from it.
pub(super) struct BCursorMark {
    pub(super) center: (f32, f32),
    pub(super) cross: Vec<((f32, f32), (f32, f32), Hsla)>,
    ink: BCursorInk,
}

impl BCursorMark {
    /// Where `matrix` (the cursor's place and turn) lands in this view;
    /// `None` behind a perspective eye (`V3D_PROJ_TEST_CLIP_NEAR`).
    pub(super) fn of(matrix: [[f32; 4]; 4], view: (f32, f32, (f32, f32), f32), setup: BViewSetup, ink: BCursorInk) -> Option<Self> {
        let (yaw, pitch, pan, zoom) = view;
        let project = |p: [f32; 3]| project3_full(p[0] * MESH_SCALE, p[2] * MESH_SCALE, p[1] * MESH_SCALE, yaw, pitch, pan, zoom);
        let loc = [matrix[3][0], matrix[3][1], matrix[3][2]];
        let at = project(loc);
        let near = 0.05 / (SCREEN * zoom * setup.fov) * 1.001;
        if setup.perspective && at.2 <= near {
            return None;
        }
        // A step small beside what the canvas shows, for the view's local
        // scale at the cursor: the screen's right is world (cos yaw, −sin yaw, 0).
        let step = 1e-3 / (SCREEN * zoom * MESH_SCALE);
        let moved = |dir: [f32; 3]| {
            let p = project(std::array::from_fn(|i| loc[i] + dir[i] * step));
            ((p.0 - at.0) / step, (p.1 - at.1) / step)
        };
        let (sy, cy) = yaw.to_radians().sin_cos();
        let per_unit = moved([cy, -sy, 0.0]).0.abs().max(1e-9);
        // A widget unit at the cursor's depth, in world units, is `WIDGET_UNIT /
        // (per_unit · width)`; on screen an axis's step is `(dx, dy / aspect)`
        // of that, in pixels.
        let scale = WIDGET_UNIT / per_unit;
        let mut cross = Vec::with_capacity(12);
        for (i, color) in ink.axis.iter().enumerate() {
            let axis = [matrix[i][0], matrix[i][1], matrix[i][2]];
            let len = axis.iter().map(|v| v * v).sum::<f32>().sqrt().max(1e-9);
            let d = moved(axis.map(|v| v / len));
            let off = |k: f32| (d.0 * k * scale, d.1 / setup.aspect * k * scale);
            for (sign, tip) in [(1.0, *color), (-1.0, Self::paler(*color))] {
                cross.push((off(sign * OUTER), off(sign * COLOR_LIMIT), tip));
                cross.push((off(sign * COLOR_LIMIT), off(sign * INNER), ink.white));
            }
        }
        Some(Self { center: (at.0, at.1), cross, ink })
    }

    /// The negative half of an axis: a quarter of the way to white, then
    /// 60 of 255 darker (`get_color_blend_shade_3fv`).
    fn paler(c: Hsla) -> Hsla {
        let rgb = c.to_rgb();
        let f = |v: f32| (v * 0.75 + 0.25 - 60.0 / 255.0).clamp(0.0, 1.0);
        Hsla::from(Rgba { r: f(rgb.r), g: f(rgb.g), b: f(rgb.b), a: c.a })
    }

    pub(super) fn paint(&self, bounds: Bounds<Pixels>, width: f32, window: &mut Window) {
        let (w, h) = (f32::from(bounds.size.width), f32::from(bounds.size.height));
        let (cx, cy) = (f32::from(bounds.origin.x) + (0.5 + self.center.0) * w, f32::from(bounds.origin.y) + (0.5 + self.center.1) * h);
        let at = |x: f32, y: f32| point(px(cx + x), px(cy + y));
        let radius = RING_RADIUS * WIDGET_UNIT;
        for k in 0..RING_SEGMENTS {
            let angle = |k: usize| std::f32::consts::TAU * k as f32 / RING_SEGMENTS as f32;
            let mut ring = PathBuilder::stroke(px(width));
            ring.move_to(at(radius * angle(k).cos(), -radius * angle(k).sin()));
            ring.line_to(at(radius * angle(k + 1).cos(), -radius * angle(k + 1).sin()));
            if let Ok(path) = ring.build() {
                window.paint_path(path, if k % 2 == 0 { self.ink.white } else { self.ink.red });
            }
        }
        for &(a, b, color) in &self.cross {
            let mut line = PathBuilder::stroke(px(width));
            line.move_to(at(a.0, a.1));
            line.line_to(at(b.0, b.1));
            if let Ok(path) = line.build() {
                window.paint_path(path, color);
            }
        }
    }
}
