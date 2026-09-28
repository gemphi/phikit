//! The Action Editor's action selector, as Blender 5.0 draws it
//! (`DOPESHEET_HT_editor_buttons._draw_action_selector`): the browse menu
//! lists every action in the file with the object's ticked; picking one
//! animates the object with it through the slot named for it; New copies
//! the action in hand (`Spin` → `Spin.001`) or, with none, makes an empty
//! `Action` with no slot; Unlink takes it off. The sidebar's Action panel
//! sets its Manual Frame Range and Cyclic, Properties ▸ Object has the same
//! selector under Animation, Undo steps it all, and Ctrl S writes it
//! (`phiphx/tests/blender_opens_saves.rs` has Blender read the same
//! writes). Over the factory scene with CubeAction, CameraAction and an
//! unused Spin (`phiphx/tests/fixtures/make_actions.py`).

use gpui::{Modifiers, TestAppContext, px, size};
use phiber::phiphx::blend::BlendFile;
use phiber::phiphx::scene::SceneModel;
use phiber::v2::app::BPhiberApp;
use phiber::v2::host::BHostType;
use phiber::v2::part::modebar::active_mode;
use phiber::v2::part::modebar::set::mode_key;
use phiber::v2::scene::action::BSceneActions;
use phiber::v2::scene::{BObject3D, scene_objects};
use phiber::v2::space::BSpaceType;
use phiber::v2::state::{BKey as _, ButsKey, CtxKey, ModalKey, OpKey};
use phiber::v2::view::BAction;
use std::path::PathBuf;

fn click(cx: &mut gpui::VisualTestContext, id: &str) {
    let at = cx.debug_bounds(Box::leak(id.to_string().into_boxed_str())).unwrap_or_else(|| panic!("{id}")).center();
    cx.simulate_click(at, Modifiers::none());
    cx.run_until_parked();
}

fn shown(cx: &mut gpui::VisualTestContext, id: &str) -> bool {
    cx.debug_bounds(Box::leak(id.to_string().into_boxed_str())).is_some()
}

fn edits(app: &gpui::Entity<BPhiberApp>, cx: &mut gpui::VisualTestContext) -> BSceneActions {
    app.read_with(cx, |a, _| BSceneActions::now(&a.state))
}

/// The Cube's world matrix at `frame`.
fn cube_at(app: &gpui::Entity<BPhiberApp>, cx: &mut gpui::VisualTestContext, frame: i32) -> [[f32; 4]; 4] {
    app.update(cx, |app, cx| {
        app.state.set(CtxKey::Frame, frame.to_string());
        cx.notify();
    });
    cx.run_until_parked();
    app.read_with(cx, |a, _| match scene_objects(&a.state).iter().find(|o| o.name() == "Cube") {
        Some(BObject3D::Loaded { matrix_world, .. }) => *matrix_world,
        _ => panic!("the Cube"),
    })
}

fn op(app: &gpui::Entity<BPhiberApp>, cx: &mut gpui::VisualTestContext, key: OpKey, value: &str) {
    app.update(cx, |app, cx| app.handle_action(&BAction::Set(key.path(), value.into()), cx));
    cx.run_until_parked();
}

