//! The legacy IK solver's segments (`intern/iksolver/intern/IK_QSegment.cc`,
//! `IK_Math.h`), in double precision as there: a bone's rotation joint —
//! spherical (3 DoF), swing (X and Z), elbow (one axis and twist),
//! revolute (one axis) or none — and, when it may stretch, a translation
//! joint along Y chained after it (a "composite").

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

/// What a segment's joint is.
#[derive(Clone, Debug)]
pub enum BJoint {
    Null,
    Spherical { new_basis: DMat3, limit: [bool; 3], min: [f64; 2], max: [f64; 2], min_y: f64, max_y: f64, locked_a: DVec3 },
    Revolute { axis: usize, angle: f64, new_angle: f64, limit: bool, min: f64, max: f64 },
    Swing { new_basis: DMat3, limit_x: bool, limit_z: bool, min: [f64; 2], max: [f64; 2] },
    Elbow { axis: usize, twist: f64, angle: f64, new_twist: f64, new_angle: f64, cos_twist: f64, sin_twist: f64, limit: bool, limit_twist: bool, min: f64, max: f64, min_twist: f64, max_twist: f64 },
    Translate { axis: [usize; 3], enabled: [bool; 3], limit: [bool; 3], min: [f64; 3], max: [f64; 3], new_translation: DVec3 },
}

/// One `IK_QSegment`.
#[derive(Clone, Debug)]
pub struct BSegment {
    pub joint: BJoint,
    pub parent: Option<usize>,
    /// Children, most recent first (`m_child` → `m_sibling`).
    pub children: Vec<usize>,
    pub composite: Option<usize>,
    pub start: DVec3,
    pub rest_basis: DMat3,
    pub basis: DMat3,
    pub translation: DVec3,
    pub orig_basis: DMat3,
    pub orig_translation: DVec3,
    pub max_extension: f64,
    pub global_start: DVec3,
    pub global_lin: DMat3,
    pub global_t: DVec3,
    pub num_dof: usize,
    pub dof_id: usize,
    pub locked: [bool; 3],
    pub translational: bool,
    pub weight: [f64; 3],
}

impl BSegment {
    pub fn new(joint: BJoint) -> Self {
        let (num_dof, translational) = match &joint {
            BJoint::Null => (0, false),
            BJoint::Spherical { .. } => (3, false),
            BJoint::Revolute { .. } => (1, false),
            BJoint::Swing { .. } | BJoint::Elbow { .. } => (2, false),
            BJoint::Translate { enabled, .. } => (enabled.iter().filter(|e| **e).count(), true),
        };
        Self {
            joint,
            parent: None,
            children: Vec::new(),
            composite: None,
            start: DVec3::ZERO,
            rest_basis: DMat3::IDENTITY,
            basis: DMat3::IDENTITY,
            translation: DVec3::ZERO,
            orig_basis: DMat3::IDENTITY,
            orig_translation: DVec3::ZERO,
            max_extension: 0.0,
            global_start: DVec3::ZERO,
            global_lin: DMat3::IDENTITY,
            global_t: DVec3::ZERO,
            num_dof,
            dof_id: 0,
            locked: [false; 3],
            translational,
            weight: [1.0; 3],
        }
    }

    /// A translation joint along `axes` (in order).
    pub fn translate(axes: &[usize]) -> BJoint {
        let mut enabled = [false; 3];
        let mut axis = [0; 3];
        for (i, &a) in axes.iter().enumerate() {
            enabled[a] = true;
            axis[i] = a;
        }
        BJoint::Translate { axis, enabled, limit: [false; 3], min: [0.0; 3], max: [0.0; 3], new_translation: DVec3::ZERO }
    }

    /// `SetBasis`, by joint.
    pub fn set_basis(&mut self, basis: DMat3) {
        self.basis = match &mut self.joint {
            BJoint::Null => DMat3::IDENTITY,
            BJoint::Revolute { axis, angle, .. } => {
                if *axis == 1 {
                    *angle = BIkMath::twist(&basis);
                    BIkMath::rotation_angle(*angle, 1)
                } else {
                    *angle = BIkMath::euler_angle(&basis, *axis);
                    BIkMath::rotation_angle(*angle, *axis)
                }
            }
            BJoint::Swing { .. } => BIkMath::remove_twist(&basis),
            BJoint::Elbow { axis, twist, angle, .. } => {
                *twist = BIkMath::twist(&basis);
                *angle = BIkMath::euler_angle(&basis, *axis);
                BIkMath::rotation_angle(*angle, *axis) * BIkMath::rotation_angle(*twist, 1)
            }
            _ => basis,
        };
    }

    /// `SetTransform`.
    pub fn set_transform(&mut self, start: DVec3, rest: DMat3, basis: DMat3, length: f64) {
        self.max_extension = start.length() + length;
        self.start = start;
        self.rest_basis = rest;
        self.orig_basis = basis;
        self.set_basis(basis);
        self.translation = DVec3::new(0.0, length, 0.0);
        self.orig_translation = self.translation;
    }

