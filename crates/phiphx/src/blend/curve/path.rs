//! A curve's path (`BKE_anim_path_calc_data`, `BKE_where_on_path`): its
//! first spline's bevel list with the running length at each point, and
//! the place, rotation and radius a fraction of the way along it — what an
//! object parented to the curve rides (`ob_parcurve`) and what Follow Path
//! puts its owner on (`followpath_get_tarmat`).

use super::bevel::{BBevelList, BBevelPoint, KEY_BSPLINE, KEY_CARDINAL, KEY_LINEAR, position_weights};
use super::{BCurve, CU_FOLLOW, CU_NURB_CYCLIC, CU_PATH, CU_PATH_CLAMP, CU_PATH_RADIUS, CU_POLY, CU_BEZIER, CU_NURBS};
use crate::blend::rig::math::{BMat4, BQuat};
use glam::{Mat4, Vec3};

/// `FOLLOWPATH_FOLLOW`, `FOLLOWPATH_STATIC`, `FOLLOWPATH_RADIUS`.
pub const FOLLOWPATH_FOLLOW: u32 = 1 << 0;
pub const FOLLOWPATH_STATIC: u32 = 1 << 1;
pub const FOLLOWPATH_RADIUS: u32 = 1 << 2;

/// A curve as a path.
#[derive(Clone, Debug, PartialEq)]
pub struct BCurvePath {
    list: BBevelList,
    /// `anim_path_accum_length`: the length to the end of each segment.
    accum: Vec<f32>,
    /// The first spline's `Nurb.type` (how positions interpolate).
    kind: u16,
    /// `Curve.flag`, `pathlen`, and whether the first spline is closed.
    pub flag: u32,
    pub pathlen: f32,
    pub closed: bool,
    /// The curve's control-point box (`BCurve::bounds`), which Clamp To
    /// measures its owner against.
    pub bounds: Option<(Vec3, Vec3)>,
}

/// A place on the path (`BKE_where_on_path`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BPathPoint {
    pub vec: Vec3,
    pub tilt: f32,
    pub quat: BQuat,
    pub radius: f32,
}

/// A path at a frame: the curve's evaluation time there (`Curve.ctime`).
#[derive(Clone, Copy, Debug)]
pub struct BPathAt<'a> {
    pub path: &'a BCurvePath,
    pub ctime: f32,
}

impl BCurvePath {
    /// `curve`'s path, when its first spline has at least one segment.
    pub fn of(curve: &BCurve) -> Option<Self> {
        let list = BBevelList::first(curve)?;
        let n = list.points.len();
        if n < 2 {
            return None;
        }
        let seg = if list.cyclic { n } else { n - 1 };
        let mut accum = Vec::with_capacity(seg);
        let mut len = 0.0f32;
        for w in list.points.windows(2) {
            len += w[0].vec.distance(w[1].vec);
            accum.push(len);
        }
        if list.cyclic {
            accum.push(len + list.points[0].vec.distance(list.points[n - 1].vec));
        }
        let first = &curve.nurbs[0];
        let closed = first.flagu & CU_NURB_CYCLIC != 0;
        Some(Self { list, accum, kind: first.kind, flag: curve.flag, pathlen: curve.pathlen, closed, bounds: curve.bounds() })
    }

    /// `get_curve_points_from_idx`: the four points around segment `idx`.
    fn points(&self, idx: usize) -> [&BBevelPoint; 4] {
        let p = &self.list.points;
        let n = p.len();
        let cyclic = self.list.cyclic;
        if idx == 0 {
            let p0 = if cyclic { &p[n - 1] } else { &p[0] };
            return [p0, &p[0], &p[1], if n > 2 { &p[2] } else { &p[1] }];
        }
        if idx == n - 2 {
            return [&p[idx - 1], &p[idx], &p[idx + 1], if cyclic { &p[0] } else { &p[idx + 1] }];
        }
        if idx == n - 1 {
            return [&p[idx - 1], &p[idx], &p[0], &p[1]];
        }
        [&p[idx - 1], &p[idx], &p[idx + 1], &p[idx + 2]]
    }

    /// `binary_search_anim_path`: the segment holding `goal`, and how far
    /// into it.
    fn find(&self, goal: f32) -> Option<(usize, f32)> {
        let a = &self.accum;
        let seg = a.len();
        if seg == 1 {
            return Some((0, goal / a[0]));
        }
        let (mut base, mut step) = (0usize, seg - 1);
        loop {
            let cur = base + step / 2;
            let (left, right) = (a[cur], a[cur + 1]);
            if left <= goal && right > goal {
                return Some((cur + 1, (goal - left) / (right - left)));
            }
            if cur == 0 {
                return Some((0, goal / a[0]));
            }
            if step == 0 {
                return None;
            }
            if left < goal {
                base = cur + 1;
                step -= 1;
            }
            step /= 2;
        }
    }

