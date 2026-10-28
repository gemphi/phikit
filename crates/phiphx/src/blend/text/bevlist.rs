//! The text's bevel lists (`BKE_curve_bevelList_make`, `curve.cc`) for a
//! flat curve: every spline evaluated (a Bézier segment by forward
//! differencing at the curve's resolution, a straight one by its start
//! alone), duplicate points dropped, holes found (`bevelinside`) and each
//! outline turned so outlines and holes run opposite ways, then the 2D
//! bevel directions (`make_bevel_list_2D`, `calc_bevel_sin_cos`).
//!
//! STEP 3 ([`autohole`]) and the flat STEP 4 ([`cosines`]) serve curve
//! objects too, whose lists come from `blend/curve/bevel`.

use super::{BText, TNurb, TSpline, HD_VECT};
use crate::blend::rig::math::BVec3;
use glam::Vec3;

/// `BevPoint`, as much as a sweep reads.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TBevPoint {
    pub vec: Vec3,
    pub sina: f32,
    pub cosa: f32,
    /// The radius the profile is scaled by (a text's is 1).
    pub radius: f32,
    pub dir: Vec3,
    /// The 3D rotation (`make_bevel_list_3D`).
    pub quat: [f32; 4],
    pub(crate) dupe: bool,
}

/// `BevList`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TBevList {
    pub points: Vec<TBevPoint>,
    /// `-1` open, otherwise the cyclic outline's number (from 1).
    pub poly: i32,
    pub hole: bool,
    pub reversed: bool,
    pub charidx: i32,
}

/// `BKE_curve_forward_diff_bezier`: `it` points from `q0` (the end left out).
pub fn forward_diff(q0: f32, q1: f32, q2: f32, q3: f32, it: usize) -> impl Iterator<Item = f32> {
    let mut f = it as f32;
    let rt0 = q0;
    let rt1 = 3.0 * (q1 - q0) / f;
    f *= f;
    let rt2 = 3.0 * (q0 - 2.0 * q1 + q2) / f;
    f *= it as f32;
    let rt3 = (q3 - q0 + 3.0 * (q1 - q2)) / f;
    let (mut a, mut b, mut c, d) = (rt0, rt1 + rt2 + rt3, 2.0 * rt2 + 6.0 * rt3, 6.0 * rt3);
    (0..it).map(move |_| {
        let v = a;
        a += b;
        b += c;
        c += d;
        v
    })
}

/// One segment of a cyclic Bézier spline as its evaluated points: the
/// start alone between two vector handles, else `resolu` steps.
pub fn bezier_points(bezt: &[super::TBez], resolu: usize) -> Vec<Vec3> {
    let n = bezt.len();
    let mut out = Vec::with_capacity(n * resolu);
    for i in 0..n {
        let (prev, b) = (&bezt[(i + n - 1) % n], &bezt[i]);
        if prev.h2 == HD_VECT && b.h1 == HD_VECT {
            out.push(prev.vec[1]);
            continue;
        }
        let [xs, ys, zs] = [0, 1, 2].map(|k| forward_diff(prev.vec[1][k], prev.vec[2][k], b.vec[0][k], b.vec[1][k], resolu).collect::<Vec<_>>());
        out.extend((0..resolu).map(|j| Vec3::new(xs[j], ys[j], zs[j])));
    }
    out
}

/// Every spline's bevel list, in spline order.
pub fn bevel_lists(cu: &BText, nurbs: &[TNurb]) -> Vec<TBevList> {
    let mut bev: Vec<TBevList> = nurbs.iter().map(step1).collect();
    // STEP 2: double points (the threshold of the last spline's resolution).
    let resolu = nurbs.last().map_or(1, |n| n.resolu.max(1));
    let limit = 0.00001 / resolu as f32;
    for bl in bev.iter_mut().filter(|bl| !bl.points.is_empty()) {
        let n = bl.points.len();
        let pairs: Vec<(usize, usize)> = match bl.poly != -1 {
            true => std::iter::once((n - 1, 0)).chain((0..n.saturating_sub(2)).map(|i| (i, i + 1))).collect(),
            false => (0..n - 1).map(|i| (i, i + 1)).collect(),
        };
        for (a, b) in pairs {
            if (bl.points[a].vec - bl.points[b].vec).abs().max_element() <= limit {
                bl.points[a].dupe = true;
            }
        }
        bl.points.retain(|p| !p.dupe);
    }
    autohole(&mut bev, cu.flag & super::CU_3D == 0);
    cosines(&mut bev);
    bev
}

/// STEP 4 for a flat curve: the 2D cosines of every list.
pub fn cosines(bev: &mut [TBevList]) {
    for bl in bev.iter_mut() {
        match bl.points.len() {
            0 | 1 => {}
            // `make_bevel_list_segment_2D`, its direction from `_3D`.
            2 => {
                let d = bl.points[1].vec - bl.points[0].vec;
                let (s, c) = sin_cos(d.x, d.y, -d.x, -d.y);
                for p in bl.points.iter_mut() {
                    (p.sina, p.cosa, p.dir) = (s, c, d.normalize_or_zero());
                }
            }
            _ => flat(bl),
        }
    }
}

/// STEP 1 for one spline.
fn step1(nu: &TNurb) -> TBevList {
    let points: Vec<Vec3> = match &nu.spline {
        TSpline::Bezier(b) if b.len() > 1 => bezier_points(b, nu.resolu),
        TSpline::Poly(p) if p.len() > 1 => p.clone(),
        _ => Vec::new(),
    };
    TBevList { points: points.into_iter().map(|vec| TBevPoint { vec, radius: 1.0, ..Default::default() }).collect(), poly: 0, charidx: nu.charidx, ..Default::default() }
}

