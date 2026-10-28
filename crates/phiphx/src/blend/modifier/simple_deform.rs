//! Simple Deform (`eModifierType_SimpleDeform` = 28) — Blender
//! `MOD_simpledeform.cc`, ported.
//!
//! Each vertex is taken into the origin object's space when there is one
//! (`BLI_SPACE_TRANSFORM_SETUP(ob, origin)`), clamped to the limits along
//! the limit axis (the part cut off, `dcut`, is added back after), its axes
//! rotated so the deform axis is Z (`axis_map_table`; Bend always maps Z),
//! twisted, bent, tapered or stretched, mapped back, blended with its
//! vertex-group weight and taken back to object space. The limits are
//! fractions of the mesh's extent along the limit axis, and the factor is
//! divided by the limited extent, so the deform is the same however large
//! the mesh is.

use super::driver::BModValue;
use super::{BModifier, BlendEvalMesh, EvalCtx};
use crate::blend::file::types::*;
use glam::{Mat4, Vec3};

/// `BEND_EPS`.
const BEND_EPS: f32 = 0.000001;
/// `MOD_SIMPLEDEFORM_FLAG_INVERT_VGROUP`.
const INVERT_VGROUP: u32 = 1 << 0;
/// `MOD_SIMPLEDEFORM_LOCK_AXIS_X`, `_Y`, `_Z`.
const LOCK: [u32; 3] = [1 << 0, 1 << 1, 1 << 2];

/// `axis_map_table`: the deform axis brought to Z.
const AXIS_MAP: [[usize; 3]; 3] = [[1, 2, 0], [2, 0, 1], [0, 1, 2]];

/// `SimpleDeformModifierMode`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum BSimpleMode {
    Twist,
    Bend,
    Taper,
    Stretch,
}

impl BSimpleMode {
    fn from_dna(mode: u32) -> Option<Self> {
        match mode {
            1 => Some(Self::Twist),
            2 => Some(Self::Bend),
            3 => Some(Self::Taper),
            4 => Some(Self::Stretch),
            _ => None,
        }
    }

    /// `simpleDeform_twist` / `_bend` / `_taper` / `_stretch` on a mapped
    /// coordinate.
    fn deform(self, factor: f32, axis: usize, dcut: Vec3, co: Vec3) -> Vec3 {
        let (x, y, z) = (co.x, co.y, co.z);
        match self {
            Self::Taper => {
                let scale = z * factor;
                Vec3::new(x + x * scale, y + y * scale, z) + dcut
            }
            Self::Stretch => {
                let scale = z * z * factor - factor + 1.0;
                Vec3::new(x * scale, y * scale, z * (1.0 + factor)) + dcut
            }
            Self::Twist => {
                let (sint, cost) = (z * factor).sin_cos();
                Vec3::new(x * cost - y * sint, x * sint + y * cost, z) + dcut
            }
            Self::Bend => bend(factor, axis, dcut, x, y, z),
        }
    }
}

/// `simpleDeform_bend`, in its own order of operations (#85470).
fn bend(factor: f32, axis: usize, dcut: Vec3, x: f32, y: f32, z: f32) -> Vec3 {
    let theta = match axis {
        0 | 1 => z * factor,
        _ => x * factor,
    };
    let (sint, cost) = theta.sin_cos();
    match axis {
        0 => Vec3::new(
            x + dcut.x,
            y * cost + (1.0 - cost) / factor + sint * dcut.z,
            -(y - 1.0 / factor) * sint + cost * dcut.z,
        ),
        1 => Vec3::new(
            x * cost + (1.0 - cost) / factor + sint * dcut.z,
            y + dcut.y,
            -(x - 1.0 / factor) * sint + cost * dcut.z,
        ),
        _ => Vec3::new(
            -(y - 1.0 / factor) * sint + cost * dcut.x,
            y * cost + (1.0 - cost) / factor + sint * dcut.x,
            z + dcut.z,
        ),
    }
}

/// `axis_limit`: clamp `co[axis]` into `limits`, keeping what was cut.
fn axis_limit(axis: usize, limits: [f32; 2], co: &mut Vec3, dcut: &mut Vec3) {
    let val = co[axis].max(limits[0]).min(limits[1]);
    dcut[axis] = co[axis] - val;
    co[axis] = val;
}

#[derive(Clone)]
pub(crate) struct BSimpleDeform {
    /// `None`: a mode the file names that Blender does not know, which
    /// leaves every vertex (`default: return`).
    pub mode: Option<BSimpleMode>,
    /// The origin object's saved address, 0 for none.
    pub origin: u64,
    pub factor: f32,
    pub limit: [f32; 2],
    /// `SimpleDeformModifierData.axis`: the lock axes.
    pub lock: u32,
    /// `deform_axis`, clamped to 0–2.
    pub deform_axis: usize,
    /// Index of the modifier's vertex group in the mesh's groups, if found.
    pub group: Option<usize>,
    pub invert: bool,
}

