//! Curve deform (`eModifierType_Curve` = 3) — Blender `MOD_curve.cc` +
//! `curve_deform.cc`, ported.
//!
//! The mesh is taken into the curve object's space (`init_curve_deform`'s
//! `curvespace`), and each vertex's coordinate along the deform axis becomes
//! a fraction of the way along the curve's path (`BKE_where_on_path`, the
//! same path an object parented to the curve rides, `blend/curve/path.rs`).
//! The other two coordinates are turned by the path's rotation there —
//! re-aimed for the axis (`quat_apply_track` / `vec_apply_track`) — scaled
//! by its radius with `CU_PATH_RADIUS`, and set down at its position; then
//! back into object space (`objectspace`).
//!
//! The fraction is measured from the mesh's own box in curve space unless
//! the curve has `CU_DEFORM_BOUNDS_OFF` (Blender's default for a new curve:
//! the box is then `0..1`, or `-1..0` on a negative axis), and divided by
//! the path's length, or by the box's own extent with `CU_STRETCH`. Past
//! either end of an open path the position runs on along the end segment
//! (`where_on` extrapolates linearly); a cyclic one wraps.
//!
//! The deform asks the depsgraph for the path (`DAG_EVAL_NEED_CURVE_PATH`),
//! so it is evaluated whether or not the curve has `CU_PATH`; and
//! `CU_PATH_CLAMP` is a path-animation setting the deform never reads.
//!
//! What zips the CW-POSES garments: the stitching and zip teeth are arrays
//! bent along their seams.

use super::{BModifier, BlendEvalMesh, EvalCtx};
use crate::blend::curve::bevel::BBevelList;
use crate::blend::curve::{BCurvePath, CU_PATH_RADIUS};
use crate::blend::file::types::*;
use crate::blend::rig::math::BQuat;
use glam::{Mat4, Vec3};

/// `MOD_CURVE_INVERT_VGROUP`.
const INVERT_VGROUP: u32 = 1 << 0;
/// `Curve.flag`: `CU_DEFORM_BOUNDS_OFF`, `CU_STRETCH`.
const CU_DEFORM_BOUNDS_OFF: u32 = 1 << 6;
const CU_STRETCH: u32 = 1 << 7;
/// `OB_CURVES_LEGACY`.
const OB_CURVES_LEGACY: u32 = 2;

pub(crate) struct BCurveDeform {
    /// The curve object's saved address (its world comes from `ctx.worlds`).
    pub curve: u64,
    /// The curve's path; `None` when it has none (`anim_path_accum_length`
    /// null: an empty curve, a single point), and every vertex stays.
    pub path: Option<BCurvePath>,
    /// `BKE_anim_path_get_length`.
    pub length: f32,
    /// `Curve.flag`.
    pub flag: u32,
    /// `defaxis - 1`, clamped to 0–5: ±X, ±Y, ±Z.
    pub axis: usize,
    /// Index of the modifier's vertex group in the object's groups, if named.
    pub group: Option<usize>,
    pub invert: bool,
}

/// `BKE_anim_path_get_length`: the running length to the end of the last
/// segment (`anim_path_accum_length`), the closing one on a cyclic list.
fn path_length(list: &BBevelList) -> f32 {
    let p = &list.points;
    let mut len = 0.0f32;
    for w in p.windows(2) {
        len += w[0].vec.distance(w[1].vec);
    }
    match (list.cyclic, p.len()) {
        (true, n) if n > 1 => len + p[0].vec.distance(p[n - 1].vec),
        _ => len,
    }
}

pub(super) fn read(f: &BlendFile, md: &BlendBlock, ob: &BlendBlock, dna: &DnaType) -> Option<Box<dyn BModifier>> {
    let curve = f.ptr(md, Some(f.off("CurveModifierData", "*object", dna)?));
    let cu_ob = f.block_at_code(curve, "OB")?;
    // `is_disabled`'s type check: a surface or a text object is a `Curve`
    // datablock too, but no path.
    if f.int_field(cu_ob, "Object", "type", dna) != OB_CURVES_LEGACY {
        return None;
    }
    let cu = f.block_at_code(f.ptr(cu_ob, f.off("Object", "*data", dna)), "CU")?;
    // The splines are the curve's own DATA, resolved in its scope.
    let data = f.scoped(cu, || f.object_curve(cu_ob, dna))?;
    let length = BBevelList::first(&data).map_or(0.0, |l| path_length(&l));
    let name = f.off("CurveModifierData", "name[64]", dna).map(|o| super::armature::name64(&md.data, o)).unwrap_or_default();
    let group = (!name.is_empty())
        .then(|| f.mesh_block_of(ob, dna).and_then(|me| f.vertex_group_names(ob, me, dna).iter().position(|g| *g == name)))
        .flatten();
    // "Silly that defaxis and BKE_curve_deform_coords are off by 1".
    let defaxis = f.int_field(md, "CurveModifierData", "defaxis", dna) as u16 as i16;
    Some(Box::new(BCurveDeform {
        curve,
        path: BCurvePath::of(&data),
        length,
        flag: data.flag,
        axis: (defaxis as i32 - 1).clamp(0, 5) as usize,
        group,
        invert: f.int_field(md, "CurveModifierData", "flag", dna) & INVERT_VGROUP != 0,
    }))
}

