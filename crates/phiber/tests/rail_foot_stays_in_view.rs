//! A vertical strip's expand / paging cluster is pinned to the foot of the
//! strip: when the strip is shorter than its entries, the entries scroll
//! under it and the cluster stays in view. It used to be pushed off the
//! bottom of the Properties navbar and out of the window — dividing the
//! height pins the cluster only while the entries fit, and `justify:
//! between` cannot take space that is not there. So a rail with a cluster is
//! two bands: the entries take the slack and scroll, the cluster keeps its
//! size (`view/draw/rail.rs`, `BRailPart`).
//!
//! The cluster sits at the foot of the rail's *content*, which is the region
//! less the lead it pads — the same lead above the first entry as below the
//! last, so the strip reads as evenly inset at both ends.

use gpui::{TestAppContext, px, size};
use phiber::v2::app::BPhiberApp;
use phiber::v2::state::{CtxKey, ModalKey};

fn bottom(b: gpui::Bounds<gpui::Pixels>) -> f32 {
    f32::from(b.origin.y) + f32::from(b.size.height)
}

#[gpui::test]
async fn the_cluster_sits_a_cap_gap_off_the_foot_of_a_short_strip(cx: &mut TestAppContext) {
    let (app, cx) = cx.add_window_view(|_, _| BPhiberApp::new());
    // Short enough that the Properties navbar's fifteen tabs do not fit.
    cx.simulate_resize(size(px(1400.0), px(560.0)));
    app.update(cx, |app, cx| {
        app.state.remove(ModalKey::Open);
        app.state.set(CtxKey::Space, "layout");
        cx.notify();
    });
    cx.run_until_parked();
    let style = phiber::v2::theme::ui_style_defaults(phiber::v2::theme::BPaletteName::DEFAULT);
    // The lead the region pads above its first entry, and below its last —
    // `BResolvedUiStyle::rail_lead` for a card column: the host's content
    // pad (`pad_sm` in Properties), the body column's two borders, and its
    // inner pad, so a rail entry and the row beside it share one grid.
    let lead = style.spacing.pad_sm + style.strokes.border + style.spacing.pad_sm;
    let rail = cx.debug_bounds("region-NavBar-Buts").expect("the Properties navbar");
    let chevron = cx.debug_bounds("tab-chevron_right-chevron_right").expect("the expand arrow");
    assert!(bottom(chevron) <= 560.0, "the arrow is in the window: {chevron:?}");
    let in_rail = chevron.origin.x >= rail.origin.x && chevron.origin.x < rail.origin.x + rail.size.width;
    assert!(in_rail, "the arrow found is the navbar's: {chevron:?} vs {rail:?}");
    // The arrow's bar ends at the foot of the rail's content — the region's
    // bottom less the lead it pads, the same lead as above the first tab.
    assert!(bottom(chevron) <= bottom(rail) - lead + 0.51, "the arrow {chevron:?} stays inside the navbar {rail:?}");
    assert!(bottom(chevron) >= bottom(rail) - lead - 0.51, "and sits at its foot: {chevron:?} in {rail:?}");
    assert!(bottom(rail) <= 560.0, "the navbar itself fits the window: {rail:?}");
    // The tabs above it are what scrolls: the last one is cut off by the
    // entries band rather than pushing the cluster out of the strip.
    let first = cx.debug_bounds("tab-render-render").expect("the first Properties tab");
    assert!(first.origin.y >= rail.origin.y, "the first tab starts inside the rail: {first:?}");
    assert!(bottom(first) < bottom(chevron), "the tabs stack above the cluster");
}

#[gpui::test]
async fn the_viewport_strips_end_a_cap_gap_off_the_canvas_bottom(cx: &mut TestAppContext) {
    let (app, cx) = cx.add_window_view(|_, _| BPhiberApp::new());
    cx.simulate_resize(size(px(1400.0), px(560.0)));
    app.update(cx, |app, cx| {
        app.state.remove(ModalKey::Open);
        app.state.set(CtxKey::Space, "layout");
        cx.notify();
    });
    cx.run_until_parked();
    let gap = phiber::v2::theme::ui_style_defaults(phiber::v2::theme::BPaletteName::DEFAULT).dimensions.menu_top_gap;
    let canvas = cx.debug_bounds("view3d-canvas").expect("the viewport");
    for side in ["pane-floating-Left-View3D", "pane-floating-Right-View3D"] {
        let strip = cx.debug_bounds(side).expect("a floating strip");
        assert!((bottom(canvas) - bottom(strip) - gap).abs() < 0.51, "{side} ends a cap gap off the canvas's foot: {strip:?} in {canvas:?}");
        assert!(bottom(strip) <= 560.0);
    }
}
