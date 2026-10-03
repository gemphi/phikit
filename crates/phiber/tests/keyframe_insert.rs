//! I inserts keyframes as Blender 5.0 does. Over the 3D Viewport
//! (`anim.keyframe_insert`, Object ▸ Animation ▸ Insert Keyframe) it keys
//! the selected objects' Location, Rotation and Scale where they stand — an
//! object with no action gets `<Object>Action` and the slot named for it —
//! and from then on the animation drives them; over the Dope Sheet
//! (`action.keyframe_insert`, Key ▸ Insert Keyframes) it keys every curve
//! they have at its value now. Keys go in with Blender's handles
//! (`BFCurve::insert`), Undo steps them, and Ctrl S writes them
//! (`phiphx/tests/blender_opens_saves.rs` has Blender read the same
//! writes). Over the factory scene, whose Cube has no animation.

use gpui::{Modifiers, TestAppContext, px, size};
use phiber::phiphx::blend::BlendFile;
use phiber::phiphx::scene::{SceneAction, SceneModel};
use phiber::v2::app::BPhiberApp;
use phiber::v2::scene::action::BSceneActions;
use phiber::v2::scene::transform::{BTransform, BTransforms};
use phiber::v2::scene::{BObject3D, scene_objects};
use phiber::v2::space::BSpaceType;
use phiber::v2::state::{BKey as _, CtxKey, ModalKey};
use phiber::v2::view::BAction;
use std::path::PathBuf;

type Cx<'a> = &'a mut gpui::VisualTestContext;

fn edits(app: &gpui::Entity<BPhiberApp>, cx: Cx) -> BSceneActions {
    app.read_with(cx, |a, _| BSceneActions::now(&a.state))
}

fn frame(app: &gpui::Entity<BPhiberApp>, cx: Cx, frame: i32) {
    app.update(cx, |app, cx| {
        app.state.set(CtxKey::Frame, frame.to_string());
        cx.notify();
    });
    cx.run_until_parked();
}

/// I with the pointer over the window called `over`.
fn press_i(cx: Cx, over: &'static str) {
    let at = cx.debug_bounds(over).unwrap_or_else(|| panic!("{over}")).center();
    cx.simulate_mouse_move(at, None, Modifiers::none());
    cx.run_until_parked();
    cx.simulate_keystrokes("i");
    cx.run_until_parked();
}

/// The Cube's action as the editors read it.
fn cube_action(cx: Cx) -> SceneAction {
    cx.run_until_parked();
    phiber::v2::scene::source::with_loaded(|m| m.and_then(|m| m.actions.iter().find(|a| a.name == "CubeAction").cloned())).expect("CubeAction")
}

fn cube_x(app: &gpui::Entity<BPhiberApp>, cx: Cx) -> f32 {
    app.read_with(cx, |a, _| match scene_objects(&a.state).iter().find(|o| o.name() == "Cube") {
        Some(BObject3D::Loaded { matrix_world, .. }) => matrix_world[3][0],
        _ => panic!("the Cube"),
    })
}

