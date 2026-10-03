//! The sidebar column (`BColumn::Sidebar`) — a fixed-width column the user
//! can drag wider or narrower by its inner edge (`prim/resize.rs`).

use super::color::to_hsla;
use super::ctx::BRenderCtx;
use super::flex::BFill;
use super::walk;
use super::resize::{BPanelEdge, handle, k_sidebar_width};
use super::scroller::{scroll_id, scroller};
use crate::v2::theme::{BStyleRole, BSurfaceTone};
use crate::v2::view::BView;
use gpui::prelude::*;
use gpui::{AnyElement, div, px};

pub struct BSidebar<'a>(pub &'a [BView]);

impl BSidebar<'_> {
    /// A column that resizes by its trailing edge (the file browser's
    /// source list, a dialog's sidebar).
    pub fn render(&self, ctx: &BRenderCtx) -> AnyElement {
        self.render_at(BPanelEdge::trailing(), ctx)
    }

    /// The column with its grab strip on `edge` — the viewport's N panel
    /// takes it on its leading edge, beside the tab strip on its right.
    pub fn render_at(&self, edge: BPanelEdge, ctx: &BRenderCtx) -> AnyElement {
        // Dragged shut (`view::size`): only the edge is left, and clicking
        // it brings the sidebar back — Blender's hidden-region arrow.
        if !edge.leading && crate::v2::view::size::sidebar_hidden(ctx.state) {
            return restore(ctx);
        }
        // Outer surface owns paint and geometry; the bounded inner child scrolls.
        let outer_padding = ctx.style.spacing.pad_sm;
        // The dragged width, or the metric default until the user drags.
        let width = crate::v2::view::size::sidebar_column_width(ctx.state, ctx.style.sidebar_min_width(), ctx.style.dimensions.sidebar_min_width);
        // Inside a Tools region the region owns the column's width (the
        // file browser's source list, `region.rs`), so the panel fills it.
        let in_tools = ctx.region == Some(crate::v2::region::BRegionType::Tools);
        let outer = div()
            .relative()
            .id(gpui::SharedString::from("panel-sidebar-column"))
            .debug_selector(|| "panel-sidebar-column".to_string())
            // Over a canvas (the N panel) a click on the column is the
            // column's, never a pick in the viewport under it.
            .occlude()
            .flex()
            .flex_col()
            .h_full()
            .min_h_0()
            .flex_shrink_0();
        // In a Tools region the column is the region's content and its cards
        // are the surfaces, as Properties' cards sit in their region: no
        // frame of its own round them (that drew a box in a box), and the
        // region's pad is the only inset.
        let outer = if in_tools {
            outer.w_full()
        } else {
            outer
                .w(px(width))
                .bg(BFill::Tone(BSurfaceTone::Body).background(ctx))
                .border(px(ctx.style.strokes.border))
                .border_color(to_hsla(ctx.style.colors.border))
                .rounded(px(ctx.style.radius_for(BStyleRole::Panel)))
        };
        let outer_padding = if in_tools { 0.0 } else { outer_padding };
        let id = scroll_id("panel-sidebar", self.0);
        let mut inner = div()
            .id(id.clone())
            .flex()
            .flex_col()
            .flex_1()
            .w_full()
            .min_h_0()
            .min_w_0()
            .p(px(outer_padding))
            .gap(px(ctx.style.spacing.gap_xs))
            .overflow_y_scroll();
        for child in self.0 {
            inner = inner.child(walk(child, ctx));
        }
        // The grab strip floats on the column's inner edge rather than
        // sitting beside it, so the sidebar costs exactly its own width and
        // the gap to the pane is the boundary gap, not a handle. In a Tools
        // region the drag writes the same key, which the region reads.
        outer
            .child(scroller(inner, &id, ctx))
            .child(handle(k_sidebar_width(), width, edge, ctx))
            .into_any_element()
    }
}

/// A hidden sidebar's edge: a strip one row wide, the arrow at its top; a
/// click anywhere on it shows the sidebar again (Blender's region azone).
fn restore(ctx: &BRenderCtx) -> AnyElement {
    use crate::v2::view::prim::BAction;
    let show = BAction::Set(crate::v2::view::size::k_sidebar_hidden(), crate::v2::state::val::V_FALSE.to_string());
    let handler = ctx.handler.clone();
    use super::flex::{BAlign, BFlex, BSpace};
    BFlex::column()
        .align(BAlign::Center)
        .full()
        .keeps_size()
        .pad(BSpace::PadSm)
        .div(ctx)
        .id(gpui::SharedString::from("panel-sidebar-restore"))
        .debug_selector(|| "panel-sidebar-restore".to_string())
        .cursor(gpui::CursorStyle::PointingHand)
        .on_click(move |_, window, app| (std::rc::Rc::clone(&handler))(&show, window, app))
        .child(walk(&BView::icon("triangle_right"), ctx))
        .into_any_element()
}
