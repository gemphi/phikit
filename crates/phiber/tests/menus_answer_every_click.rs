//! Every menu item does something. A leaf either runs an operator, opens a
//! dialog or the browser, flips a checkable, or says in the status bar that
//! it is not here yet — never the old silent close. The top bar's operators
//! run through the app the way a click runs them: New starts a clean file,
//! Undo takes an Add back and Redo brings it again, Export writes the file
//! the browser was pointed at, Frame All frames the scene, Show Status Bar
//! hides the bottom zone.

use gpui::{TestAppContext, px, size};
use phiber::v2::app::BPhiberApp;
use phiber::v2::host::BHostType;
use phiber::v2::part::menubar::item::{BMenuItem, BMenuKind, BMenuSpec};
use phiber::v2::part::menubar::tree::leaf_action;
use phiber::v2::scene::{OBJECTS, scene_objects};
use phiber::v2::state::{BKey as _, BState, CtxKey, FileKey, MenuKey, ModalKey, OpKey, ZoneKey, val::V_FALSE};
use phiber::v2::view::BAction;
use phiber::v2::zone::BZoneType;

fn walk(host: BHostType, trail: &mut Vec<&'static str>, items: &'static [BMenuItem], out: &mut Vec<(String, BAction)>) {
    for item in items {
        match item.kind {
            BMenuKind::Sep | BMenuKind::Recent => {}
            BMenuKind::Sub(subs) => {
                trail.push(item.label);
                walk(host, trail, subs, out);
                trail.pop();
            }
            _ => out.push((format!("{} ▸ {} ▸ {}", host.slug(), trail.join(" ▸ "), item.label), leaf_action(host.slug(), trail, item))),
        }
    }
}

fn every_leaf() -> Vec<(String, BAction)> {
    let mut out = Vec::new();
    for host in BHostType::ALL {
        for menu in BMenuSpec::list_for(*host) {
            walk(*host, &mut vec![menu.label], menu.items, &mut out);
        }
    }
    out
}

#[test]
fn no_menu_item_is_silent_and_most_run_an_operator() {
    let leaves = every_leaf();
    assert!(leaves.len() > 200, "{} leaves", leaves.len());
    let silent: Vec<&String> = leaves
        .iter()
        .filter(|(_, action)| matches!(action, BAction::Set(path, _) if MenuKey::owns(path)))
        .map(|(name, _)| name)
        .collect();
    assert!(silent.is_empty(), "these only close the menu:\n  {}", silent.iter().map(|s| s.as_str()).collect::<Vec<_>>().join("\n  "));
    let reports = |list: &[&(String, BAction)]| list.iter().filter(|(_, a)| matches!(a, BAction::Set(p, _) if *p == OpKey::Report.path())).count();
    let all: Vec<&(String, BAction)> = leaves.iter().collect();
    let reported = reports(&all);
    let working = leaves.len() - reported;
    println!("{working} of {} menu items run an operator, {reported} report themselves", leaves.len());
    assert!(working >= 120, "at least 120 items run something: {working}");
    // The top bar and the 3D viewport in particular: more work than report.
    for host in ["topbar", "view3d"] {
        let own: Vec<&(String, BAction)> = leaves.iter().filter(|(n, _)| n.starts_with(host)).collect();
        let own_reported = reports(&own);
        assert!(own_reported * 2 < own.len(), "{host} mostly works: {own_reported} of {} report", own.len());
    }
}

fn names(state: &BState) -> Vec<String> {
    scene_objects(state).iter().map(|o| o.name().to_string()).collect()
}

