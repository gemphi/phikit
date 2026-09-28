//! The top bar wraps when the window is too narrow for its three cells in
//! one line: the left cell (the brand and the File… menus) keeps the first
//! line and the workspace tabs and the right-hand controls go under it,
//! the bar growing a line. Wide enough, it is one strip tall as ever. An
//! editor's header never wraps — Blender clips and scrolls it — so an
//! area's chrome keeps its height whatever its width.

use gpui::{Bounds, Pixels, TestAppContext, px, size};
use phiber::v2::app::BPhiberApp;
use phiber::v2::space::BSpaceType;
use phiber::v2::state::{BState, CtxKey, ModalKey};
use phiber::v2::theme::METRICS;

fn bounds(cx: &mut gpui::VisualTestContext, ids: &[&'static str]) -> Option<Bounds<Pixels>> {
    ids.iter().find_map(|id| cx.debug_bounds(id))
}

const TOPBAR: &[&str] = &["region-Header-Topbar", "region-ToolHeader-Topbar"];

#[gpui::test]
async fn a_narrow_window_puts_the_tabs_and_controls_under_the_menus(cx: &mut TestAppContext) {
    let (app, cx) = cx.add_window_view(|_, _| BPhiberApp::new());
    cx.simulate_resize(size(px(1600.0), px(900.0)));
    app.update(cx, |app, cx| {
        app.state = BState::new();
        app.state.set(CtxKey::Space, BSpaceType::Layout.slug());
        app.state.set(ModalKey::Open, "");
        cx.notify();
    });
    cx.run_until_parked();
    let one = bounds(cx, TOPBAR).expect("the top bar");
    assert_eq!(f32::from(one.size.height), METRICS.h_header, "wide: one strip tall");
    let viewport_one = bounds(cx, &["region-Header-View3D"]).expect("the viewport header");

    cx.simulate_resize(size(px(800.0), px(900.0)));
    cx.run_until_parked();
    let two = bounds(cx, TOPBAR).expect("the top bar");
    assert_eq!(f32::from(two.size.height), 2.0 * METRICS.h_header + METRICS.gap_xs, "narrow: two lines");
    let left = bounds(cx, &["pane-Left-Topbar"]).expect("the left cell");
    let center = bounds(cx, &["pane-Center-Topbar"]).expect("the centre cell");
    let right = bounds(cx, &["pane-Right-Topbar"]).expect("the right cell");
    assert_eq!(f32::from(left.size.width), 800.0 - 2.0 * METRICS.pad_md, "the left cell takes the first line");
    assert!(center.origin.y > left.origin.y && right.origin.y == center.origin.y, "the tabs and the controls share the second line");
    assert!(right.origin.x + right.size.width <= two.origin.x + two.size.width, "the controls stay in the window");
    // An editor's header keeps its height.
    let viewport_two = bounds(cx, &["region-Header-View3D"]).expect("the viewport header");
    assert_eq!(viewport_two.size.height, viewport_one.size.height);

    cx.simulate_resize(size(px(1600.0), px(900.0)));
    cx.run_until_parked();
    assert_eq!(bounds(cx, TOPBAR).map(|b| b.size.height), Some(one.size.height), "and back to one");
}
