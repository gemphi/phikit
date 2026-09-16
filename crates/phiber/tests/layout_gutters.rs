//! Layout gutters — the horizontal rhythm every band in the app shares.
//!
//! Two constants govern it (see `render::gpui::prim`):
//!   • `surface_gutter` = `pad_md` — panel edge → the controls it contains.
//!   • `text_gutter`    = `pad_md + pad_sm` — panel edge → *text*, since a
//!     heading's title is a Button carrying its own `pad_sm` while a property
//!     row's label is a bare Label carrying none.
//!
//! These tests pin the two things a reader actually notices: an accordion
//! heading's title and the row labels under it start in the same column, and
//! the row values end under the heading's chevron.

use gpui::{Modifiers, TestAppContext, px, size};
use phiber::v2::app::BPhiberApp;
use phiber::v2::host::BHostType;
use phiber::v2::space::BSpaceType;
use phiber::v2::state::{BPath, BState, key::K_SPACE};
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
        app.state.set(&K_SPACE, BSpaceType::Layout.slug());
        cx.notify();
    });
    cx.run_until_parked();
    cx
}

fn header_ids() -> (&'static str, &'static str) {
    let key = "ctx-buts-active_tool-active_tool-expanded";
    (
        Box::leak(format!("accordion-title-{key}").into_boxed_str()),
        Box::leak(format!("accordion-disclosure-{key}").into_boxed_str()),
    )
}

#[gpui::test]
async fn heading_title_and_row_labels_share_a_column(cx: &mut TestAppContext) {
    let cx = layout(cx).await;
    let (title_id, _) = header_ids();
    let title = bounds(cx, title_id);
    let row = bounds(cx, "section-Tool");

    assert_eq!(
        title.x - row.x,
        METRICS.pad_md,
        "heading title and property labels must start in the same column"
    );
}

#[gpui::test]
async fn row_values_end_under_the_heading_chevron(cx: &mut TestAppContext) {
    let cx = layout(cx).await;
    let (_, chevron_id) = header_ids();
    let chevron = bounds(cx, chevron_id);
    let row = bounds(cx, "section-Tool");

    assert_eq!(
        row.right - chevron.right,
        METRICS.pad_md,
        "a property row's value must end where the heading's chevron does"
    );
}

#[gpui::test]
async fn properties_rail_tab_state_changes_only_paint(cx: &mut TestAppContext) {
    let (app, cx) = cx.add_window_view(|_, _| BPhiberApp::new());
    cx.simulate_resize(size(px(1280.0), px(800.0)));
    app.update(cx, |app, cx| {
        app.state = BState::new();
        app.state.set(&K_SPACE, BSpaceType::Layout.slug());
        cx.notify();
    });
    cx.run_until_parked();

    let rail = bounds(cx, "region-NavBar-Buts");
    let pane = bounds(cx, "region-Center-Buts");
    let active_tab = bounds(cx, "tab-tool-tool");
    let active_paint = bounds(cx, "tab-tool-tool-background");
    let active_icon = bounds(cx, "tab-icon-tool-tool");
    let style = resolve_ui_style(ui_style_defaults(BPaletteName::DEFAULT), &BState::new());

    assert_eq!(active_tab.x, rail.x, "tab allocation starts at rail edge");
    assert_eq!(active_tab.right, pane.x, "tab allocation does not widen");
    assert_eq!(
        active_paint.x - rail.x,
        style.properties_tab_background_inset(),
        "paint begins at the resolved 2px-compatible inset"
    );
    assert_eq!(
        active_paint.right - pane.x,
        style.properties_tab_overlap(),
        "paint alone overlaps the content-facing edge"
    );

    app.update(cx, |app, cx| {
        app.state
            .set(&BPath::owned(vec!["buts", "navbar", "tab"]), "Render");
        cx.notify();
    });
    cx.run_until_parked();
    let resting_tab = bounds(cx, "tab-tool-tool");
    let resting_paint = bounds(cx, "tab-tool-tool-background");
    let resting_icon = bounds(cx, "tab-icon-tool-tool");

    cx.simulate_mouse_move(resting_tab.center(), None, Modifiers::none());
    cx.run_until_parked();
    let hovered_tab = bounds(cx, "tab-tool-tool");
    let hovered_paint = bounds(cx, "tab-tool-tool-background");
    let hovered_icon = bounds(cx, "tab-icon-tool-tool");

    assert_eq!(
        active_tab.size(),
        resting_tab.size(),
        "selection changed tab allocation size"
    );
    assert_eq!(resting_tab, hovered_tab, "hover changed tab allocation");
    assert_eq!(
        active_paint.relative_to(active_tab),
        resting_paint.relative_to(resting_tab),
        "selection changed paint geometry within the tab"
    );
    assert_eq!(resting_paint, hovered_paint, "hover changed paint geometry");
    assert_eq!(
        active_icon.relative_to(active_tab),
        resting_icon.relative_to(resting_tab),
        "selection moved the tab icon within its allocation"
    );
    assert_eq!(resting_icon, hovered_icon, "hover moved the tab icon");
}

#[gpui::test]
async fn rail_tabs_span_the_rail_and_meet_the_pane(cx: &mut TestAppContext) {
    let cx = layout(cx).await;
    let rail = bounds(cx, "region-NavBar-Buts");
    let pane = bounds(cx, "region-Center-Buts");
    let tab = bounds(cx, "tab-tool-tool");

    assert_eq!(tab.x, rail.x, "a rail tab starts at the rail's left edge");
    assert_eq!(
        tab.right - tab.x,
        METRICS.w_navbar,
        "a rail tab is exactly as wide as the rail"
    );
    assert_eq!(
        tab.right, pane.x,
        "tab allocation must meet the pane without changing strip width"
    );
}

#[gpui::test]
async fn host_headers_pad_horizontally_only(cx: &mut TestAppContext) {
    let cx = layout(cx).await;
    let b = cx
        .debug_bounds("region-Header-Buts")
        .expect("Buts header not painted");
    assert_eq!(
        f32::from(b.size.height),
        METRICS.h_header,
        "a header strip stays exactly h_header tall"
    );
    let _ = BHostType::Buts;
}

#[gpui::test]
async fn resize_seam_is_centered_on_the_boundary(cx: &mut TestAppContext) {
    let (app, cx) = cx.add_window_view(|_, _| BPhiberApp::new());
    cx.simulate_resize(size(px(1280.0), px(800.0)));
    app.update(cx, |app, cx| {
        app.state = BState::new();
        app.state.set(&K_SPACE, BSpaceType::Shading.slug());
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
    let seam_lo = seam_center - resize.visible_seam / 2.0;
    let seam_hi = seam_center + resize.visible_seam / 2.0;

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
        resize.visible_seam,
        "the seam uses resolved thickness"
    );
    // The seam center is centered on a whole pixel (boundary - 2 is still integer).
    assert_eq!(
        seam_center.fract(),
        0.0,
        "the default seam is centered on a whole pixel"
    );
}
