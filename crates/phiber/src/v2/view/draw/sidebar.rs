//! The sidebar column (`BColumn::Sidebar`) — a fixed-width column the user
//! can drag wider or narrower by its inner edge (`prim/resize.rs`).

use super::ctx::BRenderCtx;
use super::flex::{BFill, BFlex, BLen, BRadius, BSpace};
use super::walk;
use super::resize::{BPanelEdge, handle, k_sidebar_width};
use super::scroller::{scroll_id, scroller};
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
        // A details column (a UI region: the file browser's, the Project
        // Browser's inspector) is its own column: it does not share the
        // source list's dragged width or its shut state, and has no grip.
        let info = ctx.region == Some(crate::v2::region::BRegionType::UI);
        if !edge.leading && !info && crate::v2::view::size::sidebar_hidden(ctx.state) {
            return restore(ctx);
        }
        // The outer column owns geometry; the bounded inner child scrolls.
        // The dragged width, or the metric default until the user drags.
        // In the timing editors the sidebar opens as wide as their channel
        // list on the other side, so the two columns either side of the keys
        // match.
        use crate::v2::host::BHostType as H;
        let default = match ctx.host {
            H::Action | H::Time | H::Nla | H::Ipo | H::Drivers => super::canvas::time::row::CHANNEL_WIDTH,
            _ => ctx.style.sidebar_min_width(),
        };
        let width = crate::v2::view::size::sidebar_column_width(ctx.state, default, ctx.style.dimensions.sidebar_min_width);
        // Inside a Tools region the region owns the column's width (the
        // file browser's source list, `region.rs`), so the panel fills it.
        let in_tools = ctx.region == Some(crate::v2::region::BRegionType::Tools);
        let column_id = if info { "panel-info-column" } else { "panel-sidebar-column" };
        let outer = div()
            .relative()
            .id(gpui::SharedString::from(column_id))
            .debug_selector(move || column_id.to_string())
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
        // Its sections have no surface of their own (`BCardKind::Section`),
        // so the column is one: the sidebar's tone, a step under the listing
        // beside it, as Properties' and the N panel's columns are.
        // Beside a rail (the N panel) the column paints nothing either: its
        // cards are the surfaces, standing at the top of the column. A
        // full-height box of its own round them was a panel left dangling —
        // an empty well under the last card all the way down the column.
        // In a Tools region (the file browser's source list, the Project
        // Browser's navigator) the column is a rail: the same rounded ground,
        // in the floating chrome's shade and padded the same, that the
        // viewport's toolbar and every docked rail stand on (`rail.rs`).
        // A details column is its region's width less the region's pad, as
        // the source list is its own region's: at a fixed width it ran a pad
        // past the region and its cards met the window's edge.
        let outer = match (in_tools, info) {
            (true, _) => BFlex::column()
                .width(BLen::Full)
                .pad(BSpace::PadXs)
                .radius(BRadius::Role(crate::v2::theme::BStyleRole::Panel))
                .fill(BFill::Floating)
                .styled(outer, ctx),
            (false, true) => BFlex::column().width(BLen::Full).styled(outer, ctx),
            (false, false) => outer.w(px(width)),
        };
        let id = scroll_id("panel-sidebar", self.0);
        let mut inner = div()
            .id(id.clone())
            .flex()
            .flex_col()
            .flex_1()
            .w_full()
            .min_h_0()
            .min_w_0()
            .gap(px(ctx.style.spacing.gap_xs))
            .overflow_y_scroll();
        for child in self.0 {
            inner = inner.child(walk(child, ctx));
        }
        // The grab strip floats on the column's inner edge rather than
        // sitting beside it, so the sidebar costs exactly its own width and
        // the gap to the pane is the boundary gap, not a handle. In a Tools
        // region the drag writes the same key, which the region reads.
        let outer = outer.child(scroller(inner, &id, ctx));
        match info {
            true => outer.into_any_element(),
            false => outer.child(handle(k_sidebar_width(), width, edge, ctx)).into_any_element(),
        }
    }
}

/// A hidden sidebar's edge: a strip one row wide, the arrow at its top; a
/// click anywhere on it shows the sidebar again (Blender's region azone).
fn restore(ctx: &BRenderCtx) -> AnyElement {
    use crate::v2::view::prim::BAction;
    let show = BAction::Set(crate::v2::view::size::k_sidebar_hidden(), crate::v2::state::val::V_FALSE.to_string());
    let handler = ctx.handler.clone();
    use super::flex::BAlign;
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