/// STEP 3: number the outlines, find the holes and, for a flat curve,
/// turn the outlines.
pub fn autohole(bev: &mut [TBevList], is_2d: bool) {
    let mut poly = 0;
    let mut sort: Vec<(usize, f32, bool)> = Vec::new();
    for (k, bl) in bev.iter_mut().enumerate() {
        if bl.points.is_empty() || bl.poly < 0 {
            continue;
        }
        poly += 1;
        bl.poly = poly;
        bl.hole = false;
        // The extreme left point and the turning direction there.
        let p = &bl.points;
        let n = p.len();
        let left = (0..n).fold(0, |m, i| if p[m].vec.x > p[i].vec.x { i } else { m });
        let (b0, b1, b2) = (p[(left + n - 1) % n].vec, p[left].vec, p[(left + 1) % n].vec);
        let inp = (b1.x - b0.x) * (b0.y - b2.y) + (b0.y - b1.y) * (b0.x - b2.x);
        sort.push((k, p[left].vec.x, inp > 0.0));
    }
    sort.sort_by(|a, b| a.1.total_cmp(&b.1));
    for a in 1..sort.len() {
        let k = sort[a].0;
        for b in (0..a).rev() {
            let outer = sort[b].0;
            if bev[outer].charidx == bev[k].charidx && inside(&bev[outer], &bev[k]) {
                bev[k].hole = !bev[outer].hole;
                break;
            }
        }
    }
    if is_2d {
        for &(k, _, dir) in &sort {
            if bev[k].hole == dir {
                bev[k].reversed = true;
                bev[k].points.reverse();
            }
        }
    }
}

/// `bevelinside`: whether `bl2`'s first point is inside `bl1`, counting
/// edge crossings of a horizontal line to either side.
fn inside(bl1: &TBevList, bl2: &TBevList) -> bool {
    let h = bl2.points[0].vec;
    let (mut left, mut right) = (0, 0);
    let p = &bl1.points;
    let n = p.len();
    for i in 0..n {
        let (a, b) = (p[(i + n - 1) % n].vec, p[i].vec);
        let (min, max) = (a.y.min(b.y), a.y.max(b.y));
        if min == max || !(min <= h.y && max >= h.y) {
            continue;
        }
        // `cu_isectLL` with the line from `h` to `h + (1000, 0)`.
        let h2 = h + Vec3::new(1000.0, 0.0, 0.0);
        let deler = (a.x - b.x) * (h.y - h2.y) - (h.x - h2.x) * (a.y - b.y);
        if deler == 0.0 {
            continue;
        }
        let lab = -(((a.y - h.y) * (h.x - h2.x) - (a.x - h.x) * (h.y - h2.y)) / deler);
        let x = lab * (b.x - a.x) + a.x;
        if lab != 0.0 {
            match x < h.x {
                true => left += 1,
                false => right += 1,
            }
        }
    }
    left & 1 == 1 && right & 1 == 1
}

/// `make_bevel_list_2D`: each point's cosines and bisected direction (a
/// cap's normal); an open list's ends by their own directions.
fn flat(bl: &mut TBevList) {
    let n = bl.points.len();
    let triples: Vec<(usize, usize, usize)> = match bl.poly != -1 {
        true => (0..n).map(|i| ((i + n - 1) % n, i, (i + 1) % n)).collect(),
        false => (1..n - 1).map(|i| (i - 1, i, i + 1)).collect(),
    };
    for (a, i, c) in triples {
        let (p0, p1, p2) = (bl.points[a].vec, bl.points[i].vec, bl.points[c].vec);
        (bl.points[i].sina, bl.points[i].cosa) = sin_cos(p1.x - p0.x, p1.y - p0.y, p1.x - p2.x, p1.y - p2.y);
        bl.points[i].dir = p1.bisect(p0, p2);
    }
    if bl.poly == -1 {
        for i in [0, n - 1] {
            let d = bl.points[i].dir;
            let angle = d.x.atan2(d.y) - std::f32::consts::FRAC_PI_2;
            (bl.points[i].sina, bl.points[i].cosa) = (angle.sin(), angle.cos());
        }
    }
}

/// `calc_bevel_sin_cos`: the bisector's direction, lengthened so an
/// offset along it keeps both edges at the same distance.
fn sin_cos(x1: f32, y1: f32, x2: f32, y2: f32) -> (f32, f32) {
    let t01 = match (x1 * x1 + y1 * y1).sqrt() {
        0.0 => 1.0,
        t => t,
    };
    let t02 = match (x2 * x2 + y2 * y2).sqrt() {
        0.0 => 1.0,
        t => t,
    };
    let (x1, y1, x2, y2) = (x1 / t01, y1 / t01, x2 / t02, y2 / t02);
    let dot = x1 * x2 + y1 * y2;
    let half = if dot.abs() >= 1.0 { std::f32::consts::FRAC_PI_2 } else { dot.clamp(-1.0, 1.0).acos() / 2.0 };
    let s = match half.sin() {
        0.0 => 1.0,
        s => s,
    };
    let (mut x3, mut y3) = (x1 - x2, y1 - y2);
    if x3 == 0.0 && y3 == 0.0 {
        (x3, y3) = (y1, -x1);
    } else {
        let t = (x3 * x3 + y3 * y3).sqrt();
        (x3, y3) = (x3 / t, y3 / t);
    }
    (-y3 / s, x3 / s)
}
