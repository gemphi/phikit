//! Legacy curves (`Curve`, `Nurb`) as paths: what an object parented to a
//! curve rides along, and what the Follow Path and Clamp To constraints
//! read (Clamp To its control-point box too, `BCurve::bounds`).
//!
//! Blender turns a curve's splines into its bevel list
//! (`BKE_curve_bevelList_make`, [`bevel`]): each spline evaluated at its
//! resolution — Bézier by forward differencing, NURBS by its basis, poly as
//! is — with tilt, radius and weight between the points, duplicates
//! dropped, and a rotation at every point by the curve's twist method.
//! The path is the first spline's list with its running length
//! (`BKE_anim_path_calc_data`), and a place on it is found by length
//! (`BKE_where_on_path`, [`path`]).
//!
//! Not read: deform modifiers and shape keys on the curve, and hooks.

pub mod bevel;
mod orient;
pub mod path;
pub(crate) mod read;
#[cfg(test)]
mod tests;

pub use path::{BCurvePath, BPathAt, BPathPoint};

use glam::{Vec3, Vec4};

/// `Curve.flag` bits a path reads.
pub const CU_3D: u32 = 1 << 0;
pub const CU_PATH: u32 = 1 << 3;
pub const CU_FOLLOW: u32 = 1 << 4;
pub const CU_PATH_CLAMP: u32 = 1 << 5;
pub const CU_PATH_RADIUS: u32 = 1 << 12;

/// `Nurb.type`.
pub const CU_POLY: u16 = 0;
pub const CU_BEZIER: u16 = 1;
pub const CU_NURBS: u16 = 4;

/// `Nurb.flagu`.
pub const CU_NURB_CYCLIC: u16 = 1 << 0;
pub const CU_NURB_ENDPOINT: u16 = 1 << 1;
pub const CU_NURB_BEZIER: u16 = 1 << 2;

/// `Curve.twist_mode`.
pub const CU_TWIST_MINIMUM: u16 = 3;
pub const CU_TWIST_TANGENT: u16 = 4;

/// One Bézier control point (`BezTriple`): left handle, point, right
/// handle.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BBezPoint {
    pub vec: [Vec3; 3],
    pub tilt: f32,
    pub weight: f32,
    pub radius: f32,
    /// `h1`, `h2`: the handle types (`HD_VECT` = 2).
    pub h1: u8,
    pub h2: u8,
}

/// One NURBS or poly control point (`BPoint`): position and weight in `vec`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BCurvePoint {
    pub vec: Vec4,
    pub tilt: f32,
    pub weight: f32,
    pub radius: f32,
}

/// One spline (`Nurb`).
#[derive(Clone, Debug, PartialEq)]
pub struct BNurb {
    /// `CU_POLY`, `CU_BEZIER` or `CU_NURBS`.
    pub kind: u16,
    pub flagu: u16,
    pub pntsu: usize,
    pub pntsv: usize,
    pub resolu: usize,
    pub orderu: usize,
    /// `knotsu` as saved (NURBS only).
    pub knots: Vec<f32>,
    pub bezt: Vec<BBezPoint>,
    pub bp: Vec<BCurvePoint>,
    /// `KEY_*` for tilt and radius between Bézier points.
    pub tilt_interp: i16,
    pub radius_interp: i16,
}

/// A curve datablock, as much as a path needs.
#[derive(Clone, Debug, PartialEq)]
pub struct BCurve {
    pub flag: u32,
    /// `pathlen` (Path Animation ▸ Frames) and `ctime` (Evaluation Time,
    /// as saved).
    pub pathlen: f32,
    pub ctime: f32,
    pub twist_mode: u16,
    pub twist_smooth: f32,
    /// `CU_DO_RADIUS`: the points carry their radius into the bevel list.
    pub do_radius: bool,
    pub nurbs: Vec<BNurb>,
}

impl BNurb {
    pub fn cyclic(&self) -> bool {
        self.flagu & CU_NURB_CYCLIC != 0
    }

    /// `SEGMENTSU`.
    pub fn segments(&self) -> usize {
        if self.cyclic() { self.pntsu } else { self.pntsu.saturating_sub(1) }
    }

    /// `BKE_nurb_check_valid_u`.
    pub fn valid(&self) -> bool {
        if self.pntsu <= 1 {
            return false;
        }
        if self.kind != CU_NURBS {
            return true;
        }
        if self.pntsu < self.orderu {
            return false;
        }
        if self.flagu & CU_NURB_BEZIER != 0 {
            let order = self.orderu.max(2);
            let needed = if self.cyclic() {
                let remainder = self.pntsu % (order - 1);
                if remainder > 0 { order - 1 - remainder } else { 0 }
            } else if self.flagu & CU_NURB_ENDPOINT == 0 && self.pntsu <= order {
                order + 1 - self.pntsu
            } else {
                0
            };
            return needed == 0;
        }
        true
    }
}

impl BCurve {
    /// `BKE_curve_minmax` with the radius (`calc_nurb_minmax`): the box
    /// round every control point — a Bézier point's handles too, and a
    /// point grown by its radius on every axis (a surface's rows aside) —
    /// in the curve's own space. `None` with no splines.
    pub fn bounds(&self) -> Option<(Vec3, Vec3)> {
        if self.nurbs.is_empty() {
            return None;
        }
        let (mut min, mut max) = (Vec3::splat(f32::MAX), Vec3::splat(f32::MIN));
        let mut take = |p: Vec3| {
            min = min.min(p);
            max = max.max(p);
        };
        for nu in &self.nurbs {
            for b in nu.bezt.iter().filter(|_| nu.kind == CU_BEZIER) {
                take(b.vec[1] + Vec3::splat(b.radius));
                take(b.vec[1] - Vec3::splat(b.radius));
                take(b.vec[0]);
                take(b.vec[2]);
            }
            for p in nu.bp.iter().filter(|_| nu.kind != CU_BEZIER) {
                let r = if nu.pntsv == 1 { p.radius } else { 0.0 };
                take(p.vec.truncate() + Vec3::splat(r));
                take(p.vec.truncate() - Vec3::splat(r));
            }
        }
        Some((min, max))
    }
}
