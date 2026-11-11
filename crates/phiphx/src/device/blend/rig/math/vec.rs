//! Vector and 3x3 matrix helpers (`math_vector_c.cc`, `math_matrix_c.cc`).
use super::quat::BQuat;
use super::symmetric::BSymmetric;
use glam::{DMat3, DVec3, Mat3, Vec3};

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
