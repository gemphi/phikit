//! The sweep of a text's or a curve's display list
//! (`evaluate_curve_type_object`, `displist.cc`): with no bevel profile and
//! no width correction the splines' own outlines; else every bevel list
//! with its points moved along their bisectors (width correction alone),
//! or each profile list rotated to every point of it (`rotateBevelPiece`)
//! and scaled by the point's radius or the taper (`displist_calc_taper`),
//! with an open spline's ends capped (`fillBevelCap`); then the flat fill
//! (`curve_to_filledpoly`).
//!
//! The bevel factors are read as Blender's default mapping: every point,
//! no blending at the ends (`calc_bevfac_mapping_default`).

use super::bevlist::{TBevList, TBevPoint};
use super::displist::{curve_to_filledpoly, fill, TDispList, TDlKind, DL_BACK_CURVE, DL_CYCL_U, DL_CYCL_V, DL_FRONT_CURVE};
use super::profile::bevel_profile;
use super::{TCurveShape, CU_3D, CU_FILL_CAPS};
use crate::blend::rig::math::BQuat;
use glam::Vec3;

/// What a spline (`Nurb`) gives its display lists: `mat_nr`, `charidx`
/// and `flag & CU_SMOOTH`.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TSplineInfo {
    pub col: i16,
    pub charidx: i32,
    pub smooth: bool,
}

/// `evaluate_curve_type_object` from the bevel lists on: `plain` is the
/// `curve_to_displist` of the splines, for a curve with neither profile
/// nor width correction.
pub fn sweep(cu: &TCurveShape, splines: &[TSplineInfo], bev: &[TBevList], plain: impl FnOnce() -> Vec<TDispList>) -> Vec<TDispList> {
    let dlbev = bevel_profile(cu);
    let mut disp = Vec::new();
    match (dlbev.is_empty(), cu.offset == 1.0) {
        (true, true) => disp = plain(),
        _ => {
            let widfac = cu.offset - 1.0;
            for (bl, nu) in bev.iter().zip(splines) {
                let n = bl.points.len();
                if n == 0 {
                    continue;
                }
                if dlbev.is_empty() {
                    // Width correction alone: the outline moved along its bisectors.
                    let verts = bl.points.iter().map(|p| p.vec + Vec3::new(widfac * p.sina, widfac * p.cosa, 0.0)).collect();
                    let (kind, flag) = match bl.poly != -1 {
                        true => (TDlKind::Poly, 0),
                        false => (TDlKind::Segm, DL_FRONT_CURVE | DL_BACK_CURVE),
                    };
                    disp.push(TDispList::new(kind, verts, n, 1, flag, nu));
                    continue;
                }
                sweep_list(cu, bl, nu, &dlbev, widfac, &mut disp);
            }
        }
    }
    curve_to_filledpoly(cu.flag, &mut disp);
    disp
}

/// One bevel list swept by every profile list, then its caps filled.
fn sweep_list(cu: &TCurveShape, bl: &TBevList, nu: &TSplineInfo, dlbev: &[TDispList], widfac: f32, disp: &mut Vec<TDispList>) {
    let steps = bl.points.len();
    let caps = cu.flag & CU_FILL_CAPS != 0 && bl.poly == -1;
    let (mut bottom, mut top) = (Vec::new(), Vec::new());
    let (mut bottom_no, mut top_no) = (Vec3::ZERO, Vec3::ZERO);
    let cap = |verts: &[Vec3]| TDispList::new(TDlKind::Poly, verts.to_vec(), verts.len(), 1, 0, nu);
    for dlb in dlbev {
        let nr = dlb.verts.len();
        let mut verts = Vec::with_capacity(nr * steps);
        for (a, p) in bl.points.iter().enumerate() {
            let at = verts.len();
            rotate_piece(cu, p, dlb, widfac, radius_factor(cu, p, a, steps), &mut verts);
            if caps && a == 1 && p.dir.length_squared() > 0.0 {
                bottom.push(cap(&verts[at - nr..at]));
                bottom_no = p.dir;
            }
            if caps && a == steps - 1 && p.dir.length_squared() > 0.0 {
                top.push(cap(&verts[at..]));
                top_no = -p.dir;
            }
        }
        let mut flag = dlb.flag & (DL_FRONT_CURVE | DL_BACK_CURVE);
        if dlb.kind == TDlKind::Poly {
            flag |= DL_CYCL_U;
        }
        if bl.poly >= 0 && steps > 2 {
            flag |= DL_CYCL_V;
        }
        disp.push(TDispList::new(TDlKind::Surf, verts, nr, steps, flag, nu));
    }
    for (caps, no) in [(bottom, bottom_no), (top, top_no)] {
        for f in fill(&caps, no, false) {
            disp.insert(0, f);
        }
    }
}

/// The profile's scale at point `i` of `n`: its radius, or the taper
/// there (`BKE_displist_calc_taper`) as the radius mode combines them.
fn radius_factor(cu: &TCurveShape, p: &TBevPoint, i: usize, n: usize) -> f32 {
    let Some(taper) = &cu.taper else { return p.radius };
    let r = calc_taper(taper, i as f32 / (n - 1) as f32);
    match cu.taper_radius_mode {
        1 => r * p.radius,
        2 => r + p.radius,
        _ => r,
    }
}

/// `displist_calc_taper`: the taper curve's height where its run from the
/// first point to the last reaches `fac`, between its points.
fn calc_taper(dl: &[Vec3], fac: f32) -> f32 {
    let Some(first) = dl.first() else { return 1.0 };
    let minx = first.x;
    let dx = dl[dl.len() - 1].x - minx;
    if !(dx > 0.0) {
        return 1.0;
    }
    for (a, fp) in dl.iter().enumerate() {
        if (fp.x - minx) / dx >= fac {
            if a > 0 {
                let prev = dl[a - 1];
                let (fac1, fac2) = ((prev.x - minx) / dx, (fp.x - minx) / dx);
                if fac1 != fac2 {
                    return fp.y * (fac1 - fac) / (fac1 - fac2) + prev.y * (fac - fac2) / (fac1 - fac2);
                }
            }
            return fp.y;
        }
    }
    dl[dl.len() - 1].y
}

/// `rotateBevelPiece` with no blending: the profile placed at `p`, by its
/// rotation for a 3D curve, by its 2D cosines for a flat one.
fn rotate_piece(cu: &TCurveShape, p: &TBevPoint, dlb: &TDispList, widfac: f32, rf: f32, out: &mut Vec<Vec3>) {
    for fp in &dlb.verts {
        match cu.flag & CU_3D != 0 {
            true => {
                let v = BQuat(p.quat).rotate(Vec3::new(fp.y + widfac, fp.z, 0.0));
                out.push(p.vec + Vec3::new(rf * v.x, rf * v.y, rf * v.z));
            }
            false => out.push(p.vec + Vec3::new(rf * (widfac + fp.y) * p.sina, rf * (widfac + fp.y) * p.cosa, rf * fp.z)),
        }
    }
}
