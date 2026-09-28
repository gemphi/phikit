//! Unsaved changes, as Blender keeps `wm->file_saved`: an edit puts `* `
//! in the window's title, a save takes it off; while there are changes,
//! Quit, New File and Open first ask "Save changes before closing?" —
//! Cancel stays, Don't Save goes on, Save writes the file and goes on.

use gpui::{TestAppContext, px, size};
use phiber::phiphx::blend::BlendFile;
use phiber::phiphx::scene::SceneModel;
use phiber::v2::app::BPhiberApp;
use phiber::v2::state::{BKey as _, CtxKey, FileKey, ModalKey, OpKey};
use phiber::v2::view::BAction;
use std::path::PathBuf;

fn fixture_copy(dir: &std::path::Path) -> PathBuf {
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../phiphx/tests/fixtures/nla_cube.blend");
    let copy = dir.join("cube.blend");
    std::fs::copy(source, &copy).expect("copy the fixture");
    copy
}

fn frame_in(path: &std::path::Path) -> i32 {
    BlendFile::parse_bytes(&std::fs::read(path).unwrap()).unwrap().extract_scene().unwrap().props.frame_current
}

#[gpui::test]
async fn an_edit_marks_the_title_and_quit_new_and_open_ask_before_losing_it(cx: &mut TestAppContext) {
    let dir = std::env::temp_dir().join(format!("phiber_unsaved_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let blend = fixture_copy(&dir);
    let path = blend.to_string_lossy().to_string();
    let model = SceneModel::from_blend_owned(BlendFile::parse_bytes(&std::fs::read(&blend).unwrap()).unwrap().extract_scene().unwrap(), &path);

    let (app, cx) = cx.add_window_view(|_, _| BPhiberApp::new());
    cx.simulate_resize(size(px(1280.0), px(800.0)));
    cx.run_until_parked();
    assert_eq!(cx.window_title().as_deref(), Some("(Unsaved) - Phiber"), "nothing open yet");
    app.update(cx, |app, cx| {
        app.state.set(ModalKey::Open, "");
        app.state.set(&phiber::v2::scene::loaded_path_key(), path.clone());
        app.scene_cache = Some((path.clone(), std::sync::Arc::new(model)));
        cx.notify();
    });
    cx.run_until_parked();
    assert_eq!(cx.window_title().as_deref(), Some(format!("cube [{path}] - Phiber").as_str()), "the file's name and path");

    // An edit: the document has unsaved changes.
    app.update(cx, |app, cx| app.handle_action(&BAction::Set(CtxKey::Frame.path(), "40".into()), cx));
    cx.run_until_parked();
    assert_eq!(cx.window_title().as_deref(), Some(format!("* cube [{path}] - Phiber").as_str()), "`* ` while unsaved");
    let modal = |cx: &mut gpui::VisualTestContext| app.read_with(cx, |a, _| a.state.get(ModalKey::Open).map(str::to_string));

    // Ctrl Q: the dialog, not the end. Cancel stays, with the changes.
    cx.simulate_keystrokes("ctrl-q");
    cx.run_until_parked();
    assert_eq!(modal(cx).as_deref(), Some("file_close"));
    app.update(cx, |app, cx| app.handle_action(&BAction::Set(OpKey::FileClose.path(), "cancel".into()), cx));
    cx.run_until_parked();
    assert_eq!(modal(cx).as_deref(), Some(""));
    assert!(cx.window_title().unwrap().starts_with("* "), "still unsaved");

    // Open asks too; Save writes the file and goes on to the browser.
    app.update(cx, |app, cx| app.handle_action(&BAction::Set(FileKey::Purpose.path(), "open".into()), cx));
    cx.run_until_parked();
    assert_eq!(modal(cx).as_deref(), Some("file_close"), "asked before the browser opens");
    assert_eq!(app.read_with(cx, |a, _| a.state.get(&FileKey::Purpose.path()).map(str::to_string)), None, "the browser has not been asked for yet");
    app.update(cx, |app, cx| app.handle_action(&BAction::Set(OpKey::FileClose.path(), "save".into()), cx));
    cx.run_until_parked();
    assert_eq!(frame_in(&blend), 40, "saved");
    assert_eq!(modal(cx).as_deref(), Some("file_browser"), "then the browser, for the open");
    assert_eq!(app.read_with(cx, |a, _| a.state.get(&FileKey::Purpose.path()).map(str::to_string)), Some("open".into()));
    assert_eq!(cx.window_title().as_deref(), Some(format!("cube [{path}] - Phiber").as_str()), "saved: no `* `");

    // Once saved, Open does not ask again.
    app.update(cx, |app, cx| {
        app.state.set(ModalKey::Open, "");
        app.handle_action(&BAction::Set(FileKey::Purpose.path(), "open".into()), cx);
    });
    cx.run_until_parked();
    assert_eq!(modal(cx).as_deref(), Some("file_browser"));

    // Edit again; New File asks; Don't Save goes on to the new file.
    app.update(cx, |app, cx| {
        app.state.set(ModalKey::Open, "");
        app.handle_action(&BAction::Set(CtxKey::Frame.path(), "50".into()), cx);
        app.handle_action(&BAction::Set(OpKey::NewFile.path(), "Layout".into()), cx);
    });
    cx.run_until_parked();
    assert_eq!(modal(cx).as_deref(), Some("file_close"));
    assert!(app.read_with(cx, |a, _| a.scene_cache.as_ref().is_some_and(|(p, _)| *p == path)), "nothing replaced yet");
    app.update(cx, |app, cx| app.handle_action(&BAction::Set(OpKey::FileClose.path(), "discard".into()), cx));
    for _ in 0..50 {
        cx.run_until_parked();
        cx.executor().advance_clock(std::time::Duration::from_millis(50));
        if app.read_with(cx, |a, _| a.scene_cache.as_ref().is_some_and(|(_, m)| m.lights.iter().any(|l| l.name == "Light"))) {
            break;
        }
    }
    assert!(app.read_with(cx, |a, _| a.scene_cache.as_ref().is_some_and(|(_, m)| m.lights.iter().any(|l| l.name == "Light"))), "the new file is the factory scene");
    assert_eq!(frame_in(&blend), 40, "not written");
    cx.run_until_parked();
    assert_eq!(cx.window_title().as_deref(), Some("(Unsaved) - Phiber"), "a fresh, untitled document");

    // An untitled document with changes: Save from the dialog asks where,
    // and stops there (`wm_block_file_close_save`).
    app.update(cx, |app, cx| {
        app.handle_action(&BAction::Set(CtxKey::Frame.path(), "7".into()), cx);
        app.handle_action(&BAction::Set(OpKey::Quit.path(), String::new()), cx);
    });
    cx.run_until_parked();
    assert_eq!(modal(cx).as_deref(), Some("file_close"));
    app.update(cx, |app, cx| app.handle_action(&BAction::Set(OpKey::FileClose.path(), "save".into()), cx));
    cx.run_until_parked();
    assert_eq!(modal(cx).as_deref(), Some("file_browser"));
    assert_eq!(app.read_with(cx, |a, _| a.state.get(&FileKey::Purpose.path()).map(str::to_string)), Some("save_as".into()));
    let _ = std::fs::remove_dir_all(&dir);
}
