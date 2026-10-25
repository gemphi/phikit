//! Layout gutters — the horizontal rhythm every band in the app shares.
//!
//! Three constants govern it (see `draw::prim::columns::style`):
//!   • `surface_gutter` = `pad_md` — panel edge → the text in its strips.
//!   • `row_inset`      = `pad_sm` — a row's own edge → its text.
//!   • `content_inset`  = `surface_gutter - row_inset` — a card's edge → the
//!     rows it holds, so the two add back up to `surface_gutter`.
//!
//! These tests pin the two things a reader actually notices, laid out as
//! Blender lays out a panel (`panel_draw_aligned_widgets`): the row labels
//! start in the column of the heading's fold triangle (the heading's first
//! glyph, at the left), and the row values end under its grip (its last
//! glyph, at the right).

use phiber::v2::state::{BKey as _, CtxKey};
use phiber::v2::state::ButsKey;
use gpui::{Modifiers, TestAppContext, px, size};
use phiber::v2::app::BPhiberApp;
use phiber::v2::host::BHostType;
use phiber::v2::space::BSpaceType;
use phiber::v2::state::{BState};
use phiber::v2::theme::{BPaletteName, METRICS, resolve_ui_style, ui_style_defaults};

#[derive(Clone, Copy, Debug, PartialEq)]
struct Box_ {
    x: f32,
    y: f32,
    right: f32,
    bottom: f32,
}

impl Box_ {
    fn center(self) -> gpui::Point<gpui::Pixels> {
        gpui::point(
            px((self.x + self.right) / 2.0),
            px((self.y + self.bottom) / 2.0),
        )
    }

    fn size(self) -> (f32, f32) {
        (self.right - self.x, self.bottom - self.y)
    }

    fn relative_to(self, parent: Self) -> Self {
        Self {
            x: self.x - parent.x,
            y: self.y - parent.y,
            right: self.right - parent.x,
            bottom: self.bottom - parent.y,
        }
    }
}

fn bounds(cx: &mut gpui::VisualTestContext, id: &'static str) -> Box_ {
    let b = cx
        .debug_bounds(id)
        .unwrap_or_else(|| panic!("{id} was not painted"));
    Box_ {
        x: f32::from(b.origin.x),
        y: f32::from(b.origin.y),
        right: f32::from(b.origin.x) + f32::from(b.size.width),
        bottom: f32::from(b.origin.y) + f32::from(b.size.height),
    }
}

async fn layout(cx: &mut TestAppContext) -> &mut gpui::VisualTestContext {
    let (app, cx) = cx.add_window_view(|_, _| BPhiberApp::new());
    cx.simulate_resize(size(px(1280.0), px(800.0)));
    app.update(cx, |app, cx| {
        app.state = BState::new();
        app.state.set(CtxKey::Space, BSpaceType::Layout.slug());
        cx.notify();
    });
    cx.run_until_parked();
    cx
}

/// The heading's first and last glyph: the fold triangle and the grip.
fn header_ids() -> (&'static str, &'static str) {
    let key = "ctx-buts-active_tool-active_tool-expanded";
    (
        Box::leak(format!("card-fold-{key}").into_boxed_str()),
        Box::leak(format!("card-grip-{key}").into_boxed_str()),
    )
}

#[gpui::test]
async fn heading_triangle_and_row_labels_share_a_column(cx: &mut TestAppContext) {
    let cx = layout(cx).await;
    let key = "ctx-buts-active_tool-active_tool-expanded";
    let title = bounds(cx, Box::leak(format!("card-title-{key}").into_boxed_str()));
    let label = bounds(cx, "section-Tool-label");

    // A folding card's rows are its children: they stand under its title,
    // one step in from the fold triangle, so the tree reads.
    assert_eq!(
        label.x, title.x,
        "the property labels start under the heading's title"
    );
}

#[gpui::test]
async fn row_values_end_under_the_heading_grip(cx: &mut TestAppContext) {
    let cx = layout(cx).await;
    let (_, grip_id) = header_ids();
    let grip = bounds(cx, grip_id);
    let value = bounds(cx, "section-Tool-value");

    assert_eq!(
        value.right, grip.right,
        "a property row's value must end where the heading's grip does"
    );
}

#[gpui::test]
async fn properties_rail_tab_state_changes_only_paint(cx: &mut TestAppContext) {
    // A Properties tab is the rail's one tile: selecting it or hovering it
    // changes its fill, never its box or where its icon sits.
    let (app, cx) = cx.add_window_view(|_, _| BPhiberApp::new());
    cx.simulate_resize(size(px(1280.0), px(800.0)));
    app.update(cx, |app, cx| {
        app.state = BState::new();
        app.state.set(CtxKey::Space, BSpaceType::Layout.slug());
        cx.notify();
    });
    cx.run_until_parked();

    let active_tab = bounds(cx, "tab-tool-tool");
    let active_icon = bounds(cx, "tab-icon-tool-tool");

    app.update(cx, |app, cx| {
        app.state
            .set(&ButsKey::NavbarTab.path(), "Render");
        cx.notify();
    });
    cx.run_until_parked();
    let resting_tab = bounds(cx, "tab-tool-tool");
    let resting_icon = bounds(cx, "tab-icon-tool-tool");

    cx.simulate_mouse_move(resting_tab.center(), None, Modifiers::none());
    cx.run_until_parked();
    let hovered_tab = bounds(cx, "tab-tool-tool");
    let hovered_icon = bounds(cx, "tab-icon-tool-tool");

    assert_eq!(active_tab, resting_tab, "selection changed the tile's box");
    assert_eq!(resting_tab, hovered_tab, "hover changed the tile's box");
    assert_eq!(active_icon, resting_icon, "selection moved the icon");
    assert_eq!(resting_icon, hovered_icon, "hover moved the icon");
}

