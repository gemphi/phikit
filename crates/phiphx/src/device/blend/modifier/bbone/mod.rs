//! B-Bone segments — Blender `armature.cc` (`BKE_pchan_bbone_spline_compute`,
//! `BKE_pchan_bbone_segments_cache_compute`,
//! `find_bbone_segment_index_straight`) and `armature_deform.cc`
//! (`b_bone_deform`), ported.
//!
//! A bone with `segments > 1` does not deform rigidly: Blender fits a cubic
//! Bezier through the bone, cuts it into equal arcs, and builds one matrix per
//! cut. A vertex is placed by how far it lies along the bone and takes a blend
//! of the two matrices either side of it. Rigify uses this for nearly every
//! deform bone — spine, thigh, shin, face — so without it a character's
//! surface lands about a millimetre off wherever a bone is bent, which is more
//! than a garment's thickness.
//!
//! Not ported: the `BBONE_MAPPING_CURVED` segment lookup (Blender's BSP
//! boundary planes); a curved-mapping bone falls back to the straight one.

mod curve;
mod pose;
mod read;
mod roll;
mod shape;
mod spline;

pub(crate) use shape::{BoneShape, ChanShape, Pose, Shape};
use curve::*;
use roll::*;
use spline::*;

use crate::device::blend::file::kind::*;
use glam::{Mat3, Mat4, Vec3};
use std::collections::HashMap;

/// `MAX_BBONE_SUBDIV`.
const MAX_SUBDIV: usize = 32;

const BONE_CONNECTED: u32 = 1 << 4;
const BBONE_HANDLE_AUTO: u8 = 0;
const BBONE_HANDLE_RELATIVE: u8 = 2;
const BBONE_HANDLE_TANGENT: u8 = 3;
const BBONE_ADD_PARENT_END_ROLL: u32 = 1 << 0;
const BBONE_SCALE_EASING: u32 = 1 << 1;
/// `BBONE_HANDLE_SCALE_X / _Y / _Z / _EASE`.
const SCALE_X: u32 = 1 << 0;
const SCALE_Y: u32 = 1 << 1;
const SCALE_Z: u32 = 1 << 2;
const SCALE_EASE: u32 = 1 << 3;

/// One bone's segment matrices, ready to deform with.
#[derive(Clone, Debug)]
pub(crate) struct BBone {
    /// `bbone_deform_mats[0]` — armature space to bone-local, for finding
    /// which segment a point belongs to.
    to_bone: Mat4,
    /// `bbone_deform_mats[1..]` — `segments + 1` matrices, armature space.
    mats: Vec<Mat4>,
    length: f32,
}

impl BBone {
    /// `find_bbone_segment_index_straight` + `..._deform_clamp_segment_index`:
    /// the first of the two segments this point falls between, and how far it
    /// leans towards the second.
    pub(crate) fn segment(&self, co: Vec3) -> (usize, f32) {
        let segments = self.mats.len() - 1;
        let y = self.to_bone.transform_point3(co).y;
        let head_tail = (y / self.length).clamp(0.0, 1.0);
        let pre = head_tail * segments as f32;
        let index = (pre.floor() as isize).clamp(0, segments as isize - 1) as usize;
        (index, (pre - index as f32).clamp(0.0, 1.0))
    }

    /// The matrix that deforms `co`. `b_bone_deform` adds the two segments'
    /// contributions with weights `1 − blend` and `blend`, and both are affine
    /// in the matrix, so the blended matrix gives the same answer.
    pub(crate) fn matrix(&self, co: Vec3) -> Mat4 {
        let (i, blend) = self.segment(co);
        let (a, b) = (self.mats[i], self.mats[i + 1]);
        Mat4::from_cols_array(&std::array::from_fn(|k| {
            let (x, y) = (a.to_cols_array()[k], b.to_cols_array()[k]);
            x + (y - x) * blend
        }))
    }
}

impl BBone {
    /// The segment matrices, in the order the skinner expects them.
    pub(crate) fn slots(&self) -> impl Iterator<Item = Mat4> + '_ {
        self.mats.iter().copied()
    }
}
