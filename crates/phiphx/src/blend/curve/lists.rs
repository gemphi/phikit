//! A curve's bevel lists for its display list (`BKE_curve_bevelList_make`,
//! `curve.cc`, over every spline): STEP 1 each spline evaluated with tilt,
//! radius and weight ([`super::bevel`]); STEP 2 duplicate points dropped at
//! the last valid spline's resolution; STEP 3 the closed outlines numbered,
//! holes found and, for a flat curve, turned (`blend/text/bevlist`'s
//! `autohole`); STEP 4 a 3D curve's rotations by its twist method,
//! smoothed and tilted ([`super::orient`]), or a flat one's 2D cosines.

use super::bevel::BBevelList;
use super::{BCurve, CU_3D, CU_BEZIER, CU_NURBS, CU_POLY, CU_TWIST_TANGENT};
use crate::blend::rig::math::BQuat;
use crate::blend::text::bevlist::{autohole, cosines, TBevList, TBevPoint};

/// Every spline's bevel list, in spline order (an invalid spline's empty).
pub fn bevel_lists(curve: &BCurve) -> Vec<TBevList> {
    let is_3d = curve.flag & CU_3D != 0;
    let tangent = curve.twist_mode == CU_TWIST_TANGENT;
    // STEP 1 (`resolu` is left at the last valid spline's).
    let mut resolu = 1;
    let mut lists: Vec<BBevelList> = curve
        .nurbs
        .iter()
        .map(|nu| {
            if !nu.valid() {
                return BBevelList { points: Vec::new(), cyclic: nu.cyclic() };
            }
            resolu = nu.resolu.max(1);
            let points = match nu.kind {
                CU_POLY => BBevelList::poly(nu),
                CU_BEZIER => BBevelList::bezier(nu, is_3d, curve.do_radius, tangent),
                CU_NURBS if nu.pntsv == 1 => BBevelList::nurbs(nu, is_3d, curve.do_radius),
                _ => Vec::new(),
            };
            BBevelList { points, cyclic: nu.cyclic() }
        })
        .collect();
    // STEP 2.
    for list in lists.iter_mut() {
        list.drop_duplicates(resolu);
    }
    // STEP 4 for a 3D curve first: STEP 3 leaves its points as they are.
    if is_3d {
        let smooth = (resolu as f32 * curve.twist_smooth) as usize;
        for list in lists.iter_mut() {
            match list.points.len() {
                0 => {}
                1 => list.points[0].quat = BQuat::IDENTITY.0,
                2 => list.segment(),
                _ => list.orient(curve.twist_mode, smooth),
            }
        }
    }
    let mut bev: Vec<TBevList> = lists
        .into_iter()
        .map(|list| TBevList {
            points: list.points.iter().map(|p| TBevPoint { vec: p.vec, radius: p.radius, dir: p.dir, quat: p.quat, ..Default::default() }).collect(),
            poly: if list.cyclic { 0 } else { -1 },
            ..Default::default()
        })
        .collect();
    autohole(&mut bev, !is_3d);
    if !is_3d {
        cosines(&mut bev);
    }
    bev
}
