//! The Object tab's Transform panel moves the object: a Location, Rotation
//! or Scale typed in moves it in the viewport at once — its children with
//! it — and Ctrl S writes it (`Object.loc / rot / size`, `rotmode` XYZ).
//! The fixture is Blender's factory scene with Arm parented to the Cube
//! and Hand to Arm (`tests/fixtures/make_parented.py`); the Arm is a mesh,
//! so the viewport draws it following.

use gpui::{TestAppContext, px, size};
use phiber::phiphx::blend::BlendFile;
use phiber::phiphx::scene::SceneModel;
use phiber::v2::app::BPhiberApp;
use phiber::v2::scene::{BObject3D, scene_objects};
use phiber::v2::state::{BKey as _, ButsKey, CtxKey, ModalKey};
use phiber::v2::view::BAction;
use std::path::PathBuf;

fn world(objects: &[BObject3D], name: &str) -> [f32; 3] {
    match objects.iter().find(|o| o.name() == name).unwrap_or_else(|| panic!("{name}")) {
        BObject3D::Loaded { matrix_world, .. } => [matrix_world[3][0], matrix_world[3][1], matrix_world[3][2]],
        BObject3D::Primitive { .. } => panic!("{name} is a loaded object"),
    }
}

#[gpui::test]
async fn typing_a_location_moves_the_object_and_its_children_and_saves(cx: &mut TestAppContext) {
    let dir = std::env::temp_dir().join(format!("phiber_transform_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let blend = dir.join("parented.blend");
    std::fs::copy(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/parented.blend"), &blend).unwrap();
    let path = blend.to_string_lossy().to_string();
    let scene = BlendFile::parse_bytes(&std::fs::read(&blend).unwrap()).unwrap().extract_scene().unwrap();
    let model = SceneModel::from_blend_owned(scene, &path);
    let (app, cx) = cx.add_window_view(|_, _| BPhiberApp::new());
    cx.simulate_resize(size(px(1280.0), px(800.0)));
    app.update(cx, |app, cx| {
        app.state.set(ModalKey::Open, "");
        app.state.set(&phiber::v2::scene::loaded_path_key(), path.clone());
        app.scene_cache = Some((path.clone(), std::sync::Arc::new(model)));
        cx.notify();
    });
    cx.run_until_parked();
    // Picking the Cube fills the panel with its values.
    app.update(cx, |app, cx| app.handle_action(&BAction::Set(CtxKey::Object.path(), "Cube".into()), cx));
    cx.run_until_parked();
    let before = app.read_with(cx, |a, _| {
        let objects = scene_objects(&a.state);
        (world(&objects, "Cube"), world(&objects, "Arm"))
    });

    // Location X: 2 m further along.
    app.update(cx, |app, cx| app.handle_action(&BAction::Set(ButsKey::Axis("location", "x").path(), "2".into()), cx));
    cx.run_until_parked();
    let after = app.read_with(cx, |a, _| {
        let objects = scene_objects(&a.state);
        (world(&objects, "Cube"), world(&objects, "Arm"))
    });
    assert!((after.0[0] - 2.0).abs() < 1e-5, "the Cube stands at x = 2: {:?}", after.0);
    let (b, a) = (before.1, after.1);
    assert!((a[0] - b[0] - (2.0 - before.0[0])).abs() < 1e-4 && (a[1] - b[1]).abs() < 1e-5, "the Arm (its child) came along: {b:?} → {a:?}");

    // Picking another object and back shows the typed value.
    app.update(cx, |app, cx| {
        app.handle_action(&BAction::Set(CtxKey::Object.path(), "Light".into()), cx);
        app.handle_action(&BAction::Set(CtxKey::Object.path(), "Cube".into()), cx);
    });
    cx.run_until_parked();
    assert_eq!(app.read_with(cx, |a, _| a.state.get(ButsKey::Axis("location", "x")).map(str::to_string)).as_deref(), Some("2"));

    cx.simulate_keystrokes("ctrl-s");
    cx.run_until_parked();
    let saved = BlendFile::parse_bytes(&std::fs::read(&blend).unwrap()).unwrap().extract_scene().unwrap();
    let cube = saved.objects.iter().find(|o| o.name == "Cube").unwrap();
    assert!((cube.location[0] - 2.0).abs() < 1e-6, "saved: {:?}", cube.location);
    let _ = std::fs::remove_dir_all(&dir);
}
