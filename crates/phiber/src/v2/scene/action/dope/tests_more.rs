//! Second half of `dope/tests.rs` — split so each file sits under the
//! 240-line read cap (`tests/level_structure.rs`).

//! Over phiphx's `fixtures/actions.blend` (`make_actions.py`): the Cube,
//! selected and active, moves by CubeAction's location keys at 1 and 20.

use super::*;
use crate::v2::scene::action::BSceneActions;
use crate::v2::state::action::apply_action;
use crate::v2::state::{BKey as _, OpKey};
use crate::v2::view::prim::BAction;

fn model() -> SceneModel {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../phiphx/tests/fixtures/actions.blend");
    let scene = crate::phiphx::blend::BlendFile::parse_bytes(&std::fs::read(&path).unwrap()).unwrap().extract_scene().unwrap();
    SceneModel::from_blend_owned(scene, &path.to_string_lossy())
}

fn op(s: &mut BState, value: &str) {
    apply_action(s, &BAction::Set(OpKey::Keys.path(), value.to_string()));
}

fn picks(s: &BState) -> Vec<(String, f32)> {
    BKeySelection::now(s).0.into_iter().map(|p| (p.object, p.frame)).collect()
}

fn cube_x(s: &BState, model: &SceneModel) -> Vec<f32> {
    let actions = BSceneActions::now(s).keyed_actions(model);
    let action = actions.iter().find(|a| a.name == "CubeAction").unwrap();
    action.curves.iter().find(|c| c.path == "location" && c.index == 0).map_or(Vec::new(), |c| c.keys.iter().map(|k| k.co[0]).collect())
}

#[test]
fn mirror_reflects_the_picked_columns_over_a_pivot() {
    let model = model();
    crate::v2::scene::source::set_loaded_scene(Some(model.clone()));
    let mut s = BState::new();
    s.set(CtxKey::Object, "Cube");
    s.set(CtxKey::Frame, "10");
    op(&mut s, "click:1:20");
    // Over the current frame (10): 20 reflects to 2·10 − 20 = 0.
    op(&mut s, "mirror:current");
    assert_eq!(cube_x(&s, &model), [0.0, 1.0], "the key at 20 reflects to 0");
    assert_eq!(picks(&s), [("Cube".to_string(), 0.0)], "the pick follows the reflection");
    crate::v2::scene::source::set_loaded_scene(None);
}

#[test]
fn duplicate_copies_the_picked_column_at_the_next_free_frame_and_the_picks_follow() {
    let model = model();
    crate::v2::scene::source::set_loaded_scene(Some(model.clone()));
    let mut s = BState::new();
    s.set(CtxKey::Object, "Cube");
    // Nothing picked: nothing duplicates.
    op(&mut s, "duplicate");
    assert!(BSceneActions::now(&s).keyed.is_empty(), "no selection, no duplicate");
    op(&mut s, "click:1:20");
    op(&mut s, "duplicate");
    // Frame 1 and 20 are taken; the copy lands on 21.
    assert_eq!(cube_x(&s, &model), [1.0, 20.0, 21.0], "the copy lands on frame 21");
    assert_eq!(picks(&s), [("Cube".to_string(), 21.0)], "the picks follow to the copy");
    // The curves' value at the copy matches Blender evaluating the originals
    // there (the key was read with `evaluate(20.0)`).
    let actions = BSceneActions::now(&s).keyed_actions(&model);
    let a = actions.iter().find(|a| a.name == "CubeAction").unwrap();
    let x = a.curves.iter().find(|c| c.path == "location" && c.index == 0).unwrap();
    assert!((x.evaluate(21.0) - x.evaluate(20.0)).abs() < 1e-6, "the copy's x matches the source's");
    // A second duplicate on the same (now-moved) pick skips the frame the
    // first batch just filled.
    op(&mut s, "duplicate");
    assert_eq!(cube_x(&s, &model), [1.0, 20.0, 21.0, 22.0], "the second copy lands on 22");
    assert_eq!(picks(&s), [("Cube".to_string(), 22.0)]);
    crate::v2::scene::source::set_loaded_scene(None);
}

