//! A header too narrow for what it holds scrolls sideways under the wheel,
//! as Blender's header regions do, instead of clipping its right end.

use gpui::{Point, ScrollDelta, ScrollWheelEvent, TestAppContext, px, size};
use phiber::v2::app::BPhiberApp;
use phiber::v2::state::ModalKey;

const MAGNET: &str = "btn-icon-set-ctx.buts.use_snap_playhead-true-magnet";

#[gpui::test]
async fn the_wheel_pans_a_narrow_header(cx: &mut TestAppContext) {
    let (app, cx) = cx.add_window_view(|_, _| BPhiberApp::new());
    cx.simulate_resize(size(px(640.0), px(700.0)));
    app.update(cx, |app, cx| {
        app.state.remove(ModalKey::Open);
        cx.notify();
    });
    cx.run_until_parked();
    let header = cx.debug_bounds("region-Header-Time").expect("the Timeline's header");
    let before = cx.debug_bounds(MAGNET).expect("the transport's magnet");
    assert!(before.right() > header.right(), "at this width the header holds more than it shows");

    cx.simulate_event(ScrollWheelEvent { position: header.center(), delta: ScrollDelta::Lines(Point::new(0.0, -4.0)), ..Default::default() });
    cx.run_until_parked();
    let after = cx.debug_bounds(MAGNET).unwrap();
    assert!(after.origin.x < before.origin.x, "the wheel moved the strip sideways: {:?} → {:?}", before.origin.x, after.origin.x);
    assert_eq!(after.origin.y, before.origin.y, "and only sideways");
    assert_eq!(cx.debug_bounds("region-Header-Time").unwrap(), header, "the header itself stays put");
}

#[gpui::test]
async fn a_middle_drag_pans_a_narrow_header_and_stops_at_its_start(cx: &mut TestAppContext) {
    use gpui::{Modifiers, MouseButton};
    let (app, cx) = cx.add_window_view(|_, _| BPhiberApp::new());
    cx.simulate_resize(size(px(640.0), px(700.0)));
    app.update(cx, |app, cx| {
        app.state.remove(ModalKey::Open);
        cx.notify();
    });
    cx.run_until_parked();
    let header = cx.debug_bounds("region-Header-Time").expect("the Timeline's header");
    let before = cx.debug_bounds(MAGNET).unwrap();
    let from = header.center();
    let to = Point::new(from.x - px(80.0), from.y);
    cx.simulate_mouse_down(from, MouseButton::Middle, Modifiers::none());
    cx.simulate_mouse_move(to, Some(MouseButton::Middle), Modifiers::none());
    cx.simulate_mouse_up(to, MouseButton::Middle, Modifiers::none());
    cx.run_until_parked();
    let panned = cx.debug_bounds(MAGNET).unwrap();
    assert_eq!(panned.origin.x, before.origin.x - px(80.0), "the strip follows the pointer");

    // Dragging back past the start stops at the start.
    let back = Point::new(from.x + px(300.0), from.y);
    cx.simulate_mouse_down(from, MouseButton::Middle, Modifiers::none());
    cx.simulate_mouse_move(back, Some(MouseButton::Middle), Modifiers::none());
    cx.simulate_mouse_up(back, MouseButton::Middle, Modifiers::none());
    cx.run_until_parked();
    assert_eq!(cx.debug_bounds(MAGNET).unwrap().origin.x, before.origin.x, "no further than its start");
}
