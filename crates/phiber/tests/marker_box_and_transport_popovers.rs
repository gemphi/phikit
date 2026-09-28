//! A drag over the marker band's empty part sweeps a box and selects the
//! markers in it (`marker.select_box`); the transport carries Blender's two
//! small popovers — Auto Keyframing after the record button, Time Jump
//! after the jump arrows — and the jump moves by what Time Jump says.

use gpui::{Modifiers, MouseButton, Point, TestAppContext, px, size};
use phiber::v2::app::BPhiberApp;
use phiber::v2::part::playback::settings;
use phiber::v2::scene::marker::BMarkerList;
use phiber::v2::state::{BKey as _, CtxKey, ModalKey, val::BValue};
use phiber::v2::view::BAction;
use phiber::v2::view::draw::canvas::time::marker::BAND_HEIGHT;

fn pt(x: f32, y: f32) -> Point<gpui::Pixels> {
    Point::new(px(x), px(y))
}

#[gpui::test]
async fn a_drag_over_the_empty_band_box_selects_the_markers_it_spans(cx: &mut TestAppContext) {
    let (app, cx) = cx.add_window_view(|_, _| BPhiberApp::new());
    cx.simulate_resize(size(px(1400.0), px(800.0)));
    app.update(cx, |app, cx| {
        app.state.remove(ModalKey::Open);
        app.state.set(CtxKey::Markers, "40=A|80=B|120=C|200=D");
        cx.notify();
    });
    cx.run_until_parked();
    let window = cx.debug_bounds("time-window").expect("the Timeline window is painted");
    let (left, width) = (f32::from(window.origin.x), f32::from(window.size.width));
    let (top, height) = (f32::from(window.origin.y), f32::from(window.size.height));
    let x_of = |frame: f32| left + width * ((frame + 9.0) / 269.0);
    let band_y = top + height - BAND_HEIGHT / 2.0;
    let frame_before = app.read_with(cx, |a, _| a.state.get(CtxKey::Frame).map(str::to_string));

    // From frame 60 (between A and B) to 140: B and C.
    cx.simulate_mouse_down(pt(x_of(60.0), band_y), MouseButton::Left, Modifiers::none());
    for frame in [70.0, 90.0, 110.0, 130.0, 140.0] {
        cx.simulate_mouse_move(pt(x_of(frame), band_y), Some(MouseButton::Left), Modifiers::none());
        cx.run_until_parked();
    }
    assert_eq!(app.read_with(cx, |a, _| BMarkerList::selected(&a.state)), [80, 120], "live, while the box is swept");
    assert!(app.read_with(cx, |a, _| phiber::v2::scene::grab::BMarkerBox::now(&a.state)).is_some(), "the box is drawn while the button is down");
    cx.simulate_mouse_up(pt(x_of(140.0), band_y), MouseButton::Left, Modifiers::none());
    cx.run_until_parked();
    assert!(app.read_with(cx, |a, _| phiber::v2::scene::grab::BMarkerBox::now(&a.state)).is_none(), "released: the box is gone");
    assert_eq!(app.read_with(cx, |a, _| BMarkerList::selected(&a.state)), [80, 120], "and B and C stay selected");
    assert_eq!(app.read_with(cx, |a, _| a.state.get(CtxKey::Frame).map(str::to_string)), frame_before, "a box is not a scrub");

    // Shift: a second box adds D to the selection.
    cx.simulate_mouse_down(pt(x_of(180.0), band_y), MouseButton::Left, Modifiers::shift());
    for frame in [190.0, 210.0, 220.0] {
        cx.simulate_mouse_move(pt(x_of(frame), band_y), Some(MouseButton::Left), Modifiers::shift());
        cx.run_until_parked();
    }
    cx.simulate_mouse_up(pt(x_of(220.0), band_y), MouseButton::Left, Modifiers::shift());
    cx.run_until_parked();
    assert_eq!(app.read_with(cx, |a, _| BMarkerList::selected(&a.state)), [80, 120, 200]);
}

#[gpui::test]
async fn the_time_jump_popover_sets_how_far_the_jump_buttons_go(cx: &mut TestAppContext) {
    let (app, cx) = cx.add_window_view(|_, _| BPhiberApp::new());
    cx.simulate_resize(size(px(1400.0), px(800.0)));
    app.update(cx, |app, cx| {
        app.state.remove(ModalKey::Open);
        app.state.set(CtxKey::Frame, "100");
        cx.notify();
    });
    cx.run_until_parked();
    for anchor in ["btn-icon-set-ctx.header.time.panel.Time Jump-true-chevron_down", "btn-icon-set-ctx.header.time.panel.Auto Keyframing-true-chevron_down"] {
        assert!(cx.debug_bounds(anchor).is_some(), "{anchor} is painted in the transport");
    }
    let open = cx.debug_bounds("btn-icon-set-ctx.header.time.panel.Time Jump-true-chevron_down").unwrap();
    cx.simulate_mouse_down(open.center(), MouseButton::Left, Modifiers::none());
    cx.simulate_mouse_up(open.center(), MouseButton::Left, Modifiers::none());
    cx.run_until_parked();
    assert!(cx.debug_bounds("section-Jump Unit").is_some() && cx.debug_bounds("section-Delta").is_some(), "the popover's rows");

    // Frames, 5: the forward jump moves five frames.
    app.update(cx, |app, cx| {
        app.handle_action(&BAction::Set(settings::key(settings::JUMP_UNIT_KEY), BValue::Enum("FRAME".into()).encode()), cx);
        app.handle_action(&BAction::Set(settings::key(settings::JUMP_DELTA_KEY), BValue::Float(5.0).encode()), cx);
    });
    cx.run_until_parked();
    let forward = cx.debug_bounds("btn-icon-set-ctx.frame_step-+5-chevron_right").expect("the forward jump now steps five");
    cx.simulate_mouse_down(forward.center(), MouseButton::Left, Modifiers::none());
    cx.simulate_mouse_up(forward.center(), MouseButton::Left, Modifiers::none());
    cx.run_until_parked();
    assert_eq!(app.read_with(cx, |a, _| a.state.get(CtxKey::Frame).map(str::to_string)), Some("105".into()));
}
