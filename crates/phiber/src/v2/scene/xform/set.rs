//! The three transforms G, R and S start, and what the pointer's travel
//! means to each — Blender's `transform_mode_translate.cc`,
//! `transform_mode_rotate.cc` and `transform_mode_resize.cc` in the view:
//!
//! - **Move** keeps the pointer over the same point of the plane through
//!   the pivot, facing the view; with an axis it slides along that axis
//!   through the pivot to the point nearest the pointer's ray.
//! - **Rotate** turns about the view axis through the pivot by the angle
//!   the pointer swept round the pivot on screen; with an axis, about that
//!   axis, the same way round as seen.
//! - **Scale** scales about the pivot by how much farther the pointer is
//!   from it on screen than where it started; with an axis, along it alone.

use super::constraint::BConstraint;
use crate::v2::scene::pick::{BPickRay, BPickView};
use glam::{Mat4, Vec3};

/// `transform.translate`, `transform.rotate`, `transform.resize`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BTransformMode {
    Translate,
    Rotate,
    Resize,
}

/// The pointer's travel in one viewport: where it started and where it is,
/// in pixels on a canvas of `size`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BPointerTravel {
    pub from: (f32, f32),
    pub to: (f32, f32),
    pub size: (f32, f32),
}

pub static MODES: [BTransformMode; 3] = [BTransformMode::Translate, BTransformMode::Rotate, BTransformMode::Resize];

