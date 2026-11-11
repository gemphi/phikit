//! The legacy IK solver's double-precision math (`IK_Math.h`): rotations, swing and twist, ellipse clamping.

use glam::{DMat3, DVec3};

/// `IK_EPSILON`.
pub const IK_EPSILON: f64 = 1e-20;

/// Eigen-style matrix from rows.
pub fn rows(r: [[f64; 3]; 3]) -> DMat3 {
    DMat3::from_cols_array_2d(&r).transpose()
}

/// `M(r, c)`.
pub fn at(m: &DMat3, r: usize, c: usize) -> f64 {
    m.col(c)[r]
}

/// The math of `IK_Math.h`.
pub struct BIkMath;

impl BIkMath {
    pub fn fuzzy_zero(x: f64) -> bool {
        x.abs() < IK_EPSILON
    }

    pub fn rotation(sine: f64, cosine: f64, axis: usize) -> DMat3 {
        match axis {
            0 => rows([[1.0, 0.0, 0.0], [0.0, cosine, -sine], [0.0, sine, cosine]]),
            1 => rows([[cosine, 0.0, sine], [0.0, 1.0, 0.0], [-sine, 0.0, cosine]]),
            _ => rows([[cosine, -sine, 0.0], [sine, cosine, 0.0], [0.0, 0.0, 1.0]]),
        }
    }

    pub fn rotation_angle(angle: f64, axis: usize) -> DMat3 {
        Self::rotation(angle.sin(), angle.cos(), axis)
    }

    /// `EulerAngleFromMatrix`.
    pub fn euler_angle(r: &DMat3, axis: usize) -> f64 {
        let t = (at(r, 0, 0) * at(r, 0, 0) + at(r, 0, 1) * at(r, 0, 1)).sqrt();
        if t > 16.0 * IK_EPSILON {
            match axis {
                0 => -at(r, 1, 2).atan2(at(r, 2, 2)),
                1 => (-at(r, 0, 2)).atan2(t),
                _ => -at(r, 0, 1).atan2(at(r, 0, 0)),
            }
        } else {
            match axis {
                0 => -(-at(r, 2, 1)).atan2(at(r, 1, 1)),
                1 => (-at(r, 0, 2)).atan2(t),
                _ => 0.0,
            }
        }
    }

    pub fn safe_acos(f: f64) -> f64 {
        if f <= -1.0 {
            std::f64::consts::PI
        } else if f >= 1.0 {
            0.0
        } else {
            f.acos()
        }
    }

    pub fn normalize(v: DVec3) -> DVec3 {
        let len = v.length();
        if Self::fuzzy_zero(len) { DVec3::ZERO } else { v / len }
    }

    /// `ComputeTwist`: the twist about Y.
    pub fn twist(r: &DMat3) -> f64 {
        let qy = at(r, 0, 2) - at(r, 2, 0);
        let qw = at(r, 0, 0) + at(r, 1, 1) + at(r, 2, 2) + 1.0;
        2.0 * qy.atan2(qw)
    }

    pub fn remove_twist(r: &DMat3) -> DMat3 {
        *r * Self::rotation_angle(Self::twist(r), 1).transpose()
    }

    /// `SphericalRangeParameters`: swing X, twist, swing Z.
    pub fn spherical_range(r: &DMat3) -> DVec3 {
        let tau = Self::twist(r);
        let num = 2.0 * (1.0 + at(r, 1, 1));
        if num.abs() < IK_EPSILON {
            return DVec3::new(0.0, tau, 1.0);
        }
        let num = 1.0 / num.sqrt();
        DVec3::new(-at(r, 2, 1) * num, tau, at(r, 0, 1) * num)
    }

    /// `ComputeSwingMatrix`: `Quaterniond(-cos, ax, 0, az)` as a matrix.
    pub fn swing(ax: f64, az: f64) -> DMat3 {
        let sine2 = ax * ax + az * az;
        let cosine2 = if sine2 >= 1.0 { 0.0 } else { (1.0 - sine2).sqrt() };
        let (w, x, y, z) = (-cosine2, ax, 0.0, az);
        rows([
            [1.0 - 2.0 * (y * y + z * z), 2.0 * (x * y - z * w), 2.0 * (x * z + y * w)],
            [2.0 * (x * y + z * w), 1.0 - 2.0 * (x * x + z * z), 2.0 * (y * z - x * w)],
            [2.0 * (x * z - y * w), 2.0 * (y * z + x * w), 1.0 - 2.0 * (x * x + y * y)],
        ])
    }

    /// `MatrixToAxisAngle`.
    pub fn axis_angle(r: &DMat3) -> DVec3 {
        let delta = DVec3::new(at(r, 2, 1) - at(r, 1, 2), at(r, 0, 2) - at(r, 2, 0), at(r, 1, 0) - at(r, 0, 1));
        let c = Self::safe_acos((at(r, 0, 0) + at(r, 1, 1) + at(r, 2, 2) - 1.0) / 2.0);
        let l = delta.length();
        if Self::fuzzy_zero(l) { delta } else { delta * (c / l) }
    }

    /// `EllipseClamp`.
    pub fn ellipse_clamp(ax: &mut f64, az: &mut f64, amin: &[f64; 2], amax: &[f64; 2]) -> bool {
        let (mut x, xlim) = if *ax < 0.0 { (-*ax, -amin[0]) } else { (*ax, amax[0]) };
        let (mut z, zlim) = if *az < 0.0 { (-*az, -amin[1]) } else { (*az, amax[1]) };
        if Self::fuzzy_zero(xlim) || Self::fuzzy_zero(zlim) {
            if x <= xlim && z <= zlim {
                return false;
            }
            x = x.min(xlim);
            z = z.min(zlim);
        } else {
            let (invx, invz) = (1.0 / (xlim * xlim), 1.0 / (zlim * zlim));
            if x * x * invx + z * z * invz <= 1.0 {
                return false;
            }
            if Self::fuzzy_zero(x) {
                x = 0.0;
                z = zlim;
            } else {
                let rico = z / x;
                let old_x = x;
                x = (1.0 / (invx + invz * rico * rico)).sqrt();
                if old_x < 0.0 {
                    x = -x;
                }
                z = rico * x;
            }
        }
        *ax = if *ax < 0.0 { -x } else { x };
        *az = if *az < 0.0 { -z } else { z };
        true
    }

    /// The rotation `dq` (a rotation vector) as a matrix, as the spherical
    /// and swing segments build it.
    pub fn from_rotation_vector(dq: DVec3) -> Option<DMat3> {
        let theta = dq.length();
        if Self::fuzzy_zero(theta) {
            return None;
        }
        let w = dq * (1.0 / theta);
        let (sine, cosine) = theta.sin_cos();
        let ci = 1.0 - cosine;
        let (xs, ys, zs) = (w.x * sine, w.y * sine, w.z * sine);
        let (xx, xy, xz, yy, yz, zz) = (w.x * w.x * ci, w.x * w.y * ci, w.x * w.z * ci, w.y * w.y * ci, w.y * w.z * ci, w.z * w.z * ci);
        Some(rows([[cosine + xx, -zs + xy, ys + xz], [zs + xy, cosine + yy, -xs + yz], [-ys + xz, xs + yz, cosine + zz]]))
    }
}
