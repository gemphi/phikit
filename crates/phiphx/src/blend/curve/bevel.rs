//! A spline's bevel list (`BKE_curve_bevelList_make` for one spline): its
//! evaluated points with tilt, radius and weight, duplicates dropped, and
//! each point's direction and rotation (`make_bevel_list_3D_*`,
//! `make_bevel_list_2D`).

use super::{BCurve, BNurb, CU_3D, CU_BEZIER, CU_NURBS, CU_POLY, CU_TWIST_TANGENT};
use crate::blend::rig::math::{BQuat, BVec3};
use glam::Vec3;

/// `HD_VECT`.
const HD_VECT: u8 = 2;
/// `KEY_LINEAR`, `KEY_CARDINAL`, `KEY_BSPLINE`, `KEY_CATMULL_ROM`,
/// `KEY_CU_EASE`.
pub const KEY_LINEAR: i16 = 0;
pub const KEY_CARDINAL: i16 = 1;
pub const KEY_BSPLINE: i16 = 2;
const KEY_CATMULL_ROM: i16 = 3;
const KEY_CU_EASE: i16 = 3;

/// One evaluated point (`BevPoint`).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct BBevelPoint {
    pub vec: Vec3,
    pub tilt: f32,
    pub radius: f32,
    pub weight: f32,
    pub dir: Vec3,
    pub tan: Vec3,
    pub quat: [f32; 4],
    dupe: bool,
}

/// One spline's bevel list (`BevList`).
#[derive(Clone, Debug, PartialEq)]
pub struct BBevelList {
    pub points: Vec<BBevelPoint>,
    pub cyclic: bool,
}

/// `key_curve_position_weights`.
pub fn position_weights(t: f32, kind: i16) -> [f32; 4] {
    let (t2, t3) = (t * t, t * t * t);
    let hermite = |fc: f32| [-fc * t3 + 2.0 * fc * t2 - fc * t, (2.0 - fc) * t3 + (fc - 3.0) * t2 + 1.0, (fc - 2.0) * t3 + (3.0 - 2.0 * fc) * t2 + fc * t, fc * t3 - fc * t2];
    match kind {
        KEY_CARDINAL => hermite(0.71),
        KEY_BSPLINE => [-0.16666666 * t3 + 0.5 * t2 - 0.5 * t + 0.16666666, 0.5 * t3 - t2 + 0.66666666, -0.5 * t3 + 0.5 * t2 + 0.5 * t + 0.16666666, 0.16666666 * t3],
        KEY_CATMULL_ROM => hermite(0.5),
        _ => [0.0, 1.0 - t, t, 0.0],
    }
}

impl BBevelList {
    /// The bevel list of `curve`'s first spline; `None` when it has no
    /// points (an invalid spline makes an empty list, which no path reads).
    pub fn first(curve: &BCurve) -> Option<Self> {
        let nu = curve.nurbs.first()?;
        if !nu.valid() {
            return None;
        }
        let points = match nu.kind {
            CU_POLY => Self::poly(nu),
            CU_BEZIER => Self::bezier(nu, curve.flag & CU_3D != 0, curve.do_radius, curve.twist_mode == CU_TWIST_TANGENT),
            CU_NURBS if nu.pntsv == 1 => Self::nurbs(nu, curve.flag & CU_3D != 0, curve.do_radius),
            _ => return None,
        };
        let mut list = Self { points, cyclic: nu.cyclic() };
        list.drop_duplicates(nu.resolu);
        if list.points.is_empty() {
            return None;
        }
        if curve.flag & CU_3D == 0 {
            list.reverse_2d();
        }
        match (list.points.len(), curve.flag & CU_3D != 0) {
            (1, _) => list.points[0].quat = BQuat::IDENTITY.0,
            (2, _) => list.segment(),
            (_, false) => list.flat(),
            (_, true) => list.orient(curve.twist_mode, (nu.resolu as f32 * curve.twist_smooth) as usize),
        }
        Some(list)
    }

