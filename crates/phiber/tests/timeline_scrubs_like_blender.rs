//! End to end: the Timeline window is painted where the region is, a press
//! in it moves the current frame to the frame under the pointer
//! (`anim.change_frame`), a drag keeps moving it, and the header's transport
//! is a row of real buttons. This drives the built window, not the view
//! tree, so a canvas that painted but never took the pointer would fail.

use gpui::{Modifiers, MouseButton, Point, TestAppContext, px, size};
use phiber::v2::app::BPhiberApp;
use phiber::v2::state::{BKey as _, CtxKey};

fn pt(x: f32, y: f32) -> Point<gpui::Pixels> {
    Point::new(px(x), px(y))
}

#[gpui::test]
async fn pressing_and_dragging_in_the_timeline_window_scrubs_the_frame(cx: &mut TestAppContext) {
    let (app, cx) = cx.add_window_view(|_, _| BPhiberApp::new());
    cx.simulate_resize(size(px(1400.0), px(800.0)));
    app.update(cx, |app, cx| {
        // Boot opens the splash; the Timeline is under it.
        app.state.remove(phiber::v2::state::ModalKey::Open);
        cx.notify();
    });
    cx.run_until_parked();

    let window = cx.debug_bounds("time-window").expect("the Timeline window is painted");
    assert!(f32::from(window.size.height) >= 40.0, "the window has room for the scrub strip and a row: {:?}", window.size);
    let (left, width) = (f32::from(window.origin.x), f32::from(window.size.width));
    // The view opens at `start − 10 .. end + 10` (`action_create`), so the
    // frame under a pointer at fraction `f` of the width is `−9 + 270 f`.
    let frame_at = |f: f32| (-9.0 + 270.0 * f).round() as i32;
    let y = f32::from(window.origin.y) + f32::from(window.size.height) * 0.6;

    let press = pt(left + width * 0.5, y);
    cx.simulate_mouse_down(press, MouseButton::Left, Modifiers::none());
    cx.run_until_parked();
    let frame: i32 = app.read_with(cx, |a, _| a.state.get(CtxKey::Frame).unwrap().parse().unwrap());
    assert!((frame - frame_at(0.5)).abs() <= 1, "a press sets the frame under the pointer: got {frame}, expected about {}", frame_at(0.5));

    for step in 1..=8 {
        cx.simulate_mouse_move(pt(left + width * (0.5 + 0.03 * step as f32), y), Some(MouseButton::Left), Modifiers::none());
        cx.run_until_parked();
    }
    cx.simulate_mouse_up(pt(left + width * 0.74, y), MouseButton::Left, Modifiers::none());
    cx.run_until_parked();
    let dragged: i32 = app.read_with(cx, |a, _| a.state.get(CtxKey::Frame).unwrap().parse().unwrap());
    assert!(dragged > frame + 40, "a drag keeps scrubbing: {frame} → {dragged}");
    assert!((dragged - frame_at(0.74)).abs() <= 2, "and lands under the pointer: got {dragged}, expected about {}", frame_at(0.74));
}

#[gpui::test]
async fn the_transport_buttons_are_painted_in_the_header(cx: &mut TestAppContext) {
    let (app, cx) = cx.add_window_view(|_, _| BPhiberApp::new());
    cx.simulate_resize(size(px(1400.0), px(800.0)));
    app.update(cx, |app, cx| {
        app.state.remove(phiber::v2::state::ModalKey::Open);
        cx.notify();
    });
    cx.run_until_parked();
    let play = cx.debug_bounds("btn-icon-set-ctx.playing-true-play").expect("the play button is painted");
    let header = cx.debug_bounds("region-Header-Time").expect("the Timeline header is painted");
    assert!(play.origin.y >= header.origin.y && play.origin.y + play.size.height <= header.origin.y + header.size.height, "play sits in the Timeline header");
    cx.simulate_mouse_down(play.center(), MouseButton::Left, Modifiers::none());
    cx.simulate_mouse_up(play.center(), MouseButton::Left, Modifiers::none());
    cx.run_until_parked();
    let playing = app.read_with(cx, |a, _| a.state.get(CtxKey::Playing).map(str::to_string));
    assert_eq!(playing.as_deref(), Some("true"), "clicking play starts playback");
    let _ = CtxKey::Playing.path();
}
