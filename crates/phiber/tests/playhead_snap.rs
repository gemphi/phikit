//! The transport's magnet (`use_snap_playhead`) and its Playhead popover
//! (`TIME_PT_playhead_snapping`): with it on, a scrub lands on a marker
//! within reach; Ctrl flips it for that scrub; and the settings go to the
//! file on Ctrl S.

use gpui::{Modifiers, MouseButton, Point, TestAppContext, px, size};
use phiber::v2::app::BPhiberApp;
use phiber::v2::part::playback::{settings, snap};
use phiber::v2::state::{BKey as _, CtxKey, ModalKey, val::BValue};
use phiber::v2::view::BAction;

fn pt(x: f32, y: f32) -> Point<gpui::Pixels> {
    Point::new(px(x), px(y))
}

#[gpui::test]
async fn the_magnet_snaps_a_scrub_to_a_marker_and_ctrl_flips_it(cx: &mut TestAppContext) {
    let (app, cx) = cx.add_window_view(|_, _| BPhiberApp::new());
    cx.simulate_resize(size(px(1400.0), px(800.0)));
    app.update(cx, |app, cx| {
        app.state.remove(ModalKey::Open);
        app.state.set(CtxKey::Markers, "40=A");
        cx.notify();
    });
    cx.run_until_parked();
    assert!(cx.debug_bounds("btn-icon-set-ctx.header.time.panel.Playhead-true-chevron_down").is_some(), "the Playhead popover sits after the magnet");
    let window = cx.debug_bounds("time-window").expect("the Timeline window");
    let (left, width) = (f32::from(window.origin.x), f32::from(window.size.width));
    let mid_y = f32::from(window.origin.y) + f32::from(window.size.height) / 3.0;
    let x_of = |frame: f32| left + width * ((frame + 9.0) / 269.0);
    let press = |cx: &mut gpui::VisualTestContext, frame: f32, mods: Modifiers| {
        cx.simulate_mouse_down(pt(x_of(frame), mid_y), MouseButton::Left, mods);
        cx.simulate_mouse_up(pt(x_of(frame), mid_y), MouseButton::Left, mods);
        cx.run_until_parked();
    };
    let frame = |cx: &mut gpui::VisualTestContext| app.read_with(cx, |a, _| a.state.get(CtxKey::Frame).map(str::to_string));

    // Off: the scrub lands where it is.
    press(cx, 38.2, Modifiers::none());
    assert_eq!(frame(cx), Some("38".into()));

    // On, snapping to markers: the marker is within reach.
    app.update(cx, |app, cx| {
        app.handle_action(&BAction::Set(settings::key(snap::ON_KEY), BValue::Bool(true).encode()), cx);
        app.handle_action(&BAction::Set(settings::key(snap::TARGETS[2].key), BValue::Bool(true).encode()), cx);
    });
    cx.run_until_parked();
    press(cx, 38.2, Modifiers::none());
    assert_eq!(frame(cx), Some("40".into()), "snapped to marker A");

    // Ctrl flips the magnet for the scrub.
    press(cx, 38.2, Modifiers::control());
    assert_eq!(frame(cx), Some("38".into()));

    // What a save writes.
    let saved = app.read_with(cx, |a, _| settings::now(&a.state));
    assert!(saved.snap_playhead && saved.snap_to & phiber::phiphx::blend::SCE_SNAP_TO_MARKERS != 0);
}
