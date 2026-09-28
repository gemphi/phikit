//! G / R / S the rest of the way Blender has them: a number typed drives
//! the transform (G Z 1.5 Return, R Z 90 Return, S 2 Return), Shift X / Y /
//! Z holds it to the plane of the other two, and while it runs the viewport
//! reads out how far it went and what it is held to. Over the factory scene
//! with the Arm parented to the Cube (`make_parented.py`).

use gpui::{Modifiers, TestAppContext, point, px, size};
use phiber::phiphx::blend::BlendFile;
use phiber::phiphx::scene::SceneModel;
use phiber::v2::app::BPhiberApp;
use phiber::v2::scene::transform::BTransforms;
use phiber::v2::scene::xform::{BTransformModal, BTransformMode};
use phiber::v2::scene::{BObject3D, scene_objects};
use phiber::v2::state::{BKey as _, BState, CtxKey, ModalKey};
use phiber::v2::view::BAction;
use glam::Vec3;
use std::path::PathBuf;

fn at(s: &BState, name: &str) -> Vec3 {
    match scene_objects(s).iter().find(|o| o.name() == name).unwrap_or_else(|| panic!("{name}")) {
        BObject3D::Loaded { matrix_world, .. } => Vec3::new(matrix_world[3][0], matrix_world[3][1], matrix_world[3][2]),
        BObject3D::Primitive { .. } => panic!("{name} is a loaded object"),
    }
}

async fn scene(cx: &mut TestAppContext) -> (gpui::Entity<BPhiberApp>, &mut gpui::VisualTestContext) {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/parented.blend");
    let scene = BlendFile::parse_bytes(&std::fs::read(&path).unwrap()).unwrap().extract_scene().unwrap();
    let key = path.to_string_lossy().to_string();
    let model = SceneModel::from_blend_owned(scene, &key);
    let (app, cx) = cx.add_window_view(|_, _| BPhiberApp::new());
    cx.simulate_resize(size(px(1280.0), px(800.0)));
    app.update(cx, |app, cx| {
        app.state.set(ModalKey::Open, "");
        app.state.set(&phiber::v2::scene::loaded_path_key(), key.clone());
        app.scene_cache = Some((key, std::sync::Arc::new(model)));
        cx.notify();
    });
    cx.run_until_parked();
    app.update(cx, |app, cx| app.handle_action(&BAction::Set(CtxKey::Object.path(), "Cube".into()), cx));
    cx.run_until_parked();
    // The pointer over the viewport: the keys are the viewport's.
    let centre = cx.debug_bounds("view3d-canvas").expect("the viewport").center();
    cx.simulate_mouse_move(centre, None, Modifiers::none());
    cx.run_until_parked();
    (app, cx)
}

fn cube(app: &gpui::Entity<BPhiberApp>, cx: &mut gpui::VisualTestContext) -> phiber::v2::scene::transform::BTransform {
    app.read_with(cx, |a, _| BTransforms::now(&a.state).get("Cube")).expect("the Cube's edit")
}

