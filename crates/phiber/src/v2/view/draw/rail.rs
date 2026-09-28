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
//! it owns layout only, never behaviour.

use super::ctx::BRenderCtx;
use super::flex::{BFlex, BLen, BRadius};
use super::{strip, tab};
use crate::v2::theme::BStyleRole;
use crate::v2::view::prim::{BRailEntry, BRailGroup, BRailKind, BRailView};
use gpui::prelude::*;
use gpui::{AnyElement, px};

pub fn view(rail: &BRailView, ctx: &BRenderCtx) -> AnyElement {
    BRail(rail).render(ctx)
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
        let groups: Vec<Vec<AnyElement>> =
            [BRailGroup::Top, BRailGroup::Center, BRailGroup::Bottom]
                .into_iter()
                .map(|group| self.group(group, ctx))
                .collect();

        let element =
            strip::place(strip::BStripAxis::Column, strip::BStripFit::Divide, groups, ctx)
                .group(group_name)
                .h_full()
                .min_h_0();
        let element = if rail.kind == BRailKind::Activity {
            // The activity rail fills its area's card and centres its buttons
            // in it. Its pane hugs, so without a width of its own the rail
            // would be exactly one button wide and the buttons would sit on
            // the card's left edge instead of down its centre line.
            let inner = ctx.style.dimensions.activity_width - 2.0 * ctx.style.strokes.border;
            element.w(px(inner)).items_center()
        } else {
            element.w_full().items_stretch()
        };
        element
            // The leading inset belongs to the region, which knows what the
            // content beside the rail is inset by (`style.rail_lead`).
            .gap(px(ctx.style.dimensions.menu_top_gap))
            .into_any_element()
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
            let mut arrows = Self::bar(ctx).flex().flex_col().w_full();
            for e in entries {
                arrows = arrows.child(self.entry(e, rail.expanded, ctx));
            }
            return vec![arrows.into_any_element()];
        }
        entries.into_iter().map(|e| self.entry(e, rail.expanded, ctx)).collect()
    }

    /// The actions bar at a rail's foot — the collapse control and the paging
    /// arrows sit on it, not on the rail: painted as a floating control is
    /// (`chrome::paint`), the control shade at rest and the hover shade under
    /// the pointer, so nothing sits directly on the rail unmarked.
    fn bar(ctx: &BRenderCtx) -> gpui::Stateful<gpui::Div> {
        let element = BFlex::column()
            .width(BLen::Full)
            .radius(BRadius::Role(BStyleRole::Button))
            .div(ctx)
            .id("rail-actions");
        super::chrome::paint(element, false, &ctx.floating())
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
        let (side, kind) = (self.0.side, self.0.kind);
        let mut ctl = e.control.clone();
        ctl.tooltip = ctl.label.clone();
        if !expanded {
            ctl.label = None;
        }
        if kind == BRailKind::Activity {
            if !expanded {
                return tab::rail_stack(&e.control, ctx);
            }
            return tab::rail_button(&ctl, ctx);
        }
        tab::rail(&ctl, side, ctx)
    }
}
