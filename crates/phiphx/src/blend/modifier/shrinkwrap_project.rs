//! Project mode: `shrinkwrap_calc_normal_projection` and
//! `BKE_shrinkwrap_project_normal` in Blender's `shrinkwrap.cc`, translated.

use super::math::{interp, normalize, vertex_normals};
use super::snap::snap_point_to_surface;
use super::tree::{Hit, Target};
use super::{BShrinkwrap, Space};
use crate::blend::modifier::BlendEvalMesh;
use glam::Vec3;

const ALLOW_POS_DIR: u32 = 1 << 0;
const ALLOW_NEG_DIR: u32 = 1 << 1;
const CULL_FRONTFACE: u32 = 1 << 3;
const CULL_BACKFACE: u32 = 1 << 4;
const CULL_MASK: u32 = CULL_FRONTFACE | CULL_BACKFACE;
const INVERT_CULL_TARGET: u32 = 1 << 7;
/// `MOD_SHRINKWRAP_PROJECT_OVER_NORMAL` and the axis bits.
const OVER_NORMAL: u32 = 0;
/// `BVH_RAYCAST_DIST_MAX`.
const RAYCAST_DIST_MAX: f32 = f32::MAX / 2.0;

/// `BKE_shrinkwrap_project_normal`: cast `vert` along `dir` (both in the
/// modified object's space) at the target, keep the hit only if it is the
/// closest yet (`hit.dist`, measured in the target's space) and survives
/// the face culling.
fn project_normal(options: u32, vert: Vec3, dir: Vec3, transf: &Space, tree: &Target, hit: &mut Hit) -> bool {
    let co = transf.apply(vert);
    let no = transf.apply_normal(dir);
    let mut tmp = Hit { index: None, ..*hit };
    tree.ray_cast(co, no, &mut tmp);
    if tmp.index.is_none() {
        return false;
    }
    // Invert the normal first, so culling works on rotated objects.
    tmp.no = transf.invert_normal(tmp.no);
    if options & CULL_MASK != 0 {
        let dot = dir.dot(tmp.no);
        if (options & CULL_FRONTFACE != 0 && dot <= 0.0) || (options & CULL_BACKFACE != 0 && dot >= 0.0) {
            return false;
        }
    }
    tmp.co = transf.invert(tmp.co);
    *hit = tmp;
    true
}

impl BShrinkwrap {
    /// `shrinkwrap_calc_normal_projection` + `_cb_ex`, one vertex at a time.
    ///
    /// The ray starts at the vertex and runs along its normal (Blender's
    /// `mesh->vert_normals()`, before any vertex moves) or the chosen axis.
    /// `subsurfLevels` would first move the ray origins to the subdivision
    /// limit (`shrinkwrap_calc_subdivided_positions`); that is not ported,
    /// so the coarse positions are used.
    pub(super) fn project(&self, mesh: &mut BlendEvalMesh, tree: &Target, local2target: &Space, aux: Option<(&Target, Space)>) {
        let opts = self.opts;
        if opts & (ALLOW_POS_DIR | ALLOW_NEG_DIR) == 0 {
            return;
        }
        let positions: Vec<Vec3> = mesh.vertices.iter().map(|v| Vec3::from(*v)).collect();
        let (normals, proj_axis) = match self.proj_axis {
            OVER_NORMAL => {
                let face_normals: Vec<Vec3> = mesh.faces.iter().map(|f| super::math::face_normal(&positions, f)).collect();
                (vertex_normals(&positions, mesh.faces.iter(), &face_normals), Vec3::ZERO)
            }
            axis => {
                let bit = |b: u32| match axis & b != 0 { true => 1.0, false => 0.0 };
                let a = normalize(Vec3::new(bit(1), bit(2), bit(4)));
                if a.length_squared() < f32::EPSILON {
                    return;
                }
                (Vec::new(), a)
            }
        };
        let proj_limit_squared = self.proj_limit * self.proj_limit;
        for i in 0..positions.len() {
            let weight = self.weight(mesh, i);
            if weight == 0.0 {
                continue;
            }
            let co = positions[i];
            let (tmp_co, tmp_no) = match normals.is_empty() {
                false => (positions[i], normals[i]),
                true => (co, proj_axis),
            };
            let mut hit = Hit { index: None, dist: RAYCAST_DIST_MAX, co: Vec3::ZERO, no: Vec3::ZERO };
            let mut is_aux = false;
            let mut cast = |options: u32, dir: Vec3, hit: &mut Hit| {
                if let Some((aux_tree, local2aux)) = &aux {
                    if project_normal(0, tmp_co, dir, local2aux, aux_tree, hit) {
                        is_aux = true;
                    }
                }
                if project_normal(options, tmp_co, dir, local2target, tree, hit) {
                    is_aux = false;
                }
            };
            // Over the positive direction of the axis.
            if opts & ALLOW_POS_DIR != 0 {
                cast(opts, tmp_no, &mut hit);
            }
            // Over the negative direction, the culling flipped if asked.
            if opts & ALLOW_NEG_DIR != 0 {
                let mut options = opts;
                if options & INVERT_CULL_TARGET != 0 && options & CULL_MASK != 0 {
                    options ^= CULL_MASK;
                }
                cast(options, -tmp_no, &mut hit);
            }
            if proj_limit_squared != 0.0 && hit.index.is_some() && hit.co.distance_squared(co) > proj_limit_squared {
                hit.index = None;
            }
            let Some(index) = hit.index else { continue };
            let snapped = match (is_aux, &aux) {
                (true, Some((aux_tree, local2aux))) => {
                    snap_point_to_surface(aux_tree, Some(local2aux), self.mode, index, hit.co, hit.no, self.keep_dist, tmp_co)
                }
                _ => snap_point_to_surface(tree, Some(local2target), self.mode, index, hit.co, hit.no, self.keep_dist, tmp_co),
            };
            mesh.vertices[i] = interp(co, snapped, weight).into();
        }
    }
}
