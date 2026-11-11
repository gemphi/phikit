//! Stable orthogonalisation (`orthogonalize_stable`).
use crate::device::blend::rig::math::vec::BVec3;
use glam::Vec3;

/// `orthogonalize_stable` (the plane projection that keeps the volume).
pub(super) struct BOrtho;

impl BOrtho {
    pub(super) fn stable(v1: &mut Vec3, v2: &mut Vec3, v3: &mut Vec3, normalize: bool) {
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
