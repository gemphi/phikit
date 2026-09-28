//! The action log at the foot of the window, measured on the painted UI.
//!
//! The status bar starts folded: one row, the newest line beside a chevron.
//! Every action the user takes is written as a sentence; unfolding opens a
//! drawer upward over the workspace that holds them all in a scroller, and
//! the status bar itself keeps its height.

use phiber::v2::state::{BKey as _, AppKey, CtxKey};
use gpui::{Pixels as px_t, TestAppContext, px, size};
use phiber::v2::app::BPhiberApp;
use phiber::v2::space::BSpaceType;
use phiber::v2::state::log;
use phiber::v2::state::{BPath, BState};
use phiber::v2::view::prim::BAction;

fn zoom(value: &str) -> BAction {
    BAction::Set(BPath::owned(vec!["view3d", "zoom"]), value.to_string())
}

/// The Layout workspace, with motion off so a drawer is painted at rest.
async fn workspace(
    cx: &mut TestAppContext,
) -> (gpui::Entity<BPhiberApp>, &mut gpui::VisualTestContext) {
    log::clear();
    cx.update(|cx| cx.set_reduce_motion(true));
    let (app, cx) = cx.add_window_view(|_, _| BPhiberApp::new());
    cx.simulate_resize(size(px(1280.0), px(800.0)));
    app.update(cx, |app, cx| {
        app.state = BState::new();
        app.state.set(CtxKey::Space, BSpaceType::Layout.slug());
        cx.notify();
    });
    cx.run_until_parked();
    (app, cx)
}

fn bounds(cx: &mut gpui::VisualTestContext, id: &'static str) -> gpui::Bounds<px_t> {
    cx.debug_bounds(id).unwrap_or_else(|| panic!("{id} was not painted"))
}

#[gpui::test]
async fn the_log_starts_folded_in_the_status_bar(cx: &mut TestAppContext) {
    let (_app, cx) = workspace(cx).await;
    let head = bounds(cx, "status-log-head");
    let bar = bounds(cx, "region-Header-Statusbar");
    assert!(cx.debug_bounds("status-log-drawer").is_none(), "the log must start folded");
    assert!(
        head.origin.y >= bar.origin.y && head.origin.y + head.size.height <= bar.origin.y + bar.size.height,
        "the folded log is one row inside the status bar"
    );
}

#[gpui::test]
async fn every_action_writes_a_line(cx: &mut TestAppContext) {
    let (app, cx) = workspace(cx).await;
    app.update(cx, |app, cx| {
        app.handle_action(&zoom("1.7"), cx);
        app.handle_action(&BAction::Set(BPath::owned(vec!["space"]), "shading".to_string()), cx);
    });
    cx.run_until_parked();
    let lines: Vec<String> = log::entries().iter().map(|e| e.text.clone()).collect();
    assert_eq!(lines, ["Switched to Shading", "Zoomed to 1.70×"], "newest first, one sentence each");
}

#[gpui::test]
async fn unfolding_opens_a_drawer_above_the_bar_and_keeps_the_bar_its_height(
    cx: &mut TestAppContext,
) {
    let (app, cx) = workspace(cx).await;
    let bar_before = bounds(cx, "region-Header-Statusbar");
    app.update(cx, |app, cx| {
        app.handle_action(&zoom("2"), cx);
        app.handle_action(&BAction::Toggle(AppKey::StatusLogOpen.path()), cx);
    });
    cx.run_until_parked();

    let bar = bounds(cx, "region-Header-Statusbar");
    let drawer = bounds(cx, "status-log-drawer");
    let list = bounds(cx, "status-log-list");
    let head = bounds(cx, "status-log-head");
    assert_eq!(bar, bar_before, "the status bar never grows; the log is a drawer over the workspace");
    assert_eq!(
        drawer.origin.y + drawer.size.height,
        head.origin.y,
        "the drawer's foot sits on the head row it opened from"
    );
    assert!(drawer.size.height > head.size.height * 3.0, "the drawer shows several rows");
    assert!(
        list.origin.y >= drawer.origin.y && list.size.height <= drawer.size.height,
        "the list scrolls inside the drawer"
    );
    // Opening the log is itself an action — it is on the list.
    let lines: Vec<String> = log::entries().iter().map(|e| e.text.clone()).collect();
    assert!(lines.iter().any(|l| l.contains("Zoomed to 2.00×")), "{lines:?}");
}