#[gpui::test]
async fn the_action_editor_lists_assigns_makes_and_unlinks_actions(cx: &mut TestAppContext) {
    let dir = std::env::temp_dir().join(format!("phiber_action_editor_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let blend = dir.join("actions.blend");
    std::fs::copy(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../phiphx/tests/fixtures/actions.blend"), &blend).unwrap();
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
    cx.run_until_parked();

    // The Dope Sheet mode shows no selector; the Action Editor does — and
    // the 3D viewport keeps its own mode.
    let browse = "btn-label-toggle-ctx.menu.action.action_browse-text";
    assert!(!shown(cx, browse), "no selector in Dope Sheet mode");
    let viewport = app.read_with(cx, |a, _| active_mode(&a.state, BHostType::View3D));
    app.update(cx, |app, cx| {
        app.state.set(&mode_key(BHostType::Action), "Action Editor");
        cx.notify();
    });
    cx.run_until_parked();
    let modes = app.read_with(cx, |a, _| (active_mode(&a.state, BHostType::Action), active_mode(&a.state, BHostType::View3D)));
    assert_eq!(modes, ("Action Editor", viewport), "each editor keeps its own mode");
    assert!(shown(cx, "btn-label-toggle-ctx.menu.action.action_slot-text"), "the slot selector beside it");

    // Browse: every action, the Cube's ticked; pick Spin.
    let moved = cube_at(&app, cx, 20);
    assert!((moved[3][0] - 3.0).abs() < 1e-3, "CubeAction moves it to x 3 at frame 20: {moved:?}");
    click(cx, browse);
    for name in ["CameraAction", "CubeAction", "Spin"] {
        assert!(shown(cx, &format!("menu-{name}")), "{name} in the browse menu");
    }
    click(cx, "menu-Spin");
    let e = edits(&app, cx);
    assert_eq!((e.assigned[0].action.as_str(), e.assigned[0].slot.as_str()), ("Spin", "OBCube"), "through the slot named for it");
    let spun = cube_at(&app, cx, 24);
    assert!(spun[3][0].abs() < 1e-3 && (spun[0][0] + 1.0).abs() < 1e-3, "Spin turns it half round and moves it not: {spun:?}");

    // New copies Spin and takes the copy; Unlink; New again: an empty Action.
    click(cx, "btn-icon-set-ctx.op.action-new-copy");
    assert_eq!(edits(&app, cx).made, [("Spin.001".to_string(), "Spin".to_string())]);
    assert_eq!(edits(&app, cx).assigned[0].action, "Spin.001");
    click(cx, "btn-icon-set-ctx.op.action-unlink-x");
    assert!(edits(&app, cx).assigned[0].action.is_empty(), "unlinked");
    assert!(shown(cx, "btn-label-set-ctx.op.action-new-text"), "with no action, New is a labelled button");
    let still = cube_at(&app, cx, 20);
    assert!(still[3][0].abs() < 1e-3, "no action: it stays");
    click(cx, "btn-label-set-ctx.op.action-new-text");
    let e = edits(&app, cx);
    assert_eq!(e.made.last(), Some(&("Action".to_string(), String::new())));
    assert_eq!((e.assigned[0].action.as_str(), e.assigned[0].slot.as_str()), ("Action", ""), "a new action has no slot yet");

    // Undo twice: back on Spin.001.
    cx.simulate_keystrokes("ctrl-z ctrl-z");
    cx.run_until_parked();
    assert_eq!(edits(&app, cx).assigned[0].action, "Spin.001", "Undo steps the actions");

    // The sidebar's Action panel: Manual Frame Range from 3, and Cyclic.
    app.update(cx, |app, cx| {
        app.state.set(&phiber::v2::part::sidebar::show::BSidebarShown(BHostType::Action).key(), "true");
        cx.notify();
    });
    cx.run_until_parked();
    assert!(shown(cx, "card-ctx-action-action-action-expanded") && shown(cx, "card-ctx-action-action-slot-expanded"), "the Action and Slot panels");
    op(&app, cx, OpKey::ActionRange, "true");
    op(&app, cx, OpKey::ActionStart, "3");
    op(&app, cx, OpKey::ActionCyclic, "true");
    let range = edits(&app, cx).ranges;
    assert_eq!((range[0].action.as_str(), range[0].range, range[0].cyclic), ("Spin.001", Some((3.0, 24.0)), true), "from the keys' span, the start typed");

    // Properties ▸ Object ▸ Animation: the same selector, closed until opened.
    app.update(cx, |app, cx| {
        app.state.set(ButsKey::NavbarTab, "Object");
        cx.notify();
    });
    cx.run_until_parked();
    assert!(shown(cx, "card-ctx-buts-object-animation-expanded"), "the Animation panel");
    app.update(cx, |app, cx| {
        app.state.set(&phiber::v2::part::props::BPanelItem::expand_key(BHostType::Buts, "Object", "Animation"), "true");
        cx.notify();
    });
    cx.run_until_parked();
    assert!(shown(cx, "btn-label-toggle-ctx.menu.buts.action_browse-text"), "its browse menu");

    // Ctrl S: the file has Spin.001 on the Cube with its range.
    cx.simulate_keystrokes("ctrl-s");
    cx.run_until_parked();
    let saved = BlendFile::parse_bytes(&std::fs::read(&blend).unwrap()).unwrap().extract_scene().unwrap();
    let copy = saved.props.actions.iter().find(|a| a.name == "Spin.001").expect("Spin.001 saved");
    assert_eq!(copy.users, ["Cube"]);
    assert_eq!(copy.bindings.first().and_then(|(_, h)| copy.slots.iter().find(|s| s.handle == *h)).map(|s| s.identifier.as_str()), Some("OBCube"));
    assert_eq!((copy.manual_range, copy.cyclic), (Some((3.0, 24.0)), true));
    assert!(saved.props.actions.iter().find(|a| a.name == "CubeAction").is_some_and(|a| a.users.is_empty()), "CubeAction kept, unused");
    let _ = std::fs::remove_dir_all(&dir);
}
