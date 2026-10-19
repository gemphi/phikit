//! Tests for `mod` — split out so the module itself sits under the
//! 240-line read cap (`tests/level_structure.rs`).

use super::*;

#[test]
fn the_panel_values_make_blenders_matrix() {
    let t = BTransform { location: [1.0, 2.0, 3.0], rotation: [0.0, 0.0, 90.0], scale: [2.0, 2.0, 2.0] };
    let m = t.matrix();
    // 90° about Z doubled: X lands on +2Y; then moved.
    assert!((m.transform_vector3(Vec3::X) - Vec3::new(0.0, 2.0, 0.0)).length() < 1e-5);
    assert!((m.w_axis.truncate() - Vec3::new(1.0, 2.0, 3.0)).length() < 1e-6);
    // XYZ order: X first, then Y, then Z.
    let xyz = BTransform { location: [0.0; 3], rotation: [90.0, 0.0, 90.0], scale: [1.0; 3] }.matrix();
    assert!((xyz.transform_vector3(Vec3::Y) - Vec3::new(0.0, 0.0, 1.0)).length() < 1e-5, "Y → Z by X, stays Z under Z");
    let mut s = BState::new();
    BTransforms { list: vec![("Cube".into(), t)] }.write(&mut s);
    assert_eq!(BTransforms::now(&s).get("Cube"), Some(t));
}

#[test]
fn clear_sends_a_channel_back_to_identity_keeping_the_others() {
    use crate::phiphx::scene::{SceneCorners, SceneMesh, SceneModel, SceneNormals, SceneObject};
    use crate::v2::state::action::apply_action;
    use crate::v2::view::prim::BAction;

    let mesh = SceneMesh {
        vertices: vec![[0.0; 3]],
        indices: Vec::new(),
        normals: SceneNormals::PerFace(Vec::new()),
        corners: SceneCorners::default(),
    };
    let mut model = SceneModel { objects: vec![SceneObject::identity("Cube", mesh)], ..Default::default() };
    model.selection.selected = vec!["Cube".into()];
    crate::v2::scene::source::set_loaded_scene(Some(model));

    let mut s = BState::new();
    let posed = BTransform { location: [5.0, 1.0, -2.0], rotation: [0.0, 0.0, 90.0], scale: [2.0, 2.0, 2.0] };
    BTransforms { list: vec![("Cube".into(), posed)] }.write(&mut s);

    let clear = |s: &mut BState, which: &str| apply_action(s, &BAction::Set(OpKey::ObjectClear.path(), which.to_string()));

    clear(&mut s, "location");
    let after = BTransforms::now(&s).get("Cube").unwrap();
    assert_eq!(after.location, [0.0; 3], "location cleared");
    assert_eq!((after.rotation, after.scale), (posed.rotation, posed.scale), "rotation and scale kept");

    clear(&mut s, "rotation");
    assert_eq!(BTransforms::now(&s).get("Cube").unwrap().rotation, [0.0; 3], "rotation cleared");

    clear(&mut s, "scale");
    assert_eq!(BTransforms::now(&s).get("Cube").unwrap().scale, [1.0; 3], "scale back to one");

    crate::v2::scene::source::clear_loaded_scene();
}
