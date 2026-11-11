//! Quaternions as a curve's path turns them (`math_rotation_c.cc`):
//! building one from an axis and angle or from a direction to track, turning
//! a vector by one, and re-aiming one for an object's track and up axes.

use super::math::{BQuat, BVec3};
use glam::Vec3;

/// `M_SQRT1_2`.
const SQRT_1_2: f32 = std::f32::consts::FRAC_1_SQRT_2;

impl BQuat {
    pub const IDENTITY: Self = Self([1.0, 0.0, 0.0, 0.0]);

    /// `normalize_qt`: unit length, the identity's axis for a zero one.
    pub fn normalized(self) -> Self {
        let len = self.0.iter().map(|v| v * v).sum::<f32>().sqrt();
        if len != 0.0 { Self(self.0.map(|v| v * (1.0 / len))) } else { Self([0.0, 1.0, 0.0, 0.0]) }
    }

    /// `mul_qt_qtqt`: `self · b`.
    pub fn then(self, b: Self) -> Self {
        Self(Self::mul(self.0, b.0))
    }

    /// `axis_angle_to_quat`: a turn of `angle` about `axis` (normalised
    /// here; a zero axis is no turn).
    pub fn from_axis_angle(axis: Vec3, angle: f32) -> Self {
        let mut n = axis;
        if n.normalize_len() == 0.0 {
            return Self::IDENTITY;
        }
        let (si, co) = (0.5 * angle).sin_cos();
        Self([co, n.x * si, n.y * si, n.z * si])
    }

    /// `mul_qt_v3`: `v` turned by this quaternion.
    pub fn rotate(self, v: Vec3) -> Vec3 {
        let q = self.0;
        let t0 = -q[1] * v.x - q[2] * v.y - q[3] * v.z;
        let t1 = q[0] * v.x + q[2] * v.z - q[3] * v.y;
        let t2 = q[0] * v.y + q[3] * v.x - q[1] * v.z;
        let r2 = q[0] * v.z + q[1] * v.y - q[2] * v.x;
        let (r0, r1) = (t1, t2);
        Vec3::new(
            t0 * -q[1] + r0 * q[0] - r1 * q[3] + r2 * q[2],
            t0 * -q[2] + r1 * q[0] - r2 * q[1] + r0 * q[3],
            t0 * -q[3] + r2 * q[0] - r0 * q[2] + r1 * q[1],
        )
    }

    /// `vec_to_quat`: the rotation that points `axis` (0–5, `TRACK_*`: the
    /// first three aim against `vec`) along `vec`, rolled so `upflag`
    /// (0–2) leans up.
    pub fn of_track(vec: Vec3, axis: usize, upflag: usize) -> Self {
        let len = vec.length();
        if len == 0.0 {
            return Self::IDENTITY;
        }
        let (tvec, axis) = if axis > 2 { (vec, axis - 3) } else { (-vec, axis) };
        let eps = 1e-4;
        let (mut nor, co) = match axis {
            0 => {
                let n = Vec3::new(0.0, -tvec.z, tvec.y);
                (if tvec.y.abs() + tvec.z.abs() < eps { Vec3::new(n.x, 1.0, n.z) } else { n }, tvec.x)
            }
            1 => {
                let n = Vec3::new(tvec.z, 0.0, -tvec.x);
                (if tvec.x.abs() + tvec.z.abs() < eps { Vec3::new(n.x, n.y, 1.0) } else { n }, tvec.y)
            }
            _ => {
                let n = Vec3::new(-tvec.y, tvec.x, 0.0);
                (if tvec.x.abs() + tvec.y.abs() < eps { Vec3::new(1.0, n.y, n.z) } else { n }, tvec.z)
            }
        };
        let co = co / len;
        nor.normalize_len();
        let (si, c) = (0.5 * co.clamp(-1.0, 1.0).acos()).sin_cos();
        let q = Self([c, nor.x * si, nor.y * si, nor.z * si]);
        if axis == upflag {
            return q;
        }
        // The roll that brings the up axis round (`mat[2]`, the matrix's Z
        // column).
        let fp = q.to_mat3().z_axis;
        let angle = match (axis, upflag) {
            (0, 1) => 0.5 * fp.z.atan2(fp.y),
            (0, _) => -0.5 * fp.y.atan2(fp.z),
            (1, 0) => -0.5 * fp.z.atan2(fp.x),
            (1, _) => 0.5 * fp.x.atan2(fp.z),
            (_, 0) => 0.5 * (-fp.y).atan2(-fp.x),
            _ => -0.5 * (-fp.x).atan2(-fp.y),
        };
        let (si, co) = angle.sin_cos();
        let si = si / len;
        Self([co, tvec.x * si, tvec.y * si, tvec.z * si]).then(q)
    }

    /// `quat_apply_track`: re-aimed so `axis` (0–5) runs where `vec_to_quat`
    /// put the track axis, with `upflag` (0–2) up.
    pub fn track(self, axis: usize, upflag: usize) -> Self {
        const TRACK: [[f32; 4]; 6] = [
            [SQRT_1_2, 0.0, -SQRT_1_2, 0.0],
            [0.5, 0.5, 0.5, 0.5],
            [SQRT_1_2, 0.0, 0.0, SQRT_1_2],
            [SQRT_1_2, 0.0, SQRT_1_2, 0.0],
            [0.5, -0.5, -0.5, 0.5],
            [0.0, SQRT_1_2, SQRT_1_2, 0.0],
        ];
        let axis = axis.min(5);
        let mut q = self.then(Self(TRACK[axis]));
        let a = if axis > 2 { axis - 3 } else { axis };
        if upflag.min(2) != (2 - a) >> 1 {
            let mut r = [SQRT_1_2, 0.0, 0.0, 0.0];
            r[a + 1] = if a == 1 { SQRT_1_2 } else { -SQRT_1_2 };
            q = q.then(Self(r));
        }
        q
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_quaternion_turns_a_vector_as_its_matrix_does() {
        let q = BQuat([0.8, 0.2, -0.4, 0.4]).normalized();
        let v = Vec3::new(0.3, -1.2, 0.7);
        assert!((q.rotate(v) - q.to_mat3() * v).length() < 1e-5);
    }

    #[test]
    fn tracking_points_the_axis_along_the_vector() {
        // Axis 5 (`-Z` in `TRACK_*` terms) points +Z along the vector here,
        // as the curve's bevel list uses it.
        let dir = Vec3::new(0.3, 0.8, -0.2).normalize();
        let q = BQuat::of_track(dir, 5, 1);
        assert!((q.to_mat3().z_axis - dir).length() < 1e-5);
    }
}
