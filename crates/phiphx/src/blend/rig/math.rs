//! Blender's matrix and rotation math, as the pose solver needs it
//! (`blenlib/intern/math_matrix_c.cc`, `math_rotation_c.cc`), in single
//! precision where Blender works in single precision. Matrices are glam's,
//! whose columns are Blender's `m[0..4]`: `m[i][j]` is `col(i)[j]`.

use glam::{DMat3, DVec3, Mat3, Mat4, Vec3};

/// A rotation order, `eRotationModes` 1–6 (`ROT_MODE_XYZ` … `ZYX`); the
/// quaternion and axis-angle modes (0, −1) read as XYZ, as Blender's
/// `get_rotation_order_info` does.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BEulerOrder(pub i16);

/// `EULER_HYPOT_EPSILON`.
const EULER_HYPOT_EPSILON: f32 = 0.000_037_5;

impl BEulerOrder {
    /// `rotOrders[]`: the axes in order, and the parity.
    fn info(self) -> ([usize; 3], bool) {
        match self.0 {
            2 => ([0, 2, 1], true),
            3 => ([1, 0, 2], true),
            4 => ([1, 2, 0], false),
            5 => ([2, 0, 1], false),
            6 => ([2, 1, 0], true),
            _ => ([0, 1, 2], false),
        }
    }

    /// `eulO_to_mat3` (worked in double, as Blender does).
    pub fn to_mat3(self, e: Vec3) -> Mat3 {
        let ([i, j, k], parity) = self.info();
        let s = if parity { -1.0 } else { 1.0 };
        let (ti, tj, th) = (s * e[i] as f64, s * e[j] as f64, s * e[k] as f64);
        let (ci, cj, ch) = (ti.cos(), tj.cos(), th.cos());
        let (si, sj, sh) = (ti.sin(), tj.sin(), th.sin());
        let (cc, cs, sc, ss) = (ci * ch, ci * sh, si * ch, si * sh);
        let mut m = [[0.0f64; 3]; 3];
        m[i][i] = cj * ch;
        m[j][i] = sj * sc - cs;
        m[k][i] = sj * cc + ss;
        m[i][j] = cj * sh;
        m[j][j] = sj * ss + cc;
        m[k][j] = sj * cs - sc;
        m[i][k] = -sj;
        m[j][k] = cj * si;
        m[k][k] = cj * ci;
        Mat3::from_cols_array_2d(&m.map(|c| c.map(|v| v as f32)))
    }

    /// `mat3_normalized_to_eulo2`: both Euler solutions of a unit matrix.
    fn both(self, m: &Mat3) -> (Vec3, Vec3) {
        let ([i, j, k], parity) = self.info();
        let at = |c: usize, r: usize| m.col(c)[r];
        let cy = at(i, i).hypot(at(i, j));
        let (mut a, mut b) = (Vec3::ZERO, Vec3::ZERO);
        if cy > EULER_HYPOT_EPSILON {
            a[i] = at(j, k).atan2(at(k, k));
            a[j] = (-at(i, k)).atan2(cy);
            a[k] = at(i, j).atan2(at(i, i));
            b[i] = (-at(j, k)).atan2(-at(k, k));
            b[j] = (-at(i, k)).atan2(-cy);
            b[k] = (-at(i, j)).atan2(-at(i, i));
        } else {
            a[i] = (-at(k, j)).atan2(at(j, j));
            a[j] = (-at(i, k)).atan2(cy);
            a[k] = 0.0;
            b = a;
        }
        if parity { (-a, -b) } else { (a, b) }
    }

    /// `mat3_normalized_to_eulO`: the solution with the smaller angles.
    pub fn of_normalized(self, m: &Mat3) -> Vec3 {
        let (a, b) = self.both(m);
        if a.abs().element_sum() > b.abs().element_sum() { b } else { a }
    }

    /// `mat4_to_eulO`.
    pub fn of_mat4(self, m: &Mat4) -> Vec3 {
        self.of_normalized(&Mat3::from_mat4(*m).normalized_cols())
    }

    /// `mat4_to_compatible_eulO`: the solution nearest `old`.
    pub fn compatible_of_mat4(self, old: Vec3, m: &Mat4) -> Vec3 {
        let (a, b) = self.both(&Mat3::from_mat4(*m).normalized_cols());
        let (a, b) = (Self::compatible(a, old), Self::compatible(b, old));
        if (a - old).abs().element_sum() > (b - old).abs().element_sum() { b } else { a }
    }

