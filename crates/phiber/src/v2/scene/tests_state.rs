//! State-key + view-accessor tests. Covers `yaw`/`zoom`/`pitch`/`pan` reads
//! from `BState`, blend-path predicates, and canonical `_key()` segments.
//! Split from `mod.rs` so all tests files stay under the 200-line cap.

use super::*;

#[test]
fn default_scene_has_three_objects() {
    assert_eq!(OBJECTS.len(), 3);
    assert_eq!(OBJECTS[0].shape, BShape::Cube);
    assert_eq!(OBJECTS[1].shape, BShape::Sphere);
    assert_eq!(OBJECTS[2].shape, BShape::Gem);
}

#[test]
fn yaw_defaults_to_thirty() {
    let s = BState::new();
    assert!((view_yaw(&s) - 30.0).abs() < 1e-6);
}

#[test]
fn yaw_round_trips_through_state() {
    let mut s = BState::new();
    s.set(&yaw_key(), "90");
    assert!((view_yaw(&s) - 90.0).abs() < 1e-6);
}

#[test]
fn zoom_defaults_to_one() {
    let s = BState::new();
    assert!((view_zoom(&s) - 1.0).abs() < 1e-6);
}

#[test]
fn zoom_is_clamped_on_read() {
    let mut s = BState::new();
    s.set(&zoom_key(), "999");
    assert!((view_zoom(&s) - ZOOM_MAX).abs() < 1e-6);
    s.set(&zoom_key(), "0.001");
    assert!((view_zoom(&s) - ZOOM_MIN).abs() < 1e-6);
}

#[test]
fn zoom_step_clamps_both_ends() {
    assert!((zoom_step(ZOOM_MAX, 2.0) - ZOOM_MAX).abs() < 1e-6);
    assert!((zoom_step(ZOOM_MIN, 0.5) - ZOOM_MIN).abs() < 1e-6);
    assert!((zoom_step(1.0, 1.5) - 1.5).abs() < 1e-6);
}

#[test]
fn mesh_type_maps_primitives() {
    use crate::phiphx::spatial::PhiQuiMeshType;
    assert_eq!(BShape::Cube.mesh_type(), PhiQuiMeshType::Cube);
    assert_eq!(BShape::Sphere.mesh_type(), PhiQuiMeshType::Sphere);
    assert_eq!(BShape::Gem.mesh_type(), PhiQuiMeshType::Icosahedron);
}

#[test]
fn state_keys_return_documented_segments() {
    assert_eq!(selected_key().segs.as_ref(), &["file", "selected"]);
    assert_eq!(loading_key().segs.as_ref(), &["scene", "loading"]);
    assert_eq!(error_key().segs.as_ref(), &["scene", "error"]);
    assert_eq!(loaded_path_key().segs.as_ref(), &["scene", "loaded_path"]);
}

#[test]
fn is_blend_path_accepts_blend_suffix() {
    assert!(is_blend_path("a.blend"));
    assert!(is_blend_path("A.BLEND"));
    assert!(is_blend_path("scene.Blend"));
    assert!(is_blend_path("/some/dir/model.blend"));
}

#[test]
fn is_blend_path_rejects_non_blend() {
    assert!(!is_blend_path("a.txt"));
    assert!(!is_blend_path("blend"));
    assert!(!is_blend_path(""));
    assert!(!is_blend_path("a.blende"));
    assert!(!is_blend_path("blend.txt"));
}
