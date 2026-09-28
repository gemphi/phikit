//! The viewport's own chrome: the tool rail on the left, the sidebar on the
//! right, and the outliner showing the scene in every workspace.

use phiber::v2::state::{CtxKey};
use gpui::{TestAppContext, px, size};
use phiber::v2::app::BPhiberApp;
use phiber::v2::space::BSpaceType;
use phiber::v2::state::{BState};

async fn workspace(cx: &mut TestAppContext, space: BSpaceType) -> &mut gpui::VisualTestContext {
    let (app, cx) = cx.add_window_view(|_, _| BPhiberApp::new());
    cx.simulate_resize(size(px(1600.0), px(900.0)));
    app.update(cx, |app, cx| {
        app.state = BState::new();
        app.state.set(CtxKey::Space, space.slug());
        cx.notify();
    });
    cx.run_until_parked();
    cx
}

#[gpui::test]
async fn the_viewport_shows_its_tool_rail_and_sidebar(cx: &mut TestAppContext) {
    let cx = workspace(cx, BSpaceType::Layout).await;
    let canvas = cx.debug_bounds("view3d-canvas").expect("the viewport canvas");
    let rail = cx
        .debug_bounds("pane-floating-Left-View3D")
        .expect("the viewport's tool rail is painted");
    let side = cx
        .debug_bounds("pane-floating-Right-View3D")
        .expect("the viewport's sidebar is painted");
    assert!(
        f32::from(rail.origin.x) >= f32::from(canvas.origin.x) - 1.0,
        "the rail sits inside the viewport"
    );
    assert!(
        f32::from(side.origin.x) + f32::from(side.size.width)
            <= f32::from(canvas.origin.x) + f32::from(canvas.size.width) + 1.0,
        "and so does the sidebar"
    );
}
