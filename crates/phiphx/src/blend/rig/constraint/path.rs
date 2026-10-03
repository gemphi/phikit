//! The two constraints that put their owner on a curve's path: Follow Path
//! (`followpath_evaluate`) carries it to its place on the path, turned with
//! it when it follows, its own scale kept unless the curve's radius should
//! scale it; Clamp To (`clampto_evaluate`) moves it onto the path at the
//! fraction its own place makes along one axis of the curve's box.

use super::BCob;
use crate::blend::curve::BCurvePath;
use crate::blend::curve::path::FOLLOWPATH_RADIUS;
use crate::blend::rig::math::BMat4;
use glam::{Mat4, Vec3};

/// `bClampToConstraint.flag`: the axis measured, `CLAMPTO_AUTO` (the box's
/// longest) or `CLAMPTO_X` + 0, 1, 2; `flag2`: `CLAMPTO_CYCLIC`.
pub const CLAMPTO_AUTO: u32 = 0;
pub const CLAMPTO_CYCLIC: u32 = 1 << 0;

impl BCob {
    /// `target` the place on the path in world space
    /// (`followpath_get_tarmat`).
    pub(super) fn follow_path(cob: &mut BCob, target: Mat4, flag: u32) {
        let size = cob.matrix.size();
        cob.matrix = target * cob.matrix;
        if flag & FOLLOWPATH_RADIUS == 0 {
            let now = cob.matrix.size();
            for i in 0..3 {
                if now[i] != 0.0 {
                    let col = cob.matrix.col_mut(i);
                    *col = (col.truncate() * (size[i] / now[i])).extend(col.w);
                }
            }
        }
    }

    /// Clamp To onto `path`, the curve at `world`. The owner's place in
    /// the world is measured against the curve's box in the curve's own
    /// space, as Blender measures it; only the owner's place changes.
    pub(super) fn clamp_to(cob: &mut BCob, world: Mat4, path: &BCurvePath, axis: u32, cyclic: bool) {
        // `INIT_MINMAX` when the curve has no points.
        let (min, max) = path.bounds.unwrap_or((Vec3::splat(f32::MAX), Vec3::splat(f32::MIN)));
        let size = max - min;
        let a = match axis {
            CLAMPTO_AUTO if size.z > size.x && size.z > size.y => 2,
            CLAMPTO_AUTO if size.y > size.x && size.y > size.z => 1,
            CLAMPTO_AUTO => 0,
            n => (n as usize - 1).min(2),
        };
        let (lo, hi, at) = (min[a], max[a], cob.matrix.w_axis[a]);
        let len = hi - lo;
        let flat = len.abs() < f32::EPSILON;
        let time = if cyclic {
            // Past either end, the box is laid again end to end.
            match () {
                _ if flat => 0.0,
                _ if at < lo => (at - (lo - ((lo - at) / len).ceil() * len)) / len,
                _ if at > hi => (at - (hi + (((at - hi) / len) as i32) as f32 * len)) / len,
                _ => (at - lo) / len,
            }
        } else if at <= lo {
            0.0
        } else if at >= hi {
            1.0
        } else if !flat {
            (at - lo) / len
        } else {
            0.0
        };
        // No place on the path leaves the target matrix at the origin.
        let place = path.where_on(time).map_or(Vec3::ZERO, |p| world.transform_point3(p.vec));
        cob.matrix.w_axis = place.extend(cob.matrix.w_axis.w);
    }
}