    /// `SetLimit` (angles in radians; a translation joint in length).
    pub fn set_limit(&mut self, axis: usize, lmin: f64, lmax: f64) {
        use std::f64::consts::PI;
        match &mut self.joint {
            BJoint::Spherical { limit, min, max, min_y, max_y, .. } if lmin <= lmax => {
                let (lo, hi) = (lmin.clamp(-PI, PI), lmax.clamp(-PI, PI));
                if axis == 1 {
                    (*min_y, *max_y, limit[1]) = (lo, hi, true);
                } else {
                    let (lo, hi) = ((lo * 0.5).sin(), (hi * 0.5).sin());
                    let k = if axis == 0 { 0 } else { 1 };
                    (min[k], max[k], limit[axis]) = (-hi, -lo, true);
                }
            }
            BJoint::Revolute { axis: a, limit, min, max, .. } if lmin <= lmax && *a == axis => {
                (*min, *max, *limit) = (lmin.clamp(-PI, PI), lmax.clamp(-PI, PI), true);
            }
            BJoint::Swing { limit_x, limit_z, min, max, .. } if lmin <= lmax => {
                let (lo, hi) = ((lmin.clamp(-PI, PI) * 0.5).sin(), (lmax.clamp(-PI, PI) * 0.5).sin());
                if axis == 0 {
                    (min[0], max[0], *limit_x) = (-hi, -lo, true);
                } else if axis == 2 {
                    (min[1], max[1], *limit_z) = (-hi, -lo, true);
                }
            }
            BJoint::Elbow { axis: a, limit, limit_twist, min, max, min_twist, max_twist, .. } if lmin <= lmax => {
                let (lo, hi) = (lmin.clamp(-PI, PI), lmax.clamp(-PI, PI));
                if axis == 1 {
                    (*min_twist, *max_twist, *limit_twist) = (lo, hi, true);
                } else if axis == *a {
                    (*min, *max, *limit) = (lo, hi, true);
                }
            }
            BJoint::Translate { limit, min, max, .. } if lmax >= lmin => {
                (min[axis], max[axis], limit[axis]) = (lmin, lmax, true);
            }
            _ => {}
        }
    }

    /// `SetWeight`, by joint.
    pub fn set_weight(&mut self, axis: usize, weight: f64) {
        match &self.joint {
            BJoint::Spherical { .. } => self.weight[axis] = weight,
            BJoint::Revolute { axis: a, .. } if *a == axis => self.weight[0] = weight,
            BJoint::Swing { .. } if axis == 0 => self.weight[0] = weight,
            BJoint::Swing { .. } if axis == 2 => self.weight[1] = weight,
            BJoint::Elbow { axis: a, .. } if *a == axis => self.weight[0] = weight,
            BJoint::Elbow { .. } if axis == 1 => self.weight[1] = weight,
            BJoint::Translate { axis: axes, .. } => {
                for i in 0..self.num_dof {
                    if axes[i] == axis {
                        self.weight[i] = weight;
                    }
                }
            }
            _ => {}
        }
    }

    /// `Axis(dof)`: the joint's axis in world space.
    pub fn axis(&self, dof: usize) -> DVec3 {
        match &self.joint {
            BJoint::Null => DVec3::ZERO,
            BJoint::Spherical { .. } => self.global_lin.col(dof),
            BJoint::Revolute { axis, .. } => self.global_lin.col(*axis),
            BJoint::Swing { .. } => self.global_lin.col(if dof == 0 { 0 } else { 2 }),
            BJoint::Elbow { axis, cos_twist, sin_twist, .. } => {
                if dof == 0 {
                    let v = if *axis == 0 { DVec3::new(*cos_twist, 0.0, *sin_twist) } else { DVec3::new(-*sin_twist, 0.0, *cos_twist) };
                    self.global_lin * v
                } else {
                    self.global_lin.col(1)
                }
            }
            BJoint::Translate { axis, .. } => self.global_lin.col(axis[dof]),
        }
    }

    /// `Scale`.
    pub fn scale(&mut self, s: f64) {
        self.start *= s;
        self.translation *= s;
        self.orig_translation *= s;
        self.global_start *= s;
        self.global_t *= s;
        self.max_extension *= s;
        if let BJoint::Translate { min, max, new_translation, .. } = &mut self.joint {
            // As Blender has it: the loop scales `m_min[0]` and `m_max[1]`,
            // three times over.
            for _ in 0..3 {
                min[0] *= s;
                max[1] *= s;
            }
            *new_translation *= s;
        }
    }

    /// `BasisChange`.
    pub fn basis_change(&self) -> DMat3 {
        self.orig_basis.transpose() * self.basis
    }

    /// `TranslationChange`.
    pub fn translation_change(&self) -> DVec3 {
        self.translation - self.orig_translation
    }

    /// `PrependBasis`.
    pub fn prepend_basis(&mut self, mat: &DMat3) {
        self.basis = self.rest_basis.inverse() * *mat * self.rest_basis * self.basis;
    }
}
