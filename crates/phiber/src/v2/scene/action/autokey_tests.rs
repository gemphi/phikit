//! Against Blender 5.0.1 (`bpy`, the factory scene, auto keying on,
//! `transform.translate / rotate / resize` at frame 10): see the module.

use super::*;
use crate::phiphx::blend::BlendFile;
use crate::phiphx::scene::SceneModel;
use crate::v2::scene::action::retime::BKeyStep;
use crate::v2::state::CtxKey;
use crate::v2::state::val::{BValue, V_TRUE};

fn factory() -> SceneModel {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets/blend/startup-5.blend");
    let scene = BlendFile::parse_bytes(&std::fs::read(&path).unwrap()).unwrap().extract_scene().unwrap();
    SceneModel::from_blend_owned(scene, &path.to_string_lossy())
}

fn state(mode: &str) -> BState {
    let mut s = BState::new();
    s.set(CtxKey::AutoKey, V_TRUE);
    s.set(&crate::v2::part::playback::settings::key(crate::v2::part::playback::settings::AUTO_KEY_MODE_KEY), BValue::Enum(mode.into()).encode());
    s.set(CtxKey::Frame, "10");
    s
}

/// Every curve of the Cube's slot: `(path, index, keyed frames and values)`.
fn curves(s: &BState, model: &SceneModel) -> Vec<(String, u32, Vec<(f32, f32)>)> {
    let actions = BSceneActions::now(s).keyed_actions(model);
    let Some(action) = actions.iter().find(|a| a.name == "CubeAction") else { return Vec::new() };
    action.curves.iter().map(|c| (c.path.clone(), c.index, c.keys.iter().map(|k| (k.co[0], (k.co[1] * 1000.0).round() / 1000.0)).collect())).collect()
}

fn moved(s: &mut BState, mode: BTransformMode, by: [f32; 3]) {
    let mut t = BTransforms::now(s);
    let mut e = t.get("Cube").unwrap_or(crate::v2::scene::transform::BTransform { location: [0.0; 3], rotation: [0.0; 3], scale: [1.0; 3] });
    match mode {
        BTransformMode::Translate => e.location = [e.location[0] + by[0], e.location[1] + by[1], e.location[2] + by[2]],
        BTransformMode::Rotate => e.rotation = [e.rotation[0] + by[0], e.rotation[1] + by[1], e.rotation[2] + by[2]],
        BTransformMode::Resize => e.scale = [e.scale[0] * by[0], e.scale[1] * by[1], e.scale[2] * by[2]],
    }
    t.set("Cube", e);
    t.write(s);
}

#[test]
fn add_and_replace_keys_the_transform_s_channels_only() {
    let model = factory();
    crate::v2::scene::source::set_loaded_scene(Some(model.clone()));
    let mut s = state("ADD_REPLACE_KEYS");
    moved(&mut s, BTransformMode::Translate, [1.5, 0.0, 0.0]);
    assert_eq!(BAutoKey::transformed(&mut s, BTransformMode::Translate, &["Cube".into()]), ["Cube"]);
    // Blender: location[0..3] at (10, 1.5), (10, 0), (10, 0).
    let got = curves(&s, &model);
    assert_eq!(got, [("location".to_string(), 0, vec![(10.0, 1.5)]), ("location".to_string(), 1, vec![(10.0, 0.0)]), ("location".to_string(), 2, vec![(10.0, 0.0)])]);
    assert!(BTransforms::now(&s).get("Cube").is_none(), "its curves drive it now");
    // R about Z by 0.5 rad at frame 10: rotation_euler, the three of them
    // (an edit's rotation is in degrees).
    moved(&mut s, BTransformMode::Rotate, [0.0, 0.0, -0.5f32.to_degrees()]);
    BAutoKey::transformed(&mut s, BTransformMode::Rotate, &["Cube".into()]);
    let got = curves(&s, &model);
    assert_eq!(got.len(), 6, "{got:?}");
    assert_eq!(got[5], ("rotation_euler".to_string(), 2, vec![(10.0, -0.5)]));
    // S at 20: scale, the three.
    s.set(CtxKey::Frame, "20");
    moved(&mut s, BTransformMode::Resize, [2.0, 2.0, 2.0]);
    BAutoKey::transformed(&mut s, BTransformMode::Resize, &["Cube".into()]);
    let got = curves(&s, &model);
    assert_eq!(got.len(), 9);
    assert_eq!(got[6], ("scale".to_string(), 0, vec![(20.0, 2.0)]));
    crate::v2::scene::source::set_loaded_scene(None);
}

