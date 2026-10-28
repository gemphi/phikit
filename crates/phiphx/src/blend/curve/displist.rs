//! A curve object's display list (`evaluate_curve_type_object`,
//! `displist.cc`): its bevel lists ([`super::lists`]) swept by the bevel
//! profile, or, with neither profile nor width correction, every valid
//! spline's evaluated outline (`curve_to_displist`); then the flat fill.
//! The sweep, the profile and the fill are the text pipeline's
//! (`blend/text/sweep`), which is Blender's one pipeline for both.

use super::bevel::BBevelList;
use super::lists::bevel_lists;
use super::{BCurve, CU_BEZIER, CU_NURBS, CU_POLY};
use crate::blend::text::bevlist::forward_diff;
use crate::blend::text::displist::{TDispList, TDlKind};
use crate::blend::text::sweep::{sweep, TSplineInfo};
use crate::blend::text::{TCurveShape, HD_VECT};
use glam::Vec3;

/// The display list of `curve` with its shape and its splines' materials.
pub fn curve_displist(curve: &BCurve, shape: &TCurveShape, splines: &[TSplineInfo]) -> Vec<TDispList> {
    let bev = bevel_lists(curve);
    sweep(shape, splines, &bev, || curve_to_displist(curve, splines))
}

/// `curve_to_displist`: a Bézier spline sampled `resolu` times a segment
/// (once between two vector handles), its last point closing an open one;
/// a NURBS by its basis (`BKE_nurb_makeCurve`); a poly as it is. Cyclic
/// lists are `DL_POLY` (but two points make a segment).
pub fn curve_to_displist(curve: &BCurve, splines: &[TSplineInfo]) -> Vec<TDispList> {
    let mut disp = Vec::new();
    for (nu, info) in curve.nurbs.iter().zip(splines) {
        if !nu.valid() {
            continue;
        }
        let cyclic = nu.cyclic();
        let verts: Vec<Vec3> = match nu.kind {
            CU_BEZIER => {
                let (b, n, res) = (&nu.bezt, nu.pntsu, nu.resolu);
                let mut v = Vec::new();
                let mut segment = |prev: &super::BBezPoint, bezt: &super::BBezPoint| match prev.h2 == HD_VECT && bezt.h1 == HD_VECT {
                    true => v.push(prev.vec[1]),
                    false => {
                        let [xs, ys, zs] = [0, 1, 2].map(|k| forward_diff(prev.vec[1][k], prev.vec[2][k], bezt.vec[0][k], bezt.vec[1][k], res).collect::<Vec<_>>());
                        v.extend((0..res).map(|j| Vec3::new(xs[j], ys[j], zs[j])));
                    }
                };
                for i in 1..n {
                    segment(&b[i - 1], &b[i]);
                }
                match cyclic {
                    true => segment(&b[n - 1], &b[0]),
                    false => v.push(b[n - 1].vec[1]),
                }
                v
            }
            CU_NURBS => BBevelList::nurbs(nu, false, false).iter().map(|p| p.vec).collect(),
            CU_POLY => nu.bp.iter().take(nu.pntsu).map(|p| p.vec.truncate()).collect(),
            _ => continue,
        };
        let kind = match (cyclic, verts.len() != 2 || nu.kind == CU_NURBS) {
            (true, true) => TDlKind::Poly,
            _ => TDlKind::Segm,
        };
        let nr = verts.len();
        // `curve_to_displist` leaves `rt` unset: flat shaded.
        disp.push(TDispList::new(kind, verts, nr, 1, 0, &TSplineInfo { smooth: false, ..*info }));
    }
    disp
}