    /// `compatible_eul`: `eul` wrapped by whole turns towards `old`.
    pub fn compatible(mut eul: Vec3, old: Vec3) -> Vec3 {
        let (pi, pi_x2) = (std::f32::consts::PI, 2.0 * std::f32::consts::PI);
        let mut d = Vec3::ZERO;
        for i in 0..3 {
            d[i] = eul[i] - old[i];
            if d[i] > pi {
                eul[i] -= ((d[i] / pi_x2) + 0.5).floor() * pi_x2;
                d[i] = eul[i] - old[i];
            } else if d[i] < -pi {
                eul[i] += ((-d[i] / pi_x2) + 0.5).floor() * pi_x2;
                d[i] = eul[i] - old[i];
            }
        }
        let (mut i, mut j, mut k) = (0usize, 1usize, 2usize);
        while i < 3 {
            let half = std::f32::consts::FRAC_PI_2;
            if d[i].abs() > pi && d[j].abs() < half && d[k].abs() < half {
                eul[i] += if d[i] > 0.0 { -pi_x2 } else { pi_x2 };
            }
            // `for (i = 0; i < 3; j = k, k = i++)`.
            j = k;
            k = i;
            i += 1;
        }
        eul
    }
}

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

/// Blender's `normalize_v3`: the length, the vector scaled to 1 (zero
/// below 1e-35 squared).
pub trait BVec3 {
    fn normalize_len(&mut self) -> f32;
    /// `angle_normalized_v3v3`: the angle between two unit vectors, by the
    /// chord (more exact than `acos` of the dot).
    fn angle_normalized(&self, b: Vec3) -> f32;
    /// `bisect_v3_v3v3v3`: the unit direction halfway between `a → self`
    /// and `self → c`.
    fn bisect(&self, a: Vec3, c: Vec3) -> Vec3;
}

impl BVec3 for Vec3 {
    fn normalize_len(&mut self) -> f32 {
        let d = self.dot(*self);
        if d > 1.0e-35 {
            let d = d.sqrt();
            *self *= 1.0 / d;
            d
        } else {
            *self = Vec3::ZERO;
            0.0
        }
    }

    fn angle_normalized(&self, b: Vec3) -> f32 {
        let half = |d: f32| (d / 2.0).clamp(-1.0, 1.0).asin();
        if self.dot(b) >= 0.0 { 2.0 * half(self.distance(b)) } else { std::f32::consts::PI - 2.0 * half(self.distance(-b)) }
    }

    fn bisect(&self, a: Vec3, c: Vec3) -> Vec3 {
        let (mut d12, mut d23) = (*self - a, c - *self);
        d12.normalize_len();
        d23.normalize_len();
        let mut r = d12 + d23;
        r.normalize_len();
        r
    }
}

/// 3×3 matrix helpers.
pub trait BMat3 {
    fn normalized_cols(&self) -> Mat3;
    /// `mat3_to_size`.
    fn size3(&self) -> Vec3;
    /// `mat3_to_rot_size`: unit columns and their lengths, a mirror moved
    /// into the lengths.
    fn rot_size(&self) -> (Mat3, Vec3);
    /// `mat3_polar_decompose`: `self = U · P`, `U` the nearest orthogonal.
    fn polar(&self) -> (Mat3, Mat3);
    /// `interp_m3_m3m3`.
    fn interp(&self, b: &Mat3, t: f32) -> Mat3;
    /// `blend_m3_m3m3(out, self, src, w)`.
    fn blend(&self, src: &Mat3, w: f32) -> Mat3;
}

impl BMat3 for Mat3 {
    fn normalized_cols(&self) -> Mat3 {
        let mut c = [self.x_axis, self.y_axis, self.z_axis];
        c.iter_mut().for_each(|v| _ = v.normalize_len());
        Mat3::from_cols(c[0], c[1], c[2])
    }

    fn size3(&self) -> Vec3 {
        Vec3::new(self.x_axis.length(), self.y_axis.length(), self.z_axis.length())
    }

    fn rot_size(&self) -> (Mat3, Vec3) {
        let mut c = [self.x_axis, self.y_axis, self.z_axis];
        let size = Vec3::from_array(std::array::from_fn(|i| c[i].normalize_len()));
        let rot = Mat3::from_cols(c[0], c[1], c[2]);
        if rot.determinant() < 0.0 { (-rot, -size) } else { (rot, size) }
    }

