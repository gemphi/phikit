//! A camera or light the Transform panel or G / R / S moved is where the
//! camera view, the rendered lights and F12 look from: the render pass
//! scopes the edits with the frame (`motion::set_moved`), so every reader
//! without state in hand — the tracer's lamps, the camera view, a final
//! render's camera — sees the moved one. Over the factory scene with the
//! Light parented to the Cube (`tests/fixtures/make_parented.py`).

use gpui::{TestAppContext, px, size};
use phiber::phiphx::blend::BlendFile;
use phiber::phiphx::scene::SceneModel;
use phiber::v2::app::BPhiberApp;
use phiber::v2::scene::motion::{active_camera_at, scoped_frame};
use phiber::v2::scene::source::with_loaded;
use phiber::v2::scene::transform::{BTransform, BTransforms};
use phiber::v2::scene::{scene_cameras, scene_lights};
use phiber::v2::state::{BKey as _, ModalKey, SceneKey};
use phiber::v2::view::BAction;
use std::path::PathBuf;

fn close(a: [f32; 3], b: [f32; 3]) -> bool {
    a.iter().zip(b).all(|(x, y)| (x - y).abs() < 1e-3)
}

#[gpui::test]
async fn the_camera_view_the_lamps_and_f12_follow_the_edits(cx: &mut TestAppContext) {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/parented.blend");
    let scene = BlendFile::parse_bytes(&std::fs::read(&path).unwrap()).unwrap().extract_scene().unwrap();
    let key = path.to_string_lossy().to_string();
    let model = SceneModel::from_blend_owned(scene, &key);
    let light0 = model.lights[0].location;
    let (app, cx) = cx.add_window_view(|_, _| BPhiberApp::new());
    cx.simulate_resize(size(px(1280.0), px(800.0)));
    app.update(cx, |app, cx| {
        app.state.set(ModalKey::Open, "");
        app.state.set(&phiber::v2::scene::loaded_path_key(), key.clone());
        app.scene_cache = Some((key, std::sync::Arc::new(model)));
        cx.notify();
    });
    cx.run_until_parked();
    assert!(close(scene_cameras()[0].location, [7.359, -6.926, 4.958]), "the file's camera first");

    // The camera moved and turned to look along +Y; the Cube (the Light's
    // parent) two metres along X.
    let edits = BTransforms {
        list: vec![
            ("Camera".into(), BTransform { location: [0.0, -10.0, 1.0], rotation: [90.0, 0.0, 0.0], scale: [1.0; 3] }),
            ("Cube".into(), BTransform { location: [2.0, 0.0, 0.0], rotation: [0.0; 3], scale: [1.0; 3] }),
        ],
    };
    app.update(cx, |app, cx| app.handle_action(&BAction::Set(SceneKey::Transformed.path(), edits.encode()), cx));
    cx.run_until_parked();

    let camera = scene_cameras().into_iter().next().expect("a camera");
    assert!(close(camera.location, [0.0, -10.0, 1.0]), "the camera stands where it was moved: {:?}", camera.location);
    // It looks down its −Z: turned 90° about X, that is +Y.
    let look = [-camera.matrix_world[2][0], -camera.matrix_world[2][1], -camera.matrix_world[2][2]];
    assert!(close(look, [0.0, 1.0, 0.0]), "{look:?}");
    // The camera view and F12 look through the same one.
    let through = with_loaded(|m| active_camera_at(m.unwrap(), scoped_frame())).expect("the active camera");
    assert!(close(through.location, [0.0, -10.0, 1.0]));
    // F12's camera: the eye (`pivot + back · dist`) is the moved camera.
    let render = with_loaded(|m| {
        let m = m.unwrap();
        let (w, h) = phiber::v2::scene::render_size(m);
        phiber::v2::scene::camera_of(&active_camera_at(m, scoped_frame()).unwrap(), w, h)
    });
    let eye: [f32; 3] = std::array::from_fn(|k| render.pivot[k] + render.back[k] * render.dist);
    assert!(close(eye, [0.0, -10.0, 1.0]), "the render's eye: {eye:?}");

    // The Light follows its parent for the tracer too.
    let light = scene_lights().into_iter().next().expect("a light");
    assert!(close(light.location, [light0[0] + 2.0, light0[1], light0[2]]), "{:?} vs {light0:?}", light.location);

    // Undo the edits: back where the file put them.
    app.update(cx, |app, cx| app.handle_action(&BAction::Set(SceneKey::Transformed.path(), String::new()), cx));
    cx.run_until_parked();
    assert!(close(scene_lights()[0].location, light0));
    assert!(close(scene_cameras()[0].location, [7.359, -6.926, 4.958]));
}
