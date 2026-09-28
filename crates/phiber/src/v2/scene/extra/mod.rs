//! Viewport overlay "extras" — the wire shapes Blender's overlay engine draws
//! for cameras, lights and empties (`overlay_extra.cc`, `overlay_empty.hh`,
//! `BKE_camera_view_frame_ex`). Pure: returns world-space segments; the
//! canvas projects and strokes them.

pub mod set;

use crate::phiphx::scene::{SceneCamera, SceneEmpty, SceneLight, SceneModel};
use glam::{Mat4, Vec3};
use set::*;

/// What a segment belongs to — the painter picks a colour per kind.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BExtraKind {
    Camera,
    Light,
    Empty,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BExtraLine {
    pub kind: BExtraKind,
    pub a: [f32; 3],
    pub b: [f32; 3],
    pub name_index: usize,
    /// The object's origin, where it stands — what box, circle and lasso
    /// select test (`pick::extra`).
    pub origin: [f32; 3],
}

impl BExtraLine {
    /// The object the line belongs to, by its name in the file.
    pub fn file_name<'a>(&self, model: &'a SceneModel) -> Option<&'a str> {
        match self.kind {
            BExtraKind::Camera => model.cameras.get(self.name_index).map(|c| c.name.as_str()),
            BExtraKind::Light => model.lights.get(self.name_index).map(|l| l.name.as_str()),
            BExtraKind::Empty => model.empties.get(self.name_index).map(|e| e.name.as_str()),
        }
    }
}

fn ring(r: f32, plane: (usize, usize), offset: [f32; 3], n: usize) -> Vec<BSeg> {
    (0..n)
        .map(|i| {
            let p = |k: usize| {
                let t = k as f32 / n as f32 * std::f32::consts::TAU;
                let mut v = offset;
                v[plane.0] += r * t.cos();
                v[plane.1] += r * t.sin();
                v
            };
            (p(i), p(i + 1))
        })
        .collect()
}

/// Blender `BKE_camera_view_frame_ex` (scale = 1, no shift) + the pyramid
/// and the "up" triangle `overlay_extra` draws above the frame.
pub fn camera_shape(c: &SceneCamera, aspect: [f32; 2]) -> Vec<BSeg> {
    let (fx, fy, depth) = if c.kind == "ORTHO" {
        let f = 0.5 * c.ortho_scale;
        (f * aspect[0], f * aspect[1], -c.display_size)
    } else {
        let half_sensor = 0.5 * c.sensor_width.max(1e-3);
        let size = c.display_size / 2.0;
        (size * aspect[0], size * aspect[1], size * c.focal_length_mm / -half_sensor)
    };
    let f = [[fx, fy, depth], [fx, -fy, depth], [-fx, -fy, depth], [-fx, fy, depth]];
    let mut out = Vec::with_capacity(11);
    for i in 0..4 {
        out.push((f[i], f[(i + 1) % 4]));
        if c.kind != "ORTHO" {
            out.push(([0.0; 3], f[i]));
        }
    }
    // Up-indicator triangle above the frame's top edge.
    let tip = [0.0, fy * 1.7, depth];
    out.push(([-fx * 0.7, fy * 1.1, depth], tip));
    out.push((tip, [fx * 0.7, fy * 1.1, depth]));
    out.push(([fx * 0.7, fy * 1.1, depth], [-fx * 0.7, fy * 1.1, depth]));
    out
}

/// Light gizmos: a small ring for every light, plus the type-specific part.
pub fn light_shape(l: &SceneLight) -> Vec<BSeg> {
    let mut out = ring(0.1, PLANE_XY, [0.0; 3], 16);
    match l.kind.as_str() {
        "SUN" => out.push(([0.0; 3], [0.0, 0.0, -1.0])),
        "SPOT" => {
            let r = (l.spot_size * 0.5).tan();
            let base = ring(r, PLANE_XY, [0.0, 0.0, -1.0], CONE_SEGMENTS * 2);
            for s in base.iter().step_by(4) {
                out.push(([0.0; 3], s.0));
            }
            out.extend(base);
        }
        "AREA" => {
            let h = l.size * 0.5;
            let q = [[h, h, 0.0], [h, -h, 0.0], [-h, -h, 0.0], [-h, h, 0.0]];
            out.extend((0..4).map(|i| (q[i], q[(i + 1) % 4])));
            out.push(([0.0; 3], [0.0, 0.0, -0.5]));
        }
        _ => out.extend(ring(l.size.max(0.05), PLANE_XY, [0.0; 3], 16)),
    }
    out
}