    fn polar(&self) -> (Mat3, Mat3) {
        // From the eigen-decomposition of Mᵀ·M = V·S²·Vᵀ (in double):
        // P = V·S·Vᵀ, U = M·V·S⁻¹·Vᵀ — the SVD's U = W·Vᵀ, P = V·S·Vᵀ.
        let m = self.as_dmat3();
        let (vals, vecs) = BSymmetric::eigen(m.transpose() * m);
        let s = vals.map(|v| v.max(0.0).sqrt());
        let p = vecs * DMat3::from_diagonal(s) * vecs.transpose();
        let inv = DVec3::from_array(s.to_array().map(|v| if v > 1e-12 { 1.0 / v } else { 0.0 }));
        let u = m * vecs * DMat3::from_diagonal(inv) * vecs.transpose();
        (u.as_mat3(), p.as_mat3())
    }

    fn interp(&self, b: &Mat3, t: f32) -> Mat3 {
        let (mut ua, mut pa) = self.polar();
        let (mut ub, mut pb) = b.polar();
        if ua.determinant() < 0.0 {
            (ua, pa) = (-ua, -pa);
        }
        if ub.determinant() < 0.0 {
            (ub, pb) = (-ub, -pb);
        }
        let q = BQuat::interp(BQuat::of_mat3(&ua), BQuat::of_mat3(&ub), t);
        let p = Mat3::from_cols(pa.x_axis.lerp(pb.x_axis, t), pa.y_axis.lerp(pb.y_axis, t), pa.z_axis.lerp(pb.z_axis, t));
        q.to_mat3() * p
    }

    fn blend(&self, src: &Mat3, w: f32) -> Mat3 {
        let (drot, dscale) = self.rot_size();
        let (srot, sscale) = src.rot_size();
        let q = BQuat::interp(BQuat::of_normalized_fast(&drot), BQuat::of_normalized_fast(&srot), w);
        q.to_mat3() * Mat3::from_diagonal(dscale.lerp(sscale, w))
    }
}

/// 4×4 matrix helpers.
pub trait BMat4 {
    /// `orthogonalize_m4_stable`: the other axes made square to `axis`,
    /// keeping the determinant.
    fn orthogonalize_stable(&mut self, axis: usize, normalize: bool);
    /// `normalize_m4_ex`: unit axes, their lengths returned.
    fn normalize_ex(&mut self) -> Vec3;
    /// `mat4_to_size`.
    fn size(&self) -> Vec3;
    /// `mat4_to_size_fix_shear`.
    fn size_fix_shear(&self) -> Vec3;
    /// `mat4_to_volume_scale`: the 3×3 determinant.
    fn volume(&self) -> f32;
    /// `rescale_m4`.
    fn rescale(&mut self, s: Vec3);
    /// `mat4_to_loc_rot_size`.
    fn loc_rot_size(&self) -> (Vec3, Mat3, Vec3);
    /// `interp_m4_m4m4`.
    fn interp(&self, b: &Mat4, t: f32) -> Mat4;
    /// The 3×3 part replaced, translation kept.
    fn with_m3(&self, m: Mat3) -> Mat4;
    fn loc(&self) -> Vec3;
    fn set_loc(&mut self, v: Vec3);
    /// `invert_m4_m4_safe`: a degenerate matrix nudged along its diagonal,
    /// the identity when even that will not invert.
    fn inverse_safe(&self) -> Mat4;
}

impl BMat4 for Mat4 {
    fn orthogonalize_stable(&mut self, axis: usize, normalize: bool) {
        let mut c = [self.x_axis.truncate(), self.y_axis.truncate(), self.z_axis.truncate()];
        let (a, b, d) = match axis {
            0 => (0, 1, 2),
            1 => (1, 0, 2),
            _ => (2, 0, 1),
        };
        let (mut v1, mut v2, mut v3) = (c[a], c[b], c[d]);
        BOrtho::stable(&mut v1, &mut v2, &mut v3, normalize);
        (c[a], c[b], c[d]) = (v1, v2, v3);
        self.x_axis = c[0].extend(self.x_axis.w);
        self.y_axis = c[1].extend(self.y_axis.w);
        self.z_axis = c[2].extend(self.z_axis.w);
    }

