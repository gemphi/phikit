//! The viewport's first GPU image lands without a hand on the mouse.
//!
//! The render thread's frames reach the window through the ray ticker
//! (`ensure_ray_ticker`), armed from the last paint's verdict that a
//! viewport left its image on its way (`rendered::refining`). `render`
//! asked after `drawing` had cleared that flag for the draw in hand, so it
//! never saw it, and the viewport refreshed only on input: a session's
//! first image waited for the first click (four hours, in one journal),
//! and so did every frame a refinement finished with. Now `render` asks
//! first, and a paint that leaves its view refining with no ticker running
//! asks for one more frame itself (`wake::ticking`).

use gpui::{TestAppContext, px, size};
use phiber::v2::app::BPhiberApp;

#[gpui::test]
async fn the_ticker_runs_after_the_first_paint_with_no_input(cx: &mut TestAppContext) {
    if phiray::Gpu::shared().is_none() {
        return;
    }
    let (app, cx) = cx.add_window_view(|_, _| BPhiberApp::new());
    cx.simulate_resize(size(px(1200.0), px(700.0)));
    app.update(cx, |app, cx| {
        app.state.remove(phiber::v2::state::ModalKey::Open);
        cx.notify();
    });
    cx.run_until_parked();
    assert!(phiber::v2::view::draw::canvas::view3d::rendered::views_posted() > 0, "the viewport posted its view");
    // No input, no notify: the frames the paint itself asks for are all
    // there is — the platform delivers those (`simulate_next_frame` stands
    // in for its frame loop) — and they must be enough to start the ticker.
    for _ in 0..2 {
        app.update_in(cx, |_, window, cx| window.simulate_next_frame(cx));
        cx.run_until_parked();
    }
    assert!(app.read_with(cx, |a, _| a.draws_on_frames()), "the ticker never started: the first image would wait for a click");
}