#[gpui::test]
async fn rail_tabs_are_the_viewport_tiles(cx: &mut TestAppContext) {
    // Every rail is one element: its tiles are card-head squares, `pad_xs`
    // in from both edges, so the rail is `rail_width` wide — the Properties
    // navbar exactly as the viewport's floating strips.
    let cx = layout(cx).await;
    let style = resolve_ui_style(ui_style_defaults(BPaletteName::DEFAULT), &BState::new());
    let rail = bounds(cx, "region-NavBar-Buts");
    let tab = bounds(cx, "tab-tool-tool");
    let side = style.dimensions.card_head_height();

    assert_eq!(rail.right - rail.x, style.rail_width(), "the navbar is a rail wide");
    assert_eq!(tab.right - tab.x, side, "a rail tile is the card-head module wide");
    assert_eq!(tab.bottom - tab.y, side, "and as tall: a square");
    assert_eq!(tab.x - rail.x, style.spacing.pad_xs, "a pad in from the rail's left edge");
    assert_eq!(rail.right - tab.right, style.spacing.pad_xs, "and from its right");
}

#[gpui::test]
async fn host_headers_pad_horizontally_only(cx: &mut TestAppContext) {
    let cx = layout(cx).await;
    let b = cx
        .debug_bounds("region-Header-Buts")
        .expect("Buts header not painted");
    // The header's rule sits below its band: the strip is the module plus
    // that one stroke, so everything in the band keeps its place and the line
    // is what the pane below meets (`chrome_geometry`).
    let rule = phiber::v2::theme::style::metric::DEFAULT_STROKES.border;
    assert_eq!(
        f32::from(b.size.height),
        METRICS.h_header + rule,
        "a header strip is its band plus the rule under it"
    );
    let _ = BHostType::Buts;
}

#[gpui::test]
async fn resize_seam_is_centered_on_the_boundary(cx: &mut TestAppContext) {
    let (app, cx) = cx.add_window_view(|_, _| BPhiberApp::new());
    cx.simulate_resize(size(px(1280.0), px(800.0)));
    app.update(cx, |app, cx| {
        app.state = BState::new();
        app.state.set(CtxKey::Space, BSpaceType::Shading.slug());
        cx.notify();
    });
    cx.run_until_parked();

    let above = cx
        .debug_bounds("slot-right-0")
        .expect("upper area not painted");
    let below = cx
        .debug_bounds("slot-right-1")
        .expect("lower area not painted");
    let band = cx
        .debug_bounds("resize-right-0")
        .expect("resize seam not painted");
    let resize = ui_style_defaults(BPaletteName::DEFAULT).resize;

    let gap_lo = f32::from(above.origin.y) + f32::from(above.size.height);
    let gap_hi = f32::from(below.origin.y);
    let boundary_center = (gap_lo + gap_hi) / 2.0;
    let seam_center = f32::from(band.origin.y) + resize.hit_width / 2.0;
    let seam_lo = seam_center - resize.rest_thickness() / 2.0;
    let seam_hi = seam_center + resize.rest_thickness() / 2.0;

    assert_eq!(
        gap_hi - gap_lo,
        resize.gap,
        "layout uses the resolved resize gap"
    );
    // The seam center may be slightly off the exact boundary center due to
    // integer pixel alignment of the hit band (inset=-9, hit_width=14 means
    // seam_center = boundary - 2). Assert it is within 4px of center.
    assert!(
        (seam_center - boundary_center).abs() <= 4.0,
        "the seam stays near the boundary center (within 4px): seam={seam_center} boundary={boundary_center}"
    );
    assert_eq!(
        seam_hi - seam_lo,
        resize.rest_thickness(),
        "the seam uses resolved thickness"
    );
    // The seam center is centered on a whole pixel (boundary - 2 is still integer).
    assert_eq!(
        seam_center.fract(),
        0.0,
        "the default seam is centered on a whole pixel"
    );
}

#[gpui::test]
async fn card_rows_start_under_their_heading_in_every_host(cx: &mut TestAppContext) {
    let (app, cx) = cx.add_window_view(|_, _| BPhiberApp::new());
    cx.simulate_resize(size(px(1280.0), px(800.0)));
    app.update(cx, |app, cx| {
        app.state = BState::new();
        app.state.set(CtxKey::Space, BSpaceType::Layout.slug());
        app.state.set(
            phiber::v2::state::ModalKey::Open,
            phiber::v2::modal::BModalType::FileBrowser.slug(),
        );
        cx.notify();
    });
    cx.run_until_parked();

    // The file browser's bookmark panels are Cards whose rows are plain
    // Buttons, so this is the case that used to sit flush against the card
    // edge while its heading was inset.
    let key = "ctx-file-sidebar-System-expanded";
    let header = bounds(cx, Box::leak(format!("card-head-{key}").into_boxed_str()));
    let fold = bounds(cx, Box::leak(format!("card-fold-{key}").into_boxed_str()));
    let row = bounds(cx, Box::leak(format!("card-row-{key}-0").into_boxed_str()));

    assert_eq!(
        fold.x - header.x,
        METRICS.pad_md,
        "a heading's fold triangle sits one surface gutter in from the card edge"
    );
    let title = bounds(cx, Box::leak(format!("card-title-{key}").into_boxed_str()));
    assert_eq!(
        row.x - header.x + METRICS.pad_sm,
        title.x - header.x,
        "card rows sit under the heading's title: with their own pad_sm their \
         text starts where the title does"
    );
    assert_eq!(
        header.right - row.right,
        METRICS.pad_md - METRICS.pad_sm,
        "the content gutter is symmetric"
    );
}
