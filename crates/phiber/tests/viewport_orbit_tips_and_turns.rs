//! A drag on the 3D viewport orbits as Blender's turntable does
//! (`scene::orbit::BTurntable`): across turns the view about the world's Z
//! axis, up and down tips it, so the Z axis leans as X and Y swing round —
//! until now a drag only ever turned it across, and Z stood straight up.
//! The tip goes on over the top, as Blender's does.

use gpui::{Modifiers, MouseButton, MouseDownEvent, MouseUpEvent, TestAppContext, point, px, size};
use phiber::v2::app::BPhiberApp;
use phiber::v2::scene::view_slot::BViewSlot;
use phiber::v2::scene::{view_pitch, view_yaw};
use phiber::v2::state::ModalKey;

fn angles(app: &gpui::Entity<BPhiberApp>, cx: &mut gpui::VisualTestContext) -> (f32, f32) {
    app.read_with(cx, |a, _| {
        let scope = phiber::v2::space::BSpaceList::active(&a.state).slug();
        BViewSlot::new(scope, "Center").with(|| (view_yaw(&a.state), view_pitch(&a.state)))
    })
}

#[gpui::test]
async fn a_drag_turns_across_and_tips_up_and_down_over_the_top(cx: &mut TestAppContext) {
    let (app, cx) = cx.add_window_view(|_, _| BPhiberApp::new());
    cx.simulate_resize(size(px(1280.0), px(800.0)));
    app.update(cx, |app, cx| {
        app.state.set(ModalKey::Open, "");
        cx.notify();
    });
    cx.run_until_parked();
    let (yaw0, pitch0) = angles(&app, cx);
    assert_eq!((yaw0, pitch0), (30.0, 30.0), "Blender's startup view");

    let canvas = cx.debug_bounds("view3d-canvas").expect("the viewport");
    let at = point(canvas.origin.x + canvas.size.width / 4.0, canvas.origin.y + canvas.size.height / 4.0);
    let moved = |dx: f32, dy: f32| point(at.x + px(dx), at.y + px(dy));
    cx.simulate_event(MouseDownEvent { position: at, modifiers: Modifiers::none(), button: MouseButton::Left, click_count: 1, first_mouse: false });
    cx.simulate_mouse_move(moved(6.0, 6.0), MouseButton::Left, Modifiers::none());
    cx.simulate_mouse_move(moved(8.0, 8.0), MouseButton::Left, Modifiers::none());
    cx.run_until_parked();
    let (yaw1, pitch1) = angles(&app, cx);
    // Measured from the canvas, not the window: the first step is a few
    // degrees, not the canvas's offset in the window.
    assert!((yaw1 - yaw0).abs() <= 4.0 && (pitch1 - pitch0).abs() <= 4.0, "no jump: {yaw1}, {pitch1}");

    cx.simulate_mouse_move(moved(48.0, 68.0), MouseButton::Left, Modifiers::none());
    cx.run_until_parked();
    let (yaw2, pitch2) = angles(&app, cx);
    assert!((yaw2 - yaw1 - 20.0).abs() < 1e-3, "across: half a degree a pixel about Z ({yaw1} → {yaw2})");
    assert!((pitch2 - pitch1 - 30.0).abs() < 1e-3, "down: the view tips toward Top ({pitch1} → {pitch2})");

    // On down, past the pole: the view goes over the top, not stopped at 90.
    cx.simulate_mouse_move(moved(48.0, 208.0), MouseButton::Left, Modifiers::none());
    cx.run_until_parked();
    let (_, pitch3) = angles(&app, cx);
    assert!((pitch3 - pitch2 - 70.0).abs() < 1e-3 && pitch3 > 90.0, "over the top: {pitch3}");
    cx.simulate_event(MouseUpEvent { position: moved(48.0, 208.0), modifiers: Modifiers::none(), button: MouseButton::Left, click_count: 1 });
    cx.run_until_parked();
}
