//! Preferences sections are Blender's `USERPREF_PT_*` panels: each folds,
//! and the theme picker (`USERPREF_PT_theme`, `HIDE_HEADER`) has no heading.

use gpui::{TestAppContext, px, size};
use phiber::v2::app::BPhiberApp;
use phiber::v2::modal::BModalType;
use phiber::v2::space::BSpaceType;
use phiber::v2::state::{BState, CtxKey, ModalKey, action::apply_action};
use phiber::v2::view::prim::BAction;

async fn settings(cx: &mut TestAppContext) -> (gpui::Entity<BPhiberApp>, &mut gpui::VisualTestContext) {
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
    (app, cx)
}

const HEAD: &str = "card-head-ctx-prefs-interface-dimensions-expanded";
const FOLD: &str = "card-fold-ctx-prefs-interface-dimensions-expanded";
const BODY: &str = "card-content-ctx-prefs-interface-dimensions-expanded";

#[gpui::test]
async fn a_preference_section_is_a_panel_that_folds(cx: &mut TestAppContext) {
    let (app, cx) = settings(cx).await;
    assert!(cx.debug_bounds(HEAD).is_some(), "the Dimensions panel header is painted");
    assert!(cx.debug_bounds(FOLD).is_some(), "it has Blender's fold triangle");
    assert!(cx.debug_bounds(BODY).is_some(), "and is open by default");

    app.update(cx, |app, cx| {
        let key = phiber::v2::part::props::BPanelItem::expand_key(
            phiber::v2::host::BHostType::Prefs,
            "Interface",
            "Dimensions",
        );
        apply_action(&mut app.state, &BAction::Set(key, "false".into()));
        cx.notify();
    });
    cx.run_until_parked();
    assert!(cx.debug_bounds(HEAD).is_some(), "a closed panel keeps its header");
    assert!(cx.debug_bounds(BODY).is_none(), "and drops its body");
}