/// `vec_apply_track`: `co` as the axis sees it (only ±X and −Y, −Z move).
fn vec_apply_track(v: Vec3, axis: usize) -> Vec3 {
    match axis {
        0 | 3 => Vec3::new(v.x, v.z, -v.y),
        4 => Vec3::new(-v.z, v.y, v.x),
        5 => Vec3::new(-v.x, -v.y, v.z),
        _ => v,
    }
}

impl BCurveDeform {
    /// `calc_curve_deform`: `co` (curve space) onto the path. `false` when
    /// the path has no place for it, and `co` is left as it was.
    fn calc(&self, path: &BCurvePath, co: &mut Vec3, dmin: Vec3, dmax: Vec3) -> bool {
        let neg = self.axis > 2;
        let index = self.axis % 3;
        let divisor = match self.flag & CU_STRETCH != 0 {
            true => dmax[index] - dmin[index],
            false => self.length,
        };
        let fac = match (divisor > f32::EPSILON, neg) {
            (false, _) => 0.0,
            (true, true) => -(co[index] - dmax[index]) / divisor,
            (true, false) => (co[index] - dmin[index]) / divisor,
        };
        let Some(at) = path.where_on(fac) else { return false };
        // The `upflag` is a dummy, set so no roll is added.
        let up = match self.axis {
            0 | 2 => 1,
            _ => 0,
        };
        let quat: BQuat = at.quat.track(self.axis, up);
        let mut cent = vec_apply_track(*co, self.axis);
        cent[index] = 0.0;
        if self.flag & CU_PATH_RADIUS != 0 {
            cent *= at.radius;
        }
        *co = quat.normalized().rotate(cent) + at.vec;
        true
    }

    /// The vertex's weight in the modifier's group (`BKE_defvert_find_weight`,
    /// inverted with `MOD_CURVE_INVERT_VGROUP`); `None` without one, where
    /// every vertex deforms in full.
    fn weight(&self, mesh: &BlendEvalMesh, n: usize, group: Option<usize>) -> Option<f32> {
        let gi = group?;
        let w = mesh.dverts[n].iter().find(|(g, _)| *g as usize == gi).map_or(0.0, |(_, w)| *w);
        Some(if self.invert { 1.0 - w } else { w })
    }
}

impl BModifier for BCurveDeform {
    fn clone_box(&self) -> Box<dyn BModifier> {
        Box::new(Self {
            curve: self.curve,
            path: self.path.clone(),
            length: self.length,
            flag: self.flag,
            axis: self.axis,
            group: self.group,
            invert: self.invert,
        })
    }

    /// `MOD_curve.cc`'s `is_disabled`: `!object || object->type !=
    /// OB_CURVES_LEGACY`; the reader drops the second case.
    fn disabled(&self) -> bool {
        self.curve == 0
    }

    fn wants_weights(&self) -> bool {
        self.group.is_some()
    }

    /// `curve_deform_coords_impl`, without the edit-mesh branch.
    fn apply(&self, mesh: &mut BlendEvalMesh, ctx: &EvalCtx) {
        let Some(curve_world) = ctx.worlds.get(&self.curve) else { return };
        let Some(path) = &self.path else { return };
        // `init_curve_deform`.
        let objectspace: Mat4 = ctx.world.inverse() * *curve_world;
        let curvespace = objectspace.inverse();
        let (mut dmin, mut dmax) = match (self.flag & CU_DEFORM_BOUNDS_OFF != 0, self.axis > 2) {
            (true, false) => (Vec3::ZERO, Vec3::ONE),
            (true, true) => (Vec3::NEG_ONE, Vec3::ZERO),
            (false, _) => (Vec3::splat(f32::MAX), Vec3::splat(-f32::MAX)),
        };
        let bounds = self.flag & CU_DEFORM_BOUNDS_OFF == 0;
        // `MOD_get_vgroup`: no deform-vertex layer is no group at all.
        let group = self.group.filter(|_| mesh.dverts.len() == mesh.vertices.len());
        // Pass one: into curve space — only a vertex with weight, under a
        // group — and the box round them (`DEFORM_OP_MINMAX`).
        let mut weights = Vec::with_capacity(mesh.vertices.len());
        for n in 0..mesh.vertices.len() {
            let w = self.weight(mesh, n, group);
            if w.is_some_and(|w| w <= 0.0) {
                weights.push(None);
                continue;
            }
            let co = curvespace.transform_point3(Vec3::from(mesh.vertices[n]));
            mesh.vertices[n] = co.into();
            if bounds {
                dmin = dmin.min(co);
                dmax = dmax.max(co);
            }
            weights.push(Some(w));
        }
        // Pass two: onto the path, blended by weight, and back
        // (`DEFORM_OP_CLAMPED`).
        for (co, w) in mesh.vertices.iter_mut().zip(weights) {
            let Some(w) = w else { continue };
            let mut p = Vec3::from(*co);
            match w {
                None => {
                    self.calc(path, &mut p, dmin, dmax);
                }
                Some(t) => {
                    let mut vec = p;
                    self.calc(path, &mut vec, dmin, dmax);
                    // `interp_v3_v3v3`.
                    p = p * (1.0 - t) + vec * t;
                }
            }
            *co = objectspace.transform_point3(p).into();
        }
    }
}
