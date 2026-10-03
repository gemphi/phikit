//! Track To and Locked Track (`trackto_evaluate` with `vectomat`,
//! `locktrack_evaluate`).

use super::BCob;
use crate::blend::rig::math::{BMat3, BMat4, BVec3};
use glam::{Mat3, Mat4, Vec3};

/// `TARGET_Z_UP`.
const TARGET_Z_UP: u32 = 1 << 0;

impl BCob {
    /// `trackto_evaluate`: the owner's rotation cleared, its scale kept, and
    /// `track` (`TRACK_*`) aimed at the target with `up` (`UP_*`) towards
    /// world Z, or the target's Z with `TARGET_Z_UP`.
    pub(super) fn track_to(cob: &mut BCob, target: Mat4, track: u32, up: u32, flags: u32) {
        let size = cob.matrix.size();
        let vec = cob.matrix.loc() - target.loc();
        let totmat = Self::vectomat(vec, target.z_axis.truncate(), track.min(5) as usize, up.min(2) as usize, flags);
        let m = totmat * Mat3::from_diagonal(size);
        cob.matrix = cob.matrix.with_m3(m);
    }

    /// `vectomat`: the rotation whose `axis` (0–5, the negative ones
    /// flipping the direction) runs along `vec` and whose `upflag` axis
    /// leans towards the up vector.
    fn vectomat(vec: Vec3, target_up: Vec3, axis: usize, upflag: usize, flags: u32) -> Mat3 {
        let mut n = vec;
        if n.normalize_len() == 0.0 {
            n = Vec3::Z;
        }
        let axis = if axis > 2 { axis - 3 } else {
            n = -n;
            axis
        };
        let u = if flags & TARGET_Z_UP != 0 { target_up } else { Vec3::Z };
        // `project_v3_v3v3`, then onto the plane.
        let nn = n.dot(n);
        let mut proj = u - if nn != 0.0 { n * (u.dot(n) / nn) } else { Vec3::ZERO };
        if proj.normalize_len() == 0.0 {
            proj = Vec3::Y;
        }
        let mut right = proj.cross(n);
        right.normalize_len();
        if axis == upflag {
            return Mat3::IDENTITY;
        }
        let mut cols = [Vec3::ZERO; 3];
        let right_index = 3 - axis - upflag;
        // `basis_cross`.
        let neg = match axis as i32 - upflag as i32 {
            1 | -2 => 1.0,
            -1 | 2 => -1.0,
            _ => 0.0,
        };
        cols[right_index] = right * neg;
        cols[upflag] = proj;
        cols[axis] = n;
        Mat3::from_cols(cols[0], cols[1], cols[2])
    }

    /// `locktrack_evaluate`: the `lock` axis (`LOCK_*`) kept, the `track`
    /// axis (`TRACK_*`) turned about it towards the target, the owner's
    /// heading kept.
    pub(super) fn locked_track(cob: &mut BCob, target: Mat4, track: u32, lock: u32) {
        let vec = target.loc() - cob.matrix.loc();
        let own = Mat3::from_mat4(cob.matrix);
        let (lock, track) = (lock as usize, track as usize);
        let t = track % 3;
        let mut totmat = if lock > 2 || track > 5 || t == lock {
            Mat3::IDENTITY
        } else {
            let mut cols = [Vec3::ZERO; 3];
            let locked = own.col(lock);
            // `project_v3_v3v3`: the target direction in the plane of the
            // locked axis.
            let ll = locked.dot(locked);
            let mut aim = vec - if ll != 0.0 { locked * (vec.dot(locked) / ll) } else { Vec3::ZERO };
            aim.normalize_len();
            cols[t] = if track > 2 { -aim } else { aim };
            let mut l = locked;
            l.normalize_len();
            cols[lock] = l;
            // The third axis completes a right-handed basis.
            let k = 3 - t - lock;
            cols[k] = cols[(k + 1) % 3].cross(cols[(k + 2) % 3]);
            Mat3::from_cols(cols[0], cols[1], cols[2])
        };
        // Keep the heading: relative to the owner's own rotation.
        let heading = own.normalized_cols();
        totmat = totmat * heading.inverse();
        if totmat.determinant() == 0.0 {
            totmat = Mat3::IDENTITY;
        }
        cob.matrix = cob.matrix.with_m3(totmat * own);
    }
}
