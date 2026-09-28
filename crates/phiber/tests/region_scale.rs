//! Dragging a region's edge, as Blender's `SCREEN_OT_region_scale`
//! (`editors/screen/screen_ops.cc`, `region_scale_modal`): the drag resizes
//! within the region's range; dragged below one unit the region hides and
//! keeps its size; clicking the arrow left at its edge shows it again.
//! Checked on the file browser's source list (a sidebar in a Tools region).

use gpui::{Modifiers, MouseButton, TestAppContext, px, size};
use phiber::v2::app::BPhiberApp;
use phiber::v2::state::val::V_MODAL_FILE_BROWSER;
use phiber::v2::state::{BState, CtxKey, ModalKey};

async fn dialog(cx: &mut TestAppContext) -> (gpui::Entity<BPhiberApp>, &mut gpui::VisualTestContext) {
    let (app, cx) = cx.add_window_view(|_, _| BPhiberApp::new());
    cx.simulate_resize(size(px(1400.0), px(900.0)));
    app.update(cx, |app, cx| {
        app.state = BState::new();
        app.state.set(CtxKey::Space, "layout");
        app.state.set(ModalKey::Open, V_MODAL_FILE_BROWSER);
        cx.notify();
    });
    cx.run_until_parked();
    (app, cx)
}

fn width(cx: &mut gpui::VisualTestContext) -> f32 {
    f32::from(cx.debug_bounds("panel-sidebar-column").expect("the source list").size.width)
}

fn drag(cx: &mut gpui::VisualTestContext, dx: f32) {
    let handle = cx.debug_bounds("panel-resize-handle").expect("the source list's edge").center();
    cx.simulate_mouse_down(handle, MouseButton::Left, Modifiers::none());
    // A step past GPUI's drag threshold starts the drag; the next move is
    // the first the drag sees, so the width is measured from there.
    let at = |x: f32| gpui::point(handle.x + px(x), handle.y);
    cx.simulate_mouse_move(at(4.0), Some(MouseButton::Left), Modifiers::none());
    cx.simulate_mouse_move(at(4.0), Some(MouseButton::Left), Modifiers::none());
    cx.simulate_mouse_move(at(4.0 + dx), Some(MouseButton::Left), Modifiers::none());
    cx.run_until_parked();
    cx.simulate_mouse_up(gpui::point(handle.x + px(dx), handle.y), MouseButton::Left, Modifiers::none());
    cx.run_until_parked();
}

#[gpui::test]
async fn a_region_dragged_below_a_unit_hides_and_comes_back_at_its_size(cx: &mut TestAppContext) {
    let (_app, cx) = dialog(cx).await;
    let start = width(cx);

    drag(cx, 60.0);
    let wider = width(cx);
    assert!(wider > start + 40.0, "dragging out widens it ({start} → {wider})");

    drag(cx, -(wider + 40.0));
    assert!(cx.debug_bounds("panel-sidebar-column").is_none(), "dragged shut, the column is gone");
    let arrow = cx.debug_bounds("panel-sidebar-restore").expect("an arrow is left at its edge");

    cx.simulate_click(arrow.center(), Modifiers::none());
    cx.run_until_parked();
    assert!(cx.debug_bounds("panel-sidebar-restore").is_none());
    let back = width(cx);
    assert!((back - wider).abs() <= 1.0, "it comes back at the size it had ({wider} → {back})");
}
