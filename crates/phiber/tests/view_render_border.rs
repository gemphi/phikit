//! The viewport's own render region (`View3D.render_border`,
//! `V3D_RENDER_BORDER`): Ctrl B outside the camera view sets it — the
//! scene's region (`Scene.r.border`) untouched — the sidebar's View ▸
//! Render Region is its switch, Ctrl Alt B clears whichever the viewport at
//! hand is looking at, and a file's view opens with its own.

use gpui::{TestAppContext, px, size};
use phiber::phiphx::blend::{BlendRenderBorder, BlendViewBorder};
use phiber::v2::app::BPhiberApp;
use phiber::v2::scene::render_border::BRenderBorder;
use phiber::v2::scene::view_border::BViewBorder;
use phiber::v2::scene::view_slot::BViewSlot;
use phiber::v2::state::{BKey as _, CtxKey, ModalKey, OpKey, View3dKey, val::V_TRUE};
use phiber::v2::view::BAction;

fn region(v: &str) -> BAction {
    BAction::Set(OpKey::RenderBorder.path(), v.into())
}

#[gpui::test]
async fn ctrl_b_outside_the_camera_view_sets_the_viewports_own_region(cx: &mut TestAppContext) {
    let (app, cx) = cx.add_window_view(|_, _| BPhiberApp::new());
    cx.simulate_resize(size(px(1400.0), px(800.0)));
    app.update(cx, |app, cx| {
        app.state.remove(ModalKey::Open);
        app.handle_action(&BAction::Set(CtxKey::Object.path(), "Cube".into()), cx);
    });
    cx.run_until_parked();
    // The viewport of the workspace showing, the pointer over it, not
    // looking through the camera.
    let scope = app.read_with(cx, |a, _| phiber::v2::space::BSpaceList::active(&a.state).slug());
    let slot = BViewSlot::new(scope, "Center");
    slot.hover();
    assert!(app.read_with(cx, |a, _| a.state.get(View3dKey::Camera(scope, "Center")) != Some(V_TRUE)));

    // Ctrl B arms the gesture; its box, outside the camera view, is the
    // viewport's own region (`render_border_exec`'s other branch).
    app.update(cx, |app, cx| app.handle_action(&region("arm"), cx));
    assert_eq!(app.read_with(cx, |a, _| a.state.get(View3dKey::RenderBorder) == Some(V_TRUE)), true);
    let rect = BViewBorder::from_region_box((700.0, 400.0, 1400.0, 800.0), (1400.0, 800.0)).unwrap();
    assert_eq!(rect, [0.5, 1.0, 0.0, 0.5]);
    app.update(cx, |app, cx| {
        app.handle_action(&BAction::Set(View3dKey::RenderBorder.path(), String::new()), cx);
        app.handle_action(&region(&format!("view:{}", BRenderBorder::encode(rect))), cx);
    });
    cx.run_until_parked();
    app.read_with(cx, |a, _| {
        assert_eq!(BViewBorder::now(&a.state), BlendViewBorder { on: true, rect });
        assert_eq!(BRenderBorder::now(&a.state, BlendRenderBorder::default()), BlendRenderBorder::default(), "the scene's region is not touched");
        // Only an engine that draws the viewport itself fills the region.
        assert_eq!(BViewBorder::fills(&a.state, "CYCLES", 1400, 800), Some((700, 400, 700, 400)));
        assert_eq!(BViewBorder::fills(&a.state, "BLENDER_WORKBENCH", 1400, 800), None);
    });

    // The sidebar's View tab: Render Region is the region's switch.
    let view = cx.debug_bounds("tab-view_layer-view_layer").expect("the View tab").center();
    cx.simulate_click(view, gpui::Modifiers::none());
    cx.run_until_parked();
    let id: &'static str = Box::leak(format!("btn-icon-set-{}-false-check_square", View3dKey::BorderUse.path().encode()).into_boxed_str());
    assert!(cx.debug_bounds(id).is_some(), "View ▸ Render Region, on, offers to turn it off");
    app.update(cx, |app, cx| app.handle_action(&BAction::Set(View3dKey::BorderUse.path(), "false".into()), cx));
    cx.run_until_parked();
    assert_eq!(app.read_with(cx, |a, _| BViewBorder::now(&a.state)), BlendViewBorder { on: false, rect }, "off keeps the rectangle");
    app.update(cx, |app, cx| app.handle_action(&BAction::Set(View3dKey::BorderUse.path(), "true".into()), cx));

    // Ctrl Alt B over this viewport clears its own region; the scene's,
    // set meanwhile, stays.
    app.update(cx, |app, cx| {
        app.handle_action(&region("set:0.25,0.75,0.25,0.75"), cx);
        app.handle_action(&region("clear"), cx);
    });
    cx.run_until_parked();
    app.read_with(cx, |a, _| {
        assert_eq!(BViewBorder::now(&a.state), BlendViewBorder::default());
        assert_eq!(BRenderBorder::now(&a.state, BlendRenderBorder::default()).rect, [0.25, 0.75, 0.25, 0.75]);
    });
    // Looking through the camera, Ctrl Alt B clears the scene's instead.
    app.update(cx, |app, cx| {
        app.handle_action(&region(&format!("view:{}", BRenderBorder::encode(rect))), cx);
        app.state.set(View3dKey::Camera(scope, "Center"), V_TRUE);
        app.handle_action(&region("clear"), cx);
    });
    app.read_with(cx, |a, _| {
        assert_eq!(BRenderBorder::now(&a.state, BlendRenderBorder::default()), BlendRenderBorder::default());
        assert_eq!(BViewBorder::now(&a.state), BlendViewBorder { on: true, rect }, "the viewport's own is kept");
    });
    BViewSlot::forget_hover();
}