#[gpui::test]
async fn the_top_bars_operators_run_through_the_app(cx: &mut TestAppContext) {
    let (app, cx) = cx.add_window_view(|_, _| BPhiberApp::new());
    cx.simulate_resize(size(px(1280.0), px(800.0)));
    let run = |app: &gpui::Entity<BPhiberApp>, cx: &mut gpui::VisualTestContext, action: BAction| {
        app.update(cx, |app, cx| app.handle_action(&action, cx));
        cx.run_until_parked();
    };
    app.update(cx, |app, cx| {
        app.state.set(ModalKey::Open, "");
        cx.notify();
    });
    cx.run_until_parked();
    let default_count = OBJECTS.len();

    // Add ▸ Mesh ▸ Cube, then Undo, then Redo.
    run(&app, cx, BAction::Set(OpKey::SceneEdit.path(), "add:cube".into()));
    assert!(app.read_with(cx, |a, _| names(&a.state).contains(&"Cube.001".to_string())));
    run(&app, cx, BAction::Set(OpKey::Undo.path(), String::new()));
    assert_eq!(app.read_with(cx, |a, _| names(&a.state).len()), default_count, "undo took the cube back");
    run(&app, cx, BAction::Set(OpKey::Redo.path(), String::new()));
    assert!(app.read_with(cx, |a, _| names(&a.state).contains(&"Cube.001".to_string())), "redo brought it again");

    // Export ▸ Wavefront: the browser opens for the export, the commit writes.
    run(&app, cx, BAction::Set(FileKey::Purpose.path(), "export:obj".into()));
    assert_eq!(app.read_with(cx, |a, _| (a.state.get(FileKey::Purpose).map(str::to_string), a.state.get(OpKey::ExportFormat).map(str::to_string))), (Some("export".into()), Some("obj".into())));
    let dir = std::env::temp_dir().join(format!("phiber_menu_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let target = dir.join("scene.obj");
    run(&app, cx, BAction::Set(FileKey::Selected.path(), target.to_string_lossy().to_string()));
    let obj = std::fs::read_to_string(&target).expect("the export was written");
    assert!(obj.contains("o Cube_001") && obj.contains("o Sphere"), "every visible object, the added one too");
    assert!(app.read_with(cx, |a, _| a.state.get(OpKey::ExportFormat).is_none() && a.state.get(FileKey::Purpose).is_none()));
    let _ = std::fs::remove_dir_all(&dir);

    // View ▸ Frame All, Window ▸ Show Status Bar, Window ▸ Next Workspace.
    run(&app, cx, BAction::Set(OpKey::View3d.path(), "frame_all".into()));
    assert!(app.read_with(cx, |a, _| a.state.get(phiber::v2::state::View3dKey::Pivot).is_some()));
    run(&app, cx, BAction::Set(OpKey::Flip.path(), format!("!{}", ZoneKey::Shown(BZoneType::Bottom).path().encode())));
    assert_eq!(app.read_with(cx, |a, _| a.state.get(ZoneKey::Shown(BZoneType::Bottom)).map(str::to_string)), Some(V_FALSE.into()), "the status bar zone hides");
    let before = app.read_with(cx, |a, _| a.state.get(CtxKey::Space).map(str::to_string));
    run(&app, cx, BAction::Set(OpKey::SpaceStep.path(), "+1".into()));
    assert_ne!(app.read_with(cx, |a, _| a.state.get(CtxKey::Space).map(str::to_string)), before);

    // File ▸ New ▸ General: with the added cube unsaved it asks first; Don't
    // Save gives the default scene again, nothing added, nothing to undo.
    run(&app, cx, BAction::Set(OpKey::NewFile.path(), "Layout".into()));
    assert_eq!(app.read_with(cx, |a, _| a.state.get(phiber::v2::state::ModalKey::Open).map(str::to_string)), Some("file_close".into()), "the cube is unsaved: the dialog first");
    run(&app, cx, BAction::Set(OpKey::FileClose.path(), "discard".into()));
    assert_eq!(app.read_with(cx, |a, _| names(&a.state).len()), default_count);
    assert_eq!(app.read_with(cx, |a, _| a.state.get(CtxKey::Space).map(str::to_string)), Some("Layout".into()));
    run(&app, cx, BAction::Set(OpKey::Undo.path(), String::new()));
    assert_eq!(app.read_with(cx, |a, _| names(&a.state).len()), default_count, "a new file has no history");

    // An item without an operator says so.
    run(&app, cx, BAction::Set(OpKey::Report.path(), "File ▸ Save".into()));
    assert!(phiber::v2::state::log::entries().iter().any(|e| e.text == "File ▸ Save: not available yet"));
}
