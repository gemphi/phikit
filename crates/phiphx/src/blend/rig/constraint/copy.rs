//! Copy Transforms, Copy Location, Copy Rotation and Copy Scale
//! (`translike_evaluate`, `loclike_evaluate`, `rotlike_evaluate`,
//! `sizelike_evaluate`).

use super::BCob;
use crate::blend::rig::math::{BCompose, BEulerOrder, BMat4};
use glam::{Mat3, Mat4, Vec3};

/// `TRANSLIKE_REMOVE_TARGET_SHEAR`.
const TRANSLIKE_REMOVE_TARGET_SHEAR: u32 = 1 << 0;
/// `TRANSLIKE_MIX_*`.
const TRANS_REPLACE: u8 = 0;
const TRANS_BEFORE: u8 = 1;
const TRANS_AFTER: u8 = 2;
const TRANS_BEFORE_SPLIT: u8 = 3;
const TRANS_AFTER_SPLIT: u8 = 4;
const TRANS_BEFORE_FULL: u8 = 5;
const TRANS_AFTER_FULL: u8 = 6;
/// `LOCLIKE_*` / `ROTLIKE_*` / `SIZELIKE_*` axis bits.
const AXIS: [u32; 3] = [1 << 0, 1 << 1, 1 << 2];
const INVERT: [u32; 3] = [1 << 4, 1 << 5, 1 << 6];
const LOCLIKE_OFFSET: u32 = 1 << 7;
const SIZELIKE_OFFSET: u32 = 1 << 3;
const SIZELIKE_MULTIPLY: u32 = 1 << 4;
const SIZELIKE_UNIFORM: u32 = 1 << 5;
/// `ROTLIKE_MIX_*`.
const ROT_REPLACE: u8 = 0;
const ROT_OFFSET: u8 = 1;
const ROT_ADD: u8 = 2;
const ROT_BEFORE: u8 = 3;
const ROT_AFTER: u8 = 4;

impl BCob {
    /// `mul_m4_m4m4_aligned_scale`.
    pub(super) fn aligned_scale(a: &Mat4, b: &Mat4) -> Mat4 {
        let ((_, ra, sa), (lb, rb, sb)) = (a.loc_rot_size(), b.loc_rot_size());
        BCompose::loc_rot_size(a.transform_point3(lb), ra * rb, sa * sb)
    }

    /// `mul_m4_m4m4_split_channels`.
    pub(super) fn split_channels(a: &Mat4, b: &Mat4) -> Mat4 {
        let ((la, ra, sa), (lb, rb, sb)) = (a.loc_rot_size(), b.loc_rot_size());
        BCompose::loc_rot_size(la + lb, ra * rb, sa * sb)
    }

    pub(super) fn copy_transforms(cob: &mut BCob, mut target: Mat4, mix: u8, flag: u32) {
        if flag & TRANSLIKE_REMOVE_TARGET_SHEAR != 0 {
            target.orthogonalize_stable(1, false);
        }
        let own = cob.matrix;
        cob.matrix = match mix {
            TRANS_REPLACE => target,
            TRANS_BEFORE_FULL => target * own,
            TRANS_AFTER_FULL => own * target,
            TRANS_BEFORE => Self::aligned_scale(&target, &own),
            TRANS_AFTER => Self::aligned_scale(&own, &target),
            TRANS_BEFORE_SPLIT => Self::split_channels(&target, &own),
            TRANS_AFTER_SPLIT => Self::split_channels(&own, &target),
            _ => own,
        };
    }

    pub(super) fn copy_location(cob: &mut BCob, target: Mat4, flag: u32) {
        let offset = if flag & LOCLIKE_OFFSET != 0 { cob.matrix.loc() } else { Vec3::ZERO };
        let mut loc = cob.matrix.loc();
        for i in 0..3 {
            if flag & AXIS[i] != 0 {
                loc[i] = target.loc()[i];
                if flag & INVERT[i] != 0 {
                    loc[i] = -loc[i];
                }
                loc[i] += offset[i];
            }
        }
        cob.matrix.set_loc(loc);
    }

    pub(super) fn copy_rotation(cob: &mut BCob, target: Mat4, flag: u32, order: u8, mix: u8) {
        let (loc, oldrot, size) = cob.matrix.loc_rot_size();
        let order = BEulerOrder(if order != 0 { order as i16 } else { cob.rot_order });
        let obeul = order.of_mat4(&cob.matrix);
        let mut mat = target;
        mat.orthogonalize_stable(1, true);
        let mut eul = order.compatible_of_mat4(obeul, &mat);
        let legacy_offset = mix == ROT_OFFSET;
        let defeul = if matches!(mix, ROT_OFFSET | ROT_REPLACE) { obeul } else { Vec3::ZERO };
        for (i, axis) in ['X', 'Y', 'Z'].into_iter().enumerate() {
            if flag & AXIS[i] == 0 {
                eul[i] = defeul[i];
                continue;
            }
            if legacy_offset {
                eul = Self::rotate_euler(eul, order, axis, obeul[i]);
            }
            if flag & INVERT[i] != 0 {
                eul[i] = -eul[i];
            }
        }
        if mix == ROT_ADD {
            eul += obeul;
        }
        let eul = BEulerOrder::compatible(eul, obeul);
        let mut newrot = order.to_mat3(eul);
        match mix {
            ROT_BEFORE => newrot *= oldrot,
            ROT_AFTER => newrot = oldrot * newrot,
            _ => {}
        }
        cob.matrix = BCompose::loc_rot_size(loc, newrot, size);
    }

    /// `rotate_eulO`: `eul` turned `angle` about one axis, first.
    fn rotate_euler(eul: Vec3, order: BEulerOrder, axis: char, angle: f32) -> Vec3 {
        let mut e = Vec3::ZERO;
        e[(axis as u8 - b'X') as usize] = angle;
        let m: Mat3 = order.to_mat3(e) * order.to_mat3(eul);
        order.of_normalized(&m)
    }

    pub(super) fn copy_scale(cob: &mut BCob, target: Mat4, flag: u32, power: f32) {
        let obsize = cob.matrix.size();
        let mut size = if flag & SIZELIKE_UNIFORM != 0 {
            let all = AXIS[0] | AXIS[1] | AXIS[2];
            let total = if flag & all == all {
                target.volume().abs()
            } else {
                let s = target.size();
                (0..3).filter(|&i| flag & AXIS[i] != 0).map(|i| s[i]).product()
            };
            Vec3::splat(total.cbrt())
        } else {
            target.size()
        };
        size = Vec3::from_array(size.to_array().map(|v| v.powf(power)));
        if flag & SIZELIKE_OFFSET != 0 {
            size = if flag & SIZELIKE_MULTIPLY != 0 { size * obsize } else { size + obsize - Vec3::ONE };
        }
        for i in 0..3 {
            if flag & (AXIS[i] | SIZELIKE_UNIFORM) != 0 && obsize[i] != 0.0 {
                let col = cob.matrix.col_mut(i);
                *col = (col.truncate() * (size[i] / obsize[i])).extend(col.w);
            }
        }
    }
}
