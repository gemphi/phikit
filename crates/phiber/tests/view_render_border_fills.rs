//! Rendered shading fills the viewport's own render region alone
//! (`View3D.render_border`, `ED_view3d_calc_render_border`), and Workbench
//! as the engine fills the whole viewport whatever the region says. Its
//! own binary: it drives the GPU tracer to full size twice, and shares a
//! process badly with the other viewport test.

use gpui::{TestAppContext, px, size};
use phiber::v2::app::BPhiberApp;
use phiber::v2::state::{BKey as _, ModalKey, OpKey};
use phiber::v2::view::BAction;

fn region(v: &str) -> BAction {
    BAction::Set(OpKey::RenderBorder.path(), v.into())
}

/// The image traced is the region's pixels of the viewport's frame.
#[gpui::test]
async fn rendered_shading_fills_the_region_alone(cx: &mut TestAppContext) {
    if phiray::Gpu::shared().is_none() {
        return;
    }
    use phiber::v2::view::draw::canvas::view3d::rendered::shown_image;
    let (app, cx) = cx.add_window_view(|_, _| BPhiberApp::new());
    cx.simulate_resize(size(px(1200.0), px(700.0)));
    app.update(cx, |app, cx| {
        app.state.remove(ModalKey::Open);
        app.state.set(&phiber::v2::scene::shading_key(), "rendered");
        app.state.set(phiber::v2::state::ButsKey::Value("engine"), "CYCLES");
        app.handle_action(&region("view:0.5,1,0,0.5"), cx);
        cx.notify();
    });
    cx.run_until_parked();
    // The tracer starts a view small and doubles it until full size
    // (`coarse`): the view's full size is where the images stop growing.
    let full_size = |cx: &mut gpui::VisualTestContext, app: &gpui::Entity<BPhiberApp>| {
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(240);
        let (mut best, mut grew) = (None, std::time::Instant::now());
        loop {
            app.update_in(cx, |_, window, cx| window.simulate_next_frame(cx));
            cx.executor().advance_clock(std::time::Duration::from_millis(16));
            cx.run_until_parked();
            if let Some((size, true)) = shown_image()
                && best.is_none_or(|b: (u32, u32)| size.0 > b.0)
            {
                (best, grew) = (Some(size), std::time::Instant::now());
            }
            // Settled: no larger image for a while — long enough that a
            // full-size sample on a loaded machine still arrives in time.
            if let Some(size) = best
                && grew.elapsed() > std::time::Duration::from_secs(12)
            {
                return size;
            }
            assert!(std::time::Instant::now() < deadline, "the view never settled on a size; so far {best:?}");
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
    };
    let (rw, rh) = full_size(cx, &app);
    // Workbench: the whole viewport, whatever the region says — which is
    // also what says how large the viewport is.
    app.update(cx, |app, cx| {
        app.state.set(phiber::v2::state::ButsKey::Value("engine"), "BLENDER_WORKBENCH");
        cx.notify();
    });
    cx.run_until_parked();
    let (w, h) = full_size(cx, &app);
    assert!(w > rw && h > rh, "the region is smaller than the viewport: {rw}×{rh} of {w}×{h}");
    assert!((rw as i64 - (w / 2) as i64).abs() <= 1 && (rh as i64 - (h / 2) as i64).abs() <= 1, "the lower right quarter: {rw}×{rh} of {w}×{h}");
}
