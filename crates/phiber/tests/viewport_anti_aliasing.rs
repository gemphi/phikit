//! Preferences ▸ Viewport ▸ Quality ▸ Viewport Anti-Aliasing
//! (`U.viewport_aa`): Blender's seven choices over `ctx.pref.viewport_aa`,
//! eight samples until set, and a change reaches the Solid viewport as a
//! new view — the tracer's count (`Tracer::set_aa_samples`) rides with the
//! shot, so the samples start over at the new count.

use gpui::{TestAppContext, px, size};
use phiber::v2::app::BPhiberApp;
use phiber::v2::modal::BModalType;
use phiber::v2::scene::viewport_aa::BViewportAa;
use phiber::v2::space::BSpaceType;
use phiber::v2::state::{AppKey, BKey as _, BPath, BState, CtxKey, ModalKey};
use phiber::v2::view::draw::canvas::view3d::rendered::views_posted;
use phiber::v2::view::prim::BAction;

#[gpui::test]
async fn the_quality_panel_offers_the_counts_and_a_choice_reposts_the_view(cx: &mut TestAppContext) {
    cx.update(|cx| cx.set_reduce_motion(true));
    let (app, cx) = cx.add_window_view(|_, _| BPhiberApp::new());
    cx.simulate_resize(size(px(1600.0), px(900.0)));
    app.update(cx, |app, cx| {
        app.state = BState::new();
        app.state.set(CtxKey::Space, BSpaceType::Layout.slug());
        app.state.set(ModalKey::Open, BModalType::Settings.slug());
        app.state.set(&BPath::owned(vec![phiber::v2::host::BHostType::Prefs.slug(), "navbar", "tab"]), "Viewport");
        cx.notify();
    });
    cx.run_until_parked();
    assert!(cx.debug_bounds("card-head-ctx-prefs-viewport-quality-expanded").is_some(), "the Quality panel under Viewport");
    let id: &'static str = Box::leak(format!("btn-label-toggle-{}-text", BPath::owned(vec!["buts", "dropdown", "pref.viewport_aa"]).encode()).into_boxed_str());
    assert!(cx.debug_bounds(id).is_some(), "the Viewport Anti-Aliasing select");
    assert_eq!(app.read_with(cx, |a, _| BViewportAa::samples(&a.state)), 8, "`U.viewport_aa`'s default");

    // Back in the viewport, Solid: the view is posted once; a new count
    // posts it again, the old samples dropped (`SceneState::reset_taa`).
    app.update(cx, |app, cx| {
        app.state.remove(ModalKey::Open);
        cx.notify();
    });
    cx.run_until_parked();
    let posted = views_posted();
    assert!(posted > 0, "the viewport posted its view");
    app.update(cx, |app, cx| app.handle_action(&BAction::Set(AppKey::ViewportAa.path(), "FXAA".into()), cx));
    cx.run_until_parked();
    assert_eq!(app.read_with(cx, |a, _| BViewportAa::samples(&a.state)), 1);
    assert_eq!(views_posted(), posted + 1, "the count is part of the view");
    app.update(cx, |app, cx| app.handle_action(&BAction::Set(AppKey::ViewportAa.path(), "OFF".into()), cx));
    cx.run_until_parked();
    assert_eq!(views_posted(), posted + 2);
}