pub(super) fn read(f: &BlendFile, md: &BlendBlock, ob: &BlendBlock, dna: &DnaType) -> Option<Box<dyn BModifier>> {
    let st = "SimpleDeformModifierData";
    let origin = f.ptr(md, f.off(st, "*origin", dna));
    let name = f.off(st, "vgroup_name[64]", dna).map(|o| super::armature::name64(&md.data, o)).unwrap_or_default();
    let group = (!name.is_empty())
        .then(|| f.mesh_block_of(ob, dna).and_then(|me| f.vertex_group_names(ob, me, dna).iter().position(|g| *g == name)))
        .flatten();
    let limit = f.off(st, "limit[2]", dna).map_or([0.0, 1.0], |o| [f.f32_at(&md.data, o), f.f32_at(&md.data, o + 4)]);
    Some(Box::new(BSimpleDeform {
        mode: BSimpleMode::from_dna(f.int_field(md, st, "mode", dna)),
        // "Safe-check: no self references".
        origin: (origin != ob.old_address).then_some(origin).unwrap_or(0),
        factor: f.float_field(md, st, "factor", dna, 45f32.to_radians()),
        limit,
        lock: f.int_field(md, st, "axis", dna),
        deform_axis: (f.int_field(md, st, "deform_axis", dna) as i32).clamp(0, 2) as usize,
        group,
        invert: f.int_field(md, st, "flag", dna) & INVERT_VGROUP != 0,
    }))
}

impl BModifier for BSimpleDeform {
    fn clone_box(&self) -> Box<dyn BModifier> {
        Box::new(self.clone())
    }

    fn set_driven(&mut self, prop: &str, component: usize, value: BModValue) -> bool {
        match (prop, component) {
            ("factor" | "angle", _) => self.factor = value.as_f32(),
            ("limits", 0 | 1) => self.limit[component] = value.as_f32(),
            _ => return false,
        }
        true
    }

    fn wants_weights(&self) -> bool {
        self.group.is_some()
    }

    /// `SimpleDeformModifier_do`.
    fn apply(&self, mesh: &mut BlendEvalMesh, ctx: &EvalCtx) {
        let Some(mode) = self.mode else { return };
        let axis = self.deform_axis;
        let lock = match mode {
            // Bend has no lock axis; the others never lock the deform axis,
            // which would flatten the mesh.
            BSimpleMode::Bend => 0,
            _ => self.lock & !LOCK[axis],
        };
        let limit0 = self.limit[0].clamp(0.0, 1.0).min(self.limit[1]);
        // `BLI_SPACE_TRANSFORM_SETUP(transf, ob, origin)`.
        let transf: Option<(Mat4, Mat4)> = (self.origin != 0).then(|| ctx.worlds.get(&self.origin)).flatten().map(|o| {
            let local2target = o.inverse() * ctx.world;
            (local2target, local2target.inverse())
        });
        let to_space = |p: Vec3| transf.map_or(p, |(m, _)| m.transform_point3(p));
        let limit_axis = match (mode, axis) {
            (BSimpleMode::Bend, 0 | 1) => 2,
            (BSimpleMode::Bend, _) => 0,
            _ => axis,
        };
        let (lower, upper) = mesh.vertices.iter().fold((f32::MAX, -f32::MAX), |(lo, hi), v| {
            let t = to_space(Vec3::from(*v))[limit_axis];
            (lo.min(t), hi.max(t))
        });
        let limits = [lower + (upper - lower) * limit0, lower + (upper - lower) * self.limit[1]];
        let factor = self.factor / f32::EPSILON.max(limits[1] - limits[0]);
        if mode == BSimpleMode::Bend && factor.abs() < BEND_EPS {
            return;
        }
        let map = AXIS_MAP[match mode {
            BSimpleMode::Bend => 2,
            _ => axis,
        }];
        let weighted = self.group.map(|g| (g, mesh.dverts.len() == mesh.vertices.len()));
        for (i, v) in mesh.vertices.iter_mut().enumerate() {
            // `BKE_defvert_array_find_weight_safe`: no group 1, a group the
            // mesh carries no weights for 0 (1 inverted).
            let weight = match weighted {
                None => 1.0,
                Some((_, false)) => f32::from(u8::from(self.invert)),
                Some((g, true)) => {
                    let w = mesh.dverts[i].iter().find(|(gr, _)| *gr as usize == g).map_or(0.0, |(_, w)| *w);
                    match self.invert {
                        true => 1.0 - w,
                        false => w,
                    }
                }
            };
            if weight == 0.0 {
                continue;
            }
            let start = to_space(Vec3::from(*v));
            let (mut co, mut dcut) = (start, Vec3::ZERO);
            for (k, bit) in LOCK.iter().enumerate() {
                if lock & bit != 0 {
                    axis_limit(k, [0.0, 0.0], &mut co, &mut dcut);
                }
            }
            axis_limit(limit_axis, limits, &mut co, &mut dcut);
            let remap = |p: Vec3| Vec3::new(p[map[0]], p[map[1]], p[map[2]]);
            let out = mode.deform(factor, axis, remap(dcut), remap(co));
            let mut back = Vec3::ZERO;
            for k in 0..3 {
                back[map[k]] = out[k];
            }
            // `interp_v3_v3v3`.
            let blended = start * (1.0 - weight) + back * weight;
            *v = transf.map_or(blended, |(_, inv)| inv.transform_point3(blended)).into();
        }
    }
}
