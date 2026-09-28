//! Lasso select — `view3d.select_lasso` (Ctrl right-drag, Shift Ctrl to
//! deselect; `view3d_select.cc::do_lasso_select_objects`): the drag draws a
//! free-hand outline, and on release every object whose origin lies inside
//! it is added to (or taken from) the selection. The outline while it is
//! drawn is `ctx.view3d.lasso.path` — `x,y;x,y;…` in canvas pixels, a point
//! kept only once the pointer has moved [`STEP`] pixels, as
//! `WM_gesture_lasso` thins its points.

use super::BObject3D;
use super::pick::BPickView;
use crate::v2::state::{BState, View3dKey};

/// Pixels the pointer moves before the outline takes another point.
pub const STEP: f32 = 4.0;

/// The outline, in canvas pixels.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct BLasso {
    pub points: Vec<(f32, f32)>,
}

impl BLasso {
    /// The outline being drawn, if a lasso is under way.
    pub fn now(s: &BState) -> Option<Self> {
        let lasso = Self::parse(s.get(View3dKey::LassoPath)?);
        (!lasso.points.is_empty()).then_some(lasso)
    }

    pub fn parse(raw: &str) -> Self {
        let points = raw
            .split(';')
            .filter_map(|p| {
                let (x, y) = p.split_once(',')?;
                Some((x.trim().parse().ok()?, y.trim().parse().ok()?))
            })
            .collect();
        Self { points }
    }

    pub fn encode(&self) -> String {
        self.points.iter().map(|(x, y)| format!("{x:.1},{y:.1}")).collect::<Vec<_>>().join(";")
    }

    /// The outline with `(x, y)` added — `None` while the pointer is still
    /// within [`STEP`] of the last point.
    pub fn extended(&self, x: f32, y: f32) -> Option<Self> {
        let far = self.points.last().is_none_or(|(lx, ly)| (x - lx).hypot(y - ly) >= STEP);
        far.then(|| {
            let mut points = self.points.clone();
            points.push((x, y));
            Self { points }
        })
    }

    /// Is `(x, y)` inside the closed outline (even–odd, `BLI_lasso_is_point_inside`)?
    pub fn contains(&self, x: f32, y: f32) -> bool {
        let n = self.points.len();
        if n < 3 {
            return false;
        }
        let mut inside = false;
        for i in 0..n {
            let (ax, ay) = self.points[i];
            let (bx, by) = self.points[(i + n - 1) % n];
            if (ay > y) != (by > y) && x < (bx - ax) * (y - ay) / (by - ay) + ax {
                inside = !inside;
            }
        }
        inside
    }

    /// The objects whose origin, on a `w × h` canvas, is inside — among
    /// those `allowed`, in scene order.
    pub fn names(&self, scene: &[BObject3D], view: &BPickView, w: f32, h: f32, allowed: impl Fn(&str) -> bool) -> Vec<String> {
        scene
            .iter()
            .filter(|object| allowed(object.name()))
            .filter(|object| view.origin_on_canvas(object, w, h).is_some_and(|(x, y)| self.contains(x, y)))
            .map(|object| object.name().to_string())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_outline_thins_its_points_and_holds_what_is_inside() {
        let start = BLasso { points: vec![(0.0, 0.0)] };
        assert!(start.extended(2.0, 2.0).is_none(), "under four pixels: no point");
        let square = BLasso { points: vec![(0.0, 0.0), (100.0, 0.0), (100.0, 100.0), (0.0, 100.0)] };
        assert!(square.contains(50.0, 50.0) && !square.contains(150.0, 50.0) && !square.contains(-1.0, 50.0));
        assert!(!BLasso { points: vec![(0.0, 0.0), (9.0, 9.0)] }.contains(4.0, 4.0), "a line holds nothing");
        assert_eq!(BLasso::parse(&square.encode()), square);
    }
}
