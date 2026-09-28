//! Outliner tree geometry — the column grid and the header/menu border line.
//!
//! The outliner lays every row on a grid of `METRICS.indent_unit` columns:
//! one column per level of indent, one for the disclosure arrow, one for the
//! data-type icon. That is what makes a child row's arrow sit directly under
//! its parent's icon, and it is easy to break — an earlier selection
//! indicator drawn as a `border_l` displaced the selected row's content by
//! 2px and knocked exactly that row off the grid.

use phiber::v2::state::{BKey as _, CtxKey};
use gpui::{TestAppContext, px, size};
use phiber::v2::app::BPhiberApp;
use phiber::v2::host::BHostType;
use phiber::v2::space::BSpaceType;
use phiber::v2::state::{BState, V_TRUE, MenuKey};
use phiber::v2::theme::METRICS;

fn arrow_x(cx: &mut gpui::VisualTestContext, id: &'static str) -> f32 {
    let b = cx
        .debug_bounds(id)
        .unwrap_or_else(|| panic!("{id} was not painted"));
    f32::from(b.origin.x)
}

#[gpui::test]
async fn each_depth_steps_by_exactly_one_grid_column(cx: &mut TestAppContext) {
    let (app, cx) = cx.add_window_view(|_, _| BPhiberApp::new());
    cx.simulate_resize(size(px(1280.0), px(800.0)));
    app.update(cx, |app, cx| {
        app.state = BState::new();
        app.state.set(CtxKey::Space, BSpaceType::Layout.slug());
        cx.notify();
    });
    cx.run_until_parked();

    let unit = METRICS.indent_unit;
    let scene = arrow_x(cx, "tree-arrow-scene_collection");
    let collection = arrow_x(cx, "tree-arrow-collection");
    // "Cube" is the active object, so this also covers the SELECTED row —
    // the row whose accent bar must not displace its contents.
    let object = arrow_x(cx, "tree-arrow-Cube");

    assert_eq!(
        collection - scene,
        unit,
        "a child's arrow must sit one grid column right of its parent's"
    );
    assert_eq!(
        object - collection,
        unit,
        "the selected row must stay on the grid — its accent bar is an \
         overlay, not a border that shifts content"
    );
}

#[gpui::test]
async fn every_arrow_column_is_one_unit_wide(cx: &mut TestAppContext) {
    let (app, cx) = cx.add_window_view(|_, _| BPhiberApp::new());
    cx.simulate_resize(size(px(1280.0), px(800.0)));
    app.update(cx, |app, cx| {
        app.state = BState::new();
        app.state.set(CtxKey::Space, BSpaceType::Layout.slug());
        cx.notify();
    });
    cx.run_until_parked();

    for id in [
        "tree-arrow-scene_collection",
        "tree-arrow-collection",
        "tree-arrow-Cube",
    ] {
        let b = cx
            .debug_bounds(id)
            .unwrap_or_else(|| panic!("{id} was not painted"));
        assert_eq!(
            f32::from(b.size.width),
            METRICS.indent_unit,
            "{id}: the disclosure column is always one grid unit wide, so \
             names at a given depth all start at the same x"
        );
    }
}

/// A menubar anchor spans the header's content box, so its accent underline
/// lands on the header's bottom border and the popover it deploys starts on
/// that same line.
#[gpui::test]
async fn menubar_anchor_fills_the_header_down_to_its_border(cx: &mut TestAppContext) {
    let (app, cx) = cx.add_window_view(|_, _| BPhiberApp::new());
    cx.simulate_resize(size(px(1280.0), px(800.0)));
    app.update(cx, |app, cx| {
        app.state = BState::new();
        app.state.set(CtxKey::Space, BSpaceType::Layout.slug());
        app.state
            .set(&MenuKey::Top(BHostType::Topbar.slug(), "File").path(), V_TRUE);
        cx.notify();
    });
    cx.run_until_parked();

    let id: &'static str = Box::leak(
        phiber::v2::view::draw::button::element_id(
            "",
            Some("File"),
            &phiber::v2::view::prim::BAction::Toggle(MenuKey::Top(BHostType::Topbar.slug(), "File").path()),
        )
        .into_boxed_str(),
    );
    let b = cx
        .debug_bounds(id)
        .expect("open File anchor was not painted");
    let strip = cx
        .debug_bounds("region-Header-Topbar")
        .expect("the menubar strip was not painted");
    // A menu anchor is a button: the button height, centred on the strip's
    // line — the centre of the cap band, the strip less the menu gap above
    // it — like the editor switcher and the icon toggles beside it, as the
    // approved mock-ups draw them (Blender's pulldowns are `UI_UNIT_Y`
    // buttons too). It no longer hangs from the strip's bottom edge.
    assert_eq!(f32::from(b.size.height), METRICS.h_button, "a menu anchor is the button height");
    let line = f32::from(strip.origin.y) + METRICS.menu_top_gap + (METRICS.h_header - METRICS.menu_top_gap) / 2.0;
    let mid = f32::from(b.origin.y) + f32::from(b.size.height) / 2.0;
    assert!((mid - line).abs() < 0.51, "the anchor sits on the strip's line: {mid} vs {line}");
}
