//! A key goes to the editor under the pointer first, as Blender's region
//! keymaps do: over the Timeline, G grabs the selected markers
//! (`marker.move` — they follow the pointer, a click keeps them, Escape
//! puts them back, and the whole grab is one undo step) and F2 renames the
//! selected marker (`marker.rename`); over the 3D viewport F2 is still
//! Rename Active Item.

use gpui::{Modifiers, MouseButton, Point, TestAppContext, px, size};
use phiber::v2::app::BPhiberApp;
use phiber::v2::scene::marker::BMarkerList;
use phiber::v2::state::{AppKey, CtxKey, ModalKey};
use phiber::v2::view::draw::canvas::time::marker::BAND_HEIGHT;

fn pt(x: f32, y: f32) -> Point<gpui::Pixels> {
    Point::new(px(x), px(y))
}

#[gpui::test]
async fn g_grabs_the_selected_markers_and_f2_renames_one_over_the_timeline(cx: &mut TestAppContext) {
    let (app, cx) = cx.add_window_view(|_, _| BPhiberApp::new());
    cx.simulate_resize(size(px(1400.0), px(800.0)));
    app.update(cx, |app, cx| {
        app.state.remove(ModalKey::Open);
        app.state.set(CtxKey::Markers, "60=Cut|120=End");
        cx.notify();
    });
    cx.run_until_parked();
    let window = cx.debug_bounds("time-window").expect("the Timeline window is painted");
    let (left, width) = (f32::from(window.origin.x), f32::from(window.size.width));
    let (top, height) = (f32::from(window.origin.y), f32::from(window.size.height));
    let x_of = |frame: f32| left + width * ((frame + 9.0) / 269.0);
    let band_y = top + height - BAND_HEIGHT / 2.0;
    let mid_y = top + height * 0.5;
    let markers = |cx: &mut gpui::VisualTestContext| app.read_with(cx, |a, _| BMarkerList::now(&a.state).list);

    // Select Cut, then G with the pointer over the Timeline.
    cx.simulate_mouse_down(pt(x_of(60.0), band_y), MouseButton::Left, Modifiers::none());
    cx.simulate_mouse_up(pt(x_of(60.0), band_y), MouseButton::Left, Modifiers::none());
    cx.run_until_parked();
    let undo_before = app.read_with(cx, |a, _| a.history.depth().0);
    let frame_before = app.read_with(cx, |a, _| a.state.get(CtxKey::Frame).map(str::to_string));
    cx.simulate_mouse_move(pt(x_of(80.0), mid_y), None, Modifiers::none());
    cx.simulate_keystrokes("g");
    cx.run_until_parked();
    // The pointer moves ten frames on: so does Cut.
    for frame in [81.0, 85.0, 90.0] {
        cx.simulate_mouse_move(pt(x_of(frame), mid_y), None, Modifiers::none());
        cx.run_until_parked();
    }
    let cut = |list: &[(i32, String)]| list.iter().find(|(_, n)| n == "Cut").map(|(f, _)| *f).unwrap();
    let moved = cut(&markers(cx));
    assert!((moved - 70).abs() <= 1, "Cut follows the pointer by ten frames: {:?}", markers(cx));
    // Escape puts it back.
    cx.simulate_keystrokes("escape");
    cx.run_until_parked();
    assert_eq!(cut(&markers(cx)), 60, "cancelled");

    // Again, and a click keeps it; one undo step for the whole grab.
    cx.simulate_mouse_move(pt(x_of(80.0), mid_y), None, Modifiers::none());
    cx.simulate_keystrokes("g");
    cx.run_until_parked();
    for frame in [80.0, 90.0, 100.0] {
        cx.simulate_mouse_move(pt(x_of(frame), mid_y), None, Modifiers::none());
        cx.run_until_parked();
    }
    cx.simulate_mouse_down(pt(x_of(100.0), mid_y), MouseButton::Left, Modifiers::none());
    cx.simulate_mouse_up(pt(x_of(100.0), mid_y), MouseButton::Left, Modifiers::none());
    cx.run_until_parked();
    let kept = cut(&markers(cx));
    assert!((kept - 80).abs() <= 1, "kept twenty frames on: {:?}", markers(cx));
    assert_eq!(app.read_with(cx, |a, _| a.history.depth().0), undo_before + 1, "one step for the whole grab");
    assert_eq!(app.read_with(cx, |a, _| a.state.get(CtxKey::Frame).map(str::to_string)), frame_before, "the confirming click did not scrub");

    // F2 over the Timeline renames the selected marker.
    cx.simulate_keystrokes("f2");
    cx.run_until_parked();
    assert_eq!(app.read_with(cx, |a, _| a.state.get(ModalKey::Open).map(str::to_string)), Some("rename".into()));
    assert_eq!(app.read_with(cx, |a, _| a.state.get(AppKey::RenameText).map(str::to_string)), Some("Cut".into()));
    app.update(cx, |app, cx| app.handle_action(&phiber::v2::view::BAction::Set(phiber::v2::state::BKey::path(&AppKey::RenameText), "Scene 2".into()), cx));
    cx.run_until_parked();
    assert!(markers(cx).iter().any(|(f, n)| *f == kept && n == "Scene 2"), "{:?}", markers(cx));
}
