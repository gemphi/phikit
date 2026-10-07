//! What a driver variable reads, and its value — `dvar_eval_singleProp`,
//! `dvar_eval_transChan`, `dvar_eval_locDiff` and `dvar_eval_rotDiff`
//! (`fcurve_driver.cc`), over the scene as the actions and the NLA left it.
//!
//! Every kind Blender has but Context Property is read here. A Context
//! Property variable names a setting of the active scene or view layer,
//! which a file alone does not fix, so it stays [`BlendDriverRead::Unsupported`]
//! and reads 0 the way a driver nobody can resolve does.

use super::chan::{BTransChan, ROT_MODE_EUL};
use super::space::BDriverSpace;
use crate::blend::idprop::BIdGroup;
use crate::blend::object::transform::{IDENTITY4, ObjectTransformRaw, object_to_mat4};
use crate::blend::rig::math::{BMat3, BQuat};
use crate::blend::rig::solve::BChannelValue;
use glam::{Mat3, Mat4, Vec3};
use std::collections::HashMap;

/// One end of a Loc Diff or Rot Diff variable: the object, and the space it
/// is read in.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct BlendDriverEnd {
    pub object: u64,
    pub space: BDriverSpace,
}

/// Where an object driver's variable reads from.
#[derive(Clone, Debug, PartialEq)]
pub enum BlendDriverRead {
    /// `DVAR_TYPE_SINGLE_PROP` on a transform channel of an object in the
    /// scene, named by its block address (`DriverTarget.id`), at `index`.
    Channel { object: u64, property: String, index: usize },
    /// `DVAR_TYPE_SINGLE_PROP` on an object's custom property (`["name"]`).
    Prop { object: u64, name: String },
    /// `DVAR_TYPE_TRANSFORM_CHAN`: one channel of a target's final matrix.
    Transform { object: u64, chan: u32, space: BDriverSpace, rot_mode: i16 },
    /// `DVAR_TYPE_LOC_DIFF`: the distance between two targets.
    LocDiff { a: BlendDriverEnd, b: BlendDriverEnd },
    /// `DVAR_TYPE_ROT_DIFF`: the angle between two targets' rotations,
    /// folded into `0..π`.
    RotDiff { a: u64, b: u64 },
    /// A kind or path this evaluator does not read — a Context Property, a
    /// material or modifier setting, a path under a bone. It reads 0, as a
    /// file whose scripts do not run does: a driver nobody can evaluate is
    /// better still than a driver evaluated from a guess.
    Unsupported,
}

/// What a driver pass needs of the scene: every object's channels as the
/// actions left them, their world matrices over that snapshot, their custom
/// properties, and where to find one by block address.
pub struct BlendDriverScene<'a> {
    pub raw: &'a [ObjectTransformRaw],
    pub world: &'a [[[f32; 4]; 4]],
    pub prop: &'a HashMap<u64, BIdGroup>,
    pub index: &'a HashMap<u64, usize>,
}

