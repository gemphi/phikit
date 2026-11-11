//! Blender's quaternion, `(w, x, y, z)` (`math_rotation_c.cc`).
use super::vec::BMat3;
use glam::{Mat3, Vec3};

/// A Blender quaternion, `(w, x, y, z)`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BQuat(pub [f32; 4]);

impl BQuat {
    /// `quat_to_mat3` (double inside, as Blender).
    pub fn to_mat3(self) -> Mat3 {
        let s = std::f64::consts::SQRT_2;
        let [q0, q1, q2, q3] = self.0.map(|v| s * v as f64);
        let (qda, qdb, qdc) = (q0 * q1, q0 * q2, q0 * q3);
        let (qaa, qab, qac, qbb, qbc, qcc) = (q1 * q1, q1 * q2, q1 * q3, q2 * q2, q2 * q3, q3 * q3);
        Mat3::from_cols_array_2d(&[
            [(1.0 - qbb - qcc) as f32, (qdc + qab) as f32, (-qdb + qac) as f32],
            [(-qdc + qab) as f32, (1.0 - qaa - qcc) as f32, (qda + qbc) as f32],
            [(qdb + qac) as f32, (-qda + qbc) as f32, (1.0 - qaa - qbb) as f32],
        ])
    }

    /// `mat3_normalized_to_quat_fast`: Mike Day's method, W kept ≥ 0.
    pub fn of_normalized_fast(m: &Mat3) -> Self {
        let at = |c: usize, r: usize| m.col(c)[r];
        let mut q = [0.0f32; 4];
        if at(2, 2) < 0.0 {
            if at(0, 0) > at(1, 1) {
                let trace = 1.0 + at(0, 0) - at(1, 1) - at(2, 2);
                let mut s = 2.0 * trace.sqrt();
                if at(1, 2) < at(2, 1) {
                    s = -s;
                }
                q[1] = 0.25 * s;
                s = 1.0 / s;
                q[0] = (at(1, 2) - at(2, 1)) * s;
                q[2] = (at(0, 1) + at(1, 0)) * s;
                q[3] = (at(2, 0) + at(0, 2)) * s;
                if trace == 1.0 && q[0] == 0.0 && q[2] == 0.0 && q[3] == 0.0 {
                    q[1] = 1.0;
                }
            } else {
                let trace = 1.0 - at(0, 0) + at(1, 1) - at(2, 2);
                let mut s = 2.0 * trace.sqrt();
                if at(2, 0) < at(0, 2) {
                    s = -s;
                }
                q[2] = 0.25 * s;
                s = 1.0 / s;
                q[0] = (at(2, 0) - at(0, 2)) * s;
                q[1] = (at(0, 1) + at(1, 0)) * s;
                q[3] = (at(1, 2) + at(2, 1)) * s;
                if trace == 1.0 && q[0] == 0.0 && q[1] == 0.0 && q[3] == 0.0 {
                    q[2] = 1.0;
                }
            }
        } else if at(0, 0) < -at(1, 1) {
            let trace = 1.0 - at(0, 0) - at(1, 1) + at(2, 2);
            let mut s = 2.0 * trace.sqrt();
            if at(0, 1) < at(1, 0) {
                s = -s;
            }
            q[3] = 0.25 * s;
            s = 1.0 / s;
            q[0] = (at(0, 1) - at(1, 0)) * s;
            q[1] = (at(2, 0) + at(0, 2)) * s;
            q[2] = (at(1, 2) + at(2, 1)) * s;
            if trace == 1.0 && q[0] == 0.0 && q[1] == 0.0 && q[2] == 0.0 {
                q[3] = 1.0;
            }
        } else {
            let trace = 1.0 + at(0, 0) + at(1, 1) + at(2, 2);
            let mut s = 2.0 * trace.sqrt();
            q[0] = 0.25 * s;
            s = 1.0 / s;
            q[1] = (at(1, 2) - at(2, 1)) * s;
            q[2] = (at(2, 0) - at(0, 2)) * s;
            q[3] = (at(0, 1) - at(1, 0)) * s;
            if trace == 1.0 && q[1] == 0.0 && q[2] == 0.0 && q[3] == 0.0 {
                q[0] = 1.0;
            }
        }
        let len_sq: f32 = q.iter().map(|v| v * v).sum();
        if (len_sq - 1.0).abs() >= 0.0002 * 3.0 {
            let len = len_sq.sqrt();
            if len != 0.0 {
                q = q.map(|v| v / len);
            }
        }
        Self(q)
    }

    /// `mat3_to_quat`: the columns normalised, a mirrored or broken matrix
    /// made sound first (`mat3_normalized_to_quat_with_checks`).
    pub fn of_mat3(m: &Mat3) -> Self {
        let mut n = m.normalized_cols();
        let det = n.determinant();
        if !det.is_finite() {
            n = Mat3::IDENTITY;
        } else if det < 0.0 {
            n = -n;
        }
        Self::of_normalized_fast(&n)
    }

    /// `mul_qt_qtqt`: `a · b`.
    pub fn mul(a: [f32; 4], b: [f32; 4]) -> [f32; 4] {
        [
            a[0] * b[0] - a[1] * b[1] - a[2] * b[2] - a[3] * b[3],
            a[0] * b[1] + a[1] * b[0] + a[2] * b[3] - a[3] * b[2],
            a[0] * b[2] + a[2] * b[0] + a[3] * b[1] - a[1] * b[3],
            a[0] * b[3] + a[3] * b[0] + a[1] * b[2] - a[2] * b[1],
        ]
    }

    /// `quat_to_axis_angle`: the axis (Y for no turn) and the angle.
    pub fn axis_angle(&self) -> (Vec3, f32) {
        let [w, x, y, z] = self.0;
        let ha = w.clamp(-1.0, 1.0).acos();
        let mut si = ha.sin();
        if si.abs() < 0.0005 {
            si = 1.0;
        }
        let axis = Vec3::new(x, y, z) / si;
        (if axis == Vec3::ZERO { Vec3::Y } else { axis }, ha * 2.0)
    }

    /// `interp_qt_qtqt`: slerp the short way round, a lerp when aligned.
    pub fn interp(a: Self, b: Self, t: f32) -> Self {
        let mut cosom: f32 = a.0.iter().zip(b.0).map(|(x, y)| x * y).sum();
        let mut q = a.0;
        if cosom < 0.0 {
            cosom = -cosom;
            q = q.map(|v| -v);
        }
        let (w0, w1) = if cosom.abs() < 1.0 - 1e-4 {
            let omega = cosom.acos();
            let sinom = omega.sin();
            (((1.0 - t) * omega).sin() / sinom, (t * omega).sin() / sinom)
        } else {
            (1.0 - t, t)
        };
        Self(std::array::from_fn(|i| w0 * q[i] + w1 * b.0[i]))
    }
}
