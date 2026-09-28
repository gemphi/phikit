//! The keyboard runs the menus: a menu item's shortcut label is its key
//! binding, and Blender's screen keys — Space, the arrows, Escape — work
//! outside any menu. Typing into a field still goes to the field.

use gpui::{TestAppContext, px, size};
use phiber::v2::app::BPhiberApp;
use phiber::v2::scene::{scene_objects, view_pitch, view_yaw};
use phiber::v2::state::{CtxKey, ModalKey, OpKey, View3dKey, val::V_TRUE};
use phiber::v2::view::BAction;

#[gpui::test]
async fn shortcuts_run_the_items_and_the_screen_keys_play_and_step(cx: &mut TestAppContext) {
    let (app, cx) = cx.add_window_view(|_, _| BPhiberApp::new());
    cx.simulate_resize(size(px(1280.0), px(800.0)));
    app.update(cx, |app, cx| {
        app.state.set(ModalKey::Open, "");
        cx.notify();
    });
    cx.run_until_parked();

    // Numpad 7 (the number row, Emulate Numpad): the top view.
    cx.simulate_keystrokes("7");
    cx.run_until_parked();
    assert_eq!(app.read_with(cx, |a, _| (view_yaw(&a.state), view_pitch(&a.state))), (0.0, 90.0));
    cx.simulate_keystrokes("1");
    cx.run_until_parked();
    assert_eq!(app.read_with(cx, |a, _| view_pitch(&a.state)), 0.0, "front");

    // Space plays; Right steps; Escape stops.
    cx.simulate_keystrokes("space");
    cx.run_until_parked();
    assert_eq!(app.read_with(cx, |a, _| a.state.get(CtxKey::Playing).map(str::to_string)), Some(V_TRUE.into()));
    cx.simulate_keystrokes("escape");
    cx.run_until_parked();
    assert_eq!(app.read_with(cx, |a, _| a.state.get(CtxKey::Playing).map(str::to_string)), Some("false".into()));
    let before = app.read_with(cx, |a, _| a.state.get(CtxKey::Frame).and_then(|f| f.parse::<i32>().ok()).unwrap_or(1));
    cx.simulate_keystrokes("right");
    cx.run_until_parked();
    assert_eq!(app.read_with(cx, |a, _| a.state.get(CtxKey::Frame).and_then(|f| f.parse::<i32>().ok())), Some(before + 1));

    // Shift D duplicates the active object; Ctrl Z takes it back.
    let count = app.read_with(cx, |a, _| scene_objects(&a.state).len());
    cx.simulate_keystrokes("shift-d");
    cx.run_until_parked();
    assert_eq!(app.read_with(cx, |a, _| scene_objects(&a.state).len()), count + 1);
    cx.simulate_keystrokes("ctrl-z");
    cx.run_until_parked();
    assert_eq!(app.read_with(cx, |a, _| scene_objects(&a.state).len()), count, "undone");

    // F3 opens the menu search; typing into its field is typing, not keys.
    cx.simulate_keystrokes("f3");
    cx.run_until_parked();
    assert_eq!(app.read_with(cx, |a, _| a.state.get(ModalKey::Open).map(str::to_string)), Some("command".into()));
    let _ = (OpKey::Report, View3dKey::Yaw, BAction::Toggle);
}

/// F2 opens the rename popup on the active object with the caret in the
/// field; what is typed goes into the field, and Return renames.
#[gpui::test]
async fn f2_renames_the_active_object(cx: &mut TestAppContext) {
    let (app, cx) = cx.add_window_view(|_, _| BPhiberApp::new());
    cx.simulate_resize(size(px(1280.0), px(800.0)));
    app.update(cx, |app, cx| {
        app.state.set(ModalKey::Open, "");
        app.state.set(CtxKey::Object, "Cube");
        cx.notify();
    });
    cx.run_until_parked();
    cx.simulate_keystrokes("f2");
    cx.run_until_parked();
    assert_eq!(app.read_with(cx, |a, _| a.state.get(ModalKey::Open).map(str::to_string)), Some("rename".into()));
    for _ in 0..4 {
        cx.simulate_keystrokes("backspace");
    }
    cx.simulate_input("Crate");
    cx.run_until_parked();
    assert_eq!(app.read_with(cx, |a, _| a.state.get(phiber::v2::state::AppKey::RenameText).map(str::to_string)), Some("Crate".into()));
    assert_eq!(app.read_with(cx, |a, _| a.state.get(CtxKey::Object).map(str::to_string)), Some("Cube".into()), "typing alone renames nothing");
    cx.simulate_keystrokes("enter");
    cx.run_until_parked();
    assert_eq!(app.read_with(cx, |a, _| a.state.get(CtxKey::Object).map(str::to_string)), Some("Crate".into()));
    assert_eq!(app.read_with(cx, |a, _| a.state.get(ModalKey::Open).map(str::to_string)), Some("".into()), "the popup closed");
    assert!(app.read_with(cx, |a, _| scene_objects(&a.state).iter().any(|o| o.name() == "Crate")));
}
