//! A vertical strip's expand / paging cluster is pinned to the strip's
//! foot, a cap gap (2px, `menu_top_gap`) off the bottom edge: when the
//! strip is shorter than its entries, the entries scroll under it and the
//! cluster stays in view. It used to be pushed off the bottom of the
//! Properties navbar and out of the window.

use gpui::{TestAppContext, px, size};
use phiber::v2::app::BPhiberApp;
use phiber::v2::state::{CtxKey, ModalKey};

fn bottom(b: gpui::Bounds<gpui::Pixels>) -> f32 {
    f32::from(b.origin.y) + f32::from(b.size.height)
}

#[gpui::test]
async fn the_cluster_sits_a_cap_gap_off_the_foot_of_a_short_strip(cx: &mut TestAppContext) {
    let (app, cx) = cx.add_window_view(|_, _| BPhiberApp::new());
    // Short enough that the Properties navbar's fifteen tabs do not fit.
    cx.simulate_resize(size(px(1400.0), px(560.0)));
    app.update(cx, |app, cx| {
        app.state.remove(ModalKey::Open);
        app.state.set(CtxKey::Space, "layout");
        cx.notify();
    });
    cx.run_until_parked();
    let gap = phiber::v2::theme::ui_style_defaults(phiber::v2::theme::BPaletteName::DEFAULT).dimensions.menu_top_gap;
    let rail = cx.debug_bounds("region-NavBar-Buts").expect("the Properties navbar");
    let chevron = cx.debug_bounds("tab-chevron_right-chevron_right").expect("the expand arrow");
    assert!(bottom(chevron) <= 560.0, "the arrow is in the window: {chevron:?}");
    let in_rail = chevron.origin.x >= rail.origin.x && chevron.origin.x < rail.origin.x + rail.size.width;
    assert!(in_rail, "the arrow found is the navbar's: {chevron:?} vs {rail:?}");
    // The arrow's bar ends a cap gap above the region's foot.
    assert!(bottom(chevron) <= bottom(rail) - gap + 0.51, "the arrow {chevron:?} stays inside the navbar {rail:?}");
    assert!(bottom(chevron) >= bottom(rail) - gap - 4.0, "and sits at its foot: {chevron:?} in {rail:?}");
    assert!(bottom(rail) <= 560.0, "the navbar itself fits the window: {rail:?}");
}

#[gpui::test]
async fn the_viewport_strips_end_a_cap_gap_off_the_canvas_bottom(cx: &mut TestAppContext) {
    let (app, cx) = cx.add_window_view(|_, _| BPhiberApp::new());
    cx.simulate_resize(size(px(1400.0), px(560.0)));
    app.update(cx, |app, cx| {
        app.state.remove(ModalKey::Open);
        app.state.set(CtxKey::Space, "layout");
        cx.notify();
    });
    cx.run_until_parked();
    let gap = phiber::v2::theme::ui_style_defaults(phiber::v2::theme::BPaletteName::DEFAULT).dimensions.menu_top_gap;
    let canvas = cx.debug_bounds("view3d-canvas").expect("the viewport");
    for side in ["pane-floating-Left-View3D", "pane-floating-Right-View3D"] {
        let strip = cx.debug_bounds(side).expect("a floating strip");
        assert!((bottom(canvas) - bottom(strip) - gap).abs() < 0.51, "{side} ends a cap gap off the canvas's foot: {strip:?} in {canvas:?}");
        assert!(bottom(strip) <= 560.0);
    }
}
