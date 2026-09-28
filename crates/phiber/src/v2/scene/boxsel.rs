//! Box select — `view3d.select_box` (`view3d_select.cc::do_object_box_select`):
//! every selectable object with something inside the dragged rectangle is
//! selected; Shift adds to the selection, Ctrl takes away. Blender reads an
//! ID pass under the box; here an object counts when its bounds, projected
//! through the view (`pick::BPickView::project`), overlap the box — the
//! same view the tracer draws with, so the box lands on what the pixels
//! show. A corner behind the eye is left out of the object's extent.
//!
//! The gesture is armed by **B** (`ctx.view3d.box.select`), the rectangle
//! while it is dragged is `ctx.view3d.box.rect`, and the release fires
//! `ctx.op.select = box:<names>` (or `box+:` / `box-:`), which
//! `select::BSelectOpRule` applies.

use super::BObject3D;
use super::pick::BPickView;
use crate::v2::state::{BState, View3dKey, val::V_TRUE};
use glam::Vec3;

/// The box in canvas coordinates (`-0.5 ..= 0.5` each way), any corner order.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BBox2 {
    pub a: (f32, f32),
    pub b: (f32, f32),
}

impl BBox2 {
    /// Is the canvas point `p` inside?
    pub fn holds(&self, p: (f32, f32)) -> bool {
        self.overlaps(p, p)
    }

    fn overlaps(&self, lo: (f32, f32), hi: (f32, f32)) -> bool {
        let (x0, x1) = (self.a.0.min(self.b.0), self.a.0.max(self.b.0));
        let (y0, y1) = (self.a.1.min(self.b.1), self.a.1.max(self.b.1));
        lo.0 <= x1 && hi.0 >= x0 && lo.1 <= y1 && hi.1 >= y0
    }
}

/// How the box changes the selection: Blender's `SEL_OP_SET / _ADD / _SUB`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BBoxMode {
    Set,
    Add,
    Sub,
}

impl BBoxMode {
    pub fn of(shift: bool, control: bool) -> Self {
        match (shift, control) {
            (_, true) => Self::Sub,
            (true, _) => Self::Add,
            _ => Self::Set,
        }
    }

    /// The `ctx.op.select` value's head.
    pub fn op(self) -> &'static str {
        match self {
            Self::Set => "box:",
            Self::Add => "box+:",
            Self::Sub => "box-:",
        }
    }

    /// The `ctx.op.select` an Add or Sub gesture over `names` fires — circle
    /// and lasso — or `None` when it would change nothing: only names not
    /// yet selected are added, only selected ones taken away, so a stroke
    /// records an undo step per change, not per pointer move.
    pub fn fire(self, names: Vec<String>, selected: impl Fn(&str) -> bool) -> Option<String> {
        let changing: Vec<String> = names.into_iter().filter(|n| match self {
            Self::Sub => selected(n),
            _ => !selected(n),
        }).collect();
        (!changing.is_empty()).then(|| format!("{}{}", self.op(), changing.join(",")))
    }
}

/// The world-space corners of `object`'s bounds.
fn corners(object: &BObject3D) -> [Vec3; 8] {
    let (lo, hi) = super::view_op::extent_of(object);
    let mut out = [Vec3::ZERO; 8];
    for (i, c) in out.iter_mut().enumerate() {
        *c = Vec3::new(
            if i & 1 == 0 { lo.x } else { hi.x },
            if i & 2 == 0 { lo.y } else { hi.y },
            if i & 4 == 0 { lo.z } else { hi.z },
        );
    }
    out
}

/// The names of the objects in `scene` whose projected bounds overlap
/// `boxed`, among those `allowed`, in scene order.
pub fn boxed_names(scene: &[BObject3D], view: &BPickView, boxed: BBox2, allowed: impl Fn(&str) -> bool) -> Vec<String> {
    let mut out = Vec::new();
    for object in scene {
        if !allowed(object.name()) {
            continue;
        }
        let mut lo = (f32::INFINITY, f32::INFINITY);
        let mut hi = (f32::NEG_INFINITY, f32::NEG_INFINITY);
        for corner in corners(object) {
            let Some((su, sv)) = view.project(corner) else { continue };
            lo = (lo.0.min(su), lo.1.min(sv));
            hi = (hi.0.max(su), hi.1.max(sv));
        }
        if lo.0.is_finite() && boxed.overlaps(lo, hi) {
            out.push(object.name().to_string());
        }
    }
    out
}

/// Whether B has armed the gesture.
pub fn armed(state: &BState) -> bool {
    state.get(View3dKey::BoxSelect) == Some(V_TRUE)
}

/// The rectangle being dragged, in canvas pixels `(x0, y0, x1, y1)`.
pub fn dragged(state: &BState) -> Option<(f32, f32, f32, f32)> {
    let raw = state.get(View3dKey::BoxRect)?;
    let mut it = raw.split(',').map(|v| v.trim().parse::<f32>().ok());
    Some((it.next()??, it.next()??, it.next()??, it.next()??))
}

pub fn encode(x0: f32, y0: f32, x1: f32, y1: f32) -> String {
    format!("{x0:.1},{y0:.1},{x1:.1},{y1:.1}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::v2::scene::camera::BViewSetup;
    use crate::v2::scene::{BShape, MESH_SCALE};

    #[test]
    fn a_box_around_one_cube_takes_it_and_not_its_neighbour_and_the_modes_read_the_modifiers() {
        let left = BObject3D::Primitive { name: "Left".into(), shape: BShape::Cube, x: -0.45 * MESH_SCALE * 10.0, y: 0.0 };
        let right = BObject3D::Primitive { name: "Right".into(), shape: BShape::Cube, x: 0.45 * MESH_SCALE * 10.0, y: 0.0 };
        let view = BPickView { yaw: 0.0, pitch: 0.0, pan: (0.0, 0.0), zoom: 1.0, setup: BViewSetup::default(), aspect: 1.0 };
        let scene = [left, right];
        let (lsu, _) = view.project(Vec3::new(-0.45 * 10.0, 0.0, 0.0)).unwrap();
        let (rsu, _) = view.project(Vec3::new(0.45 * 10.0, 0.0, 0.0)).unwrap();
        let (near, far) = (lsu.min(rsu), lsu.max(rsu));
        let mid = (near + far) / 2.0;
        // A box over the nearer half of the canvas takes the cube on that side only.
        let one = boxed_names(&scene, &view, BBox2 { a: (-0.5, -0.5), b: (mid, 0.5) }, |_| true);
        assert_eq!(one.len(), 1, "{one:?}");
        let both = boxed_names(&scene, &view, BBox2 { a: (0.5, 0.5), b: (-0.5, -0.5) }, |_| true);
        assert_eq!(both, ["Left", "Right"], "corner order does not matter");
        let none = boxed_names(&scene, &view, BBox2 { a: (-0.5, 0.45), b: (0.5, 0.5) }, |_| true);
        assert!(none.is_empty(), "a strip along the foot holds nothing: {none:?}");
        assert_eq!(boxed_names(&scene, &view, BBox2 { a: (0.5, 0.5), b: (-0.5, -0.5) }, |n| n != "Left"), ["Right"]);
        assert_eq!((BBoxMode::of(false, false), BBoxMode::of(true, false), BBoxMode::of(false, true)), (BBoxMode::Set, BBoxMode::Add, BBoxMode::Sub));
        assert_eq!(dragged(&{ let mut s = BState::new(); s.set(View3dKey::BoxRect, encode(1.0, 2.0, 30.5, 40.0)); s }), Some((1.0, 2.0, 30.5, 40.0)));
    }
}
