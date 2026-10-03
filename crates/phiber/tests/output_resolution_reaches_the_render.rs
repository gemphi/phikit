//! Output ▸ Format ▸ Resolution X, Y and % as typed in the panel are what
//! F12 renders, what the camera's frame in the viewport is shaped by, and
//! what Ctrl S writes (`Scene.r.xsch / ysch / size`) — they used to be the
//! file's whatever the panel said.

use gpui::{TestAppContext, px, size};
use phiber::phiphx::blend::BlendFile;
use phiber::phiphx::scene::SceneModel;
use phiber::v2::app::BPhiberApp;
use phiber::v2::scene::render_set::BResolution;
use phiber::v2::state::val::BValue;
use phiber::v2::state::{BKey as _, ButsKey, CtxKey, ModalKey, OpKey};
use phiber::v2::view::BAction;
use std::path::PathBuf;

fn model_of(path: &std::path::Path) -> SceneModel {
    let scene = BlendFile::parse_bytes(&std::fs::read(path).unwrap()).unwrap().extract_scene().unwrap();
    SceneModel::from_blend_owned(scene, &path.to_string_lossy())
}

#[gpui::test]
async fn the_panel_s_resolution_is_rendered_framed_and_saved(cx: &mut TestAppContext) {
    let dir = std::env::temp_dir().join(format!("phiber_resolution_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let blend = dir.join("cube.blend");
    // The factory scene, camera and all (`make_parented.py`).
    std::fs::copy(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/parented.blend"), &blend).unwrap();
    let model = model_of(&blend);
    let file = (model.render.resolution_x, model.render.resolution_y, model.render.resolution_percent);

    let (app, cx) = cx.add_window_view(|_, _| BPhiberApp::new());
    cx.simulate_resize(size(px(1280.0), px(800.0)));
    app.update(cx, |app, cx| {
        app.state.set(ModalKey::Open, "");
        app.state.set(&phiber::v2::scene::loaded_path_key(), blend.to_string_lossy().to_string());
        phiber::v2::app::seed::seed_output_props(&mut app.state, &model);
        app.scene_cache = Some((blend.to_string_lossy().to_string(), std::sync::Arc::new(model)));
        app.state.set(CtxKey::Object, "Cube");
        cx.notify();
    });
    cx.run_until_parked();
    assert_eq!(BResolution::scoped().x, Some(file.0 as u32), "seeded from the file");

    // Portrait, half size, one sample.
    let set = |key: &'static str, v: i64| BAction::Set(ButsKey::Value(key).path(), BValue::Int(v).encode());
    app.update(cx, |app, cx| {
        for action in [set("resolution_x", 300), set("resolution_y", 400), set("resolution_percentage", 50), set("cycles.samples", 1)] {
            app.handle_action(&action, cx);
        }
    });
    cx.run_until_parked();
    let scoped = BResolution::scoped();
    assert_eq!((scoped.x, scoped.y, scoped.percent), (Some(300), Some(400), Some(50)), "the camera's frame takes it");

    // Render ▸ Render Image (F12).
    app.update(cx, |app, cx| app.handle_action(&BAction::Set(OpKey::Render.path(), "image".into()), cx));
    cx.run_until_parked();
    let shot = phiber::v2::scene::render_job::shot().0.expect("F12 started a render");
    assert_eq!((shot.w, shot.h), (150, 200), "F12 renders the panel's size, not the file's {file:?}");
    phiber::v2::scene::render_job::cancel();
    app.update(cx, |app, _| app.state.set(ModalKey::Open, ""));
    cx.run_until_parked();

    cx.simulate_keystrokes("ctrl-s");
    cx.run_until_parked();
    let saved = model_of(&blend);
    assert_eq!((saved.render.resolution_x, saved.render.resolution_y, saved.render.resolution_percent), (300, 400, 50), "Ctrl S wrote it");
    let _ = std::fs::remove_dir_all(&dir);
}