    fn poly(nu: &BNurb) -> Vec<BBevelPoint> {
        let mut out: Vec<BBevelPoint> =
            nu.bp.iter().take(nu.pntsu).map(|bp| BBevelPoint { vec: bp.vec.truncate(), tilt: bp.tilt, radius: bp.radius, weight: bp.weight, ..Default::default() }).collect();
        if !nu.cyclic() {
            Self::ends_from_points(nu, &mut out);
        }
        out
    }

    /// `bevlist_firstlast_direction_calc_from_bpoint`.
    fn ends_from_points(nu: &BNurb, out: &mut [BBevelPoint]) {
        if nu.pntsu > 1 && out.len() > 1 {
            let bp = &nu.bp;
            let n = out.len();
            out[0].dir = (bp[1].vec - bp[0].vec).truncate().normalize_or_zero();
            out[n - 1].dir = (bp[nu.pntsu - 1].vec - bp[nu.pntsu - 2].vec).truncate().normalize_or_zero();
        }
    }

    fn bezier(nu: &BNurb, do_tilt: bool, do_radius: bool, tangent: bool) -> Vec<BBevelPoint> {
        let bezt = &nu.bezt;
        let resolu = nu.resolu;
        let mut out = Vec::new();
        let (mut prev, mut next, count) = if nu.cyclic() { (nu.pntsu - 1, 0, nu.pntsu) } else { (0, 1, nu.pntsu - 1) };
        let first_dir = (bezt[prev].vec[2] - bezt[prev].vec[1]).normalize_or_zero();
        // Blender writes a segment's `resolu + 1` tangents into the list, the
        // last into the next slot: the next segment overwrites it, but a
        // straight segment's point and the open end keep it.
        let mut carry = Vec3::ZERO;
        for _ in 0..count {
            let (a, b) = (&bezt[prev], &bezt[next]);
            if a.h2 == HD_VECT && b.h1 == HD_VECT {
                out.push(BBevelPoint { vec: a.vec[1], tilt: a.tilt, radius: a.radius, weight: a.weight, tan: carry, ..Default::default() });
                carry = Vec3::ZERO;
            } else {
                let start = out.len();
                let axis = |j: usize| Self::forward_diff(a.vec[1][j], a.vec[2][j], b.vec[0][j], b.vec[1][j], resolu);
                let (x, y, z) = (axis(0), axis(1), axis(2));
                let tan = tangent.then(|| Self::cotangents(a, b, resolu));
                for i in 0..resolu {
                    let mut p = BBevelPoint { vec: Vec3::new(x[i], y[i], z[i]), ..Default::default() };
                    if let Some(t) = &tan {
                        p.tan = t[i];
                    }
                    out.push(p);
                }
                carry = tan.map_or(Vec3::ZERO, |t| t[resolu]);
                Self::tilt_bezpart(nu, prev, next, &mut out[start..], do_tilt, do_radius);
            }
            prev = next;
            next += 1;
        }
        if let Some(p) = out.first_mut() {
            p.dir = first_dir;
        }
        if !nu.cyclic() {
            let last = &bezt[prev];
            out.push(BBevelPoint { vec: last.vec[1], tilt: last.tilt, radius: last.radius, weight: last.weight, dir: (last.vec[1] - last.vec[0]).normalize_or_zero(), tan: carry, ..Default::default() });
        }
        out
    }

    /// `BKE_curve_forward_diff_bezier`: `it + 1` points of one coordinate.
    fn forward_diff(q0: f32, q1: f32, q2: f32, q3: f32, it: usize) -> Vec<f32> {
        let mut f = it as f32;
        let rt0 = q0;
        let rt1 = 3.0 * (q1 - q0) / f;
        f *= f;
        let rt2 = 3.0 * (q0 - 2.0 * q1 + q2) / f;
        f *= it as f32;
        let rt3 = (q3 - q0 + 3.0 * (q1 - q2)) / f;
        let (mut a, mut b, mut c, d) = (rt0, rt1 + rt2 + rt3, 2.0 * rt2 + 6.0 * rt3, 6.0 * rt3);
        (0..=it)
            .map(|_| {
                let v = a;
                a += b;
                b += c;
                c += d;
                v
            })
            .collect()
    }

