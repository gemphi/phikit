//! `dvar_eval_transChan`'s tail and `BKE_driver_target_matrix_to_rot_channels`
//! under it (`fcurve_driver.cc`): a Transform Channel variable reads one
//! number out of a 4×4 — a location axis, a scale axis, the cube root of the
//! volume, or a rotation channel in whichever mode the target names (Auto,
//! an explicit Euler order, Quaternion, or the Swing/Twist split around an
//! axis that the "Damped Track + child" rigs lean on).

use crate::blend::rig::math::{BEulerOrder, BMat3, BQuat};
use glam::{Mat3, Mat4, Vec3};

/// `eDriverTarget_TransformChannels`.
pub const TRANSCHAN_ROTX: u32 = 3;
pub const TRANSCHAN_SCALEX: u32 = 6;
pub const TRANSCHAN_SCALE_AVG: u32 = 9;
pub const TRANSCHAN_ROTW: u32 = 10;
pub const MAX_TRANSCHAN: u32 = 11;

/// `eDriverTarget_RotationMode`.
pub const ROTMODE_AUTO: i16 = 0;
pub const ROTMODE_EULER_MIN: i16 = 1;
pub const ROTMODE_EULER_MAX: i16 = 6;
pub const ROTMODE_QUATERNION: i16 = 7;
pub const ROTMODE_SWING_TWIST_X: i16 = 8;
pub const ROTMODE_SWING_TWIST_Z: i16 = 10;

/// `eRotationModes`: `ROT_MODE_EUL` is the first Euler order; anything above
/// zero is one, so a target with an Euler rotation mode gets the
/// compatible-Euler pass Blender gives it.
pub const ROT_MODE_EUL: i16 = 1;

/// A 4×4 read the way a driver's Transform Channel target reads it.
pub trait BTransChan {
    /// The channel's value: `dvar_eval_transChan`'s tail, from the "check
    /// which transform" branch down. `old_eul` is the target's own saved
    /// Euler, which `compatible_eul` pulls an Auto-mode decomposition
    /// towards (`use_eulers`), and is `None` when the target is not on an
    /// Euler rotation mode.
    fn trans_chan(&self, chan: u32, rot_mode: i16, auto_order: i16, old_eul: Option<Vec3>) -> f32;

    /// `BKE_driver_target_matrix_to_rot_channels` with `angles = false`:
    /// `[w, x, y, z]` for Quaternion, `[0, ex, ey, ez]` for an Euler mode,
    /// and the swing as pseudo-angles with the twist in its own slot for a
    /// Swing/Twist mode.
    fn rot_channels(&self, auto_order: i16, rot_mode: i16, channel: usize) -> [f32; 4];
}

impl BTransChan for Mat4 {
    fn trans_chan(&self, chan: u32, rot_mode: i16, auto_order: i16, old_eul: Option<Vec3>) -> f32 {
        if chan >= MAX_TRANSCHAN {
            return 0.0;
        }
        if chan == TRANSCHAN_SCALE_AVG {
            // `cbrtf(mat4_to_volume_scale(mat))` — the geometric mean of the
            // three scales when the matrix carries no shear.
            return Mat3::from_mat4(*self).determinant().cbrt();
        }
        if (TRANSCHAN_SCALEX..TRANSCHAN_SCALE_AVG).contains(&chan) {
            // `len_v3(mat[chan - SCALEX])`, inline `mat4_to_size`.
            return Self::axis(self, (chan - TRANSCHAN_SCALEX) as usize).length();
        }
        if chan >= TRANSCHAN_ROTX {
            let channel = if chan == TRANSCHAN_ROTW { 0 } else { 1 + (chan - TRANSCHAN_ROTX) as usize };
            let mut buf = self.rot_channels(auto_order, rot_mode, channel);
            if let Some(old) = old_eul.filter(|_| rot_mode == ROTMODE_AUTO) {
                let eul = BEulerOrder::compatible(Vec3::new(buf[1], buf[2], buf[3]), old);
                (buf[1], buf[2], buf[3]) = (eul.x, eul.y, eul.z);
            }
            return buf[channel];
        }
        // `mat[3][transChan]`: the location axis.
        self.w_axis[chan as usize]
    }

    fn rot_channels(&self, auto_order: i16, rot_mode: i16, channel: usize) -> [f32; 4] {
        let mut buf = [0.0f32; 4];
        let euler = |buf: &mut [f32; 4], order: i16| {
            let e = BEulerOrder(order).of_mat4(self);
            (buf[1], buf[2], buf[3]) = (e.x, e.y, e.z);
        };
        match rot_mode {
            ROTMODE_AUTO => euler(&mut buf, auto_order),
            ROTMODE_EULER_MIN..=ROTMODE_EULER_MAX => euler(&mut buf, rot_mode),
            ROTMODE_QUATERNION => buf = Self::quat(self),
            ROTMODE_SWING_TWIST_X..=ROTMODE_SWING_TWIST_Z => {
                let axis = (rot_mode - ROTMODE_SWING_TWIST_X) as usize;
                let (swing, twist) = Self::swing_twist(&Self::quat(self), axis);
                if channel != axis + 1 {
                    buf = swing;
                    Self::to_angles(&mut buf, channel);
                }
                buf[axis + 1] = twist;
            }
            _ => {}
        }
        buf
    }
}

/// The pieces `BTransChan` is built from, kept off the trait so a caller
/// cannot reach for half a decomposition.
trait BRotPart {
    fn axis(m: &Mat4, i: usize) -> Vec3;
    fn quat(m: &Mat4) -> [f32; 4];
    fn swing_twist(q: &[f32; 4], axis: usize) -> ([f32; 4], f32);
    fn to_angles(buf: &mut [f32; 4], channel: usize);
}

impl BRotPart for Mat4 {
    fn axis(m: &Mat4, i: usize) -> Vec3 {
        let c = Mat3::from_mat4(*m);
        [c.x_axis, c.y_axis, c.z_axis][i.min(2)]
    }

    /// `mat4_to_quat`: the columns normalised first.
    fn quat(m: &Mat4) -> [f32; 4] {
        BQuat::of_mat3(&Mat3::from_mat4(*m).normalized_cols()).0
    }

    /// `quat_split_swing_and_twist(q, axis, swing, nullptr)`: the half-twist
    /// read straight off the canonical quaternion, the swing as `q` with the
    /// twist taken off, and `2·t` as the twist angle.
    fn swing_twist(q: &[f32; 4], axis: usize) -> ([f32; 4], f32) {
        let q = if q[0] < 0.0 { q.map(|v| -v) } else { *q };
        let t = q[axis + 1].atan2(q[0]);
        let (sin_t, cos_t) = t.sin_cos();
        let mut inv = [cos_t, 0.0, 0.0, 0.0];
        inv[axis + 1] = -sin_t;
        (BQuat::mul(q, inv), 2.0 * t)
    }

    /// `quaternion_to_angles(quat, channel)` for a channel that is asked for.
    fn to_angles(buf: &mut [f32; 4], channel: usize) {
        match channel {
            0 => buf[0] = 2.0 * buf[0].clamp(-1.0, 1.0).acos(),
            c if c < 4 => buf[c] = 2.0 * buf[c].clamp(-1.0, 1.0).asin(),
            _ => {}
        }
    }
}

#[cfg(test)]
#[path = "chan_tests.rs"]
mod chan_tests;