#[gpui::test]
async fn a_number_typed_drives_the_transform(cx: &mut TestAppContext) {
    let (app, cx) = scene(cx).await;
    let cube0 = app.read_with(cx, |a, _| at(&a.state, "Cube"));

    // G Z 1.5 Return: one and a half metres up.
    cx.simulate_keystrokes("g z 1 . 5");
    cx.run_until_parked();
    let readout = app.read_with(cx, |a, _| BTransformModal::now(&a.state).map(|m| m.readout(&a.state))).expect("under way");
    assert_eq!(readout, "D: [1.5|] along Z", "the header's read-out");
    assert!(cx.debug_bounds("view3d-transform-readout").is_some(), "shown in the viewport");
    // The pointer moving does not take over from the number.
    let centre = cx.debug_bounds("view3d-canvas").unwrap().center();
    cx.simulate_mouse_move(point(centre.x + px(90.0), centre.y), None, Modifiers::none());
    cx.run_until_parked();
    cx.simulate_keystrokes("enter");
    cx.run_until_parked();
    let cube1 = app.read_with(cx, |a, _| at(&a.state, "Cube"));
    assert!((cube1 - cube0 - Vec3::new(0.0, 0.0, 1.5)).length() < 1e-4, "{cube0} → {cube1}");
    assert!(cx.debug_bounds("view3d-transform-readout").is_none(), "the read-out goes with it");

    // R Z 90 Return: a quarter turn about Z.
    cx.simulate_keystrokes("r z 9 0 enter");
    cx.run_until_parked();
    assert!((cube(&app, cx).rotation[2] - 90.0).abs() < 1e-3, "{:?}", cube(&app, cx).rotation);

    // S 2 Return: twice the size.
    cx.simulate_keystrokes("s 2 enter");
    cx.run_until_parked();
    assert!(cube(&app, cx).scale.iter().all(|s| (s - 2.0).abs() < 1e-3), "{:?}", cube(&app, cx).scale);

    // Backspace takes a character back: G 3 Backspace 1 Return moves one metre along X.
    let before = app.read_with(cx, |a, _| at(&a.state, "Cube"));
    cx.simulate_keystrokes("g 3 backspace 1 enter");
    cx.run_until_parked();
    let after = app.read_with(cx, |a, _| at(&a.state, "Cube"));
    assert!((after - before - Vec3::X).length() < 1e-4, "free, a typed distance runs along X: {before} → {after}");
}

#[gpui::test]
async fn shift_and_an_axis_holds_it_to_the_plane_of_the_other_two(cx: &mut TestAppContext) {
    let (app, cx) = scene(cx).await;
    let cube0 = app.read_with(cx, |a, _| at(&a.state, "Cube"));
    let centre = cx.debug_bounds("view3d-canvas").unwrap().center();
    cx.simulate_keystrokes("g shift-z");
    cx.run_until_parked();
    let modal = app.read_with(cx, |a, _| BTransformModal::now(&a.state)).expect("under way");
    assert_eq!(modal.constraint, phiber::v2::scene::xform::BConstraint::Plane(2));
    for (dx, dy) in [(0.0, 0.0), (70.0, -50.0)] {
        cx.simulate_mouse_move(point(centre.x + px(dx), centre.y + px(dy)), None, Modifiers::none());
        cx.run_until_parked();
    }
    let cube1 = app.read_with(cx, |a, _| at(&a.state, "Cube"));
    assert!((cube1.z - cube0.z).abs() < 1e-4, "Z is locked: {cube1}");
    assert!((cube1 - cube0).length() > 0.1, "it moves in the X-Y plane: {cube0} → {cube1}");
    let readout = app.read_with(cx, |a, _| BTransformModal::now(&a.state).unwrap().readout(&a.state));
    assert!(readout.starts_with("D: ") && readout.ends_with(" along X and Y"), "{readout}");
    assert!(app.read_with(cx, |a, _| BTransformModal::now(&a.state).unwrap().guide()).is_none(), "a move draws no guide");

    // R shows its guide from the pivot to the pointer once the pointer moves.
    cx.simulate_keystrokes("r");
    cx.run_until_parked();
    cx.simulate_mouse_move(point(centre.x + px(40.0), centre.y), None, Modifiers::none());
    cx.run_until_parked();
    let modal = app.read_with(cx, |a, _| BTransformModal::now(&a.state)).expect("still under way");
    assert_eq!(modal.mode, BTransformMode::Rotate, "R switches the mode");
    assert!(modal.guide().is_some(), "R draws its guide");
    cx.simulate_keystrokes("escape");
    cx.run_until_parked();
    assert!((app.read_with(cx, |a, _| at(&a.state, "Cube")) - cube0).length() < 1e-5, "Escape puts it back");
}
