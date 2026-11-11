//! The bevel profile outlines are swept with (`BKE_curve_bevel_make`,
//! `curve_bevel.cc`): extrusion alone, a round bevel's full circle, the
//! general extrude-and-fill loop (`curve_bevel_make_extrude_and_fill`) for
//! the halves the fill mode keeps, its quarters round or from the custom
//! profile (`bevel_quarter_fill`), or the bevel object's outlines
//! (`curve_bevel_make_from_object`).

use super::displist::{TDispList, TDlKind, DL_BACK_CURVE, DL_FRONT_CURVE};
use super::{TCurveShape, CU_BACK, CU_FRONT};
use glam::Vec3;

/// `CU_BEV_MODE_OBJECT`, `CU_BEV_MODE_CURVE_PROFILE`.
pub const CU_BEV_MODE_OBJECT: u8 = 1;
pub const CU_BEV_MODE_CURVE_PROFILE: u8 = 2;

/// `BKE_curve_bevel_make`: the profile's lists, each of points (x unused,
/// y outward, z up) with its front / back flags, `DL_POLY` when it closes.
/// Empty without extrusion or bevel.
pub fn bevel_profile(cu: &TCurveShape) -> Vec<TDispList> {
    if cu.bevel_mode == CU_BEV_MODE_OBJECT {
        return cu.bevel_object.clone();
    }
    let (use_extrude, use_bevel) = (cu.extrude != 0.0, cu.bevel_radius != 0.0);
    if !use_extrude && !use_bevel {
        return Vec::new();
    }
    let p = |y: f32, z: f32| Vec3::new(0.0, y, z);
    let list = |kind: TDlKind, verts: Vec<Vec3>, flag: u32| vec![TDispList { kind, nr: verts.len(), parts: 1, flag, col: 0, charidx: 0, smooth: false, index: Vec::new(), verts }];
    if !use_bevel {
        // `curve_bevel_make_only_extrude`.
        return list(TDlKind::Segm, vec![p(0.0, -cu.extrude), p(0.0, cu.extrude)], DL_FRONT_CURVE | DL_BACK_CURVE);
    }
    let (front, back) = (cu.flag & CU_FRONT != 0, cu.flag & CU_BACK != 0);
    let r = cu.bevel_radius;
    let res = cu.bevresol;
    let samples = cu.profile.as_deref().filter(|s| cu.bevel_mode == CU_BEV_MODE_CURVE_PROFILE && s.len() > res);
    // `bevel_quarter_fill`: round, or the profile's samples (its first
    // point set by hand, as too few samples miss it).
    let (mut qx, mut qy) = (Vec::with_capacity(res + 1), Vec::with_capacity(res + 1));
    match samples {
        None => {
            let dangle = std::f32::consts::FRAC_PI_2 / (res + 1) as f32;
            let mut angle = 0.0f32;
            for _ in 0..res + 1 {
                qx.push(angle.cos() * r);
                qy.push(angle.sin() * r);
                angle += dangle;
            }
        }
        Some(s) => {
            qx.push(r);
            qy.push(0.0);
            for v in &s[1..res + 1] {
                qx.push(v.x * r);
                qy.push(v.y * r);
            }
        }
    }
    let fill = match (front, back) {
        (false, false) => TBevelFill::Full,
        (true, true) => TBevelFill::Half,
        (true, false) => TBevelFill::Front,
        (false, true) => TBevelFill::Back,
    };
    let full = fill == TBevelFill::Full;
    if !use_extrude && full && samples.is_none() {
        // `curve_bevel_make_full_circle`.
        let nr = 4 + 2 * res;
        let dangle = 2.0 * std::f32::consts::PI / nr as f32;
        let mut angle = -((nr - 1) as f32) * dangle;
        let verts = (0..nr)
            .map(|_| {
                let v = p(angle.cos() * r, angle.sin() * r - cu.extrude);
                angle += dangle;
                v
            })
            .collect();
        return list(TDlKind::Poly, verts, DL_BACK_CURVE);
    }
    // `curve_bevel_make_extrude_and_fill`.
    let one_side = matches!(fill, TBevelFill::Front | TBevelFill::Back);
    let mut v = Vec::new();
    if fill != TBevelFill::Front {
        v.push(p(0.0, -cu.extrude - r));
        for i in (0..=res).rev() {
            v.push(p(qx[i], -qy[i] - cu.extrude));
        }
    }
    if use_extrude && one_side {
        v.push(p(r, if fill == TBevelFill::Front { -cu.extrude } else { cu.extrude }));
    }
    if fill != TBevelFill::Back {
        // Don't duplicate the last back vertex.
        let start = if !use_extrude && !one_side { 1 } else { 0 };
        for i in start..res + 1 {
            v.push(p(qx[i], qy[i] + cu.extrude));
        }
        v.push(p(0.0, cu.extrude + r));
    }
    if full {
        for i in (1..=res).rev() {
            v.push(p(-qx[i], qy[i] + cu.extrude));
        }
        if use_extrude {
            v.push(p(-r, cu.extrude));
        }
        for i in 0..res + 1 {
            v.push(p(-qx[i], -qy[i] - cu.extrude));
        }
    }
    let flag = match fill {
        TBevelFill::Front => DL_FRONT_CURVE,
        TBevelFill::Back => DL_BACK_CURVE,
        _ => DL_FRONT_CURVE | DL_BACK_CURVE,
    };
    list(if full { TDlKind::Poly } else { TDlKind::Segm }, v, flag)
}

/// `curve_bevel_make_from_object` for one of the bevel object's display
/// lists (a `DL_POLY` or `DL_SEGM`, in its own space, scaled by its
/// object's `scale`): the outline turned into the profile plane.
pub fn from_object_list(dl: &TDispList, scale: [f32; 2]) -> TDispList {
    let mut out = dl.clone();
    if out.kind == TDlKind::Segm {
        out.flag |= DL_FRONT_CURVE | DL_BACK_CURVE;
    }
    for v in out.verts.iter_mut() {
        *v = Vec3::new(0.0, -v.x * scale[0], v.y * scale[1]);
    }
    out
}

/// `CurveBevelFillType`: which of the profile's halves are built.
#[derive(Clone, Copy, PartialEq, Eq)]
enum TBevelFill {
    Back,
    Front,
    Half,
    Full,
}
