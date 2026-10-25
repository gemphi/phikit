//! Rail atom — THE vertical strip renderer.
//!
//! One element for the Activity bar, the Toolbar, the Properties NavBar and
//! the Sidebar: a single column that fills the region slot it is placed in
//! (the region owns width and strip background) and lays its entries out by
//! group — Top entries stack from the top, Bottom entries pin to the bottom,
//! Center entries float between — with one flexible spacer between each
//! pair of consecutive non-empty groups.
//!
//! Entries render through the tab atom, which already owns the paper-tab
//! geometry, the per-host rail active/hover fills and the hover tooltip, so
//! every debug id (`tab-…`, `tab-…-background`, `tab-icon-…`) and every
//! pixel of the existing rail geometry is preserved.
//!
//! The strip kinds' fixed bottom cluster (expand chevron collapsed;
//! prev/collapse/next expanded) is just more entries in the Bottom group —
//! the part list builds them from state, so this renderer stays declarative:
//! it owns layout only, never behaviour. Those entries are the same square
//! tiles as the rest: nothing behind them lights up, because only a thing
//! you can click may change under the pointer.
//!
//! One width, one tile, everywhere: a rail is its tiles plus `pad_xs` either
//! side (`BResolvedUiStyle::rail_width`), docked in a region or floating
//! over a canvas — the floating pane pads it, a docked rail pads itself.

use super::ctx::BRenderCtx;
use super::flex::{BAlign, BDim, BFill, BFlex, BFlexed, BLen, BRadius, BSpace};
use super::{strip, tab};
use crate::v2::view::prim::{BRailEntry, BRailGroup, BRailSide, BRailView};
use gpui::prelude::*;
use gpui::{AnyElement, px};

pub fn view(rail: &BRailView, ctx: &BRenderCtx) -> AnyElement {
    BRail(rail).render(ctx)
}

/// The parts of a rail that has a pinned cluster: the column the two bands
/// stack in, the scrolling entries band, and the cluster at the foot.
enum BRailPart {
    /// The rail itself, when it is two bands rather than one strip.
    Column,
    /// The entries: they take the slack and scroll, so a rail with more
    /// entries than room hides entries, never the control that reveals them.
    Entries,
    /// The cluster at the foot, which keeps its size whatever is above it.
    Foot,
    /// The strip and the panels it opened, side by side.
    WithPanels,
}

impl BFlexed for BRailPart {
    fn flex(&self, _c: &BRenderCtx) -> BFlex {
        match self {
            Self::Column => BFlex::column().width(BLen::Full).height(BLen::Full).min_height(BLen::Zero),
            Self::Entries => BFlex::column().fills().width(BLen::Full).min_height(BLen::Zero),
            // Stretch, not `width(Full)`, so the bar's own inset comes out
            // of the strip's width instead of being added to it — a full
            // width plus a margin overflows, which ran the expand arrow 4px
            // past the edge it was being held off.
            Self::Foot => BFlex::column().width(BLen::Full).align(BAlign::Stretch).keeps_size(),
            // A column of panels always stands a gap off the strip beside it,
            // the same gap every column keeps from its neighbour.
            Self::WithPanels => BFlex::row().height(BLen::Full).min_height(BLen::Zero).keeps_size().gap(BSpace::GapXs),
        }
    }
}

/// One rail, drawn.
///
/// The group, the actions bar and the entry cell are methods of the rail
/// rather than free functions beside it: each answers a question only a rail
/// asks, so each is reached through the rail and from nowhere else.
pub struct BRail<'a>(pub &'a BRailView);

