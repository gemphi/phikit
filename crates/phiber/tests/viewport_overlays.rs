//! The viewport's overlays stay inside the viewport.
//!
//! The header's side panes were once rendered as floating overlays, which put
//! the menu bar over the canvas and the gizmo among the menus. Overlays belong
//! to the content region, and they keep clear of the floating strips that
//! share their corners.

use phiber::v2::state::{CtxKey};
use gpui::{Pixels as px_t, TestAppContext, px, size};
use phiber::v2::app::BPhiberApp;
use phiber::v2::space::BSpaceType;
use phiber::v2::state::{BState};

async fn viewport(cx: &mut TestAppContext) -> &mut gpui::VisualTestContext {
    let (app, cx) = cx.add_window_view(|_, _| BPhiberApp::new());
    cx.simulate_resize(size(px(1280.0), px(800.0)));
    app.update(cx, |app, cx| {
        app.state = BState::new();
        app.state.set(CtxKey::Space, BSpaceType::Layout.slug());
        cx.notify();
    });
    cx.run_until_parked();
    cx
}

fn bounds(cx: &mut gpui::VisualTestContext, id: &'static str) -> gpui::Bounds<px_t> {
    cx.debug_bounds(id)
        .unwrap_or_else(|| panic!("{id} was not painted"))
}

#[gpui::test]
async fn the_gizmo_sits_below_the_header_not_among_its_menus(cx: &mut TestAppContext) {
    let cx = viewport(cx).await;
    let header = bounds(cx, "region-Header-View3D");
    let gizmo = bounds(cx, "gizmo-compass");

    let header_bottom = f32::from(header.origin.y + header.size.height);
    assert!(
        f32::from(gizmo.origin.y) >= header_bottom,
        "the gizmo must start below the header: gizmo={} header ends {header_bottom}",
        f32::from(gizmo.origin.y)
    );
}

#[gpui::test]
async fn the_gizmo_stays_inside_the_content_region(cx: &mut TestAppContext) {
    let cx = viewport(cx).await;
    let content = bounds(cx, "region-Center-View3D");
    let gizmo = bounds(cx, "gizmo-compass");

    let (left, right) = (
        f32::from(content.origin.x),
        f32::from(content.origin.x + content.size.width),
    );
    let (top, bottom) = (
        f32::from(content.origin.y),
        f32::from(content.origin.y + content.size.height),
    );
    assert!(f32::from(gizmo.origin.x) >= left, "gizmo escapes to the left");
    assert!(
        f32::from(gizmo.origin.x + gizmo.size.width) <= right,
        "gizmo escapes to the right"
    );
    assert!(f32::from(gizmo.origin.y) >= top, "gizmo escapes upwards");
    assert!(
        f32::from(gizmo.origin.y + gizmo.size.height) <= bottom,
        "gizmo escapes downwards"
    );
}

#[gpui::test]
async fn the_gizmo_keeps_clear_of_the_floating_sidebar(cx: &mut TestAppContext) {
    let cx = viewport(cx).await;
    let sidebar = bounds(cx, "pane-floating-Right-View3D");
    let gizmo = bounds(cx, "gizmo-compass");

    assert!(
        f32::from(gizmo.origin.x + gizmo.size.width) <= f32::from(sidebar.origin.x),
        "the gizmo must not run under the sidebar: gizmo ends {}, sidebar starts {}",
        f32::from(gizmo.origin.x + gizmo.size.width),
        f32::from(sidebar.origin.x)
    );
}

#[gpui::test]
async fn the_canvas_fills_its_region_exactly(cx: &mut TestAppContext) {
    let cx = viewport(cx).await;
    let content = bounds(cx, "region-Center-View3D");
    let canvas = bounds(cx, "view3d-canvas");
    assert_eq!(
        (
            f32::from(canvas.origin.x),
            f32::from(canvas.origin.x + canvas.size.width)
        ),
        (
            f32::from(content.origin.x),
            f32::from(content.origin.x + content.size.width)
        ),
        "a canvas host paints edge to edge"
    );
}