pub fn empty_shape(e: &SceneEmpty) -> Vec<BSeg> {
    match e.display_type.as_str() {
        "ARROWS" => ARROWS.to_vec(),
        "SINGLE_ARROW" => SINGLE_ARROW.to_vec(),
        "CUBE" => CUBE.to_vec(),
        "CIRCLE" => ring(1.0, PLANE_XZ, [0.0; 3], RING_SEGMENTS),
        "SPHERE" => [PLANE_XY, PLANE_XZ, PLANE_YZ]
            .into_iter()
            .flat_map(|p| ring(1.0, p, [0.0; 3], RING_SEGMENTS))
            .collect(),
        "CONE" => {
            let base = ring(1.0, PLANE_XZ, [0.0; 3], CONE_SEGMENTS);
            let mut out: Vec<BSeg> = base.iter().map(|s| (s.0, [0.0, 2.0, 0.0])).collect();
            out.extend(base);
            out
        }
        _ => PLAIN_AXES.to_vec(),
    }
}

fn push(out: &mut Vec<BExtraLine>, kind: BExtraKind, idx: usize, m: &[[f32; 4]; 4], scale: f32, segs: Vec<BSeg>) {
    let m = Mat4::from_cols_array_2d(m);
    let origin = m.w_axis.truncate().to_array();
    for (a, b) in segs {
        let t = |p: [f32; 3]| m.transform_point3(Vec3::from(p) * scale).to_array();
        out.push(BExtraLine { kind, a: t(a), b: t(b), name_index: idx, origin });
    }
}

/// Every overlay segment for a loaded scene, in world space.
pub fn scene_extras(model: &SceneModel) -> Vec<BExtraLine> {
    scene_extras_at(model, super::motion::scoped_frame())
}

/// The same, with every camera, light and empty where it stands at `frame`.
pub fn scene_extras_at(model: &SceneModel, frame: Option<f32>) -> Vec<BExtraLine> {
    scene_extras_moved(model, frame, None)
}

/// What the viewport draws: every camera, light and empty where it stands
/// at the frame in scope, after the Transform panel's and G / R / S's edits
/// (`transform::BTransforms::extra_worlds`) — a light parented to a moved
/// cube comes along.
pub fn scene_extras_now(s: &crate::v2::state::BState, model: &SceneModel) -> Vec<BExtraLine> {
    let frame = super::motion::scoped_frame();
    let moved = super::transform::BTransforms::now(s).extra_worlds(s, model, frame);
    scene_extras_moved(model, frame, moved.as_ref())
}

/// `scene_extras_at`, each extra `moved` names at its world there.
pub fn scene_extras_moved(model: &SceneModel, frame: Option<f32>, moved: Option<&std::collections::HashMap<String, Mat4>>) -> Vec<BExtraLine> {
    let r = &model.render;
    let (ax, ay) = (r.resolution_x.max(1) as f32, r.resolution_y.max(1) as f32);
    let aspect = if ax >= ay { [1.0, ay / ax] } else { [ax / ay, 1.0] };
    let posed = super::motion::posed(model, frame);
    let mut out = Vec::new();
    let at = |name: &str| moved.and_then(|w| w.get(name)).map(|m| m.to_cols_array_2d());
    for (i, c) in model.cameras.iter().enumerate() {
        let m = at(&c.name).or_else(|| posed.as_ref().and_then(|p| p.camera(i))).unwrap_or(c.matrix_world);
        push(&mut out, BExtraKind::Camera, i, &m, 1.0, camera_shape(c, aspect));
    }
    for (i, l) in model.lights.iter().enumerate() {
        let m = at(&l.name).or_else(|| posed.as_ref().and_then(|p| p.light(i))).unwrap_or(l.matrix_world);
        push(&mut out, BExtraKind::Light, i, &m, 1.0, light_shape(l));
    }
    for (i, e) in model.empties.iter().enumerate() {
        let m = at(&e.name).or_else(|| posed.as_ref().and_then(|p| p.empty(i))).unwrap_or(e.matrix_world);
        push(&mut out, BExtraKind::Empty, i, &m, e.display_size, empty_shape(e));
    }
    out
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