impl BlendDriverScene<'_> {
    /// The matrix target `object` is read through in `space`.
    fn matrix(&self, object: u64, space: BDriverSpace) -> Option<Mat4> {
        let i = *self.index.get(&object)?;
        let raw = self.raw.get(i)?;
        let world = || Mat4::from_cols_array_2d(self.world.get(i).unwrap_or(&IDENTITY4));
        Some(match space {
            BDriverSpace::World => world(),
            BDriverSpace::Transform => Mat4::from_cols_array_2d(&object_to_mat4(raw)),
            // `BKE_constraint_mat_convertspace(WORLD → LOCAL)`.
            BDriverSpace::Local => Self::unparented(raw, i, self.raw, self.world, self.index).inverse() * world(),
        })
    }

    /// `diff_mat` in the WORLD → LOCAL conversion: `parent.world ·
    /// parentinv` for a parented object, and for a parent-less one its own
    /// rotation alone — `BKE_object_to_mat4` normalised with the
    /// translation zeroed, which is the "ugly hack" the comment there owns
    /// up to and which the Driver panel's Local Space nonetheless means.
    fn unparented(
        raw: &ObjectTransformRaw,
        _i: usize,
        all: &[ObjectTransformRaw],
        world: &[[[f32; 4]; 4]],
        index: &HashMap<u64, usize>,
    ) -> Mat4 {
        if let Some(p) = index.get(&raw.parent_address).filter(|p| all.get(**p).is_some()) {
            let pw = Mat4::from_cols_array_2d(world.get(*p).unwrap_or(&IDENTITY4));
            return pw * Mat4::from_cols_array_2d(&raw.parentinv);
        }
        Mat4::from_mat3(Mat3::from_mat4(Mat4::from_cols_array_2d(&object_to_mat4(raw))).normalized_cols())
    }

    /// `copy_v3_v3(tmp_loc, …)`: where one end of a Loc Diff sits. World
    /// space takes the matrix's translation, Transform space the object's
    /// own `loc` — not its matrix, as Blender does — and Local space the
    /// translation of the converted matrix.
    fn point(&self, end: BlendDriverEnd) -> Option<Vec3> {
        let i = *self.index.get(&end.object)?;
        match end.space {
            BDriverSpace::Transform => Some(Vec3::from(self.raw.get(i)?.loc)),
            space => self.matrix(end.object, space).map(|m| m.w_axis.truncate()),
        }
    }

    /// `use_eulers` in `dvar_eval_transChan`: the rotation order an Auto
    /// decomposition takes, and the saved Euler it is pulled towards — both
    /// only when the target is on an Euler rotation mode.
    fn eulers(&self, object: u64) -> (i16, Option<Vec3>) {
        let raw = self.index.get(&object).and_then(|i| self.raw.get(*i)).filter(|r| r.rotmode >= ROT_MODE_EUL);
        (raw.map_or(ROT_MODE_EUL, |r| r.rotmode), raw.map(|r| Vec3::from(r.rot)))
    }
}

/// One variable's value at a frame.
pub trait BDriverValue {
    fn value(&self, scene: &BlendDriverScene) -> f64;
}

impl BDriverValue for BlendDriverRead {
    fn value(&self, scene: &BlendDriverScene) -> f64 {
        match self {
            Self::Channel { object, property, index } => scene
                .index
                .get(object)
                .and_then(|i| scene.raw.get(*i))
                .and_then(|raw| raw.value(property, *index))
                .unwrap_or(0.0) as f64,
            Self::Prop { object, name } => {
                scene.prop.get(object).and_then(|g| g.get(name)).and_then(crate::blend::idprop::BIdValue::as_float).unwrap_or(0.0)
            }
            Self::Transform { object, chan, space, rot_mode } => {
                // `use_eulers`: a target on an Euler rotation mode lends its
                // order to an Auto decomposition, and its saved Euler to
                // `compatible_eul`.
                let (order, old) = scene.eulers(*object);
                scene.matrix(*object, *space).map(|m| m.trans_chan(*chan, *rot_mode, order, old)).unwrap_or(0.0) as f64
            }
            Self::LocDiff { a, b } => match (scene.point(*a), scene.point(*b)) {
                // `len_v3v3(loc1, loc2)`; a missing target makes the whole
                // variable 0, as `valid_targets < num_targets` does.
                (Some(p), Some(q)) => p.distance(q) as f64,
                _ => 0.0,
            },
            Self::RotDiff { a, b } => match (scene.matrix(*a, BDriverSpace::World), scene.matrix(*b, BDriverSpace::World)) {
                (Some(m), Some(n)) => Self::rot_diff(&m, &n) as f64,
                _ => 0.0,
            },
            Self::Unsupported => 0.0,
        }
    }
}

impl BlendDriverRead {
    /// `dvar_eval_rotDiff`'s tail: `2·acos` of the relative quaternion's
    /// `w`, folded back under π.
    fn rot_diff(a: &Mat4, b: &Mat4) -> f32 {
        let quat = |m: &Mat4| BQuat::of_mat3(&Mat3::from_mat4(*m).normalized_cols()).0;
        let q1 = quat(a);
        // `invert_qt_normalized`: the conjugate of a unit quaternion.
        let inv = [q1[0], -q1[1], -q1[2], -q1[3]];
        let angle = (2.0 * BQuat::mul(inv, quat(b))[0].clamp(-1.0, 1.0).acos()).abs();
        let tau = std::f32::consts::TAU;
        if angle > std::f32::consts::PI { tau - angle } else { angle }
    }
}
