//! 4x4 matrix helpers and the loc/rot/size composers (`math_matrix_c.cc`).
use super::euler::BEulerOrder;
use super::ortho::BOrtho;
use super::vec::{BMat3, BVec3};
use glam::{Mat3, Mat4, Vec3};

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