impl BRail<'_> {
    pub fn render(&self, ctx: &BRenderCtx) -> AnyElement {
        let rail = self.0;
        // A rail is a strip along the vertical axis — the same element as a
        // header row, with Top / Center / Bottom in place of start / centre /
        // end (`strip`). Its groups divide the height, which is what pins the
        // Bottom cluster to the foot of the strip.
        //
        // The rail is a hover group so its expand / paging cluster can appear
        // only while the pointer is over the strip.
        let group_name = gpui::SharedString::from(format!("rail-{:?}-{:?}", rail.kind, ctx.host));
        let mut groups: Vec<Vec<AnyElement>> =
            [BRailGroup::Top, BRailGroup::Center, BRailGroup::Bottom]
                .into_iter()
                .map(|group| self.group(group, ctx))
                .collect();

        // Dividing the height only pins the cluster while the entries fit.
        // The Properties navbar's fifteen tabs do not fit a short window, and
        // `justify: between` cannot take space that is not there — so the
        // cluster was pushed past the foot of the strip and out of the
        // window, which is the one control you need when entries are hidden.
        //
        // So a rail with a cluster is two bands: the entries, which take the
        // slack and scroll, and the cluster, which keeps its size at the
        // foot. `tests/rail_foot_stays_in_view.rs` is that rule.
        let pinned = rail.kind.has_strip_cluster() && !groups[2].is_empty();
        let foot = if pinned { std::mem::take(&mut groups[2]) } else { Vec::new() };
        let band = strip::place(strip::BStripAxis::Column, strip::BStripFit::Divide, groups, ctx)
            .w_full()
            .items_stretch();

        let element = if pinned {
            let id = gpui::SharedString::from(format!("rail-entries-{:?}-{:?}", rail.kind, ctx.host));
            let entries = BRailPart::Entries.flex(ctx).scroller(id, ctx).child(band);
            let cluster = foot
                .into_iter()
                .fold(BRailPart::Foot.flex(ctx).div(ctx), |cell, child| cell.child(child));
            BRailPart::Column.flex(ctx).div(ctx).child(entries).child(cluster)
        } else {
            band
        };
        let element = element.group(group_name).h_full().min_h_0();
        // Docked or floating, the rail is one bar: its pad, its rounded
        // ground in the floating chrome's shade — so the two are the same
        // width with the same tiles on the same bar, wherever it stands (the
        // Timeline's strip, the activity bar, the Properties tabs, the
        // viewport's toolbar).
        let bar = BFlex::column()
            .pad(BSpace::PadXs)
            .radius(BRadius::Role(crate::v2::theme::BStyleRole::Panel))
            .fill(BFill::Floating);
        let bar = if rail.expanded { bar.width(BLen::Full) } else { bar.width(BLen::Dim(BDim::RailWidth)).keeps_size() };
        let element = bar.styled(element, ctx);
        let strip = element
            // The leading inset belongs to the region, which knows what the
            // content beside the rail is inset by (`style.rail_lead`).
            //
            // The gap is the CONTENT region's gap, and has to be: `rail_lead`
            // already lines the first icon up with the first heading beside it
            // ("A rail entry is the card-head module, so the first entry's top
            // is the first heading's top"), and a rail entry is the same
            // `card_head_height` as a panel heading. So the two columns start
            // level and step by the same height — and then drifted, because
            // the strip stepped by `menu_top_gap` (2) and the region beside it
            // by `gap_xs` (4). Two pixels a row is invisible at the first and
            // most of a row out by the ninth, which is what made the icons
            // look unrelated to the panels they open.
            .gap(px(ctx.style.spacing.gap_xs))
            .into_any_element();

        // The panels beside the strip, while the sidebar is open. The part
        // hands them over (`part::sidebar`, `BRailView::panels`) and this
        // renderer dropped them on the floor — N set the state, the strip
        // redrew with the tab active, and the column it should have opened
        // was built and thrown away. `draw::sidebar::render_at` and
        // `BPanelEdge::beside_strip` were written for this and reached from
        // nowhere (`tests/viewport_sidebar.rs`).
        let Some(panels) = rail.panels.as_deref() else {
            return strip;
        };
        // A rail pinned to the editor's right edge puts its panels to its
        // left, and they take their grab edge on that inner edge; a
        // left-hand rail mirrors it.
        let inside = rail.side == BRailSide::Right;
        let column = super::sidebar::BSidebar(panels).render_at(
            if inside { super::resize::BPanelEdge::beside_strip(ctx.host) } else { super::resize::BPanelEdge::trailing() },
            ctx,
        );
        let row = BRailPart::WithPanels.flex(ctx).div(ctx);
        if inside { row.child(column).child(strip) } else { row.child(strip).child(column) }.into_any_element()
    }

    /// One group's cells. Strip kinds put their expand / paging cluster in
    /// the Bottom group; the Activity bar uses all three for its own entries.
    fn group(&self, group: BRailGroup, ctx: &BRenderCtx) -> Vec<AnyElement> {
        let rail = self.0;
        let entries: Vec<&BRailEntry> = rail.entries.iter().filter(|e| e.group == group).collect();
        if entries.is_empty() {
            return Vec::new();
        }
        let cluster = rail.kind.has_strip_cluster() && group == BRailGroup::Bottom;
        if cluster && rail.expanded {
            // Expanded, the cluster is the strip's footer: prev and next at
            // the two ends with the collapse control between them — on the
            // actions bar, not on the rail.
            let row = strip::place(
                strip::BStripAxis::Row,
                strip::BStripFit::Divide,
                entries.iter().map(|e| vec![self.entry(e, false, ctx)]).collect(),
                ctx,
            )
            .w_full();
            return vec![Self::bar(ctx).child(row).into_any_element()];
        }
        if cluster {
            // Collapsed, the cluster is one expand arrow, on the actions bar:
            // the bar is always there — the control shade, lit on hover — so
            // the affordance is never an unmarked patch of rail, as it was
            // while it showed only under the pointer.
            let mut arrows = Self::bar(ctx).flex().flex_col();
            for e in entries {
                arrows = arrows.child(self.entry(e, rail.expanded, ctx));
            }
            return vec![arrows.into_any_element()];
        }
        entries.into_iter().map(|e| self.entry(e, rail.expanded, ctx)).collect()
    }

    /// The holder of a rail's foot cluster — the collapse control and the
    /// paging arrows. It paints nothing and does not react to the pointer:
    /// it is not clickable, so it must not light up. Each control in it is
    /// the rail's own square tile (`tab::rail`), so the foot reads as more of
    /// the same strip.
    fn bar(ctx: &BRenderCtx) -> gpui::Stateful<gpui::Div> {
        BFlex::column().gap(BSpace::GapXs).div(ctx).id("rail-actions")
    }

    /// One entry cell. The entry's label renders inline when the strip is
    /// expanded and as the hover tooltip beside the strip when collapsed.
    ///
    /// The rail's side decides which edge of the cap runs into the pane, so a
    /// right-hand rail mirrors rather than repeating the left-hand shape. The
    /// activity bar's entries cap into nothing: each is a stack — the icon
    /// with its name under it, as Slack's rail is — or, expanded, a free
    /// button beside its name.
    fn entry(&self, e: &BRailEntry, expanded: bool, ctx: &BRenderCtx) -> AnyElement {
        let mut ctl = e.control.clone();
        ctl.tooltip = ctl.label.clone();
        if !expanded {
            ctl.label = None;
        }
        tab::rail(&ctl, ctx)
    }
}
