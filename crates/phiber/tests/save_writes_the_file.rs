//! Ctrl S writes the opened `.blend` back with the session's edits and
//! reports it as Blender does; Save As moves the document to the new path
//! and Save Copy does not; an added object is in the saved file with its
//! mesh.

use gpui::{TestAppContext, px, size};
use phiber::phiphx::blend::BlendFile;
use phiber::phiphx::scene::SceneModel;
use phiber::v2::app::BPhiberApp;
use phiber::v2::part::file::purpose::BFilePurpose;
use phiber::v2::state::{BKey as _, CtxKey, FileKey, ModalKey, OpKey};
use phiber::v2::view::BAction;
use std::path::PathBuf;

fn fixture_copy(dir: &std::path::Path) -> PathBuf {
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../phiphx/tests/fixtures/nla_cube.blend");
    let copy = dir.join("cube.blend");
    std::fs::copy(source, &copy).expect("copy the fixture");
    copy
}

fn read_scene(path: &std::path::Path) -> phiber::phiphx::blend::BlendScene {
    BlendFile::parse_bytes(&std::fs::read(path).unwrap()).unwrap().extract_scene().unwrap()
}

#[gpui::test]
async fn ctrl_s_saves_the_edits_save_as_moves_the_document_and_save_copy_does_not(cx: &mut TestAppContext) {
    let dir = std::env::temp_dir().join(format!("phiber_save_app_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let blend = fixture_copy(&dir);
    let model = SceneModel::from_blend_owned(read_scene(&blend), &blend.to_string_lossy());

    let (app, cx) = cx.add_window_view(|_, _| BPhiberApp::new());
    cx.simulate_resize(size(px(1280.0), px(800.0)));
    app.update(cx, |app, cx| {
        app.state.set(ModalKey::Open, "");
        app.state.set(&phiber::v2::scene::loaded_path_key(), blend.to_string_lossy().to_string());
        app.scene_cache = Some((blend.to_string_lossy().to_string(), std::sync::Arc::new(model)));
        app.state.set(CtxKey::Object, "Cube");
        cx.notify();
    });
    cx.run_until_parked();

    // Edit: the frame, a rename, a marker, an added object.
    app.update(cx, |app, cx| {
        app.handle_action(&BAction::Set(CtxKey::Frame.path(), "33".into()), cx);
        app.handle_action(&BAction::Set(OpKey::Marker.path(), "add".into()), cx);
        app.handle_action(&BAction::Set(OpKey::SceneEdit.path(), "add:cube".into()), cx);
        app.state.set(CtxKey::Object, "Cube");
        app.handle_action(&BAction::Set(OpKey::Rename.path(), String::new()), cx);
        app.handle_action(&BAction::Set(phiber::v2::state::AppKey::RenameText.path(), "Crate".into()), cx);
    });
    cx.run_until_parked();

    // Ctrl S.
    cx.simulate_keystrokes("ctrl-s");
    cx.run_until_parked();
    let log = app.read_with(cx, |a, _| phiber::v2::part::text::info::reports(&a.state).iter().map(|(level, text)| format!("{level}: {text}")).collect::<Vec<_>>().join("\n"));
    let saved = read_scene(&blend);
    assert_eq!(saved.props.frame_current, 33, "the frame was written; log:\n{log}");
    assert!(saved.objects.iter().any(|o| o.name == "Crate"), "the rename was written");
    assert_eq!(saved.props.markers.len(), 1, "the marker was written");
    assert!(dir.join("cube.blend1").is_file(), "the file before the save is kept as .blend1");
    assert!(log.contains("Saved \"cube.blend\""), "{log}");
    let names: Vec<(String, usize)> = saved.objects.iter().map(|o| (o.name.clone(), o.vertices.len())).collect();
    assert!(saved.objects.iter().any(|o| o.name.starts_with("Cube") && o.name != "Crate" && o.vertices.len() >= 8), "the added cube is written with its mesh: {names:?}");
    assert!(!log.contains("Warning"), "nothing was left out: {log}");

    // Save Copy: written beside, the document stays.
    app.update(cx, |app, cx| {
        app.handle_file_commit(&dir.join("copy").to_string_lossy(), BFilePurpose::SaveCopy, cx);
    });
    assert!(dir.join("copy.blend").is_file(), "the copy gets the extension it lacked");
    assert_eq!(app.read_with(cx, |a, _| a.state.get(&phiber::v2::scene::loaded_path_key()).map(str::to_string)), Some(blend.to_string_lossy().to_string()));

    // Save As: the document moves.
    app.update(cx, |app, cx| {
        app.handle_file_commit(&dir.join("moved.blend").to_string_lossy(), BFilePurpose::SaveAs, cx);
    });
    assert!(dir.join("moved.blend").is_file());
    assert_eq!(
        app.read_with(cx, |a, _| a.state.get(&phiber::v2::scene::loaded_path_key()).map(str::to_string)),
        Some(dir.join("moved.blend").to_string_lossy().to_string())
    );
    // Save Incremental from there: moved1.blend, and the document follows.
    app.update(cx, |app, cx| app.handle_action(&BAction::Set(OpKey::Save.path(), "incremental".into()), cx));
    assert!(dir.join("moved1.blend").is_file());
    let _ = FileKey::Purpose;
    let _ = std::fs::remove_dir_all(&dir);
}

/// New File reads the startup file as an untitled document — Blender's
/// cube, light and camera — so Ctrl S asks where, and Save As writes a
/// file with them in it.
#[gpui::test]
async fn a_new_file_is_the_startup_scene_and_saves_through_it(cx: &mut TestAppContext) {
    let dir = std::env::temp_dir().join(format!("phiber_new_file_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let (app, cx) = cx.add_window_view(|_, _| BPhiberApp::new());
    cx.simulate_resize(size(px(1280.0), px(800.0)));
    app.update(cx, |app, cx| {
        app.state.set(ModalKey::Open, "");
        app.handle_action(&BAction::Set(OpKey::NewFile.path(), "Layout".into()), cx);
    });
    for _ in 0..50 {
        cx.run_until_parked();
        cx.executor().advance_clock(std::time::Duration::from_millis(50));
        if app.read_with(cx, |a, _| a.scene_cache.as_ref().is_some_and(|(_, m)| m.objects.iter().any(|o| o.name == "Cube"))) {
            break;
        }
    }
    let names = app.read_with(cx, |a, _| a.scene_cache.as_ref().map(|(_, m)| m.objects.iter().map(|o| o.name.clone()).chain(m.cameras.iter().map(|c| c.name.clone())).chain(m.lights.iter().map(|l| l.name.clone())).collect::<Vec<_>>()).unwrap_or_default());
    assert!(names.contains(&"Cube".to_string()) && names.contains(&"Camera".to_string()) && names.contains(&"Light".to_string()), "{names:?}");
    assert_eq!(app.read_with(cx, |a, _| a.state.get(&phiber::v2::scene::loaded_path_key()).map(str::to_string)), None, "untitled");
    assert_eq!(app.read_with(cx, |a, _| a.scene_cache.as_ref().map(|(_, m)| m.report.file_version.clone())), Some("5.0".into()), "nothing was open: the newest layout");

    // Ctrl S on an untitled file asks where.
    cx.simulate_keystrokes("ctrl-s");
    cx.run_until_parked();
    assert_eq!(app.read_with(cx, |a, _| a.state.get(ModalKey::Open).map(str::to_string)), Some("file_browser".into()));
    assert_eq!(app.read_with(cx, |a, _| a.state.get(&FileKey::Purpose.path()).map(str::to_string)), Some("save_as".into()));

    // Save As: a file with the cube in it, and the document now has a path.
    let target = dir.join("fresh.blend");
    app.update(cx, |app, cx| app.handle_file_commit(&target.to_string_lossy(), BFilePurpose::SaveAs, cx));
    assert!(target.is_file());
    let saved = read_scene(&target);
    assert!(saved.objects.iter().any(|o| o.name == "Cube" && o.vertices.len() == 8), "{:?}", saved.objects.iter().map(|o| &o.name).collect::<Vec<_>>());
    assert_eq!(app.read_with(cx, |a, _| a.state.get(&phiber::v2::scene::loaded_path_key()).map(str::to_string)), Some(target.to_string_lossy().to_string()));
    let _ = std::fs::remove_dir_all(&dir);
}

/// The startup file New File reads has the layout of the Blender the open
/// document came from — a 3.6 pipeline stays 3.6 — or of the major the
/// operator names; either way the file Save As writes is that version's.
#[gpui::test]
async fn a_new_file_takes_the_open_documents_version_or_the_one_named(cx: &mut TestAppContext) {
    let dir = std::env::temp_dir().join(format!("phiber_new_file_version_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let (app, cx) = cx.add_window_view(|_, _| BPhiberApp::new());
    cx.simulate_resize(size(px(1280.0), px(800.0)));
    let version_after_new = |spec: &str, open: Option<&str>, cx: &mut gpui::VisualTestContext| -> (String, Vec<String>) {
        app.update(cx, |app, cx| {
            app.state.set(ModalKey::Open, "");
            // What is open now, as far as New File looks: its version.
            app.scene_cache = open.map(|v| {
                let mut model = SceneModel::default();
                model.report.file_version = v.to_string();
                ("open.blend".to_string(), std::sync::Arc::new(model))
            });
            app.handle_action(&BAction::Set(OpKey::NewFile.path(), spec.into()), cx);
        });
        for _ in 0..50 {
            cx.run_until_parked();
            cx.executor().advance_clock(std::time::Duration::from_millis(50));
            if app.read_with(cx, |a, _| a.scene_cache.as_ref().is_some_and(|(_, m)| m.objects.iter().any(|o| o.name == "Cube"))) {
                break;
            }
        }
        let version = app.read_with(cx, |a, _| a.scene_cache.as_ref().map(|(_, m)| m.report.file_version.clone()).unwrap_or_default());
        let names = app.read_with(cx, |a, _| a.scene_cache.as_ref().map(|(_, m)| m.objects.iter().map(|o| o.name.clone()).chain(m.cameras.iter().map(|c| c.name.clone())).chain(m.lights.iter().map(|l| l.name.clone())).collect::<Vec<_>>()).unwrap_or_default());
        (version, names)
    };
    // A 3.6 document open: the 3.6 factory scene.
    let (version, names) = version_after_new("Layout", Some("3.6"), cx);
    assert_eq!(version, "3.6", "{names:?}");
    assert!(names.contains(&"Cube".to_string()) && names.contains(&"Camera".to_string()) && names.contains(&"Light".to_string()), "{names:?}");
    // …and Save As writes a 3.6 file with the cube in it.
    let target = dir.join("three.blend");
    app.update(cx, |app, cx| app.handle_file_commit(&target.to_string_lossy(), BFilePurpose::SaveAs, cx));
    let file = BlendFile::parse_bytes(&std::fs::read(&target).unwrap()).unwrap();
    assert_eq!(file.header.version_label(), "3.6");
    assert!(file.extract_scene().unwrap().objects.iter().any(|o| o.name == "Cube" && o.vertices.len() == 8));
    // A 4.x document open: 4.2's.
    let (version, _) = version_after_new("Layout", Some("4.5"), cx);
    assert_eq!(version, "4.2");
    // A named major wins over what is open; the log says which layout.
    let (version, _) = version_after_new("Layout:5", Some("3.6"), cx);
    assert_eq!(version, "5.0");
    let log: Vec<String> = phiber::v2::state::log::entries().iter().map(|e| e.text.clone()).collect();
    assert!(log.iter().any(|l| l == "New file (Layout, Blender 5.0 layout)"), "{log:?}");
    let _ = std::fs::remove_dir_all(&dir);
}