#[test]
fn clean_simplifies_the_selected_keys_and_clean_channels_drops_a_flat_curve() {
    // Over the fixture's CubeAction: a key inserted midway at the value the
    // curve already holds there says nothing, and Clean takes it off again
    // (`ACTION_OT_clean`). Duplicate puts a copy at 21 with frame 20's exact
    // value, so 21 repeats what 20 says and is what Clean should remove.
    let model = model();
    crate::v2::scene::source::set_loaded_scene(Some(model.clone()));
    let mut s = BState::new();
    s.set(CtxKey::Object, "Cube");
    // Nothing picked: Clean Keyframes reports and changes nothing.
    op(&mut s, "clean:keys");
    assert!(BSceneActions::now(&s).keyed.is_empty(), "no selection, no clean");
    op(&mut s, "click:1:20");
    op(&mut s, "duplicate");
    assert_eq!(cube_x(&s, &model), [1.0, 20.0, 21.0], "the copy is in");
    // `all` reads the model in scope, which the app keeps edited
    // (`set_loaded_at` → `BActionScene::over`) but this test scopes raw — so
    // here it adds the file's own columns to the duplicate's own pick rather
    // than re-finding the copy. Either way the three columns end up picked.
    op(&mut s, "all");
    assert_eq!(picks(&s).len(), 3, "1, 20 and the copy at 21");
    op(&mut s, "clean:keys");
    assert_eq!(cube_x(&s, &model), [1.0, 20.0], "the copy said nothing and is gone");
    // The picks follow: the column that is no longer keyed stops being picked.
    assert!(!picks(&s).iter().any(|(_, f)| (*f - 21.0).abs() < 5e-4), "the dropped column is unpicked");
    assert!(picks(&s).iter().any(|(_, f)| (*f - 1.0).abs() < 5e-4), "the kept ones stay");
    crate::v2::scene::source::set_loaded_scene(None);
}

#[test]
fn clean_channels_needs_no_selection_and_reaches_every_shown_object() {
    // `clean_action_keys` filters by selection only when `channels` is set,
    // and `only_selected_keys = !channels` — so Clean Channels walks every
    // key of every shown object with nothing picked at all.
    let model = model();
    crate::v2::scene::source::set_loaded_scene(Some(model.clone()));
    let mut s = BState::new();
    s.set(CtxKey::Object, "Cube");
    op(&mut s, "click:none");
    assert!(picks(&s).is_empty());
    op(&mut s, "clean:channels");
    let steps = BSceneActions::now(&s).keyed;
    assert_eq!(steps.len(), 1, "one step for the Cube's (action, slot)");
    let crate::v2::scene::action::retime::BKeyStep::Clean(c) = &steps[0] else { panic!("a clean step") };
    assert!(c.channels && c.frames.is_empty(), "every key, not a selection");
    assert_eq!((c.action.as_str(), c.slot.as_str()), ("CubeAction", "OBCube"));
    crate::v2::scene::source::set_loaded_scene(None);
}

#[test]
fn jump_moves_the_playhead_to_the_nearest_selected_key() {
    let model = model();
    crate::v2::scene::source::set_loaded_scene(Some(model.clone()));
    let mut s = BState::new();
    s.set(CtxKey::Object, "Cube");
    s.set(CtxKey::Frame, "15");
    op(&mut s, "jump");
    assert_eq!(s.get(CtxKey::Frame), Some("15"), "nothing selected, the playhead stays");
    op(&mut s, "all");
    op(&mut s, "jump");
    assert_eq!(s.get(CtxKey::Frame), Some("20"), "20 is nearer 15 than 1 is");
    // No key was edited by the jump.
    assert!(BSceneActions::now(&s).keyed.is_empty());
    crate::v2::scene::source::set_loaded_scene(None);
}

#[test]
fn delete_takes_the_picked_columns_off() {
    let model = model();
    crate::v2::scene::source::set_loaded_scene(Some(model.clone()));
    let mut s = BState::new();
    s.set(CtxKey::Object, "Cube");
    op(&mut s, "delete");
    assert!(BSceneActions::now(&s).keyed.is_empty(), "nothing picked, nothing deleted");
    op(&mut s, "click:1:1");
    op(&mut s, "delete");
    assert_eq!(cube_x(&s, &model), [20.0]);
    assert!(picks(&s).is_empty());
    crate::v2::scene::source::set_loaded_scene(None);
}
