//! Circle select — `view3d.select_circle` (C, `view3d_select.cc::
//! object_circle_select`): a brush of `radius` pixels follows the pointer
//! over the viewport; pressing or dragging with the left button selects
//! every object whose origin lies under it, Shift or the middle button
//! deselects, the wheel resizes the brush (`gesture_circle_modal`: ±2 plus
//! a tenth), and the right button or Escape ends the tool. Unlike B, the
//! tool stays on after each stroke. Strokes go through `ctx.op.select` as
//! box select's do (`BBoxMode::fire`, Blender's shared `SEL_OP_ADD / SUB`).
//!
//! Keys: armed `ctx.view3d.circle.select`, the pointer `ctx.view3d.circle.at`
//! (canvas pixels), the brush `ctx.view3d.circle.radius` — kept between
//! uses, as Blender remembers the last radius.

use super::BObject3D;
use super::pick::BPickView;
use crate::v2::state::{BState, View3dKey, val::V_TRUE};

/// The operator's default `radius`.
pub const DEFAULT_RADIUS: f32 = 25.0;

/// The brush: centre in canvas pixels, radius in pixels.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BCircle {
    pub x: f32,
    pub y: f32,
    pub radius: f32,
}

impl BCircle {
    /// Whether C has turned the tool on.
    pub fn armed(s: &BState) -> bool {
        s.get(View3dKey::CircleSelect) == Some(V_TRUE)
    }

    pub fn radius(s: &BState) -> f32 {
        s.get(View3dKey::CircleRadius).and_then(|r| r.parse().ok()).unwrap_or(DEFAULT_RADIUS)
    }

    /// The brush to draw: armed and the pointer over the canvas.
    pub fn now(s: &BState) -> Option<Self> {
        if !Self::armed(s) {
            return None;
        }
        let (x, y) = s.get(View3dKey::CircleAt)?.split_once(',')?;
        Some(Self { x: x.trim().parse().ok()?, y: y.trim().parse().ok()?, radius: Self::radius(s) })
    }

    pub fn encode_at(x: f32, y: f32) -> String {
        format!("{x:.1},{y:.1}")
    }

    /// One wheel notch: larger (`grow`) or smaller, never under a pixel.
    pub fn resized(radius: f32, grow: bool) -> f32 {
        let step = 2.0 + radius / 10.0;
        if grow { radius + step } else { (radius - step).max(1.0) }
    }

    /// The objects whose origin, projected through `view` onto a `w × h`
    /// canvas, lies under the brush — among those `allowed`, in scene order.
    pub fn names(&self, scene: &[BObject3D], view: &BPickView, w: f32, h: f32, allowed: impl Fn(&str) -> bool) -> Vec<String> {
        scene
            .iter()
            .filter(|object| allowed(object.name()))
            .filter(|object| {
                view.origin_on_canvas(object, w, h).is_some_and(|(px, py)| (px - self.x).powi(2) + (py - self.y).powi(2) <= self.radius.powi(2))
            })
            .map(|object| object.name().to_string())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::v2::scene::camera::BViewSetup;
    use crate::v2::scene::{BShape, MESH_SCALE};
    use glam::Vec3;

    #[test]
    fn the_brush_takes_the_origins_under_it_and_a_stroke_only_what_changes() {
        let left = BObject3D::Primitive { name: "Left".into(), shape: BShape::Cube, x: -0.45 * MESH_SCALE * 10.0, y: 0.0 };
        let right = BObject3D::Primitive { name: "Right".into(), shape: BShape::Cube, x: 0.45 * MESH_SCALE * 10.0, y: 0.0 };
        let view = BPickView { yaw: 0.0, pitch: 0.0, pan: (0.0, 0.0), zoom: 1.0, setup: BViewSetup::default(), aspect: 1.0 };
        let scene = [left, right];
        let (su, sv) = view.project(Vec3::new(-0.45 * 10.0, 0.0, 0.0)).unwrap();
        let (w, h) = (400.0, 400.0);
        let over_left = BCircle { x: (su + 0.5) * w, y: (sv + 0.5) * h, radius: 10.0 };
        assert_eq!(over_left.names(&scene, &view, w, h, |_| true), ["Left"]);
        assert!(over_left.names(&scene, &view, w, h, |n| n != "Left").is_empty(), "the arrow toggle keeps it out");
        assert!(BCircle { x: 0.0, y: 0.0, radius: 5.0 }.names(&scene, &view, w, h, |_| true).is_empty());

        use crate::v2::scene::boxsel::BBoxMode;
        let names = vec!["Left".to_string(), "Right".to_string()];
        assert_eq!(BBoxMode::Add.fire(names.clone(), |n| n == "Left").as_deref(), Some("box+:Right"));
        assert_eq!(BBoxMode::Sub.fire(names.clone(), |n| n == "Left").as_deref(), Some("box-:Left"));
        assert_eq!(BBoxMode::Add.fire(names, |_| true), None, "already selected: nothing to record");
        assert_eq!(BCircle::resized(25.0, true), 29.5);
        assert_eq!(BCircle::resized(2.0, false), 1.0);
    }
}
