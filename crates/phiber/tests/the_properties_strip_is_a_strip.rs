//! The Properties navbar reads as a strip with the panel cut out of it.
//!
//! The vertical strips painted nothing — "the area's body under it is its
//! surface" — so the active tab had no edge to be cut out of. The fix was to
//! give the region a fill of its own, which is right; the shade chosen for it
//! was not.
//!
//! This file used to say "Blender's navbar is its own shade (`TH_BACK` in the
//! nav region)" and darken the whole strip by a ladder step. That was a
//! misreading of the source it cited. `region_background_color_id`
//! (`editors/screen/area.cc:3012`) returns `TH_HEADER` for a header,
//! `TH_PREVIEW_BACK` for a preview and `TH_BACK` for **everything else** —
//! `RGN_TYPE_NAV_BAR` included. `TH_BACK` in the nav region is not a shade of
//! its own; it is the same `TH_BACK` the panel beside it is painted in.
//!
//! What Blender actually separates is the TABS. `userdef_default_theme.c`'s
//! `wcol_tab` has `inner = 0x1d1d1d` and `inner_sel = 0x303030` against a
//! `space_properties.back` of `0x303030`: an unselected tab is a recessed chip
//! and the selected one is the background's own shade. Same two shades as the
//! old rule, opposite roles — the strip was wearing the chip's colour.
//!
//! So: the strip is the column's own surface — no colour behind it that the
//! panel beside it lacks — a resting tab paints nothing, and the selected tab
//! is painted as the accordion heading level with it (`heading_surface(Card)`,
//! lit edge and all), so the tab and the first panel head read as one row of
//! the same material. The body used to sit one step under the strip with the
//! selected tab in the body's shade, a hole cut onto the panel; that put a
//! shade behind the strip, and the hole marked the tab by what it lacked.
//! The shades across every palette and both modes are
//! `the_strips_read_as_one_surface.rs`.

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
fn the_active_tab_is_the_accordion_heading_beside_it() {
    use phiber::v2::theme::BSurfaceTone;
    let style = style();
    let tab = style.tab_active_gradient(BHostType::Buts);
    let head = style.heading_surface(BSurfaceTone::Card);
    assert_eq!(tab, head, "the selected tab is painted as a panel's head — the same surface, the same lit edge");
    assert_ne!(tab.bottom, style.rail_surface(BHostType::Buts).bottom, "and not the strip, or nothing marks it");
}

#[test]
fn the_tab_keeps_the_headings_own_axis() {
    // The paint used to be turned towards the pane (`along(side.gradient_axis())`),
    // which ran its sweep the other way from the accordion's: two materials.
    let tab = include_str!("../src/v2/view/draw/tab/mod.rs");
    let paint = tab.split("fn properties_layers").nth(1).expect("the Properties paint layer");
    let paint = &paint[..paint.find("} else {").unwrap_or(paint.len())];
    assert!(paint.contains("tab_active_gradient(ctx.host))"), "the heading's gradient as it is");
    assert!(!paint.contains("gradient_axis"), "not re-aimed at the pane");
}

#[test]
fn the_strip_is_the_column_beside_it() {
    // The step has moved three times, and this is where it landed: nowhere.
    //
    // It began on the whole strip, which made the nav bar read as a separate
    // panel — wrong, because `region_background_color_id` returns `TH_BACK` for
    // a nav bar, the same as the region beside it. It then moved to the resting
    // TAB (`wcol_tab.inner`): an unselected tab drew a chip and the selected tab
    // drew nothing, so the selection was the one tab with no mark on it. Then
    // to the BODY, one step under the strip, with the selected tab in the
    // body's shade — which still painted a colour behind the strip that the
    // column beside it did not have.
    //
    // The strip is the column's own surface, flat; a resting tab is strip; and
    // the selected tab wears the accordions' heading, so it is marked by what
    // it is, not by what it lacks.
    let style = style();
    let rail = style.rail_surface(BHostType::Buts);
    let panel = style.host_body_surface(BHostType::Buts);
    assert_eq!(rail.top, rail.bottom, "the strip is flat");
    assert_eq!(rail, panel, "the strip is the column beside it — one surface, no colour behind the strip");
    assert_eq!(
        rail.bottom,
        style.host_background(BHostType::Buts),
        "and that is the host's own background — `TH_BACK`, not a shade of its own",
    );
}

#[test]
fn the_strip_steps_by_the_same_pitch_as_the_panels_beside_it() {
    // The icons are meant to line up with the headings they open, row for row.
    // Two things have to hold for that and only one of them did.
    //
    // `rail_lead` already lines the FIRST icon up with the first heading ("A
    // rail entry is the card-head module, so the first entry's top is the
    // first heading's top"), and a rail entry is drawn at `card_head_height`,
    // the same height as a panel heading. So they start level and the items
    // are the same size. But the strip stepped by `menu_top_gap` and the
    // region beside it by `gap_xs` — two pixels a row, invisible at the first
    // icon and most of a row out by the ninth.
    //
    // Asserted against the source because the gap is written where the strip
    // is built; the point is that ONE number is shared, not what it is.
    let rail = include_str!("../src/v2/view/draw/rail.rs");
    let strip = rail.split("let strip = element").nth(1).expect("the strip column");
    let strip = &strip[..strip.find("into_any_element").unwrap_or(strip.len())];
    // Comments out: the note explaining the fix names the number it replaced,
    // and a source-reading test that counts prose finds what it is looking for
    // in the sentence saying not to do it.
    let strip: String = strip
        .lines()
        .map(|line| line.split("//").next().unwrap_or(""))
        .collect::<Vec<_>>()
        .join("\n");
    let strip = strip.as_str();
    assert!(
        strip.contains("spacing.gap_xs"),
        "the strip steps by the content region's own gap, so the rows stay level"
    );
    assert!(
        !strip.contains("menu_top_gap"),
        "and not by `menu_top_gap`, which is two pixels shorter and drifts"
    );

    // The region beside it is the other half of the pair.
    let region = include_str!("../src/v2/view/draw/region/mod.rs");
    assert!(
        region.contains(".gap(BSpace::GapXs)"),
        "the panel column's gap is the one the strip is matching"
    );

    // And the two items are the same height, or matching the gap would not be
    // enough on its own.
    let style = style();
    assert_eq!(
        style.dimensions.card_head_height(),
        style.dimensions.row_height,
        "a rail entry and a panel heading are one module tall"
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
