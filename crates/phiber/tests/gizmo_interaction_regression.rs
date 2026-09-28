//! Runtime regression for the redesigned View3D navigate gizmo (DEFECT 2).
//!
//! Feature: phiber-blender-space-parity
//! Validates: Requirement 13.6 (View3D navigate gizmo renders + is interactive)
//!
//! GPUI UI automation cannot inspect custom-painted surfaces pixel-by-pixel,
//! so instead this drives the REAL rendered gizmo in a `TestAppContext`
//! window: it selects a View3D space, asserts the axis compass and the zoom
//! controls actually prepaint with non-zero bounds (i.e. they are laid out and
//! visible), then clicks a zoom control and a compass axis ball and asserts the
//! backing `view3d` state changed. This guards that the redesign kept the
//! cluster both visible and wired to the existing yaw/zoom actions.

use phiber::v2::state::{CtxKey};
use gpui::{Modifiers, TestAppContext, px, size};
use phiber::v2::app::BPhiberApp;
use phiber::v2::host::BHostType;
use phiber::v2::scene::{view_yaw, view_zoom};
use phiber::v2::space::BSpaceType;
use phiber::v2::state::BState;

fn select_view3d(state: &mut BState, space: BSpaceType) {
    state.set(CtxKey::Space, space.slug());
    // The welcome window is seeded open on a fresh state; dismiss it so the
    // workspace under it is what these assertions see.
    state.set(phiber::v2::state::ModalKey::Open, "");
    // Area slots are keyed per space and per slot, so the editor chosen for
    // Layout's centre area cannot leak into Shading's (see `v2::area::current`).
    for area in space
        .spec()
        .areas
        .iter()
        .filter(|a| a.hosts.contains(&BHostType::View3D))
    {
        state.set(
            &phiber::v2::area::area_key(space, area),
            BHostType::View3D.slug(),
        );
    }
}

#[gpui::test]
async fn navigate_gizmo_paints_and_stays_interactive(cx: &mut TestAppContext) {
    let (app, cx) = cx.add_window_view(|_, _| BPhiberApp::new());
    cx.simulate_resize(size(px(1280.0), px(800.0)));

    app.update(cx, |app, cx| {
        select_view3d(&mut app.state, BSpaceType::Modeling);
        cx.notify();
    });
    cx.run_until_parked();

    // The axis compass must be laid out and visible.
    let compass = cx
        .debug_bounds("gizmo-compass")
        .expect("navigate gizmo compass was not painted");
    assert!(
        f32::from(compass.size.width) > 10.0 && f32::from(compass.size.height) > 10.0,
        "compass must have a real footprint, got {:?}",
        compass.size
    );

    // The zoom-in control must be laid out and visible.
    let zoom_in = cx
        .debug_bounds("gizmo-zoom_in")
        .expect("gizmo zoom-in control was not painted");
    assert!(
        f32::from(zoom_in.size.width) > 4.0 && f32::from(zoom_in.size.height) > 4.0,
        "zoom-in control must have a real footprint, got {:?}",
        zoom_in.size
    );

    // Clicking zoom-in must change the view zoom (gizmo stays wired to state)
    // — this viewport's own (`scene::view_slot`).
    let slot = phiber::v2::scene::view_slot::BViewSlot::new(BSpaceType::Modeling.slug(), "Center");
    let z0 = app.read_with(cx, |a, _| slot.with(|| view_zoom(&a.state)));
    cx.simulate_click(zoom_in.center(), Modifiers::none());
    cx.run_until_parked();
    let z1 = app.read_with(cx, |a, _| slot.with(|| view_zoom(&a.state)));
    assert!(z1 > z0, "zoom-in should increase view zoom: {z0} -> {z1}");

    // Clicking a compass axis ball must snap the view yaw.
    let y0 = app.read_with(cx, |a, _| slot.with(|| view_yaw(&a.state)));
    let xball = cx
        .debug_bounds("compass-X-pos")
        .expect("compass +X axis ball was not painted");
    cx.simulate_click(xball.center(), Modifiers::none());
    cx.run_until_parked();
    let y1 = app.read_with(cx, |a, _| slot.with(|| view_yaw(&a.state)));
    assert!(
        (y1 - y0).abs() > f32::EPSILON,
        "clicking +X axis ball should change yaw: {y0} -> {y1}"
    );
}

/// The compass is the viewport's own projection of the three world axes, so
/// it has to tip with pitch as well as turn with yaw. It used to track yaw
/// only, which left Z pinned to the top however the view was rotated.
#[gpui::test]
async fn the_compass_tips_with_pitch_not_just_yaw(cx: &mut TestAppContext) {
    let (app, cx) = cx.add_window_view(|_, _| BPhiberApp::new());
    cx.simulate_resize(size(px(1600.0), px(900.0)));
    app.update(cx, |app, cx| {
        app.state = BState::new();
        app.state.set(CtxKey::Space, BSpaceType::Layout.slug());
        app.state.set(&phiber::v2::scene::pitch_key(), "10");
        cx.notify();
    });
    cx.run_until_parked();
    let low = cx.debug_bounds("compass-Z-pos").expect("+Z cap");

    // Look down on the scene: +Z swings towards the middle of the compass.
    app.update(cx, |app, cx| {
        app.state.set(&phiber::v2::scene::pitch_key(), "80");
        cx.notify();
    });
    cx.run_until_parked();
    let steep = cx.debug_bounds("compass-Z-pos").expect("+Z cap after pitching");
    assert_ne!(
        f32::from(low.origin.y),
        f32::from(steep.origin.y),
        "the +Z cap moves when the view pitches"
    );
}
