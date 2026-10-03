//! Child Of, Floor and Pivot (`childof_evaluate`, `minmax_evaluate`,
//! `pivotcon_evaluate`).

use super::BCob;
use crate::blend::rig::math::{BCompose, BEulerOrder, BMat3, BMat4, BQuat};
use glam::{Mat3, Mat4, Vec3};

/// `CHILDOF_LOC[XYZ]`, `_ROT[XYZ]`, `_SIZE[XYZ]`, `CHILDOF_ALL`,
/// `CHILDOF_SET_INVERSE`.
const CHILDOF_LOC: [u32; 3] = [1 << 0, 1 << 1, 1 << 2];
const CHILDOF_ROT: [u32; 3] = [1 << 3, 1 << 4, 1 << 5];
const CHILDOF_SIZE: [u32; 3] = [1 << 6, 1 << 7, 1 << 8];
const CHILDOF_ALL: u32 = 511;
const CHILDOF_SET_INVERSE: u32 = 1 << 9;
/// `MINMAX_USEROT`.
const MINMAX_USEROT: u32 = 1 << 2;
/// `PIVOTCON_AXIS_NONE`, `PIVOTCON_AXIS_X`, `PIVOTCON_FLAG_OFFSET_ABS`.
const PIVOT_AXIS_NONE: i32 = -1;
const PIVOT_AXIS_X: i32 = 3;
const PIVOT_OFFSET_ABS: u32 = 1 << 0;

impl BCob {
    /// `childof_evaluate`: the owner parented to the target through the
    /// inverse matrix, only the channels `flag` keeps.
    pub(super) fn child_of(cob: &mut BCob, target: Mat4, target_order: i16, flag: u32, invmat: Mat4, base: Mat4) {
        let (parmat, mut inverse) = if flag & CHILDOF_ALL == CHILDOF_ALL {
            (target, invmat)
        } else {
            let (to, oo) = (BEulerOrder(target_order), BEulerOrder(cob.rot_order));
            let (mut loc, mut eul, mut size) = (target.loc(), to.of_mat4(&target), target.size());
            let (mut loco, mut eulo, mut sizeo) = (invmat.loc(), oo.of_mat4(&invmat), invmat.size());
            for i in 0..3 {
                if flag & CHILDOF_LOC[i] == 0 {
                    (loc[i], loco[i]) = (0.0, 0.0);
                }
                if flag & CHILDOF_ROT[i] == 0 {
                    (eul[i], eulo[i]) = (0.0, 0.0);
                }
                if flag & CHILDOF_SIZE[i] == 0 {
                    (size[i], sizeo[i]) = (1.0, 1.0);
                }
            }
            (BCompose::loc_eul_size(loc, eul, size, to), BCompose::loc_eul_size(loco, eulo, sizeo, oo))
        };
        // A Set Inverse still pending when the file was saved: the inverse
        // of the parent now, times `base` — the rig's world for a bone,
        // the identity for an object.
        if flag & CHILDOF_SET_INVERSE != 0 {
            inverse = parmat.inverse() * base;
        }
        let own = cob.matrix;
        cob.matrix = parmat * inverse * own;
        for i in 0..3 {
            if flag & CHILDOF_LOC[i] == 0 {
                cob.matrix.w_axis[i] = own.w_axis[i];
            }
        }
    }

    /// `minmax_evaluate`: kept on the far side of the target's plane along
    /// `axis` (`TRACK_*`), `offset` off it; in the target's own axes with
    /// `MINMAX_USEROT`.
    pub(super) fn floor(cob: &mut BCob, target: Mat4, axis: u32, offset: f32, flag: u32) {
        let (mut obmat, tarmat) = match flag & MINMAX_USEROT != 0 {
            true => (target.inverse() * cob.matrix, Mat4::IDENTITY),
            false => (cob.matrix, target),
        };
        let index = (axis % 3) as usize;
        let (t, o) = (tarmat.w_axis[index], obmat.w_axis[index] - offset);
        let (val1, val2) = match axis {
            0..=2 => (t, o),
            3..=5 => (o, t),
            _ => return,
        };
        if val1 > val2 {
            obmat.w_axis[index] = tarmat.w_axis[index] + offset;
            if flag & MINMAX_USEROT != 0 {
                cob.matrix = target * obmat;
            } else {
                cob.matrix.set_loc(obmat.loc());
            }
        }
    }

    /// `pivotcon_evaluate`: the owner's rotation taken about the pivot (the
    /// target plus `offset`, else the owner plus it, or `offset` itself
    /// with `PIVOTCON_FLAG_OFFSET_ABS`), only while the owner's Euler angle
    /// on `axis` has the sign asked for.
    pub(super) fn pivot(cob: &mut BCob, target: Option<Mat4>, offset: Vec3, axis: i32, flag: u32) {
        let axis = axis.clamp(PIVOT_AXIS_NONE, 5);
        if axis != PIVOT_AXIS_NONE {
            let rot = BEulerOrder(cob.rot_order).of_mat4(&cob.matrix);
            let skip = if axis < PIVOT_AXIS_X { rot[axis as usize] > 0.0 } else { rot[(axis - PIVOT_AXIS_X) as usize] < 0.0 };
            if skip {
                return;
            }
        }
        let mut pivot = match target {
            Some(t) => t.loc() + offset,
            None if flag & PIVOT_OFFSET_ABS == 0 => cob.matrix.loc() + offset,
            None => offset,
        };
        let rot = Mat3::from_mat4(cob.matrix).normalized_cols();
        // `mat3_normalized_to_axis_angle`: the pivot moved along the axis so
        // it does not translate the owner.
        let (axis_v, angle) = BQuat::of_normalized_fast(&rot).axis_angle();
        let own = cob.matrix.loc();
        if angle != 0.0 {
            let vec = pivot - own;
            pivot -= axis_v * (vec.dot(axis_v) / axis_v.dot(axis_v));
        }
        cob.matrix.set_loc(pivot + rot * (own - pivot));
    }
}
