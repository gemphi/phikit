//! Where a hit becomes a position: `BKE_shrinkwrap_snap_point_to_surface`
//! and its helpers in Blender's `shrinkwrap.cc`, translated.

use super::math::{interp, interp_weights_tri, normalize};
use super::tree::Target;
use super::Space;
use glam::Vec3;

/// `eShrinkwrapModifierMode`.
const ON_SURFACE: u32 = 0;
const INSIDE: u32 = 1;
const OUTSIDE: u32 = 2;
const OUTSIDE_SURFACE: u32 = 3;
const ABOVE_SURFACE: u32 = 4;

/// `BKE_shrinkwrap_compute_smooth_normal`: the target's normal at a hit.
///
/// Translated as written, including its test: the vertex normals are
/// interpolated when the mesh has no `sharp_face` attribute **or the face is
/// sharp**, and a smooth face of a mesh that has the attribute takes its
/// flat face normal. Corner normals stand in where the mesh's normal domain
/// is Corner; with no custom normals and no sharp edges read, a sharp face's
/// corner normals are its face normal.
fn smooth_normal(tree: &Target, transform: Option<&Space>, tri_idx: usize, hit_co: Vec3) -> Vec3 {
    let face = tree.tri_face[tri_idx] as usize;
    match tree.sharp.is_empty() || tree.sharp[face] {
        true => {
            let t = tree.tris[tri_idx];
            let no = match tree.corner_domain {
                true => [tree.face_normals[face]; 3],
                false => [0, 1, 2].map(|k| tree.vert_normals[t[k] as usize]),
            };
            let co = transform.map_or(hit_co, |s| s.apply(hit_co));
            let [a, b, c] = tree.tri(tri_idx);
            let w = interp_weights_tri(a, b, c, co);
            let r = no[0] * w[0] + no[1] * w[1] + no[2] * w[2];
            match transform {
                Some(s) => s.invert_normal(r),
                None => normalize(r),
            }
        }
        false => {
            let r = tree.face_normals[face];
            transform.map_or(r, |s| s.invert_normal(r))
        }
    }
}

/// `shrinkwrap_snap_with_side`: Inside, Outside and Outside Surface.
fn snap_with_side(point_co: Vec3, hit_co: Vec3, hit_no: Vec3, goal_dist: f32, forcesign: f32, forcesnap: bool) -> Vec3 {
    let delta = point_co - hit_co;
    let dist = delta.length();
    // Exactly on the surface: push out along the normal.
    if dist < f32::EPSILON {
        return match forcesnap || goal_dist > 0.0 {
            true => hit_co + hit_no * (goal_dist * forcesign),
            false => hit_co,
        };
    }
    // `signf`.
    let dsign = match delta.dot(hit_no) < 0.0 { true => -1.0, false => 1.0 };
    let forcesign = match forcesign == 0.0 { true => dsign, false => forcesign };
    // On the wrong side or too close: move to the right one.
    if !(forcesnap || dsign * dist * forcesign < goal_dist) {
        return point_co;
    }
    let mut delta = delta * (dsign / dist);
    // At very small distances, blend in the hit normal to keep the math stable.
    let dist_epsilon = (goal_dist.abs() + hit_co.x.abs() + hit_co.y.abs() + hit_co.z.abs()) * 1e-4;
    if dist < dist_epsilon {
        delta = interp(hit_no, delta, dist / dist_epsilon);
    }
    hit_co + delta * (goal_dist * forcesign)
}

/// `BKE_shrinkwrap_snap_point_to_surface`: the snapped position for one hit,
/// in the space `hit_co` and `point_co` are in (`transform` takes it to the
/// target's, for the smooth normal's barycentric weights).
#[allow(clippy::too_many_arguments)]
pub(super) fn snap_point_to_surface(
    tree: &Target,
    transform: Option<&Space>,
    mode: u32,
    hit_idx: usize,
    hit_co: Vec3,
    hit_no: Vec3,
    goal_dist: f32,
    point_co: Vec3,
) -> Vec3 {
    match mode {
        ON_SURFACE => match goal_dist != 0.0 {
            true => snap_with_side(point_co, hit_co, hit_no, goal_dist, 0.0, true),
            false => hit_co,
        },
        INSIDE => snap_with_side(point_co, hit_co, hit_no, goal_dist, -1.0, false),
        OUTSIDE => snap_with_side(point_co, hit_co, hit_no, goal_dist, 1.0, false),
        OUTSIDE_SURFACE => match goal_dist != 0.0 {
            true => snap_with_side(point_co, hit_co, hit_no, goal_dist, 1.0, true),
            false => hit_co,
        },
        ABOVE_SURFACE => match goal_dist != 0.0 {
            true => hit_co + smooth_normal(tree, transform, hit_idx, hit_co) * goal_dist,
            false => hit_co,
        },
        _ => hit_co,
    }
}
