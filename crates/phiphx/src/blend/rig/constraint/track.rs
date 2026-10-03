//! Damped Track and Stretch To (`damptrack_do_transform`,
//! `stretchto_evaluate`).

use super::{BCob, BConstraintKind};
use crate::blend::rig::math::{BCompose, BMat4, BVec3};
use glam::{Mat3, Mat4, Vec3};

/// `track_dir_vecs`: ±X, ±Y, ±Z by `TRACK_*`.
const TRACK_DIR: [Vec3; 6] = [Vec3::X, Vec3::Y, Vec3::Z, Vec3::NEG_X, Vec3::NEG_Y, Vec3::NEG_Z];
/// `TRACK_Y`.
const TRACK_Y: u32 = 1;
/// `VOLUME_*`, `PLANE_X` / `SWING_Y` / `PLANE_Z`.
const VOLUME_XZ: u32 = 0;
const VOLUME_X: u32 = 1;
const VOLUME_Z: u32 = 2;
const NO_VOLUME: u32 = 3;
const PLANE_X: u32 = 0;
const SWING_Y: u32 = 1;
const PLANE_Z: u32 = 2;
/// `STRETCHTOCON_USE_BULGE_MIN` / `_MAX`.
const BULGE_MIN: u32 = 1 << 0;
const BULGE_MAX: u32 = 1 << 1;

impl BCob {
    /// `damptrack_do_transform`: the smallest turn that points the `track`
    /// axis along `towards`.
    pub(super) fn damped_track(matrix: &mut Mat4, towards: Vec3, track: u32) {
        let mut tarvec = towards;
        if tarvec.normalize_len() == 0.0 {
            return;
        }
        let track = (track as usize).min(5);
        let mut obvec = Mat3::from_mat4(*matrix) * TRACK_DIR[track];
        if obvec.normalize_len() == 0.0 {
            obvec = TRACK_DIR[track];
        }
        let obloc = matrix.loc();
        // `cross_high_precision`: the cross product in double.
        let (o, t) = (obvec.as_dvec3(), tarvec.as_dvec3());
        let mut raxis = o.cross(t).as_vec3();
        let mut rangle = obvec.dot(tarvec).clamp(-1.0, 1.0).acos();
        let norm = raxis.normalize_len();
        if norm < f32::EPSILON {
            if rangle.abs() < std::f32::consts::PI - 0.01 {
                return;
            }
            rangle = std::f32::consts::PI;
            let tmp = Mat3::from_mat4(*matrix) * TRACK_DIR[(track + 1) % 6];
            raxis = obvec.cross(tmp);
            if raxis.normalize_len() == 0.0 {
                return;
            }
        } else if norm < 0.1 {
            rangle = if rangle > std::f32::consts::FRAC_PI_2 { std::f32::consts::PI - norm.asin() } else { norm.asin() };
        }
        let rmat = BCompose::axis_angle(raxis, rangle);
        // `mul_m4_m3m4(tmat, rmat, matrix)`: the 3×3 part turned, the
        // location put back.
        *matrix = Mat4::from_mat3(rmat * Mat3::from_mat4(*matrix));
        matrix.set_loc(obloc);
    }

    pub(super) fn stretch_to(cob: &mut BCob, target: Mat4, kind: &BConstraintKind) {
        let BConstraintKind::StretchTo { volume, plane, rest, bulge, bulge_min, bulge_max, bulge_smooth, flag, .. } = *kind else { return };
        if plane == SWING_Y {
            cob.matrix.orthogonalize_stable(1, false);
        }
        let mut size = cob.matrix.normalize_ex();
        let xx = cob.matrix.x_axis.truncate();
        let zz = cob.matrix.z_axis.truncate();
        let mut vec = target.loc() - cob.matrix.loc();
        let mut dist = vec.normalize_len();
        // `math::safe_divide`.
        dist = if size.y != 0.0 { dist / size.y } else { 0.0 };
        // A rest length of 0 is taken from this first evaluation.
        let rest = if rest == 0.0 { dist } else { rest };
        let mut scale = Vec3::ONE;
        scale.y = dist / rest;
        let mut b = (rest / dist).powf(bulge);
        if b > 1.0 && flag & BULGE_MAX != 0 {
            let bmax = bulge_max.max(1.0);
            let hard = b.min(bmax);
            let range = bmax - 1.0;
            let fac = if range > 0.0 { 1.0 / range } else { 0.0 };
            let soft = 1.0 + range * ((b - 1.0) * fac).atan() / std::f32::consts::FRAC_PI_2;
            b = bulge_smooth * soft + (1.0 - bulge_smooth) * hard;
        }
        if b < 1.0 && flag & BULGE_MIN != 0 {
            let bmin = bulge_min.clamp(0.0, 1.0);
            let hard = b.max(bmin);
            let range = 1.0 - bmin;
            let fac = if range > 0.0 { 1.0 / range } else { 0.0 };
            let soft = 1.0 - range * ((1.0 - b) * fac).atan() / std::f32::consts::FRAC_PI_2;
            b = bulge_smooth * soft + (1.0 - bulge_smooth) * hard;
        }
        (scale.x, scale.z) = match volume {
            VOLUME_XZ => (b.sqrt(), b.sqrt()),
            VOLUME_X => (b, 1.0),
            VOLUME_Z => (1.0, b),
            NO_VOLUME => (1.0, 1.0),
            _ => return,
        };
        size *= scale;
        match plane {
            SWING_Y => Self::damped_track(&mut cob.matrix, vec, TRACK_Y),
            PLANE_X => {
                cob.matrix.y_axis = vec.extend(cob.matrix.y_axis.w);
                let mut orth = xx.cross(vec);
                orth.normalize_len();
                cob.matrix.z_axis = orth.extend(cob.matrix.z_axis.w);
                let mut x = vec.cross(orth);
                x.normalize_len();
                cob.matrix.x_axis = x.extend(cob.matrix.x_axis.w);
            }
            PLANE_Z => {
                cob.matrix.y_axis = vec.extend(cob.matrix.y_axis.w);
                let mut orth = zz.cross(vec);
                orth.normalize_len();
                cob.matrix.x_axis = (-orth).extend(cob.matrix.x_axis.w);
                let mut z = vec.cross(orth);
                z.normalize_len();
                cob.matrix.z_axis = z.extend(cob.matrix.z_axis.w);
            }
            _ => {}
        }
        cob.matrix.rescale(size);
    }
}