    /// `BKE_where_on_path` at `ctime` (0–1 along the path).
    pub fn where_on(&self, mut ctime: f32) -> Option<BPathPoint> {
        if self.list.cyclic && !(0.0..=1.0).contains(&ctime) {
            ctime -= ctime.floor();
        }
        let seg = self.accum.len();
        let goal = ctime * self.accum[seg - 1];
        let (idx, frac) = if ctime <= 0.0 || ctime >= 1.0 {
            let idx = (ctime.clamp(0.0, 1.0) * (seg - 1) as f32) as usize;
            let frac = if idx == 0 { goal / self.accum[0] } else { (goal - self.accum[idx - 1]) / (self.accum[idx] - self.accum[idx - 1]) };
            (idx, frac)
        } else {
            self.find(goal)?
        };
        let [p0, p1, p2, p3] = self.points(idx);
        let kind = match self.kind {
            CU_POLY | CU_BEZIER | CU_NURBS => KEY_LINEAR,
            _ if std::ptr::eq(p2, p3) => KEY_CARDINAL,
            _ => KEY_BSPLINE,
        };
        let w = position_weights(frac, kind);
        let vec = p0.vec * w[0] + p1.vec * w[1] + p2.vec * w[2] + p3.vec * w[3];
        let w = w.map(|v| v.clamp(0.0, 1.0));
        let tilt = w[0] * p0.tilt + w[1] * p1.tilt + w[2] * p2.tilt + w[3] * p3.tilt;
        let blend = |a: [f32; 4], b: [f32; 4], wa: f32, wb: f32, fallback: [f32; 4]| {
            let total = wa + wb;
            if total > f32::EPSILON { BQuat::interp(BQuat(a), BQuat(b), wb / total).0 } else { fallback }
        };
        let q1 = blend(p0.quat, p3.quat, w[0], w[3], p1.quat);
        let q2 = blend(p1.quat, p2.quat, w[1], w[2], p3.quat);
        let total = w.iter().sum::<f32>();
        let quat = if total > f32::EPSILON { BQuat::interp(BQuat(q1), BQuat(q2), (w[1] + w[2]) / total) } else { BQuat(q2) };
        let radius = w[0] * p0.radius + w[1] * p1.radius + w[2] * p2.radius + w[3] * p3.radius;
        Some(BPathPoint { vec, tilt, quat, radius })
    }
}

impl BPathAt<'_> {
    /// `ob_parcurve`: where a child of the curve hangs, in the curve's
    /// space, `track` and `up` its `trackflag` and `upflag`. `None` when
    /// the curve is no path (the child takes the curve's world alone).
    pub fn parent_matrix(&self, track: i16, up: i16) -> Option<Mat4> {
        let path = self.path;
        if path.flag & CU_PATH == 0 {
            return None;
        }
        let mut ctime = if path.pathlen != 0.0 { self.ctime / path.pathlen } else { self.ctime };
        if path.flag & CU_PATH_CLAMP != 0 {
            ctime = ctime.clamp(0.0, 1.0);
        }
        let Some(at) = path.where_on(ctime) else { return Some(Mat4::IDENTITY) };
        let follow = path.flag & CU_FOLLOW != 0;
        let rot = follow.then(|| at.quat.track(track.clamp(0, 5) as usize, up.clamp(0, 2) as usize).normalized());
        Some(Self::place(at, rot, path.flag & CU_PATH_RADIUS != 0))
    }

    /// `followpath_get_tarmat`: the Follow Path target in the curve's space
    /// — `offset` frames behind, or `offset_fac` of the way with a fixed
    /// position — aimed by `track` / `up` with `FOLLOWPATH_FOLLOW`.
    pub fn follow_matrix(&self, flag: u32, offset: f32, offset_fac: f32, track: i16, up: i16) -> Option<Mat4> {
        let path = self.path;
        let ctime = if flag & FOLLOWPATH_STATIC != 0 {
            offset_fac
        } else {
            let t = (self.ctime - offset) / path.pathlen;
            if !path.closed && path.flag & CU_PATH_CLAMP != 0 { t.clamp(0.0, 1.0) } else { t }
        };
        let at = path.where_on(ctime)?;
        let rot = (flag & FOLLOWPATH_FOLLOW != 0).then(|| at.quat.track(track.clamp(0, 5) as usize, up.clamp(0, 2) as usize));
        Some(Self::place(at, rot, flag & FOLLOWPATH_RADIUS != 0))
    }

    /// The rotation (if any), scaled by the radius (if asked), at the place.
    fn place(at: BPathPoint, rot: Option<BQuat>, radius: bool) -> Mat4 {
        let mut m = rot.map_or(Mat4::IDENTITY, |q| Mat4::from_mat3(q.to_mat3()));
        if radius {
            m = Mat4::from_scale(Vec3::splat(at.radius)) * m;
        }
        m.set_loc(at.vec);
        m
    }
}
