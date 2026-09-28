//! Panel header gestures, as Blender's `handle_panel_header`
//! (`editors/interface/interface_panel.cc`): Ctrl-click opens this panel and
//! closes the rest of its region (`panels_collapse_all`); a press dragged
//! across headers flips the first and sets every header it crosses to the
//! same state (`panel_drag_collapse`). Checked on Preferences ▸ Interface,
//! whose three sections (Dimensions, Spacing, Typography) all start open.

use gpui::{Modifiers, MouseButton, TestAppContext, px, size};
use phiber::v2::app::BPhiberApp;
use phiber::v2::modal::BModalType;
use phiber::v2::space::BSpaceType;
use phiber::v2::state::{BState, CtxKey, ModalKey};

async fn settings(cx: &mut TestAppContext) -> &mut gpui::VisualTestContext {
    cx.update(|cx| cx.set_reduce_motion(true));
    let (app, cx) = cx.add_window_view(|_, _| BPhiberApp::new());
    cx.simulate_resize(size(px(1600.0), px(900.0)));
    app.update(cx, |app, cx| {
        app.state = BState::new();
        app.state.set(CtxKey::Space, BSpaceType::Layout.slug());
        app.state.set(ModalKey::Open, BModalType::Settings.slug());
        cx.notify();
    });
    cx.run_until_parked();
    cx
}

fn id(part: &str, section: &str) -> &'static str {
    Box::leak(format!("card-{part}-ctx-prefs-interface-{section}-expanded").into_boxed_str())
}

fn open(cx: &mut gpui::VisualTestContext, section: &str) -> bool {
    cx.debug_bounds(id("content", section)).is_some()
}

#[gpui::test]
async fn ctrl_click_keeps_this_panel_and_closes_the_rest(cx: &mut TestAppContext) {
    let cx = settings(cx).await;
    for s in ["dimensions", "spacing", "typography"] {
        assert!(open(cx, s), "{s} starts open");
    }
    let head = cx.debug_bounds(id("head", "spacing")).expect("Spacing header");
    cx.simulate_click(head.center(), Modifiers::control());
    cx.run_until_parked();
    assert!(open(cx, "spacing"), "the Ctrl-clicked panel stays open");
    assert!(!open(cx, "dimensions") && !open(cx, "typography"), "its region's other panels close");

    // A plain click still just folds the one header.
    let head = cx.debug_bounds(id("head", "dimensions")).expect("Dimensions header");
    cx.simulate_click(head.center(), Modifiers::none());
    cx.run_until_parked();
    assert!(open(cx, "dimensions") && open(cx, "spacing") && !open(cx, "typography"));
}

#[gpui::test]
async fn dragging_across_headers_sets_them_to_the_first_ones_new_state(cx: &mut TestAppContext) {
    let cx = settings(cx).await;
    // Close Typography first, so the sweep has a header already in its
    // target state to pass over.
    let typo = cx.debug_bounds(id("head", "typography")).expect("Typography header");
    cx.simulate_click(typo.center(), Modifiers::none());
    cx.run_until_parked();
    assert!(!open(cx, "typography"));

    let start = cx.debug_bounds(id("head", "dimensions")).expect("Dimensions header").center();
    cx.simulate_mouse_down(start, MouseButton::Left, Modifiers::none());
    // Past GPUI's drag threshold, still on the first header: it flips.
    cx.simulate_mouse_move(gpui::point(start.x + px(12.0), start.y), Some(MouseButton::Left), Modifiers::none());
    cx.run_until_parked();
    assert!(!open(cx, "dimensions"), "the first header flips when the drag starts");
    // Down over Spacing and Typography (their headers moved up as Dimensions closed).
    for s in ["spacing", "typography"] {
        let over = cx.debug_bounds(id("head", s)).expect("header").center();
        cx.simulate_mouse_move(over, Some(MouseButton::Left), Modifiers::none());
        cx.run_until_parked();
    }
    let end = cx.debug_bounds(id("head", "typography")).expect("header").center();
    cx.simulate_mouse_up(end, MouseButton::Left, Modifiers::none());
    cx.run_until_parked();
    assert!(!open(cx, "spacing"), "a header the sweep crossed took the sweep's state");
    assert!(!open(cx, "typography"), "one already in that state stays");
}