    /// `forward_diff_bezier_cotangent`: the second derivative, normalised,
    /// at each of `it + 1` steps (the tangent twist method's reference).
    fn cotangents(a: &super::BBezPoint, b: &super::BBezPoint, it: usize) -> Vec<Vec3> {
        let (p0, p1, p2, p3) = (a.vec[1], a.vec[2], b.vec[0], b.vec[1]);
        (0..=it)
            .map(|i| {
                let t = i as f32 / it as f32;
                let mut p = Vec3::from_array(std::array::from_fn(|k| {
                    (-6.0 * t + 6.0) * p0[k] + (18.0 * t - 12.0) * p1[k] + (-18.0 * t + 6.0) * p2[k] + (6.0 * t) * p3[k]
                }));
                p.normalize_len();
                p
            })
            .collect()
    }

    /// `tilt_bezpart`: tilt, radius and weight along one Bézier segment.
    fn tilt_bezpart(nu: &BNurb, prev: usize, next: usize, out: &mut [BBevelPoint], do_tilt: bool, do_radius: bool) {
        let last = nu.pntsu - 1;
        let bezt = &nu.bezt;
        let pprev = if prev == 0 { if nu.cyclic() { last } else { prev } } else { prev - 1 };
        let nnext = if next == last { if nu.cyclic() { 0 } else { next } } else { next + 1 };
        let (p0, p1, p2, p3) = (&bezt[pprev], &bezt[prev], &bezt[next], &bezt[nnext]);
        // Weight too is left unset when neither tilt nor radius is wanted.
        if !do_tilt && !do_radius {
            return;
        }
        let resolu = out.len();
        let dfac = 1.0 / resolu as f32;
        let mut fac = 0.0f32;
        let ease = |a: f32, b: f32, f: f32| a + (b - a) * (3.0 * f * f - 2.0 * f * f * f);
        for p in out.iter_mut() {
            if do_tilt {
                p.tilt = match nu.tilt_interp {
                    KEY_CU_EASE => ease(p1.tilt, p2.tilt, fac),
                    k => {
                        let t = position_weights(fac, k);
                        t[0] * p0.tilt + t[1] * p1.tilt + t[2] * p2.tilt + t[3] * p3.tilt
                    }
                };
            }
            if do_radius {
                p.radius = match nu.radius_interp {
                    KEY_CU_EASE => ease(p1.radius, p2.radius, fac),
                    k => {
                        let t = position_weights(fac, k);
                        t[0] * p0.radius + t[1] * p1.radius + t[2] * p2.radius + t[3] * p3.radius
                    }
                };
            }
            p.weight = ease(p1.weight, p2.weight, fac);
            fac += dfac;
        }
    }

