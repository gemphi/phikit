//! Probe: the painted horizontal gutter of every row family, measured.
//! Not an assertion — it prints what the UI actually does, so the gutter
//! decision comes from measurements rather than from reading the code.

use gpui::{TestAppContext, px, size};
use phiber::v2::app::BPhiberApp;
use phiber::v2::state::{CtxKey, ModalKey};
use phiber::v2::state::{BState, val::V_MODAL_FILE_BROWSER};

#[gpui::test]
async fn probe_file_browser_gutters(cx: &mut TestAppContext) {
    let (app, cx) = cx.add_window_view(|_, _| BPhiberApp::new());
    cx.simulate_resize(size(px(1400.0), px(900.0)));
    app.update(cx, |app, cx| {
        app.state = BState::new();
        app.state.set(CtxKey::Space, "layout");
        app.state.set(ModalKey::Open, V_MODAL_FILE_BROWSER);
        cx.notify();
    });
    cx.run_until_parked();
    println!("--- file browser ---");
    for (k, b) in cx.all_debug_bounds() {
        if k.contains("table") || k.contains("sort") || k.contains("panel-body") || k.contains("sidebar") || k.contains("thumb") {
            println!("{k:<52} x={:8.2} w={:8.2}", f32::from(b.origin.x), f32::from(b.size.width));
        }
    }
}

#[gpui::test]
async fn probe_outliner_gutters(cx: &mut TestAppContext) {
    let (app, cx) = cx.add_window_view(|_, _| BPhiberApp::new());
    cx.simulate_resize(size(px(1400.0), px(900.0)));
    app.update(cx, |app, cx| {
        app.state = BState::new();
        app.state.set(CtxKey::Space, "layout");
        cx.notify();
    });
    cx.run_until_parked();
    println!("--- layout space ---");
    for (k, b) in cx.all_debug_bounds() {
        if k.contains("tree-row") || k.contains("panel-body") || k.contains("split") || k.contains("card") {
            println!("{k:<52} x={:8.2} w={:8.2}", f32::from(b.origin.x), f32::from(b.size.width));
        }
    }
}
