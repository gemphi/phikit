//! Edit ▸ Undo History is Blender's `TOPBAR_MT_undo_history`: "Original"
//! and every step by name, the current one marked, and picking an entry
//! undoes or redoes to it (`ed.undo_history`).

use gpui::{TestAppContext, px, size};
use phiber::v2::app::BPhiberApp;
use phiber::v2::scene::scene_objects;
use phiber::v2::state::{BKey as _, CtxKey, MenuKey, ModalKey, OpKey};
use phiber::v2::view::BAction;

#[gpui::test]
async fn the_history_lists_every_step_by_name_and_jumps_to_the_one_picked(cx: &mut TestAppContext) {
    let (app, cx) = cx.add_window_view(|_, _| BPhiberApp::new());
    cx.simulate_resize(size(px(1400.0), px(900.0)));
    let original = app.read_with(cx, |a, _| a.state.get(CtxKey::Frame).map(str::to_string));
    app.update(cx, |app, cx| {
        app.state.remove(ModalKey::Open);
        app.handle_action(&BAction::Set(CtxKey::Frame.path(), "40".into()), cx);
        app.handle_action(&BAction::Set(OpKey::SceneEdit.path(), "add:cube".into()), cx);
        app.handle_action(&BAction::Set(CtxKey::Frame.path(), "60".into()), cx);
    });
    cx.run_until_parked();
    let listing = app.read_with(cx, |a, _| a.history.listing());
    assert_eq!(listing.names.len(), 4, "{listing:?}");
    assert_eq!(listing.names[0], "Original");
    assert_eq!(listing.at, 3);
    assert!(listing.names[1].contains("40"), "{listing:?}");

    // Open Edit ▸ Undo History: one row per step, the current one checked.
    app.update(cx, |app, cx| {
        app.handle_action(&BAction::Toggle(MenuKey::Top("topbar", "Edit").path()), cx);
        app.handle_action(&BAction::Open(MenuKey::Nested("topbar", &["Edit", "Undo History"]).path()), cx);
    });
    cx.run_until_parked();
    assert!(cx.debug_bounds("menu-Original").is_some(), "the listing is painted");
    let current: &'static str = Box::leak(format!("menu-{}", listing.names[3]).into_boxed_str());
    assert!(cx.debug_bounds(current).is_some());

    // Pick "Original": every edit undone; pick the second step: redone up to it.
    app.update(cx, |app, cx| app.handle_action(&BAction::Set(OpKey::UndoHistory.path(), "0".into()), cx));
    cx.run_until_parked();
    assert_eq!(app.read_with(cx, |a, _| a.state.get(CtxKey::Frame).map(str::to_string)), original, "back to the original");
    assert!(app.read_with(cx, |a, _| !scene_objects(&a.state).iter().any(|o| o.name() == "Cube.001")));
    assert_eq!(app.read_with(cx, |a, _| a.history.listing()).at, 0);
    app.update(cx, |app, cx| app.handle_action(&BAction::Set(OpKey::UndoHistory.path(), "2".into()), cx));
    cx.run_until_parked();
    assert_eq!(app.read_with(cx, |a, _| a.state.get(CtxKey::Frame).map(str::to_string)), Some("40".into()));
    assert!(app.read_with(cx, |a, _| scene_objects(&a.state).iter().any(|o| o.name() == "Cube.001")), "the cube is back, the later frame change not");
    assert_eq!(app.read_with(cx, |a, _| a.history.listing()).at, 2);
}
