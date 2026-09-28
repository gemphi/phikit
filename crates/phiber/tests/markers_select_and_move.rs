//! Markers take the pointer in the Timeline's marker band: a press on one
//! selects it (Shift extends), a press on the band's empty part deselects,
//! a drag begun on one moves it frame by frame, and a press above the band
//! still scrubs (`ed_marker_select`, `marker.move`, `anim.change_frame`).

use gpui::{Modifiers, MouseButton, Point, TestAppContext, px, size};
use phiber::v2::app::BPhiberApp;
use phiber::v2::scene::marker::BMarkerList;
use phiber::v2::state::CtxKey;
use phiber::v2::view::draw::canvas::time::marker::BAND_HEIGHT;

fn pt(x: f32, y: f32) -> Point<gpui::Pixels> {
    Point::new(px(x), px(y))
}

#[gpui::test]
async fn a_press_on_a_marker_selects_it_and_a_drag_moves_it(cx: &mut TestAppContext) {
    let (app, cx) = cx.add_window_view(|_, _| BPhiberApp::new());
    cx.simulate_resize(size(px(1400.0), px(800.0)));
    app.update(cx, |app, cx| {
        app.state.remove(phiber::v2::state::ModalKey::Open);
        app.state.set(CtxKey::Markers, "60=Cut|120=End");
        cx.notify();
    });
    cx.run_until_parked();

    let window = cx.debug_bounds("time-window").expect("the Timeline window is painted");
    let (left, width) = (f32::from(window.origin.x), f32::from(window.size.width));
    let (top, height) = (f32::from(window.origin.y), f32::from(window.size.height));
    // The view is `−9 .. 260` (`action_create`): a frame's x across the window.
    let x_of = |frame: f32| left + width * ((frame + 9.0) / 269.0);
    let band_y = top + height - BAND_HEIGHT / 2.0;
    let frame_before: i32 = app.read_with(cx, |a, _| a.state.get(CtxKey::Frame).map_or(1, |f| f.parse().unwrap()));

    // A press on the Cut marker selects it, and the frame does not move.
    cx.simulate_mouse_down(pt(x_of(60.0), band_y), MouseButton::Left, Modifiers::none());
    cx.simulate_mouse_up(pt(x_of(60.0), band_y), MouseButton::Left, Modifiers::none());
    cx.run_until_parked();
    assert_eq!(app.read_with(cx, |a, _| BMarkerList::selected(&a.state)), [60]);
    assert_eq!(app.read_with(cx, |a, _| a.state.get(CtxKey::Frame).map_or(1, |f| f.parse().unwrap())), frame_before, "a marker press is not a scrub");
    // Shift-press the End marker: both selected.
    cx.simulate_mouse_down(pt(x_of(120.0), band_y), MouseButton::Left, Modifiers::shift());
    cx.simulate_mouse_up(pt(x_of(120.0), band_y), MouseButton::Left, Modifiers::shift());
    cx.run_until_parked();
    assert_eq!(app.read_with(cx, |a, _| BMarkerList::selected(&a.state)), [60, 120]);
    // The band's empty part: deselect.
    cx.simulate_mouse_down(pt(x_of(200.0), band_y), MouseButton::Left, Modifiers::none());
    cx.simulate_mouse_up(pt(x_of(200.0), band_y), MouseButton::Left, Modifiers::none());
    cx.run_until_parked();
    assert!(app.read_with(cx, |a, _| BMarkerList::selected(&a.state)).is_empty());

    // Drag the Cut marker to frame 90.
    cx.simulate_mouse_down(pt(x_of(60.0), band_y), MouseButton::Left, Modifiers::none());
    for step in 1..=6 {
        cx.simulate_mouse_move(pt(x_of(60.0 + 5.0 * step as f32), band_y), Some(MouseButton::Left), Modifiers::none());
        cx.run_until_parked();
    }
    cx.simulate_mouse_up(pt(x_of(90.0), band_y), MouseButton::Left, Modifiers::none());
    cx.run_until_parked();
    let list = app.read_with(cx, |a, _| BMarkerList::now(&a.state).list);
    let cut = list.iter().find(|(_, n)| n == "Cut").map(|(f, _)| *f).expect("Cut is still there");
    let log: Vec<String> = phiber::v2::state::log::entries().iter().map(|e| e.text.clone()).collect();
    assert!((cut - 90).abs() <= 1, "dragged to about 90: {list:?}; log {log:?}");
    assert_eq!(app.read_with(cx, |a, _| BMarkerList::selected(&a.state)), [cut], "and stays selected");

    // Above the band a press still scrubs.
    let y = top + height * 0.4;
    cx.simulate_mouse_down(pt(x_of(30.0), y), MouseButton::Left, Modifiers::none());
    cx.simulate_mouse_up(pt(x_of(30.0), y), MouseButton::Left, Modifiers::none());
    cx.run_until_parked();
    let frame: i32 = app.read_with(cx, |a, _| a.state.get(CtxKey::Frame).unwrap().parse().unwrap());
    assert!((frame - 30).abs() <= 1, "scrubbed to about 30: {frame}");
}