    fn inverse_safe(&self) -> Mat4 {
        if self.determinant() != 0.0 {
            return self.inverse();
        }
        let mut m = *self;
        m.x_axis.x += 1e-8;
        m.y_axis.y += 1e-8;
        m.z_axis.z += 1e-8;
        if m.determinant() != 0.0 { m.inverse() } else { Mat4::IDENTITY }
    }

    fn normalize_ex(&mut self) -> Vec3 {
        let mut size = Vec3::ZERO;
        for i in 0..3 {
            let col = self.col_mut(i);
            let mut v = col.truncate();
            size[i] = v.normalize_len();
            let w = if size[i] != 0.0 { col.w / size[i] } else { col.w };
            *col = v.extend(w);
        }
        size
    }

    fn size(&self) -> Vec3 {
        Mat3::from_mat4(*self).size3()
    }

    fn size_fix_shear(&self) -> Vec3 {
        let size = self.size();
        let volume = size.x * size.y * size.z;
        if volume != 0.0 { size * (self.volume() / volume).abs().cbrt() } else { size }
    }

    fn volume(&self) -> f32 {
        Mat3::from_mat4(*self).determinant()
    }

    fn rescale(&mut self, s: Vec3) {
        for i in 0..3 {
            let col = self.col_mut(i);
            *col = (col.truncate() * s[i]).extend(col.w);
        }
    }

    fn loc_rot_size(&self) -> (Vec3, Mat3, Vec3) {
        let (rot, size) = Mat3::from_mat4(*self).rot_size();
        (self.loc(), rot, size)
    }

    fn interp(&self, b: &Mat4, t: f32) -> Mat4 {
        let m = Mat3::from_mat4(*self).interp(&Mat3::from_mat4(*b), t);
        let mut out = Mat4::from_mat3(m);
        out.set_loc(self.loc().lerp(b.loc(), t));
        out
    }

    fn with_m3(&self, m: Mat3) -> Mat4 {
        let mut out = Mat4::from_mat3(m);
        out.set_loc(self.loc());
        out
    }

    fn loc(&self) -> Vec3 {
        self.w_axis.truncate()
    }

    fn set_loc(&mut self, v: Vec3) {
        self.w_axis = v.extend(self.w_axis.w);
    }
}

/// `loc_rot_size_to_mat4` / `loc_eulO_size_to_mat4`.
pub struct BCompose;

impl BCompose {
    pub fn loc_rot_size(loc: Vec3, rot: Mat3, size: Vec3) -> Mat4 {
        let mut m = Mat4::from_mat3(rot);
        m.rescale(size);
        m.set_loc(loc);
        m
    }

    pub fn loc_eul_size(loc: Vec3, eul: Vec3, size: Vec3, order: BEulerOrder) -> Mat4 {
        let mut m = Mat4::from_mat3(order.to_mat3(eul) * Mat3::from_diagonal(size));
        m.set_loc(loc);
        m
    }

    /// `axis_angle_normalized_to_mat3`.
    pub fn axis_angle(axis: Vec3, angle: f32) -> Mat3 {
        let (s, c) = angle.sin_cos();
        let ico = 1.0 - c;
        let nsi = axis * s;
        let (n00, n01, n11) = (axis.x * axis.x * ico, axis.x * axis.y * ico, axis.y * axis.y * ico);
        let (n02, n12, n22) = (axis.x * axis.z * ico, axis.y * axis.z * ico, axis.z * axis.z * ico);
        Mat3::from_cols_array_2d(&[
            [n00 + c, n01 + nsi.z, n02 - nsi.y],
            [n01 - nsi.z, n11 + c, n12 + nsi.x],
            [n02 + nsi.y, n12 - nsi.x, n22 + c],
        ])
    }
}

/// `orthogonalize_stable` (the plane projection that keeps the volume).
struct BOrtho;