    /// `BKE_nurb_makeCurve` over the spline's saved knots.
    fn nurbs(nu: &BNurb, do_tilt: bool, do_radius: bool) -> Vec<BBevelPoint> {
        let order = nu.orderu;
        let cycl = if nu.cyclic() { order - 1 } else { 0 };
        let knot_count = order + nu.pntsu + cycl;
        let resolu = nu.resolu * nu.segments();
        if nu.knots.len() < knot_count || order > nu.pntsu || resolu == 0 {
            return vec![BBevelPoint::default(); resolu];
        }
        let k = &nu.knots;
        let ustart = k[order - 1];
        let uend = if nu.cyclic() { k[nu.pntsu + order - 1] } else { k[nu.pntsu] };
        let ustep = (uend - ustart) / (resolu - if nu.cyclic() { 0 } else { 1 }) as f32;
        let mut basis = vec![0.0f32; knot_count];
        let mut u = ustart;
        let mut out: Vec<BBevelPoint> = (0..resolu)
            .map(|_| {
                let (istart, iend) = Self::basis(u, order, nu.pntsu + cycl, k, &mut basis);
                let point = |i: usize| &nu.bp[if i >= nu.pntsu { i - nu.pntsu } else { i }];
                let mut sum: Vec<f32> = (istart..=iend).map(|i| basis[i] * point(i).vec.w).collect();
                let div: f32 = sum.iter().sum();
                if div != 0.0 && !(1.0 - 1e-6..=1.0 + 1e-6).contains(&div) {
                    sum.iter_mut().for_each(|s| *s /= div);
                }
                let mut p = BBevelPoint::default();
                for (s, i) in sum.iter().zip(istart..=iend) {
                    if *s != 0.0 {
                        let bp = point(i);
                        p.vec += bp.vec.truncate() * *s;
                        if do_tilt {
                            p.tilt += s * bp.tilt;
                        }
                        if do_radius {
                            p.radius += s * bp.radius;
                        }
                        p.weight += s * bp.weight;
                    }
                }
                u += ustep;
                p
            })
            .collect();
        if !nu.cyclic() {
            Self::ends_from_points(nu, &mut out);
        }
        out
    }

    /// `basisNurb`: the basis at `t`, and the first and last non-zero one.
    fn basis(t: f32, order: usize, pnts: usize, knots: &[f32], basis: &mut [f32]) -> (usize, usize) {
        let opp = order + pnts;
        let opp2 = opp - 1;
        let t = if t < knots[0] { knots[0] } else if t > knots[opp2] { knots[opp2] } else { t };
        let (mut i1, mut i2) = (0usize, 0usize);
        let mut i = 0;
        while i < opp2 {
            if knots[i] != knots[i + 1] && t >= knots[i] && t <= knots[i + 1] {
                basis[i] = 1.0;
                i1 = i.saturating_sub(order + 1);
                i2 = i;
                i += 1;
                while i < opp2 {
                    basis[i] = 0.0;
                    i += 1;
                }
                break;
            }
            basis[i] = 0.0;
            i += 1;
        }
        basis[i] = 0.0;
        for j in 2..=order {
            if i2 + j >= opp {
                i2 = opp2 - j;
            }
            for i in i1..=i2 {
                let d = if basis[i] != 0.0 { ((t - knots[i]) * basis[i]) / (knots[i + j - 1] - knots[i]) } else { 0.0 };
                let e = if basis[i + 1] != 0.0 { ((knots[i + j] - t) * basis[i + 1]) / (knots[i + j] - knots[i + 1]) } else { 0.0 };
                basis[i] = d + e;
            }
        }
        let (mut start, mut end) = (usize::MAX, 0);
        for (i, b) in basis.iter().enumerate().take(i2 + 1).skip(i1) {
            if *b > 0.0 {
                end = i;
                if start == usize::MAX {
                    start = i;
                }
            }
        }
        (if start == usize::MAX { 1000 } else { start }, end)
    }

    /// STEP 2: a point the same as the next (within `0.00001 / resolu`)
    /// is dropped — for a closed spline, the last against the first too.
    fn drop_duplicates(&mut self, resolu: usize) {
        let n = self.points.len();
        if n == 0 {
            return;
        }
        let limit = 0.00001 / resolu as f32;
        let same = |a: Vec3, b: Vec3| (a - b).abs().max_element() <= limit;
        let pairs: Vec<(usize, usize)> = if self.cyclic { std::iter::once((n - 1, 0)).chain((0..n.saturating_sub(2)).map(|i| (i, i + 1))).collect() } else { (0..n - 1).map(|i| (i, i + 1)).collect() };
        for (a, b) in pairs {
            if same(self.points[a].vec, self.points[b].vec) {
                self.points[a].dupe = true;
            }
        }
        self.points.retain(|p| !p.dupe);
    }
}
