//! Applying F-curves to a transform: the channel values Blender keys —
//! `location`, `rotation_euler`, `rotation_quaternion`, `rotation_axis_angle`,
//! `scale` — written over an [`ObjectTransformRaw`], which is what both an
//! object (`BKE_object_to_mat4`) and a pose channel (`BKE_pchan_to_mat4`)
//! turn into a matrix. A pose channel has no delta transforms, so its raw
//! keeps the defaults there.

use super::fcurve::BFCurve;
use crate::blend::file::types::*;
use crate::blend::object::rotation::object_to_mat4;
use crate::blend::object::transform::ObjectTransformRaw;

/// The transform value a curve of `property` / `index` drives, if any.
pub(crate) fn channel_mut<'r>(raw: &'r mut ObjectTransformRaw, property: &str, i: usize) -> Option<&'r mut f32> {
    match (property, i) {
        ("location", 0..=2) => Some(&mut raw.loc[i]),
        ("rotation_euler", 0..=2) => Some(&mut raw.rot[i]),
        ("rotation_quaternion", 0..=3) => Some(&mut raw.quat[i]),
        ("rotation_axis_angle", 0) => Some(&mut raw.rot_angle),
        ("rotation_axis_angle", 1..=3) => Some(&mut raw.rot_axis[i - 1]),
        ("scale", 0..=2) => Some(&mut raw.size[i]),
        _ => None,
    }
}

/// Write every curve for this target at `frame` into `raw`.
pub(crate) fn apply<'a>(raw: &mut ObjectTransformRaw, curves: impl Iterator<Item = &'a BFCurve>, frame: f32) {
    for c in curves {
        // A muted curve is switched off (`FCURVE_MUTED`): the channel keeps its rest value.
        if c.muted() {
            continue;
        }
        let Some(v) = c.eval(frame) else { continue };
        if let Some(slot) = channel_mut(raw, c.property(), c.index as usize) {
            *slot = v;
        }
    }
}

impl BlendFile {
    /// A pose channel's saved loc / rot / scale, as a transform. Blender's
    /// `bPoseChannel` carries the same channels an object does, minus the
    /// deltas, so the object matrix builder serves both.
    pub(crate) fn pchan_raw(&self, pc: &BlendBlock, dna: &DnaType) -> ObjectTransformRaw {
        let d = &pc.data;
        let off = |f: &str| self.off("bPoseChannel", f, dna);
        let f3 = |f: &str, dflt: [f32; 3]| self.f32s(d, off(f), dflt);
        let mut raw = ObjectTransformRaw::default();
        raw.loc = f3("loc[3]", [0.0; 3]);
        raw.rot = f3("eul[3]", [0.0; 3]);
        raw.size = f3("size[3]", [1.0; 3]);
        raw.quat = self.f32s(d, off("quat[4]"), [1.0, 0.0, 0.0, 0.0]);
        raw.rot_axis = f3("rotAxis[3]", [0.0, 1.0, 0.0]);
        raw.rot_angle = off("rotAngle").map_or(0.0, |o| self.f32_at(d, o));
        raw.rotmode = self.i16_at(d, off("rotmode"), 0);
        raw
    }
}

/// `BKE_pchan_to_mat4` / `BKE_object_to_mat4` at this frame.
pub(crate) fn basis_at<'a>(mut raw: ObjectTransformRaw, curves: impl Iterator<Item = &'a BFCurve>, frame: f32) -> glam::Mat4 {
    apply(&mut raw, curves, frame);
    glam::Mat4::from_cols_array_2d(&object_to_mat4(&raw))
}
