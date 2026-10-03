//! Limit Rotation, Limit Distance, Limit Location, Limit Scale and
//! Maintain Volume (`rotlimit_evaluate`, `distlimit_evaluate`,
//! `loclimit_evaluate`, `sizelimit_evaluate`, `samevolume_evaluate`).

use super::BCob;
use crate::blend::rig::math::{BCompose, BEulerOrder, BMat4};
use glam::{Mat4, Vec3};

/// `LIMIT_XROT` / `_YROT` / `_ZROT`, `LIMIT_ROT_LEGACY_BEHAVIOR`.
const LIMIT_ROT: [u32; 3] = [1 << 0, 1 << 1, 1 << 2];
const LIMIT_ROT_LEGACY: u32 = 1 << 3;
/// `LIMITDIST_USESOFT`, `LIMITDIST_INSIDE` / `_OUTSIDE`.
const LIMITDIST_USESOFT: u32 = 1 << 0;
const LIMITDIST_INSIDE: u32 = 0;
const LIMITDIST_OUTSIDE: u32 = 1;
/// `LIMIT_[XYZ]MIN`, `LIMIT_[XYZ]MAX`.
const LIMIT_MIN: [u32; 3] = [1 << 0, 1 << 2, 1 << 4];
const LIMIT_MAX: [u32; 3] = [1 << 1, 1 << 3, 1 << 5];
/// `SAMEVOL_STRICT` / `_UNIFORM` / `_SINGLE_AXIS`.
const SAMEVOL_STRICT: u32 = 0;
const SAMEVOL_UNIFORM: u32 = 1;
const SAMEVOL_SINGLE_AXIS: u32 = 2;

impl BCob {
    /// `angle_wrap_rad`.
    fn wrap(angle: f32) -> f32 {
        let pi = std::f32::consts::PI;
        (angle + pi).rem_euclid(2.0 * pi) - pi
    }

    /// `clamp_angle`: `angle` into `[min, max]` on a circle.
    fn clamp_angle(angle: f32, min: f32, max: f32) -> f32 {
        if max - min >= 2.0 * std::f32::consts::PI {
            return angle;
        }
        if max <= min {
            return min;
        }
        let (lo, hi) = (Self::wrap(min - angle), Self::wrap(max - angle));
        if lo < hi {
            return angle + 0.0f32.clamp(lo, hi);
        }
        if hi >= 0.0 || lo <= 0.0 {
            return angle;
        }
        if hi.abs() < lo.abs() { angle + hi } else { angle + lo }
    }

    pub(super) fn limit_rotation(cob: &mut BCob, min: Vec3, max: Vec3, flag: u32, order: u8) {
        cob.matrix.orthogonalize_stable(1, false);
        if flag & (LIMIT_ROT[0] | LIMIT_ROT[1] | LIMIT_ROT[2]) == 0 {
            return;
        }
        let order = BEulerOrder(if order != 0 { order as i16 } else { cob.rot_order });
        let (loc, size) = (cob.matrix.loc(), cob.matrix.size());
        let mut eul = order.of_mat4(&cob.matrix);
        for i in 0..3 {
            if flag & LIMIT_ROT[i] == 0 {
                continue;
            }
            eul[i] = if flag & LIMIT_ROT_LEGACY != 0 {
                // `clamp_f`: the upper bound first.
                if eul[i] > max[i] { max[i] } else if eul[i] < min[i] { min[i] } else { eul[i] }
            } else {
                Self::clamp_angle(eul[i], min[i], max[i])
            };
        }
        cob.matrix = BCompose::loc_eul_size(loc, eul, size, order);
    }

    pub(super) fn limit_distance(cob: &mut BCob, target: Mat4, dist_limit: f32, soft: f32, flag: u32, mode: u32) {
        let (own, tar) = (cob.matrix.loc(), target.loc());
        let dist = own.distance(tar);
        // A distance of 0 is taken from this first evaluation.
        let limit = if dist_limit == 0.0 { dist } else { dist_limit };
        let mut sfac = 1.0;
        let mut clamp = false;
        if mode == LIMITDIST_OUTSIDE {
            if dist <= limit {
                clamp = true;
                if dist != 0.0 {
                    sfac = limit / dist;
                }
            }
        } else if mode == LIMITDIST_INSIDE {
            if dist >= limit {
                clamp = true;
                if dist != 0.0 {
                    sfac = limit / dist;
                }
            } else if flag & LIMITDIST_USESOFT != 0 && dist >= limit - soft {
                sfac = soft * (1.0 - (-(dist - limit) / soft).exp()) + limit;
                if dist != 0.0 {
                    sfac /= dist;
                }
                clamp = true;
            }
        } else if (dist - limit).abs() >= f32::EPSILON {
            clamp = true;
            if dist != 0.0 {
                sfac = limit / dist;
            }
        }
        if clamp {
            cob.matrix.set_loc(tar.lerp(own, sfac));
        }
    }

    /// `loclimit_evaluate`: the location clamped per axis.
    pub(super) fn limit_location(cob: &mut BCob, min: Vec3, max: Vec3, flag: u32) {
        let mut loc = cob.matrix.loc();
        for i in 0..3 {
            if flag & LIMIT_MIN[i] != 0 {
                loc[i] = loc[i].max(min[i]);
            }
            if flag & LIMIT_MAX[i] != 0 {
                loc[i] = loc[i].min(max[i]);
            }
        }
        cob.matrix.set_loc(loc);
    }

    /// `sizelimit_evaluate`: each axis's scale clamped, its column scaled to
    /// it (an axis of no size left alone).
    pub(super) fn limit_scale(cob: &mut BCob, min: Vec3, max: Vec3, flag: u32) {
        let old = cob.matrix.size();
        let mut size = old;
        for i in 0..3 {
            if flag & LIMIT_MIN[i] != 0 {
                size[i] = size[i].max(min[i]);
            }
            if flag & LIMIT_MAX[i] != 0 {
                size[i] = size[i].min(max[i]);
            }
        }
        let scale = Vec3::from_array(std::array::from_fn(|i| if old[i] != 0.0 { size[i] / old[i] } else { 1.0 }));
        cob.matrix.rescale(scale);
    }

    /// `samevolume_evaluate`: the two axes other than `axis` scaled so the
    /// volume is `volume` (`mode`: all three axes, the free axis cubed, or
    /// the free axis alone).
    pub(super) fn maintain_volume(cob: &mut BCob, axis: u32, mode: u32, volume: f32) {
        let size = cob.matrix.size();
        let free = (axis as usize).min(2);
        let total = match mode {
            SAMEVOL_STRICT => size.x * size.y * size.z,
            SAMEVOL_UNIFORM => size[free].powi(3),
            SAMEVOL_SINGLE_AXIS => size[free],
            _ => 1.0,
        };
        let fac = if total != 0.0 { (volume / total).sqrt() } else { 1.0 };
        if axis <= 2 {
            let scale = Vec3::from_array(std::array::from_fn(|i| if i == free { 1.0 } else { fac }));
            cob.matrix.rescale(scale);
        }
    }
}
