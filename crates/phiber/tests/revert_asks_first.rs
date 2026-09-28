//! File ▸ Revert asks first — `wm_revert_mainfile_invoke`'s "Revert to the
//! Saved File" / "Any unsaved changes will be lost." — and only Revert
//! reads the file again; Cancel leaves the document as it is.

use std::path::PathBuf;

use gpui::{TestAppContext, px, size};
use phiber::phiphx::blend::BlendFile;
use phiber::phiphx::scene::SceneModel;
use phiber::v2::app::BPhiberApp;
use phiber::v2::modal::revert;
use phiber::v2::state::{BKey as _, CtxKey, ModalKey, OpKey, log};
use phiber::v2::view::BAction;

#[gpui::test]
async fn revert_asks_and_cancel_keeps_the_edits(cx: &mut TestAppContext) {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets/blend/startup-5.blend");
    let path = path.to_string_lossy().to_string();
    let scene = BlendFile::parse_bytes(&std::fs::read(&path).unwrap()).unwrap().extract_scene().unwrap();
    let model = SceneModel::from_blend_owned(scene, &path);
    let (app, cx) = cx.add_window_view(|_, _| BPhiberApp::new());
    cx.simulate_resize(size(px(1280.0), px(800.0)));
    app.update(cx, |app, cx| {
        app.state.set(ModalKey::Open, "");
        app.state.set(&phiber::v2::scene::loaded_path_key(), path.clone());
        app.scene_cache = Some((path.clone(), std::sync::Arc::new(model)));
        app.state.set(CtxKey::Frame, "42");
        app.handle_action(&BAction::Set(OpKey::Revert.path(), String::new()), cx);
    });
    cx.run_until_parked();
    assert_eq!(app.read_with(cx, |a, _| a.state.get(ModalKey::Open).map(str::to_string)), Some("revert".into()), "asked first");

    app.update(cx, |app, cx| app.handle_action(&BAction::Set(ModalKey::Open.path(), String::new()), cx));
    cx.run_until_parked();
    assert_eq!(app.read_with(cx, |a, _| a.state.get(CtxKey::Frame).map(str::to_string)), Some("42".into()), "Cancel: nothing read again");
    assert!(!log::entries().iter().any(|e| e.text == "Reverting"));

    app.update(cx, |app, cx| {
        app.handle_action(&BAction::Set(OpKey::Revert.path(), String::new()), cx);
        app.handle_action(&BAction::Set(OpKey::Revert.path(), revert::CONFIRMED.into()), cx);
    });
    cx.run_until_parked();
    assert_ne!(app.read_with(cx, |a, _| a.state.get(ModalKey::Open).map(str::to_string)), Some("revert".into()), "answered: the dialog is gone");
    assert!(log::entries().iter().any(|e| e.text == "Reverting"), "Revert reads the file again");
}
