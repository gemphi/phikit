//! The Transformation constraint (`transform_evaluate`): one of the target's
//! location, rotation or scale mapped, range to range, onto one of the
//! owner's, mixed in by that channel's mix mode.

use super::BCob;
use crate::blend::rig::math::{BCompose, BEulerOrder, BMat4, BQuat};
use glam::{Mat3, Mat4, Vec3};

/// `TRANS_ROTATION` / `_SCALE` (`TRANS_LOCATION` is the rest).
const TRANS_ROTATION: u8 = 1;
const TRANS_SCALE: u8 = 2;
/// `TRANS_MIXLOC_REPLACE`, `TRANS_MIXROT_*`, `TRANS_MIXSCALE_MULTIPLY`.
const MIXLOC_REPLACE: u8 = 1;
const MIXROT_REPLACE: u8 = 1;
const MIXROT_BEFORE: u8 = 2;
const MIXROT_AFTER: u8 = 3;
const MIXSCALE_MULTIPLY: u8 = 1;
/// `DTAR_ROTMODE_*`.
const ROTMODE_QUATERNION: u8 = 7;
const ROTMODE_SWING_TWIST_X: u8 = 8;

/// `bTransformConstraint`: `from` / `to` `TRANS_*`, `map` which source axis
/// feeds each destination axis, `expo` (extrapolate), and the ranges of
/// each kind (`[kind][min, max]`).
#[derive(Clone, Debug, PartialEq)]
pub struct BTransformData {
    pub from: u8,
    pub to: u8,
    pub map: [u8; 3],
    pub expo: bool,
    pub from_rotation_mode: u8,
    /// `to_euler_order` (`CONSTRAINT_EULER_AUTO` 0: the owner's).
    pub to_euler_order: u8,
    pub mix_loc: u8,
    pub mix_rot: u8,
    pub mix_scale: u8,
    /// `from_min/max`, `from_min/max_rot`, `from_min/max_scale`.
    pub from_range: [[Vec3; 2]; 3],
    pub to_range: [[Vec3; 2]; 3],
}

impl BTransformData {
    /// `transform_evaluate` with the target matrix `ct`.
    pub fn evaluate(&self, cob: &mut BCob, ct: Mat4) {
        let from = self.from.min(2) as usize;
        let mut dvec = match self.from {
            TRANS_SCALE => {
                // Which axis is negative cannot be told; all are, then.
                let s = ct.size();
                if Mat3::from_mat4(ct).determinant() < 0.0 { -s } else { s }
            }
            TRANS_ROTATION => self.rotation_channels(&ct, cob.rot_order),
            _ => ct.loc(),
        };
        let [from_min, from_max] = self.from_range[from];
        let sval = Vec3::from_array(std::array::from_fn(|i| {
            // `CLAMP`: the low bound first, as the macro does.
            if !self.expo {
                if dvec[i] < from_min[i] {
                    dvec[i] = from_min[i];
                } else if dvec[i] > from_max[i] {
                    dvec[i] = from_max[i];
                }
            }
            let range = from_max[i] - from_min[i];
            if range != 0.0 { (dvec[i] - from_min[i]) / range } else { 0.0 }
        }));
        let order = BEulerOrder(match self.to {
            TRANS_ROTATION if self.to_euler_order != 0 => self.to_euler_order as i16,
            _ => cob.rot_order,
        });
        let to = self.to.min(2) as usize;
        let [to_min, to_max] = self.to_range[to];
        let out = Vec3::from_array(std::array::from_fn(|i| to_min[i] + sval[self.map[i].min(2) as usize] * (to_max[i] - to_min[i])));
        let (mut loc, mut rot, mut size) = cob.matrix.loc_rot_size();
        match self.to {
            TRANS_SCALE => size = if self.mix_scale == MIXSCALE_MULTIPLY { size * out } else { out },
            TRANS_ROTATION => {
                rot = match self.mix_rot {
                    MIXROT_REPLACE => order.to_mat3(out),
                    MIXROT_BEFORE => order.to_mat3(out) * rot,
                    MIXROT_AFTER => rot * order.to_mat3(out),
                    _ => order.to_mat3(out + order.of_normalized(&rot)),
                }
            }
            _ => loc = if self.mix_loc == MIXLOC_REPLACE { out } else { loc + out },
        }
        cob.matrix = BCompose::loc_rot_size(loc, rot, size);
    }

    /// `BKE_driver_target_matrix_to_rot_channels` with pseudo-angles, as the
    /// constraint reads it: Euler angles in the owner's order (`AUTO`) or
    /// the one named, a quaternion's parts as angles, or swing and twist.
    fn rotation_channels(&self, m: &Mat4, auto_order: i16) -> Vec3 {
        let mode = self.from_rotation_mode;
        match mode {
            0 => BEulerOrder(auto_order).of_mat4(m),
            1..=6 => BEulerOrder(mode as i16).of_mat4(m),
            ROTMODE_QUATERNION => {
                let q = BQuat::of_mat3(&Mat3::from_mat4(*m)).0;
                Vec3::new(Self::asin2(q[1]), Self::asin2(q[2]), Self::asin2(q[3]))
            }
            _ => {
                let axis = (mode.saturating_sub(ROTMODE_SWING_TWIST_X) as usize).min(2);
                let raw = BQuat::of_mat3(&Mat3::from_mat4(*m)).0;
                let q = if raw[0] < 0.0 { raw.map(|v| -v) } else { raw };
                // `quat_split_swing_and_twist`: the half-twist, and the swing
                // as the quaternion with the twist taken off.
                let t = q[axis + 1].atan2(q[0]);
                let (sin_t, cos_t) = t.sin_cos();
                let mut inv = [cos_t, 0.0, 0.0, 0.0];
                inv[axis + 1] = -sin_t;
                let swing = BQuat::mul(q, inv);
                let mut out = Vec3::new(Self::asin2(swing[1]), Self::asin2(swing[2]), Self::asin2(swing[3]));
                out[axis] = 2.0 * t;
                out
            }
        }
    }

    /// `2 · safe_asinf(v)`.
    fn asin2(v: f32) -> f32 {
        2.0 * v.clamp(-1.0, 1.0).asin()
    }
}