#[gpui::test]
async fn i_keys_the_cube_where_it_stands_and_its_animation_then_drives_it(cx: &mut TestAppContext) {
    let dir = std::env::temp_dir().join(format!("phiber_keyframe_insert_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let blend = dir.join("startup.blend");
    std::fs::copy(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets/blend/startup-5.blend"), &blend).unwrap();
    let path = blend.to_string_lossy().to_string();
    let scene = BlendFile::parse_bytes(&std::fs::read(&blend).unwrap()).unwrap().extract_scene().unwrap();
    let model = SceneModel::from_blend_owned(scene, &path);
    let (app, cx) = cx.add_window_view(|_, _| BPhiberApp::new());
    cx.simulate_resize(size(px(1600.0), px(900.0)));
    app.update(cx, |app, cx| {
        app.state.set(ModalKey::Open, "");
        app.state.set(CtxKey::Space, BSpaceType::Animation.slug());
        app.state.set(&phiber::v2::scene::loaded_path_key(), path.clone());
        app.scene_cache = Some((path.clone(), std::sync::Arc::new(model)));
        cx.notify();
    });
    cx.run_until_parked();
    app.update(cx, |app, cx| app.handle_action(&BAction::Set(CtxKey::Object.path(), "Cube".into()), cx));
    frame(&app, cx, 1);

    // Frame 1 over the viewport: CubeAction, the Cube's slot, nine channels.
    press_i(cx, "view3d-canvas");
    let e = edits(&app, cx);
    assert_eq!(e.made, [("CubeAction".to_string(), String::new())], "id_action_ensure: <Object>Action");
    assert_eq!((e.assigned[0].action.as_str(), e.assigned[0].slot.as_str()), ("CubeAction", "OBCube"));
    let phiber::v2::scene::action::retime::BKeyStep::Insert(first) = &e.keyed[0] else { panic!("an insert") };
    let channels: Vec<String> = first.channels.iter().map(|c| format!("{}[{}]={}", c.path, c.index, c.value)).collect();
    assert_eq!(channels, ["location[0]=0", "location[1]=0", "location[2]=0", "rotation_euler[0]=0", "rotation_euler[1]=0", "rotation_euler[2]=0", "scale[0]=1", "scale[1]=1", "scale[2]=1"]);
    let said = phiber::v2::state::log::latest().map(|e| e.text.clone()).unwrap_or_default();
    assert_eq!(said, "Keyed Cube at frame 1");
    let action = cube_action(cx);
    assert_eq!((action.users.as_slice(), action.curves.len(), action.keys.as_slice()), (&["Cube".to_string()][..], 9, &[1.0][..]));

    // Moved to x 3 (the Transform panel), frame 20, I: the move is keyed
    // and handed to the animation.
    let moved = BTransform { location: [3.0, 0.0, 0.0], rotation: [0.0; 3], scale: [1.0; 3] };
    app.update(cx, |app, cx| {
        BTransforms { list: vec![("Cube".into(), moved)] }.write(&mut app.state);
        cx.notify();
    });
    frame(&app, cx, 20);
    press_i(cx, "view3d-canvas");
    assert!(app.read_with(cx, |a, _| BTransforms::now(&a.state).get("Cube").is_none()), "the animation drives it now");
    let action = cube_action(cx);
    let x = action.curves.iter().find(|c| c.path == "location" && c.index == 0).cloned().expect("X");
    assert_eq!(x.keys.iter().map(|k| k.co).collect::<Vec<_>>(), [[1.0, 0.0], [20.0, 3.0]]);
    // Between the keys it stands where Blender's curve puts it.
    frame(&app, cx, 10);
    let want = x.evaluate(10.0);
    assert!(want > 0.0 && want < 3.0 && (cube_x(&app, cx) - want).abs() < 1e-4, "at frame 10: {} vs {want}", cube_x(&app, cx));

    // Frame 30 over the Dope Sheet: every curve keyed at its value there.
    frame(&app, cx, 30);
    press_i(cx, "time-window");
    let action = cube_action(cx);
    assert_eq!(action.keys, [1.0, 20.0, 30.0]);
    assert!(action.curves.iter().all(|c| c.keys.len() == 3), "all nine curves");
    let x = action.curves.iter().find(|c| c.path == "location" && c.index == 0).cloned().unwrap();
    assert_eq!(x.keys[2].co, [30.0, 3.0], "held at 3 past its last key");

    // Undo steps it; I again puts it back.
    cx.simulate_keystrokes("ctrl-z");
    cx.run_until_parked();
    assert_eq!(edits(&app, cx).keyed.len(), 2, "Undo takes the Dope Sheet's keys off");
    press_i(cx, "time-window");
    assert_eq!(edits(&app, cx).keyed.len(), 3);

    // Ctrl S: the file has CubeAction on the Cube through its slot, the
    // nine curves with their keys and handles as phiber has them.
    cx.simulate_keystrokes("ctrl-s");
    cx.run_until_parked();
    let saved = BlendFile::parse_bytes(&std::fs::read(&blend).unwrap()).unwrap().extract_scene().unwrap();
    let written = saved.props.actions.iter().find(|a| a.name == "CubeAction").expect("CubeAction saved");
    assert_eq!(written.users, ["Cube"]);
    let slot = written.bindings.first().and_then(|(_, h)| written.slots.iter().find(|s| s.handle == *h)).expect("bound through a slot");
    assert_eq!(slot.identifier, "OBCube");
    let ours = cube_action(cx);
    assert_eq!(written.curves.len(), 9);
    for (w, o) in written.curves.iter().zip(&ours.curves) {
        assert_eq!((w.path.as_str(), w.index), (o.path.as_str(), o.index));
        for (a, b) in w.keys.iter().zip(&o.keys) {
            assert!(a.co == b.co && a.h1 == b.h1 && a.h2 == b.h2, "{}[{}]: {a:?} vs {b:?}", w.path, w.index);
        }
    }
    let _ = std::fs::remove_dir_all(&dir);
}
