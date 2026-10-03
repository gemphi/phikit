//! The Properties navbar reads as a strip with the panel cut out of it.
//!
//! The vertical strips painted nothing, so the active tab had no edge to be cut
//! out of. Giving the region a fill of its own was right; the shade chosen was
//! not.
//!
//! This file used to say "Blender's navbar is its own shade (`TH_BACK` in the nav
//! region)" and darken the whole strip by a ladder step. That misread the source
//! it cited. `region_background_color_id` (`editors/screen/area.cc:3012`) returns
//! `TH_HEADER` for a header, `TH_PREVIEW_BACK` for a preview and `TH_BACK` for
//! **everything else** — `RGN_TYPE_NAV_BAR` included. `TH_BACK` in the nav region
//! is not a shade of its own; it is the same `TH_BACK` the panel is painted in.
//!
//! What Blender separates is the TABS: `wcol_tab` has `inner = 0x1d1d1d` and
//! `inner_sel = 0x303030` against a `back` of `0x303030`, so an unselected tab is
//! a recessed chip and the selected one is the background's own shade. Same two
//! shades as the old rule, opposite roles — the strip was wearing the chip's.
//!
//! Two halves, and they only work together: the strip takes the strip shade
//! every horizontal strip takes, and the active tab takes the surface of the
//! region beside it — flat, not a sheen, because a gradient on one side of
//! that join makes the two read as two things again.

use gpui::{TestAppContext, px, size};
use phiber::v2::app::BPhiberApp;
use phiber::v2::host::BHostType;
use phiber::v2::space::BSpaceType;
use phiber::v2::state::{BState, CtxKey, ModalKey};
use phiber::v2::theme::{BPaletteName, BResolvedUiStyle, ui_style_defaults};

fn style() -> BResolvedUiStyle {
    BResolvedUiStyle::from(ui_style_defaults(BPaletteName::DEFAULT))
}

/// The renderer's own source: `region_fill` is private, and which fill a
/// region kind takes is exactly what this is about.
const REGION: &str = include_str!("../src/v2/view/draw/region/mod.rs");

#[test]
fn a_vertical_strip_paints_a_shade_of_its_own() {
    assert!(
        REGION.contains("BRegionType::NavBar | BRegionType::Tools => BFill::HostRail"),
        "the Properties navbar and the tool rail carry the rail shade, not nothing"
    );
}

#[test]
fn a_content_region_still_paints_nothing() {
    // Only the strips changed: a content region is still the area's body,
    // or the panel's surface would be painted twice.
    assert!(REGION.contains("_ => BFill::Clear"), "content regions paint nothing");
}

#[test]
fn the_active_tab_is_exactly_the_surface_it_selects() {
    let style = style();
    let tab = style.tab_active_gradient(BHostType::Buts);
    let panel = style.host_body_surface(BHostType::Buts);
    assert_eq!(tab, panel, "the active tab is the panel beside it, so the two meet with no seam");
}

#[test]
fn that_join_is_flat_on_both_sides() {
    // A sheen on the tab and a flat panel is the same colour twice with a
    // gradient across the join, which reads as an edge.
    let style = style();
    let tab = style.tab_active_gradient(BHostType::Buts);
    assert_eq!(tab.top, tab.bottom, "the active Properties tab is flat");
}

#[test]
fn the_rail_shade_is_the_panel_shade_and_the_resting_tab_is_the_step_under() {
    // `TH_BACK` for the nav region as for the region beside it. The step that
    // used to darken the whole strip is the resting tab's chip now
    // (`wcol_tab.inner`), which is the only thing that needs to stand apart.
    let style = style();
    let rail = style.rail_surface(BHostType::Buts);
    let panel = style.host_body_surface(BHostType::Buts);
    assert_eq!(rail.bottom, panel.bottom, "the strip is the colour of the panel beside it");
    assert_eq!(rail.top, rail.bottom, "and flat, like the tab it frames");
    assert_eq!(
        style.rail_tab_resting(BHostType::Buts),
        style.step(panel.bottom, -1),
        "a resting tab is the recessed chip the strip used to be"
    );
}

#[test]
fn the_clusters_bar_keeps_off_the_strips_sides() {
    // The expand / paging bar is a control, not a cap: a tab spans the rail
    // and meets the pane (`layout_gutters`), but a button keeps its gap.
    let rail = include_str!("../src/v2/view/draw/rail.rs");
    let bar = rail
        .split("fn bar(")
        .nth(1)
        .expect("the actions bar");
    assert!(
        bar.contains(".margin_x(BSpace::PadXs)"),
        "the bar is inset from the strip's edges"
    );
}

#[gpui::test]
async fn the_cluster_clears_both_sides_while_a_tab_still_spans_them(cx: &mut TestAppContext) {
    let (app, cx) = cx.add_window_view(|_, _| BPhiberApp::new());
    cx.simulate_resize(size(px(1400.0), px(560.0)));
    app.update(cx, |app, cx| {
        app.state = BState::new();
        app.state.remove(ModalKey::Open);
        app.state.set(CtxKey::Space, BSpaceType::Layout.slug());
        cx.notify();
    });
    cx.run_until_parked();
    let strip = cx.debug_bounds("region-NavBar-Buts").expect("the Properties navbar");
    let tab = cx.debug_bounds("tab-render-render").expect("a Properties tab");
    let arrow = cx.debug_bounds("tab-chevron_right-chevron_right").expect("the expand arrow");
    let span = |b: &gpui::Bounds<gpui::Pixels>| (f32::from(b.origin.x), f32::from(b.origin.x + b.size.width));
    let (sl, sr) = span(&strip);

    // A tab is a cap cut into the strip: it spans it and meets the pane.
    assert_eq!(span(&tab), (sl, sr), "a tab spans the strip: {tab:?} in {strip:?}");

    // The cluster's bar is a control, so it keeps a button's gap — the same
    // on both sides, and inside the strip rather than added to its width.
    // `width(Full)` plus a margin is what ran it 4px past the right edge.
    let (al, ar) = span(&arrow);
    let inset = phiber::v2::theme::style::metric::DEFAULT_SPACING.pad_xs;
    assert_eq!(al - sl, inset, "the bar clears the strip's left edge");
    assert_eq!(sr - ar, inset, "and its right edge by the same");
    assert!(ar <= sr, "and never runs past it: {arrow:?} in {strip:?}");
}