impl BTransformMode {
    pub fn slug(self) -> &'static str {
        match self {
            Self::Translate => "translate",
            Self::Rotate => "rotate",
            Self::Resize => "resize",
        }
    }

    pub fn from_slug(slug: &str) -> Option<Self> {
        MODES.iter().copied().find(|mode| mode.slug() == slug)
    }

    /// The name it has in the menus and the undo history.
    pub fn label(self) -> &'static str {
        match self {
            Self::Translate => "Move",
            Self::Rotate => "Rotate",
            Self::Resize => "Scale",
        }
    }

    /// The change in world space the travel makes, about `pivot`, held to
    /// `constraint`.
    pub fn delta(self, view: &BPickView, pivot: Vec3, constraint: BConstraint, travel: BPointerTravel) -> Mat4 {
        match self {
            Self::Translate => Mat4::from_translation(Self::slide(view, pivot, constraint, travel)),
            Self::Rotate => {
                let centre = Self::on_canvas(view, pivot, travel.size);
                let angle = |p: (f32, f32)| (centre.1 - p.1).atan2(p.0 - centre.0);
                let (_, _, back) = view.axes();
                // Turning the same way round as the pointer, as seen.
                let turn = match constraint {
                    BConstraint::Axis(i) | BConstraint::Plane(i) if BConstraint::unit(i).dot(back) < 0.0 => -BConstraint::unit(i),
                    BConstraint::Axis(i) | BConstraint::Plane(i) => BConstraint::unit(i),
                    BConstraint::Free => back,
                };
                Self::about(pivot, Mat4::from_axis_angle(turn, angle(travel.to) - angle(travel.from)))
            }
            Self::Resize => {
                let centre = Self::on_canvas(view, pivot, travel.size);
                let reach = |p: (f32, f32)| ((p.0 - centre.0).powi(2) + (p.1 - centre.1).powi(2)).sqrt();
                let factor = reach(travel.to) / reach(travel.from).max(1.0);
                Self::about(pivot, Mat4::from_scale(constraint.spread(factor)))
            }
        }
    }

    /// The change a typed `value` makes — metres along the axis, degrees
    /// about it (`back`, the view's axis toward the eye, when free), or a
    /// factor — Blender's numeric input (`transform_numinput`).
    pub fn typed(self, pivot: Vec3, constraint: BConstraint, value: f32, back: Vec3) -> Mat4 {
        match self {
            Self::Translate => Mat4::from_translation(constraint.typed_axis() * value),
            Self::Rotate => {
                let axis = match constraint {
                    BConstraint::Axis(i) | BConstraint::Plane(i) => BConstraint::unit(i),
                    BConstraint::Free => back,
                };
                Self::about(pivot, Mat4::from_axis_angle(axis, value.to_radians()))
            }
            Self::Resize => Self::about(pivot, Mat4::from_scale(constraint.spread(value))),
        }
    }

    /// `m` about `pivot`.
    fn about(pivot: Vec3, m: Mat4) -> Mat4 {
        Mat4::from_translation(pivot) * m * Mat4::from_translation(-pivot)
    }

    /// Where `pivot` lands on the canvas, in pixels.
    fn on_canvas(view: &BPickView, pivot: Vec3, (w, h): (f32, f32)) -> (f32, f32) {
        view.project(pivot).map_or((w * 0.5, h * 0.5), |(su, sv)| ((su + 0.5) * w, (sv + 0.5) * h))
    }

    /// Move's world offset.
    fn slide(view: &BPickView, pivot: Vec3, constraint: BConstraint, travel: BPointerTravel) -> Vec3 {
        let (w, h) = travel.size;
        let ray = |p: (f32, f32)| {
            let (su, sv) = BPickView::canvas_point(p.0, p.1, w, h);
            view.ray(su, sv)
        };
        let (from, to) = (ray(travel.from), ray(travel.to));
        // The point of a plane through the pivot that a ray meets.
        let on_plane = |normal: Vec3| {
            let hit = |r: BPickRay| {
                let facing = r.dir.dot(normal);
                (facing.abs() > 1e-6).then(|| r.origin + r.dir * ((pivot - r.origin).dot(normal) / facing))
            };
            hit(to).zip(hit(from)).map_or(Vec3::ZERO, |(a, b)| a - b)
        };
        match constraint {
            // The plane facing the view.
            BConstraint::Free => on_plane(view.axes().2),
            // The plane of the two free axes.
            BConstraint::Plane(i) => on_plane(BConstraint::unit(i)),
            // The point of the axis line through the pivot nearest the ray.
            BConstraint::Axis(i) => {
                let a = BConstraint::unit(i);
                let along = |r: BPickRay| {
                    let (w0, b) = (pivot - r.origin, a.dot(r.dir));
                    let denom = 1.0 - b * b;
                    (denom > 1e-6).then(|| (b * r.dir.dot(w0) - a.dot(w0)) / denom)
                };
                along(to).zip(along(from)).map_or(Vec3::ZERO, |(s1, s0)| a * (s1 - s0))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::v2::scene::camera::BViewSetup;

    /// Front view, orthographic: +X right, +Z up.
    fn front() -> BPickView {
        BPickView { yaw: 0.0, pitch: 0.0, pan: (0.0, 0.0), zoom: 1.0, setup: BViewSetup::default(), aspect: 1.0 }
    }

    fn travel(from: (f32, f32), to: (f32, f32)) -> BPointerTravel {
        BPointerTravel { from, to, size: (400.0, 400.0) }
    }

    #[test]
    fn move_keeps_the_pointer_over_the_same_point_and_an_axis_takes_its_share() {
        let view = front();
        let right = BTransformMode::Translate.delta(&view, Vec3::ZERO, BConstraint::Free, travel((200.0, 200.0), (260.0, 200.0))).w_axis.truncate();
        assert!(right.x > 0.0 && right.y.abs() < 1e-4 && right.z.abs() < 1e-4, "right on screen is +X in front view: {right:?}");
        // The pointer lands where the moved point projects.
        let (su, _) = view.project(right).unwrap();
        assert!(((su + 0.5) * 400.0 - 260.0).abs() < 0.01);
        let up = BTransformMode::Translate.delta(&view, Vec3::ZERO, BConstraint::Free, travel((200.0, 200.0), (200.0, 150.0))).w_axis.truncate();
        assert!(up.z > 0.0 && up.x.abs() < 1e-4, "up on screen is +Z: {up:?}");
        // Along Z only: the sideways part of the travel is dropped.
        let z = BTransformMode::Translate.delta(&view, Vec3::ZERO, BConstraint::Axis(2), travel((200.0, 200.0), (260.0, 150.0))).w_axis.truncate();
        assert!(z.x.abs() < 1e-4 && (z.z - up.z).abs() < 1e-3, "{z:?}");
        // Along Y, the axis the view looks down: no travel moves it.
        let y = BTransformMode::Translate.delta(&view, Vec3::ZERO, BConstraint::Axis(1), travel((200.0, 200.0), (260.0, 150.0))).w_axis.truncate();
        assert!(y.length() < 1e-4);
    }

    #[test]
    fn rotate_follows_the_pointer_round_the_pivot_and_scale_its_reach() {
        let view = front();
        // A quarter turn counter-clockwise on screen, from the right of the
        // pivot to above it: +X turns to +Z, about the axis toward the eye.
        let m = BTransformMode::Rotate.delta(&view, Vec3::ZERO, BConstraint::Free, travel((300.0, 200.0), (200.0, 100.0)));
        let x = m.transform_vector3(Vec3::X);
        assert!((x - Vec3::Z).length() < 1e-4, "{x:?}");
        let s = BTransformMode::Resize.delta(&view, Vec3::ZERO, BConstraint::Free, travel((250.0, 200.0), (300.0, 200.0)));
        assert!((s.transform_vector3(Vec3::ONE) - Vec3::splat(2.0)).length() < 1e-4, "twice as far: twice as big");
        let sx = BTransformMode::Resize.delta(&view, Vec3::ZERO, BConstraint::Axis(0), travel((250.0, 200.0), (300.0, 200.0)));
        assert!((sx.transform_vector3(Vec3::ONE) - Vec3::new(2.0, 1.0, 1.0)).length() < 1e-4, "along X alone");
        assert_eq!(BTransformMode::from_slug("resize"), Some(BTransformMode::Resize));
    }
}
