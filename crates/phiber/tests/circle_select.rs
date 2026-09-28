//! C: circle select (`view3d.select_circle`). The tool stays on across
//! strokes; the left button paints a selection, Shift paints it away, the
//! wheel resizes the brush, and the right button or Escape ends it.

use gpui::{Modifiers, MouseButton, Point, ScrollDelta, ScrollWheelEvent, TestAppContext, px, size};
use phiber::v2::app::BPhiberApp;
use phiber::v2::scene::circlesel::{BCircle, DEFAULT_RADIUS};
use phiber::v2::scene::scene_objects;
use phiber::v2::scene::select::BSelection;
use phiber::v2::state::{BKey as _, ModalKey, OpKey, View3dKey};
use phiber::v2::view::BAction;

#[gpui::test]
async fn the_brush_paints_selection_on_and_off_until_it_is_put_away(cx: &mut TestAppContext) {
    let (app, cx) = cx.add_window_view(|_, _| BPhiberApp::new());
    cx.simulate_resize(size(px(1400.0), px(800.0)));
    app.update(cx, |app, cx| {
        app.state.remove(ModalKey::Open);
        app.handle_action(&BAction::Set(OpKey::Select.path(), "none".into()), cx);
        app.handle_action(&BAction::Set(OpKey::Select.path(), "circle".into()), cx);
    });
    cx.run_until_parked();
    assert!(app.read_with(cx, |a, _| BCircle::armed(&a.state)), "C turns the tool on");
    let surface = cx.debug_bounds("view3d-circle-select").expect("the viewport takes the brush");
    let centre = surface.center();

    // The pointer moves the brush.
    cx.simulate_mouse_move(centre, None, Modifiers::none());
    cx.run_until_parked();
    let brush = app.read_with(cx, |a, _| BCircle::now(&a.state)).expect("the brush follows the pointer");
    assert_eq!(brush.radius, DEFAULT_RADIUS);

    // A brush over the whole canvas: one press selects every object.
    app.update(cx, |app, cx| app.handle_action(&BAction::Set(View3dKey::CircleRadius.path(), "5000".into()), cx));
    cx.run_until_parked();
    cx.simulate_mouse_down(centre, MouseButton::Left, Modifiers::none());
    cx.simulate_mouse_up(centre, MouseButton::Left, Modifiers::none());
    cx.run_until_parked();
    let all = app.read_with(cx, |a, _| scene_objects(&a.state).len());
    assert!(all > 0);
    assert_eq!(app.read_with(cx, |a, _| BSelection::now(&a.state).list.len()), all, "painted on");
    assert!(app.read_with(cx, |a, _| BCircle::armed(&a.state)), "still on after the stroke");

    // Shift paints it off.
    cx.simulate_mouse_down(centre, MouseButton::Left, Modifiers::shift());
    cx.simulate_mouse_up(centre, MouseButton::Left, Modifiers::shift());
    cx.run_until_parked();
    assert!(app.read_with(cx, |a, _| BSelection::now(&a.state).list.is_empty()), "painted off");

    // The wheel resizes the brush: down grows it.
    cx.simulate_event(ScrollWheelEvent { position: centre, delta: ScrollDelta::Lines(Point::new(0.0, -1.0)), ..Default::default() });
    cx.run_until_parked();
    assert!(app.read_with(cx, |a, _| BCircle::radius(&a.state)) > 5000.0);

    // The right button puts it away; the canvas orbits again.
    cx.simulate_mouse_down(centre, MouseButton::Right, Modifiers::none());
    cx.run_until_parked();
    assert!(!app.read_with(cx, |a, _| BCircle::armed(&a.state)));
    assert!(cx.debug_bounds("view3d-circle-select").is_none());

    // Escape does too.
    app.update(cx, |app, cx| app.handle_action(&BAction::Set(OpKey::Select.path(), "circle".into()), cx));
    cx.run_until_parked();
    cx.simulate_keystrokes("escape");
    cx.run_until_parked();
    assert!(!app.read_with(cx, |a, _| BCircle::armed(&a.state)), "Escape ends the tool");
}
