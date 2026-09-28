//! The window chrome's shared rules, measured on the painted UI.
//!
//! Every menu cap and labelled tab hangs from the bottom of its strip with one
//! `menu_top_gap` above it, so whatever drops below (a popover, a pane) reads
//! as the same surface. The icon rails are exactly as wide as a strip is tall,
//! which is what makes the window read as one grid.

use phiber::v2::state::{BKey as _, CtxKey, ModalKey};
use gpui::{Pixels as px_t, TestAppContext, px, size};
use phiber::v2::app::BPhiberApp;
use phiber::v2::space::BSpaceType;
use phiber::v2::state::{BState};
use phiber::v2::theme::METRICS;

/// The workspace with `space` open, laid out at a fixed window size.
async fn workspace(cx: &mut TestAppContext, space: BSpaceType) -> &mut gpui::VisualTestContext {
    let (app, cx) = cx.add_window_view(|_, _| BPhiberApp::new());
    cx.simulate_resize(size(px(1280.0), px(800.0)));
    app.update(cx, |app, cx| {
        app.state = BState::new();
        app.state.set(CtxKey::Space, space.slug());
        cx.notify();
    });
    cx.run_until_parked();
    cx
}

/// Painted bounds of the first selector that was drawn.
fn bounds(cx: &mut gpui::VisualTestContext, ids: &[&'static str], what: &str) -> gpui::Bounds<px_t> {
    for id in ids {
        if let Some(b) = cx.debug_bounds(id) {
            return b;
        }
    }
    panic!("{what} was not painted (tried {ids:?})")
}

#[gpui::test]
async fn the_rails_are_as_wide_as_a_strip_is_tall(cx: &mut TestAppContext) {
    let cx = workspace(cx, BSpaceType::Layout).await;
    // The Properties editor keeps its rails in-flow (the viewport floats its
    // toolbar over the canvas), so measure them there.
    let strip = bounds(cx, &["region-Header-Buts"], "the Properties header");
    let rail = bounds(cx, &["region-NavBar-Buts"], "the Properties rail");
    assert_eq!(
        f32::from(rail.size.width),
        f32::from(strip.size.height),
        "an icon rail is one strip-height wide, so the chrome reads as one grid"
    );
    for (name, w) in [
        ("toolbar", METRICS.w_toolbar),
        ("sidebar", METRICS.w_sidebar),
        ("navbar", METRICS.w_navbar),
    ] {
        assert_eq!(w, METRICS.h_header, "the {name} rail is one strip tall");
    }
    // The activity bar's entries are menu-cap-sized buttons, so its rail is a
    // strip plus the inset the button keeps from each edge.
    assert!(
        METRICS.w_activity > METRICS.h_header,
        "the activity rail is a strip plus an inset either side of its buttons"
    );
}

#[gpui::test]
async fn a_tab_hangs_from_its_strip_with_one_gap_above(cx: &mut TestAppContext) {
    let cx = workspace(cx, BSpaceType::Layout).await;
    // The workspace tabs live in the topbar's strips.
    let strip = bounds(cx, &["region-Header-Topbar", "region-ToolHeader-Topbar"], "the topbar strip");
    let tab = bounds(cx, &["tab-layout-grid-Layout"], "the Layout tab");
    // The cap keeps one gap above it …
    assert_eq!(
        f32::from(tab.origin.y - strip.origin.y),
        METRICS.menu_top_gap,
        "a tab keeps the menu gap above it"
    );
    // … and runs to the bottom of its strip.
    assert_eq!(
        f32::from(tab.size.height),
        METRICS.h_header - METRICS.menu_top_gap,
        "a tab fills its strip apart from that gap"
    );
}

#[gpui::test]
async fn an_activity_cap_is_a_free_button_centred_in_its_rail(cx: &mut TestAppContext) {
    let cx = workspace(cx, BSpaceType::Layout).await;
    // The activity bar reads like the toolbar: a column of controls standing
    // on their own, not a strip of caps hanging into a pane. So its entries
    // are free chrome — fully rounded, resting on a fill, and centred in the
    // rail with the same inset on both sides. (It used to run flush into the
    // pane beside it like a docked rail cap; that is what made it look unlike
    // the toolbar and sidebar it stands next to.)
    // The region is the rail: its Left pane hugs the caps, so it measures
    // nothing about them.
    let rail = bounds(cx, &["region-Center-Activity"], "the Activity rail");
    let cap = bounds(cx, &["tab-home-home"], "the Home cap");

    let lead = f32::from(cap.origin.x - rail.origin.x);
    let trail = f32::from(rail.origin.x + rail.size.width) - f32::from(cap.origin.x + cap.size.width);
    assert!(
        (lead - trail).abs() <= 0.5,
        "a free cap is centred in its rail: {lead} on the left, {trail} on the right"
    );
    assert!(
        lead > 0.0,
        "a free cap keeps clear of the rail's edge: rail {:?} wide at x={:?}, cap {:?} wide at x={:?}",
        rail.size.width, rail.origin.x, cap.size.width, cap.origin.x
    );
    // Slack's rail: the entry is a stack — the icon with its name under it —
    // so it is taller than a button (the module) and nearly the rail's width.
    assert!(
        f32::from(cap.size.height) > METRICS.h_header - METRICS.menu_top_gap,
        "a stacked entry is taller than a button: {:?}",
        cap.size
    );
    assert!(f32::from(cap.size.width) > f32::from(rail.size.width) * 0.6, "and nearly the rail's width");
}

#[gpui::test]
async fn the_first_rail_cap_sits_on_the_first_heading_beside_it(cx: &mut TestAppContext) {
    let cx = workspace(cx, BSpaceType::Layout).await;
    // The Properties rail runs alongside the accordion stack, so its first
    // cap and the first heading's title must share a centre line — otherwise
    // the strip reads as starting half a row above the content.
    let cap = bounds(cx, &["tab-tool-tool"], "the first Properties rail cap");
    let key = "ctx-buts-active_tool-active_tool-expanded";
    let title = bounds(
        cx,
        &[Box::leak(format!("card-title-{key}").into_boxed_str())],
        "the first panel title",
    );

    let cap_mid = f32::from(cap.origin.y) + f32::from(cap.size.height) / 2.0;
    let title_mid = f32::from(title.origin.y) + f32::from(title.size.height) / 2.0;
    assert!(
        (cap_mid - title_mid).abs() <= 0.5,
        "the first rail cap must sit on the first heading's centre line: \
         cap={cap_mid} heading={title_mid}"
    );
}

#[gpui::test]
async fn a_menu_popover_frames_its_rows_with_one_gutter(cx: &mut TestAppContext) {
    use phiber::v2::host::BHostType;
    use phiber::v2::state::{MenuKey, V_TRUE};

    let (app, cx) = cx.add_window_view(|_, _| BPhiberApp::new());
    cx.simulate_resize(size(px(1280.0), px(800.0)));
    app.update(cx, |app, cx| {
        app.state = BState::new();
        app.state.set(CtxKey::Space, BSpaceType::Layout.slug());
        app.state.set(&MenuKey::Top(BHostType::Topbar.slug(), "File").path(), V_TRUE);
        cx.notify();
    });
    cx.run_until_parked();

    let panel = bounds(cx, &["popover-panel"], "the open File menu");
    // "New" opens Blender's File menu and "Quit" closes it, as they do ours.
    let first = bounds(cx, &["menu-New"], "the menu's first row");
    let last = bounds(cx, &["menu-Quit"], "the menu's last row");

    // A row's hover fill stops short of the panel edge on every side, the way
    // a toolbar or sidebar button does — it never runs into the border radius.
    let top = f32::from(first.origin.y - panel.origin.y);
    let bottom =
        f32::from((panel.origin.y + panel.size.height) - (last.origin.y + last.size.height));
    let left = f32::from(first.origin.x - panel.origin.x);
    assert!(top > METRICS.stroke, "rows must be inset from the panel's top edge");
    assert_eq!(top, bottom, "the gutter above and below the rows must match");
    assert_eq!(top, left, "the gutter around the rows must be the same on every side");
}

#[gpui::test]
async fn the_sidebar_takes_the_width_it_was_dragged_to(cx: &mut TestAppContext) {
    use phiber::v2::view::draw::resize::k_sidebar_width;

    let (app, cx) = cx.add_window_view(|_, _| BPhiberApp::new());
    cx.simulate_resize(size(px(1280.0), px(800.0)));
    app.update(cx, |app, cx| {
        app.state = BState::new();
        app.state.set(CtxKey::Space, BSpaceType::Layout.slug());
        // The bookmarks sidebar lives in the file browser.
        app.state.set(
            phiber::v2::state::ModalKey::Open,
            phiber::v2::modal::BModalType::FileBrowser.slug(),
        );
        cx.notify();
    });
    cx.run_until_parked();
    let before = bounds(cx, &["panel-sidebar-column"], "the file browser sidebar");

    // What a drag writes.
    app.update(cx, |app, cx| {
        app.state.set(&k_sidebar_width(), (f32::from(before.size.width) + 60.0).to_string());
        cx.notify();
    });
    cx.run_until_parked();
    let after = bounds(cx, &["panel-sidebar-column"], "the widened sidebar");
    assert_eq!(
        f32::from(after.size.width) - f32::from(before.size.width),
        60.0,
        "the sidebar is exactly as wide as the drag left it"
    );
}

#[gpui::test]
async fn every_top_bar_control_sits_on_one_centre_line(cx: &mut TestAppContext) {
    let cx = workspace(cx, BSpaceType::Layout).await;
    let strip = bounds(cx, &["region-Header-Topbar", "region-ToolHeader-Topbar"], "the topbar strip");
    let mark = bounds(cx, &["brand-mark"], "the brand mark");
    let cap = bounds(cx, &["btn-label-toggle-ctx.menu.topbar.File-text"], "the File menu cap");
    let toggle = bounds(cx, &["topbar-zone-toggle-right"], "a zone toggle");
    let mid = |b: gpui::Bounds<px_t>| f32::from(b.origin.y) + f32::from(b.size.height) / 2.0;
    // A shade over the icon size — the mark, not a banner (batch 41: 16, down from 22.4).
    let brand = phiber::v2::theme::style::metric::DEFAULT_DIMENSIONS.brand_size;
    assert!((f32::from(mark.size.height) - brand).abs() < 0.6, "mark is {:?} tall", mark.size.height);
    let icon = phiber::v2::theme::style::metric::DEFAULT_DIMENSIONS.icon_size;
    assert!(brand <= icon + 2.0 && brand >= icon, "the mark is an icon's size, give or take: {brand}");
    // One line for everything in a strip: the centre of the cap band — the
    // strip less the menu gap above it. The mark, a menu cap and a button
    // all sit on it; nothing rides a pixel above or below its neighbours.
    let line = f32::from(strip.origin.y) + METRICS.menu_top_gap + (METRICS.h_header - METRICS.menu_top_gap) / 2.0;
    for (what, b) in [("mark", mark), ("cap", cap), ("toggle", toggle)] {
        assert!((mid(b) - line).abs() < 0.5, "{what} centre {} vs the strip's line {line}", mid(b));
    }
}

#[gpui::test]
async fn the_welcome_logo_fits_inside_its_banner(cx: &mut TestAppContext) {
        let (app, cx) = cx.add_window_view(|_, _| BPhiberApp::new());
    cx.simulate_resize(size(px(1280.0), px(800.0)));
    app.update(cx, |app, cx| {
        app.state = BState::new();
        app.state.set(CtxKey::Space, BSpaceType::Layout.slug());
        app.state.set(ModalKey::Open, "splash");
        cx.notify();
    });
    cx.run_until_parked();
    let banner = bounds(cx, &["panel-header"], "the Welcome banner");
    let mark = bounds(cx, &["brand-mark-banner"], "the Welcome mark");
    let top = |b: gpui::Bounds<px_t>| f32::from(b.origin.y);
    let bottom = |b: gpui::Bounds<px_t>| f32::from(b.origin.y) + f32::from(b.size.height);
    assert!(
        top(mark) >= top(banner) - 0.5 && bottom(mark) <= bottom(banner) + 0.5,
        "the Welcome mark ({}..{}) overflows its banner ({}..{})",
        top(mark),
        bottom(mark),
        top(banner),
        bottom(banner)
    );
    // The banner is an ordinary header strip and the mark is the header's
    // mark: the greeting is not taller than any other heading.
    assert_eq!(f32::from(banner.size.height), METRICS.h_header, "the banner is a header's height");
    let brand = phiber::v2::theme::style::metric::DEFAULT_DIMENSIONS.brand_size;
    assert!(
        (f32::from(mark.size.height) - brand).abs() <= 0.5,
        "the Welcome mark ({}) is the header mark's size ({brand})",
        f32::from(mark.size.height)
    );
}
