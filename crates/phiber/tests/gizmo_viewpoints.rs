//! Clicking a compass cap turns the view to that viewpoint.
//!
//! The Z caps need a pitch, not a yaw — writing a yaw alone left them doing
//! nothing meaningful, which is why a cap names a viewpoint and
//! `state::follow` turns it into both angles.

use phiber::v2::state::{BKey as _, View3dKey};
use phiber::v2::scene::{view_pitch, view_yaw};
use phiber::v2::state::{BState, action::apply_action};
use phiber::v2::view::prim::BAction;

fn look(name: &str) -> (f32, f32) {
    let mut s = BState::new();
    apply_action(&mut s, &BAction::Set(View3dKey::View.path(), name.to_string()));
    (view_yaw(&s), view_pitch(&s))
}

#[test]
fn every_axis_cap_turns_the_view_including_the_z_ones() {
    for (name, yaw, pitch) in [
        ("front", 0.0, 0.0),
        ("back", 180.0, 0.0),
        ("right", 90.0, 0.0),
        ("left", 270.0, 0.0),
        ("top", 0.0, 90.0),
        ("bottom", 0.0, -90.0),
    ] {
        assert_eq!(look(name), (yaw, pitch), "{name} view");
    }
}

#[test]
fn a_viewpoint_is_consumed_once_it_has_been_applied() {
    let mut s = BState::new();
    apply_action(&mut s, &BAction::Set(View3dKey::View.path(), "top".to_string()));
    assert_eq!(s.get(View3dKey::View), None, "the intent does not linger");
    assert_eq!(view_pitch(&s), 90.0);
}

#[test]
fn an_unknown_viewpoint_leaves_the_view_alone() {
    let mut s = BState::new();
    let before = (view_yaw(&s), view_pitch(&s));
    apply_action(&mut s, &BAction::Set(View3dKey::View.path(), "sideways".to_string()));
    assert_eq!((view_yaw(&s), view_pitch(&s)), before);
}

#[test]
fn every_compass_cap_names_a_viewpoint_the_follow_rule_knows() {
    let axis = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("src/v2/view/draw/canvas/gizmo/compass/axis.rs"),
    )
    .expect("compass axis source");
    for name in ["right", "left", "front", "back", "top", "bottom"] {
        assert!(
            axis.contains(&format!("\"{name}\"")),
            "no compass cap looks from the {name}"
        );
        assert!(
            phiber::v2::scene::viewpoint::view_angles(name).is_some(),
            "the follow rule does not know the {name} view"
        );
    }
    assert!(
        !axis.contains("yaw_key()"),
        "a cap must not write a bare yaw — the Z caps need a pitch"
    );
}

#[test]
fn the_compass_projects_all_three_axes() {
    let compass = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("src/v2/view/draw/canvas/gizmo/compass/mod.rs"),
    )
    .expect("compass source");
    // Z used to be pinned upright with a fixed depth, so the caps tipped with
    // pitch but the axis they sat on never did — which is what made the
    // compass look like it was not responding to rotation.
    for axis in ["x", "y", "z"] {
        assert!(
            compass.contains(&format!("{axis}_dx, {axis}_dy, {axis}_depth")),
            "the {axis} spoke must use its own projection"
        );
    }
}

#[test]
fn the_sphere_and_the_square_clusters_sit_at_opposite_ends() {
    use phiber::v2::view::draw::canvas::view3d::corner::BCorner;
    for corner in [
        BCorner::TopLeft,
        BCorner::TopRight,
        BCorner::BottomLeft,
        BCorner::BottomRight,
    ] {
        let other = corner.flipped();
        assert_ne!(corner, other, "the two groups must not share a corner");
        assert_eq!(
            corner.is_right(),
            other.is_right(),
            "they stay on the same side, so both clear the same strip"
        );
        assert_eq!(other.flipped(), corner, "flipping twice comes back");
    }
}
