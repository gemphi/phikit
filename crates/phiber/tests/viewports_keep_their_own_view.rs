//! Every 3D Viewport has a view of its own, as every Blender 3D Viewport is
//! its own `RegionView3D`: orbiting, zooming or framing the one under the
//! pointer leaves the others where they were. The Animation workspace has
//! two — the camera view on the left and the user view in the centre.

use gpui::{Modifiers, Point, ScrollDelta, ScrollWheelEvent, TestAppContext, px, size};
use phiber::v2::app::BPhiberApp;
use phiber::v2::scene::view_slot::BViewSlot;
use phiber::v2::space::BSpaceType;
use phiber::v2::state::{BKey as _, CtxKey, ModalKey, OpKey, View3dKey};
use phiber::v2::view::BAction;

#[gpui::test]
async fn the_view_keys_act_on_the_viewport_under_the_pointer(cx: &mut TestAppContext) {
    BViewSlot::forget_hover();
    let (app, cx) = cx.add_window_view(|_, _| BPhiberApp::new());
    cx.simulate_resize(size(px(1100.0), px(700.0)));
    app.update(cx, |app, cx| {
        app.state.remove(ModalKey::Open);
        app.state.set(CtxKey::Space, BSpaceType::Animation.slug());
        cx.notify();
    });
    cx.run_until_parked();
    let scope = BSpaceType::Animation.slug();
    let own = |cx: &mut gpui::VisualTestContext, slot: &'static str, field: &'static str| {
        app.read_with(cx, |a, _| a.state.get(View3dKey::Slot(scope, slot, field)).map(str::to_string))
    };
    let centre = cx.debug_bounds("view3d-canvas").expect("a 3D viewport canvas").center();
    let left = Point::new(px(200.0), px(180.0));

    // Over the centre viewport: Numpad 4 orbits it alone.
    cx.simulate_mouse_move(centre, None, Modifiers::none());
    app.update(cx, |app, cx| app.handle_action(&BAction::Set(OpKey::View3d.path(), "orbit_left".into()), cx));
    cx.run_until_parked();
    assert!(own(cx, "Center", "yaw").is_some(), "the centre viewport turned");
    assert_eq!(own(cx, "Left", "yaw"), None, "the left one did not");

    // Over the left one: Numpad 8 tilts it alone.
    cx.simulate_mouse_move(left, None, Modifiers::none());
    app.update(cx, |app, cx| app.handle_action(&BAction::Set(OpKey::View3d.path(), "orbit_up".into()), cx));
    cx.run_until_parked();
    assert!(own(cx, "Left", "pitch").is_some());
    assert_eq!(own(cx, "Center", "pitch"), None);

    // The wheel zooms the viewport it is over.
    cx.simulate_event(ScrollWheelEvent { position: centre, delta: ScrollDelta::Lines(Point::new(0.0, 2.0)), ..Default::default() });
    cx.run_until_parked();
    assert!(own(cx, "Center", "zoom").is_some() && own(cx, "Left", "zoom").is_none(), "the wheel zooms the centre alone");

    // Nothing went to the shared view.
    assert_eq!(app.read_with(cx, |a, _| a.state.get(View3dKey::Yaw).map(str::to_string)), None);
    BViewSlot::forget_hover();
}
