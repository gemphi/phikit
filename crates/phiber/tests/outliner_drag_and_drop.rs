//! The Outliner's drag and drop: an object dragged onto a collection moves
//! there, one dragged onto another object is parented to it — the rows
//! show it at once (the child one level under its parent), a parent loop
//! is refused, and Ctrl S writes both, which Blender reads back
//! (`phiphx/tests/blender_opens_saves.rs` checks the writer in Blender).

use gpui::{Modifiers, MouseButton, TestAppContext, px, size};
use phiber::phiphx::blend::BlendFile;
use phiber::phiphx::scene::SceneModel;
use phiber::v2::app::BPhiberApp;
use phiber::v2::part::outliner::tree::{BRowKind, flatten};
use phiber::v2::state::{BKey as _, ModalKey, OpKey};
use phiber::v2::view::BAction;
use std::path::PathBuf;

fn drag(cx: &mut gpui::VisualTestContext, from: &str, onto: &str) {
    let a = cx.debug_bounds(Box::leak(format!("outliner-row-{from}").into_boxed_str())).unwrap_or_else(|| panic!("row {from}")).center();
    let b = cx.debug_bounds(Box::leak(format!("outliner-row-{onto}").into_boxed_str())).unwrap_or_else(|| panic!("row {onto}")).center();
    cx.simulate_mouse_down(a, MouseButton::Left, Modifiers::none());
    cx.simulate_mouse_move(gpui::Point::new(a.x, a.y + px(6.0)), Some(MouseButton::Left), Modifiers::none());
    cx.simulate_mouse_move(b, Some(MouseButton::Left), Modifiers::none());
    cx.simulate_mouse_up(b, MouseButton::Left, Modifiers::none());
    cx.run_until_parked();
}

#[gpui::test]
async fn dragging_rows_moves_and_parents_objects_and_the_save_keeps_it(cx: &mut TestAppContext) {
    let dir = std::env::temp_dir().join(format!("phiber_outliner_dnd_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let blend = dir.join("startup.blend");
    std::fs::copy(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets/blend/startup-5.blend"), &blend).unwrap();
    let path = blend.to_string_lossy().to_string();
    let scene = BlendFile::parse_bytes(&std::fs::read(&blend).unwrap()).unwrap().extract_scene().unwrap();
    let model = SceneModel::from_blend_owned(scene, &path);
    let (app, cx) = cx.add_window_view(|_, _| BPhiberApp::new());
    cx.simulate_resize(size(px(1400.0), px(900.0)));
    app.update(cx, |app, cx| {
        app.state.set(ModalKey::Open, "");
        app.state.set(&phiber::v2::scene::loaded_path_key(), path.clone());
        app.scene_cache = Some((path.clone(), std::sync::Arc::new(model)));
        cx.notify();
    });
    cx.run_until_parked();

    // The Light out of "Collection", into the Scene Collection.
    drag(cx, "ob:Light", "col:Scene Collection");
    let rows = app.read_with(cx, |a, _| flatten(&a.state));
    let light = rows.iter().find(|r| r.label == "Light").expect("the Light row");
    assert_eq!(light.depth, 1, "straight under the Scene Collection now");

    // The Camera onto the Cube: parented, one level under it once it opens.
    drag(cx, "ob:Camera", "ob:Cube");
    app.update(cx, |app, cx| {
        app.state.set(phiber::v2::state::OutlinerKey::Expanded("Cube"), "true");
        cx.notify();
    });
    cx.run_until_parked();
    let rows = app.read_with(cx, |a, _| flatten(&a.state));
    let (cube, camera) = (rows.iter().position(|r| r.label == "Cube" && r.kind == BRowKind::Object).unwrap(), rows.iter().position(|r| r.label == "Camera").unwrap());
    assert!(camera > cube && rows[camera].depth == rows[cube].depth + 1, "the Camera sits under the Cube");

    // The Cube onto its own child: a loop, refused.
    app.update(cx, |app, cx| app.handle_action(&BAction::Set(OpKey::OutlinerDrop.path(), "ob:Cube>ob:Camera".into()), cx));
    cx.run_until_parked();
    let parented = app.read_with(cx, |a, _| phiber::v2::scene::links::BSceneLinks::now(&a.state).parented);
    assert_eq!(parented, [("Camera".to_string(), "Cube".to_string())]);

    // Ctrl S: the file has both.
    cx.simulate_keystrokes("ctrl-s");
    cx.run_until_parked();
    let saved = BlendFile::parse_bytes(&std::fs::read(&blend).unwrap()).unwrap().extract_scene().unwrap();
    let find = |name: &str| saved.objects.iter().find(|o| o.name == name).unwrap_or_else(|| panic!("{name} saved"));
    assert_eq!(find("Camera").parent_address, find("Cube").block_address, "the Camera's parent is the Cube");
    let root = &saved.collections[0];
    assert!(root.object_names.contains(&"Light".to_string()), "the Light is in the Scene Collection: {:?}", root.object_names);
    assert!(!root.children[0].object_names.contains(&"Light".to_string()), "and not in Collection");
    let _ = std::fs::remove_dir_all(&dir);
}
