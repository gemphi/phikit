//! The text's display list (`evaluate_curve_type_object`, `displist.cc`):
//! with no extrusion, bevel or offset, each spline's evaluated outline
//! (`curve_to_displist`); otherwise the bevel profile (`BKE_curve_bevel_make`,
//! `curve_bevel.cc`) swept along every bevel list ([`super::sweep`]), and
//! then the flat fill (`curve_to_filledpoly`, `bevels_to_filledpoly`).

use super::bevlist::{bezier_points, TBevList};
use super::fill::triangulate;
use super::sweep::{sweep, TSplineInfo};
use super::{BText, TNurb, TSpline, CU_3D, CU_BACK, CU_FRONT};
use glam::{Vec2, Vec3};

/// `DispList.type`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TDlKind {
    Segm,
    Poly,
    Surf,
    Index3,
}

/// `DispList.flag`.
pub const DL_CYCL_U: u32 = 1 << 0;
pub const DL_CYCL_V: u32 = 1 << 1;
pub const DL_FRONT_CURVE: u32 = 1 << 2;
pub const DL_BACK_CURVE: u32 = 1 << 3;

/// `DispList`.
#[derive(Clone, Debug, PartialEq)]
pub struct TDispList {
    pub kind: TDlKind,
    pub verts: Vec<Vec3>,
    pub nr: usize,
    pub parts: usize,
    pub flag: u32,
    pub col: i16,
    pub charidx: i32,
    /// `rt & CU_SMOOTH`: text splines are always smooth.
    pub smooth: bool,
    /// `DL_INDEX3` triangles.
    pub index: Vec<[u32; 3]>,
}

impl TDispList {
    pub fn new(kind: TDlKind, verts: Vec<Vec3>, nr: usize, parts: usize, flag: u32, nu: &TSplineInfo) -> Self {
        Self { kind, verts, nr, parts, flag, col: nu.col, charidx: nu.charidx, smooth: nu.smooth, index: Vec::new() }
    }
}

/// `CU_DO_2DFILL`.
fn do_2dfill(flag: u32) -> bool {
    flag & CU_3D == 0 && flag & (CU_FRONT | CU_BACK) != 0
}

/// The display list of the text's splines and their bevel lists.
pub fn evaluate(cu: &BText, nurbs: &[TNurb], bev: &[TBevList]) -> Vec<TDispList> {
    let splines: Vec<TSplineInfo> = nurbs.iter().map(|nu| TSplineInfo { col: nu.mat_nr, charidx: nu.charidx, smooth: true }).collect();
    sweep(&cu.shape(), &splines, bev, || curve_to_displist(nurbs))
}

/// `curve_to_displist` for text splines (all cyclic).
fn curve_to_displist(nurbs: &[TNurb]) -> Vec<TDispList> {
    let mut disp = Vec::new();
    for nu in nurbs {
        let verts = match &nu.spline {
            TSpline::Bezier(b) if b.len() > 1 => bezier_points(b, nu.resolu),
            TSpline::Poly(p) if p.len() > 1 => p.clone(),
            _ => continue,
        };
        let kind = if verts.len() != 2 { TDlKind::Poly } else { TDlKind::Segm };
        let nr = verts.len();
        // `curve_to_displist` leaves `rt` unset: the fill is flat shaded.
        disp.push(TDispList::new(kind, verts, nr, 1, 0, &TSplineInfo { col: nu.mat_nr, charidx: nu.charidx, smooth: false }));
    }
    disp
}

/// `curve_to_filledpoly`: the caps of a swept flat curve, or its flat
/// outlines filled. New fills go to the head of the list (`BLI_addhead`).
pub fn curve_to_filledpoly(flag: u32, disp: &mut Vec<TDispList>) {
    if !do_2dfill(flag) {
        return;
    }
    let polys: Vec<TDispList> = match disp.first().map(|d| d.kind) {
        Some(TDlKind::Surf) => {
            // `bevels_to_filledpoly`: the profile's first point round every
            // back outline, its last round every front one.
            let mut front = Vec::new();
            let mut back = Vec::new();
            for dl in disp.iter().filter(|d| d.kind == TDlKind::Surf && d.flag & DL_CYCL_V != 0 && d.flag & DL_CYCL_U == 0) {
                let ring = |k: usize| (0..dl.parts).map(|i| dl.verts[i * dl.nr + k]).collect::<Vec<_>>();
                if flag & CU_BACK != 0 && dl.flag & DL_BACK_CURVE != 0 {
                    front.push(TDispList { kind: TDlKind::Poly, verts: ring(0), nr: dl.parts, parts: 1, flag: DL_BACK_CURVE, smooth: false, ..dl.clone() });
                }
                if flag & CU_FRONT != 0 && dl.flag & DL_FRONT_CURVE != 0 {
                    back.push(TDispList { kind: TDlKind::Poly, verts: ring(dl.nr - 1), nr: dl.parts, parts: 1, flag: DL_FRONT_CURVE, smooth: false, ..dl.clone() });
                }
            }
            let mut out = fill(&front, Z_DOWN, true);
            out.extend(fill(&back, Z_DOWN, false));
            out
        }
        _ => fill(disp, Z_DOWN, false),
    };
    for p in polys {
        disp.insert(0, p);
    }
}

/// The flat fills' projection normal (`z_up`, which points down).
pub const Z_DOWN: Vec3 = Vec3::new(0.0, 0.0, -1.0);

/// `BKE_displist_fill`: one fill per character and material, over its
/// `DL_POLY` outlines seen along `normal`; a triangle's index order is
/// turned for `flip`.
pub fn fill(disp: &[TDispList], normal: Vec3, flip: bool) -> Vec<TDispList> {
    let mut keys: Vec<(i32, i16)> = disp.iter().filter(|d| d.kind == TDlKind::Poly).map(|d| (d.charidx, d.col)).collect();
    keys.sort();
    keys.dedup();
    // Axes `u`, `v` with `u × v = -normal`: a triangle counter-clockwise
    // in them faces against `normal`. Along -Z they are X and Y.
    let axes = match normal == Z_DOWN {
        true => None,
        false => {
            let m = -normal.normalize_or_zero();
            let u = m.any_orthonormal_vector();
            Some((u, m.cross(u)))
        }
    };
    let flat = |p: Vec3| match axes {
        None => p.truncate(),
        Some((u, v)) => Vec2::new(p.dot(u), p.dot(v)),
    };
    let mut out = Vec::new();
    for (charidx, col) in keys {
        let group: Vec<&TDispList> = disp.iter().filter(|d| d.kind == TDlKind::Poly && d.charidx == charidx && d.col == col).collect();
        let rings: Vec<Vec<Vec2>> = group.iter().map(|d| d.verts.iter().map(|v| flat(*v)).collect()).collect();
        let tris = triangulate(&rings);
        if tris.is_empty() {
            continue;
        }
        let verts: Vec<Vec3> = group.iter().flat_map(|d| d.verts.iter().copied()).collect();
        // Mesh conversion reads (0, 2, 1): store clockwise for a face up.
        let index = tris.into_iter().map(|[a, b, c]| if flip { [a, b, c] } else { [a, c, b] }).collect();
        let flag = group.iter().fold(0, |f, d| f | d.flag) & (DL_BACK_CURVE | DL_FRONT_CURVE);
        let smooth = group.iter().any(|d| d.smooth);
        out.push(TDispList { kind: TDlKind::Index3, nr: verts.len(), parts: 0, verts, flag, col, charidx: 0, smooth, index });
    }
    out
}
