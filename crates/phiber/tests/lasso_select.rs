//! Ctrl right-drag: lasso select (`view3d.select_lasso`). The outline is
//! drawn while the button is down; on release what is inside is added —
//! with Shift, taken away — and a right-drag without Ctrl draws nothing.

use gpui::{Modifiers, MouseButton, Point, TestAppContext, px, size};
use phiber::v2::app::BPhiberApp;
use phiber::v2::scene::lasso::BLasso;
use phiber::v2::scene::scene_objects;
use phiber::v2::scene::select::BSelection;
use phiber::v2::state::{BKey as _, ModalKey, OpKey};
use phiber::v2::view::BAction;

fn ctrl(shift: bool) -> Modifiers {
    Modifiers { control: true, shift, ..Default::default() }
}

/// Draw a lasso round the inside of `bounds`, inset by `inset` pixels —
/// begun at the foot's middle, clear of the toolbar and the gizmo.
fn draw(cx: &mut gpui::VisualTestContext, bounds: gpui::Bounds<gpui::Pixels>, inset: f32, mods: Modifiers) {
    let (x0, y0) = (f32::from(bounds.origin.x) + inset, f32::from(bounds.origin.y) + inset);
    let (x1, y1) = (f32::from(bounds.origin.x + bounds.size.width) - inset, f32::from(bounds.origin.y + bounds.size.height) - inset);
    let mid = (x0 + x1) / 2.0;
    cx.simulate_mouse_down(Point::new(px(mid), px(y1)), MouseButton::Right, mods);
    for (x, y) in [(x1, y1), (x1, y0), (x0, y0), (x0, y1)] {
        cx.simulate_mouse_move(Point::new(px(x), px(y)), Some(MouseButton::Right), mods);
        cx.run_until_parked();
    }
    cx.simulate_mouse_up(Point::new(px(x0), px(y1)), MouseButton::Right, mods);
    cx.run_until_parked();
}

#[gpui::test]
async fn a_lasso_round_the_canvas_selects_and_shift_deselects(cx: &mut TestAppContext) {
    let (app, cx) = cx.add_window_view(|_, _| BPhiberApp::new());
    cx.simulate_resize(size(px(1400.0), px(800.0)));
    app.update(cx, |app, cx| {
        app.state.remove(ModalKey::Open);
        app.handle_action(&BAction::Set(OpKey::Select.path(), "none".into()), cx);
    });
    cx.run_until_parked();
    let canvas = cx.debug_bounds("view3d-orbit").expect("the viewport's orbit layer");
    let all = app.read_with(cx, |a, _| scene_objects(&a.state).len());

    // While drawn, the outline is in the state; on release it is gone.
    let (x0, y0) = (f32::from(canvas.origin.x) + 10.0, f32::from(canvas.origin.y) + 10.0);
    cx.simulate_mouse_down(Point::new(px(x0), px(y0)), MouseButton::Right, ctrl(false));
    cx.simulate_mouse_move(Point::new(px(x0 + 50.0), px(y0)), Some(MouseButton::Right), ctrl(false));
    cx.run_until_parked();
    assert_eq!(app.read_with(cx, |a, _| BLasso::now(&a.state)).map(|l| l.points.len()), Some(2), "the outline grows as it is drawn");
    cx.simulate_mouse_up(Point::new(px(x0 + 50.0), px(y0)), MouseButton::Right, ctrl(false));
    cx.run_until_parked();
    assert!(app.read_with(cx, |a, _| BLasso::now(&a.state)).is_none());

    draw(cx, canvas, 60.0, ctrl(false));
    let selected = app.read_with(cx, |a, _| BSelection::now(&a.state).list.len());
    assert!(selected > 0 && selected <= all, "the objects on screen: {selected} of {all}");

    draw(cx, canvas, 60.0, ctrl(true));
    assert!(app.read_with(cx, |a, _| BSelection::now(&a.state).list.is_empty()), "Shift Ctrl takes them away");

    draw(cx, canvas, 60.0, Modifiers::none());
    assert!(app.read_with(cx, |a, _| BSelection::now(&a.state).list.is_empty()), "no Ctrl: no lasso");
}
