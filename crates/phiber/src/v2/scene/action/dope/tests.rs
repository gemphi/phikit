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
fn an_object_s_keys_are_its_slot_s_columns() {
    let model = model();
    let cube = BKeyTarget::of(&model.actions, "Cube");
    assert_eq!(cube.len(), 1);
    assert_eq!((cube[0].action.as_str(), cube[0].slot.as_str(), cube[0].columns.as_slice()), ("CubeAction", "OBCube", &[1.0, 20.0][..]));
    assert_eq!(BKeyTarget::columns(&model.actions, "Camera"), [1.0, 40.0]);
    let mut s = BState::new();
    s.set(CtxKey::Object, "Cube");
    assert_eq!(BKeyTarget::shown(&s, &model), ["Cube"], "the Camera is neither selected nor active");
}

#[test]
fn clicks_boxes_and_the_select_menu_pick_key_columns() {
    let model = model();
    crate::v2::scene::source::set_loaded_scene(Some(model));
    let mut s = BState::new();
    s.set(CtxKey::Object, "Cube");
    op(&mut s, "click:1:20");
    assert_eq!(picks(&s), [("Cube".to_string(), 20.0)]);
    op(&mut s, "click+:0:1");
    assert_eq!(picks(&s).len(), 2, "Shift adds the summary's column");
    op(&mut s, "click+:1:20");
    assert_eq!(picks(&s), [("Cube".to_string(), 1.0)], "and flips a picked one off");
    op(&mut s, "click:none");
    assert!(picks(&s).is_empty());
    op(&mut s, "all");
    assert_eq!(picks(&s).len(), 2);
    op(&mut s, "invert");
    assert!(picks(&s).is_empty());
    op(&mut s, "box:15,25,0,1");
    assert_eq!(picks(&s), [("Cube".to_string(), 20.0)], "the box spans frame 20 only");
    op(&mut s, "box:-5,25,1,1");
    assert_eq!(picks(&s).len(), 2, "grown to frame 1");
    op(&mut s, "box:end");
    assert!(s.get(CtxKey::KeyBox).is_none());
    crate::v2::scene::source::set_loaded_scene(None);
}

#[test]
fn a_move_follows_the_pointer_from_where_it_began_and_escape_puts_it_back() {
    let model = model();
    crate::v2::scene::source::set_loaded_scene(Some(model.clone()));
    let mut s = BState::new();
    s.set(CtxKey::Object, "Cube");
    op(&mut s, "click:1:20");
    op(&mut s, "grab:start");
    assert!(BKeyGrab::active(&s));
    op(&mut s, "grab:at:100");
    op(&mut s, "grab:at:104");
    op(&mut s, "grab:at:110");
    assert_eq!(cube_x(&s, &model), [1.0, 30.0], "ten frames on, not fourteen");
    // Blender 5.0.1 opens the move saved and stands the Cube at x 2.7632 at
    // frame 25 (`truth/blender_dope_keys.py`), its handles as phiber left them.
    let actions = BSceneActions::now(&s).keyed_actions(&model);
    let x = actions.iter().find(|a| a.name == "CubeAction").and_then(|a| a.curves.iter().find(|c| c.path == "location" && c.index == 0)).unwrap();
    assert!((x.evaluate(25.0) - 2.7632).abs() < 1e-4, "{}", x.evaluate(25.0));
    assert_eq!(picks(&s), [("Cube".to_string(), 30.0)]);
    assert_eq!(BSceneActions::now(&s).keyed.len(), 1, "one step, replaced as it moves");
    op(&mut s, "grab:cancel");
    assert_eq!((cube_x(&s, &model), picks(&s)), (vec![1.0, 20.0], vec![("Cube".to_string(), 20.0)]));
    op(&mut s, "grab:start:20");
    op(&mut s, "grab:at:25");
    op(&mut s, "grab:confirm");
    assert!(!BKeyGrab::active(&s));
    assert_eq!(cube_x(&s, &model), [1.0, 25.0]);
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