impl BOrtho {
    fn stable(v1: &mut Vec3, v2: &mut Vec3, v3: &mut Vec3, normalize: bool) {
        let len_sq_v1 = v1.length_squared();
        if len_sq_v1 > 0.0 {
            *v2 += *v1 * (-v2.dot(*v1) / len_sq_v1);
            *v3 += *v1 * (-v3.dot(*v1) / len_sq_v1);
            if normalize {
                *v1 *= 1.0 / len_sq_v1.sqrt();
            }
        }
        let (mut n2, mut n3) = (*v2, *v3);
        let (len2, len3) = (n2.normalize_len(), n3.normalize_len());
        let cos_angle = n2.dot(n3);
        let abs_cos = cos_angle.abs();
        if abs_cos > 1e-4 && abs_cos < 1.0 - f32::EPSILON {
            let angle = cos_angle.acos();
            let target = angle + (std::f32::consts::FRAC_PI_2 - angle) / 2.0;
            n2 += n3 * -cos_angle;
            n2 *= target.sin() / n2.length();
            n2 += n3 * target.cos();
            let tmp = n2.cross(n3);
            n3 = tmp.cross(n2);
            n3.normalize_len();
            if !normalize {
                let fac = angle.sin().sqrt();
                *v2 = n2 * (len2 * fac);
                *v3 = n3 * (len3 * fac);
            }
        }
        if normalize {
            *v2 = n2;
            *v3 = n3;
        }
    }
}

/// A symmetric 3×3 eigen-decomposition by Jacobi rotations (double).
struct BSymmetric;

impl BSymmetric {
    /// Eigenvalues and the eigenvectors as columns (cyclic Jacobi:
    /// `A ← Jᵀ·A·J` for each off-diagonal pair until it is diagonal).
    fn eigen(m: DMat3) -> (DVec3, DMat3) {
        // Row-major: `a[r][c]`; `v`'s columns become the eigenvectors.
        let mut a: [[f64; 3]; 3] = std::array::from_fn(|r| std::array::from_fn(|c| m.col(c)[r]));
        let mut v = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
        for _ in 0..64 {
            if a[0][1].abs() + a[0][2].abs() + a[1][2].abs() < 1e-15 {
                break;
            }
            for (p, q) in [(0usize, 1usize), (0, 2), (1, 2)] {
                if a[p][q].abs() < 1e-300 {
                    continue;
                }
                let theta = (a[q][q] - a[p][p]) / (2.0 * a[p][q]);
                let t = if theta == 0.0 { 1.0 } else { theta.signum() / (theta.abs() + (theta * theta + 1.0).sqrt()) };
                let c = 1.0 / (t * t + 1.0).sqrt();
                let s = t * c;
                for row in a.iter_mut() {
                    let (kp, kq) = (row[p], row[q]);
                    row[p] = c * kp - s * kq;
                    row[q] = s * kp + c * kq;
                }
                for k in 0..3 {
                    let (pk, qk) = (a[p][k], a[q][k]);
                    a[p][k] = c * pk - s * qk;
                    a[q][k] = s * pk + c * qk;
                }
                for row in v.iter_mut() {
                    let (kp, kq) = (row[p], row[q]);
                    row[p] = c * kp - s * kq;
                    row[q] = s * kp + c * kq;
                }
            }
        }
        (DVec3::new(a[0][0], a[1][1], a[2][2]), DMat3::from_cols_array_2d(&v).transpose())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn polar_splits_rotation_from_scale() {
        let r = BEulerOrder(1).to_mat3(Vec3::new(0.3, -0.7, 1.1));
        let s = Mat3::from_cols_array_2d(&[[2.0, 0.1, 0.0], [0.1, 0.5, 0.2], [0.0, 0.2, 1.3]]);
        let (u, p) = (r * s).polar();
        assert!((u - r).abs_diff_eq(Mat3::ZERO, 1e-5), "{u} vs {r}");
        assert!((p - s).abs_diff_eq(Mat3::ZERO, 1e-5), "{p} vs {s}");
    }

    #[test]
    fn eulers_round_trip_in_every_order() {
        let e = Vec3::new(0.4, -0.2, 0.9);
        for order in 1..=6 {
            let o = BEulerOrder(order);
            let back = o.of_normalized(&o.to_mat3(e));
            assert!((back - e).abs().max_element() < 1e-5, "order {order}: {back}");
        }
    }

    #[test]
    fn a_quaternion_survives_its_matrix() {
        let q = BQuat([0.8, 0.2, -0.4, 0.4]);
        let n: f32 = q.0.iter().map(|v| v * v).sum::<f32>().sqrt();
        let q = BQuat(q.0.map(|v| v / n));
        let back = BQuat::of_normalized_fast(&q.to_mat3());
        assert!(back.0.iter().zip(q.0).all(|(a, b)| (a - b).abs() < 1e-5), "{back:?}");
    }
}
