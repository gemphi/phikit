//! The orbit a drag on the viewport makes: Blender's turntable
//! (`view3d.rotate` with Preferences ▸ Navigation ▸ Orbit Method Turntable,
//! `viewrotate_apply`).
//!
//! Across, the view turns about the world's Z axis; up and down, it tips
//! about its own horizontal axis — so the Z axis leans toward or away from
//! the eye as X and Y swing round. Dragging down brings the top of the
//! scene into view (pitch toward 90°, Top), as pulling the front of a
//! turntable down would.
//!
//! The tip is not stopped at the poles: past 90° the view has gone over the
//! top and is upside down, as Blender lets it. Blender then turns the
//! across direction round (`ViewOpsData::reverse`, set when the drag starts
//! with the view upside down), so the scene still follows the pointer.

/// One orbit drag, from where the view stood when it began.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BTurntable {
    pub yaw: f32,
    pub pitch: f32,
    reverse: f32,
}

impl BTurntable {
    /// Degrees per pixel dragged — input mapping, not styling.
    pub const DEG_PER_PX: f32 = 0.5;

    pub fn start(yaw: f32, pitch: f32) -> Self {
        let pitch = Self::wrap(pitch);
        let reverse = if pitch.to_radians().cos() < 0.0 { -1.0 } else { 1.0 };
        Self { yaw, pitch, reverse }
    }

    /// The yaw and pitch after the pointer moved `(dx, dy)` pixels from where
    /// the drag began (`dy` down).
    pub fn at(&self, dx: f32, dy: f32) -> (f32, f32) {
        let yaw = (self.yaw + self.reverse * dx * Self::DEG_PER_PX).rem_euclid(360.0);
        (yaw, Self::wrap(self.pitch + dy * Self::DEG_PER_PX))
    }

    /// A pitch in `(-180, 180]`: one turn over the top and back is the same
    /// view.
    pub fn wrap(pitch: f32) -> f32 {
        let w = (pitch + 180.0).rem_euclid(360.0) - 180.0;
        if w <= -180.0 { 180.0 } else { w }
    }
}

#[cfg(test)]
mod tests {
    use super::BTurntable;
    use crate::v2::scene::pick::BPickView;
    use glam::Vec3;

    fn view(yaw: f32, pitch: f32) -> (Vec3, Vec3, Vec3) {
        BPickView { yaw, pitch, pan: (0.0, 0.0), zoom: 1.0, setup: Default::default(), aspect: 1.0 }.axes()
    }

    #[test]
    fn a_drag_down_tips_the_z_axis_toward_the_eye_and_across_turns_about_it() {
        // Blender's startup view, 30° / 30°.
        let orbit = BTurntable::start(30.0, 30.0);
        let (yaw, pitch) = orbit.at(0.0, 60.0);
        assert_eq!((yaw, pitch), (30.0, 60.0), "down: 0.5° a pixel toward Top");
        let (_, up_before, _) = view(30.0, 30.0);
        let (_, up_after, back_after) = view(yaw, pitch);
        assert!(Vec3::Z.dot(up_after) < Vec3::Z.dot(up_before), "Z leans away from screen-up");
        assert!(Vec3::Z.dot(back_after) > 0.8, "the eye is above the scene");
        assert_eq!(orbit.at(40.0, 0.0), (50.0, 30.0), "across: about Z only");
    }

    #[test]
    fn the_view_goes_over_the_top_and_stays_continuous() {
        let orbit = BTurntable::start(0.0, 80.0);
        let (_, pitch) = orbit.at(0.0, 40.0);
        assert_eq!(pitch, 100.0, "past Top, not stopped at it");
        let (right_a, up_a, back_a) = view(0.0, 89.9);
        let (right_b, up_b, back_b) = view(0.0, 90.1);
        assert!((right_a - right_b).length() < 1e-2 && (up_a - up_b).length() < 1e-2 && (back_a - back_b).length() < 1e-2);
        assert!(view(0.0, 100.0).1.z < 0.0, "upside down");
        assert_eq!(BTurntable::start(0.0, 170.0).at(0.0, 40.0).1, -170.0, "one turn wraps");
    }

    #[test]
    fn upside_down_the_across_drag_turns_the_other_way() {
        assert_eq!(BTurntable::start(0.0, 120.0).at(20.0, 0.0).0, 350.0);
        assert_eq!(BTurntable::start(0.0, 60.0).at(20.0, 0.0).0, 10.0);
    }

    #[test]
    fn pitch_wraps_into_one_turn() {
        assert_eq!(BTurntable::wrap(190.0), -170.0);
        assert_eq!(BTurntable::wrap(-180.0), 180.0);
        assert_eq!(BTurntable::wrap(450.0), 90.0);
    }
}
