//! End to end: the wheel over the Timeline window zooms its time axis one
//! `view2d.zoom_in` step, Ctrl wheel scrolls it, Home frames it again —
//! and the Dope Sheet's own view is left alone (one `View2D.cur` a region).

use gpui::{Modifiers, Point, ScrollDelta, ScrollWheelEvent, TestAppContext, px, size};
use phiber::v2::app::BPhiberApp;
use phiber::v2::host::BHostType;
use phiber::v2::state::{BKey as _, TimeViewKey};

fn stored(app: &gpui::Entity<BPhiberApp>, cx: &mut gpui::VisualTestContext, host: BHostType) -> Option<(f32, f32)> {
    app.read_with(cx, |a, _| {
        let raw = a.state.get(TimeViewKey(host))?.to_string();
        let (a, b) = raw.split_once(',')?;
        Some((a.parse().ok()?, b.parse().ok()?))
    })
}

#[gpui::test]
async fn the_wheel_zooms_the_timeline_and_home_frames_it(cx: &mut TestAppContext) {
    let (app, cx) = cx.add_window_view(|_, _| BPhiberApp::new());
    cx.simulate_resize(size(px(1400.0), px(800.0)));
    app.update(cx, |app, cx| {
        app.state.remove(phiber::v2::state::ModalKey::Open);
        cx.notify();
    });
    cx.run_until_parked();
    let window = cx.debug_bounds("time-window").expect("the Timeline window is painted");
    let centre = window.center();
    assert_eq!(stored(&app, cx, BHostType::Time), None, "the Timeline opens on `action_create`'s view");

    cx.simulate_event(ScrollWheelEvent { position: centre, delta: ScrollDelta::Lines(Point::new(0.0, 1.0)), ..Default::default() });
    cx.run_until_parked();
    let (first, last) = stored(&app, cx, BHostType::Time).expect("a notch zooms the Timeline");
    // `view_zoomfac_default`: 3.75% of the 269 frames (−9 .. 260) off each end.
    assert!((first - (-9.0 + 269.0 * 0.0375)).abs() < 1e-2 && (last - (260.0 - 269.0 * 0.0375)).abs() < 1e-2, "{first}..{last}");
    assert_eq!(stored(&app, cx, BHostType::Action), None, "the Dope Sheet keeps its own view");

    cx.simulate_event(ScrollWheelEvent { position: centre, delta: ScrollDelta::Lines(Point::new(0.0, -1.0)), modifiers: Modifiers::control(), ..Default::default() });
    cx.run_until_parked();
    let (scrolled, _) = stored(&app, cx, BHostType::Time).unwrap();
    assert!(scrolled > first + 1.0, "Ctrl wheel down scrolls right: {first} → {scrolled}");

    cx.simulate_mouse_move(centre, None, Modifiers::none());
    cx.simulate_keystrokes("home");
    cx.run_until_parked();
    let (framed, framed_last) = stored(&app, cx, BHostType::Time).unwrap();
    // No keys in the default scene: the scene range, 1 .. 250, with four
    // diamonds' margin (`ANIM_frame_range_view2d_add_xmargin`).
    assert!(framed < 1.0 && framed > -12.0 && framed_last > 250.0 && framed_last < 263.0, "Home frames the scene range: {framed}..{framed_last}");
}