#[test]
fn only_insert_needed_keys_the_channel_that_changed() {
    let model = factory();
    crate::v2::scene::source::set_loaded_scene(Some(model.clone()));
    let mut s = state("ADD_REPLACE_KEYS");
    moved(&mut s, BTransformMode::Translate, [1.5, 2.0, 0.0]);
    BAutoKey::transformed(&mut s, BTransformMode::Translate, &["Cube".into()]);
    // Frame 20, along X alone (G starts from where the curves hold it, 1.5,
    // 2, 0): Blender keys location[0] at (20, 2.5) and nothing else, since Y
    // and Z already hold their values.
    s.set(CtxKey::Frame, "20");
    moved(&mut s, BTransformMode::Translate, [2.5, 2.0, 0.0]);
    BAutoKey::transformed(&mut s, BTransformMode::Translate, &["Cube".into()]);
    let got = curves(&s, &model);
    assert_eq!(got[0], ("location".to_string(), 0, vec![(10.0, 1.5), (20.0, 2.5)]));
    assert_eq!(got[1].2, [(10.0, 2.0)], "Y was not keyed again");
    assert_eq!(got[2].2, [(10.0, 0.0)]);
    crate::v2::scene::source::set_loaded_scene(None);
}

#[test]
fn replace_only_replaces_a_key_standing_on_the_frame() {
    let model = factory();
    crate::v2::scene::source::set_loaded_scene(Some(model.clone()));
    let mut s = state("REPLACE_KEYS");
    moved(&mut s, BTransformMode::Translate, [1.5, 0.0, 0.0]);
    assert!(BAutoKey::transformed(&mut s, BTransformMode::Translate, &["Cube".into()]).is_empty(), "no curve: nothing keyed");
    assert!(curves(&s, &model).is_empty());
    assert_eq!(BTransforms::now(&s).get("Cube").map(|t| t.location), Some([1.5, 0.0, 0.0]), "the move stays an edit");
    // A key on the frame: replaced.
    s.set(CtxKey::AutoKey, crate::v2::state::val::V_FALSE);
    let mut edits = BSceneActions::now(&s);
    let (action, slot) = edits.keyed_slot("Cube", &model);
    edits.keyed.push(BKeyStep::Insert(super::super::keys::BKeyInsert { action, slot, frame: 10.0, channels: vec![super::super::keys::BKeyed { path: "location".into(), index: 0, value: 0.0 }] }));
    edits.write(&mut s);
    s.set(CtxKey::AutoKey, V_TRUE);
    BAutoKey::transformed(&mut s, BTransformMode::Translate, &["Cube".into()]);
    assert_eq!(curves(&s, &model), [("location".to_string(), 0, vec![(10.0, 1.5)])], "X replaced; Y and Z have no key to replace");
    crate::v2::scene::source::set_loaded_scene(None);
}

#[test]
fn off_nothing_is_keyed() {
    let model = factory();
    crate::v2::scene::source::set_loaded_scene(Some(model.clone()));
    let mut s = BState::new();
    moved(&mut s, BTransformMode::Translate, [1.0, 0.0, 0.0]);
    assert!(BAutoKey::transformed(&mut s, BTransformMode::Translate, &["Cube".into()]).is_empty());
    assert!(BAutoKey::now(&s).is_none());
    crate::v2::scene::source::set_loaded_scene(None);
}
