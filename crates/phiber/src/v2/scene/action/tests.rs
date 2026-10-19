//! Tests for `mod` — split out so the module itself sits under the
//! 240-line read cap (`tests/level_structure.rs`).

use super::*;

#[test]
fn the_edits_round_trip_through_their_keys_and_names_copy_as_blender_does() {
    let mut s = BState::new();
    let mut edits = BSceneActions::default();
    edits.assign("Cube", "CubeAction.001", "OBCube");
    edits.assign("Camera", "", "");
    edits.made.push(("CubeAction.001".into(), "CubeAction".into()));
    edits.set_range("CubeAction.001", Some((5.0, 30.0)), true);
    edits.set_range("Spin", None, false);
    edits.write(&mut s);
    assert_eq!(BSceneActions::now(&s), edits);
    let taken = vec!["CubeAction".to_string(), "CubeAction.001".to_string()];
    assert_eq!(BSceneActions::unique("CubeAction", &taken), "CubeAction.002");
    assert_eq!(BSceneActions::unique("CubeAction.001", &taken), "CubeAction.002");
    assert_eq!(BSceneActions::unique("Action", &taken), "Action");
}
