//! The popover: the anchor stays in flow and the open body is appended as a
//! deferred overlay.
//!
//! A popover is not a special kind of thing. It is a card that floats: the
//! panel is a card's frame at the overlay elevation, and the rows inside it
//! are a card's body — one gutter on all four sides so a row's hover fill
//! stops short of the border radius, and one `gap_xs` between rows, exactly
//! what a card body leaves. Saying that in `BFlex`, as `BCardPart`,
//! `BMenuPart` and `BModalPart` do, is what makes a menu, a filter popover
//! and an accordion read as three settings of one thing rather than three
//! hand-built panels that happen to look similar.
//!
//! This file used to build all of that with raw `div()` chains, which is why
//! the popover drifted from the cards beside it every time either moved.

use super::ctx::BRenderCtx;
use super::flex::{BAlign, BDim, BEdge, BFill, BFlex, BFlexed, BLen, BOverflow, BPlace, BRadius, BSpace};
use super::surface::compatible_inner_radius;
use super::walk;
mod leave;
pub mod place;

use crate::v2::theme::{BElevation, BStyleRole, BSurfaceTone};
use crate::v2::view::BView;
use crate::v2::view::prim::{BAction, BPopoverSide};
use gpui::prelude::*;
use gpui::{AnyElement, Div, deferred};

/// The parts of a popover — one enum for the level, as every other level
/// states its parts (`BCardPart`, `BMenuPart`, `BModalPart`).
enum BPopoverPart {
    /// The floating panel: a card's frame at the overlay elevation. Bounded
    /// on both axes, so neither a long host list nor a wide one runs past the
    /// area it opened in.
    Panel { two_column: bool },
    /// The rows inside it, as a card body holds its rows: the frame gutter on
    /// all four sides and one gap between them. It scrolls — `empty` keeps an
    /// opened submenu with nothing in it visible instead of collapsing to
    /// nothing (`viewport_layout_bug_conditions` C4).
    Content { empty: bool },
    /// The host switcher's two columns, and the row that holds them. The
    /// columns paint nothing: the panel owns every surface.
    SwitcherRow,
    SwitcherColumn { leading: bool },
    /// The anchor's own box: in flow, and the origin the overlay is placed
    /// from.
    Anchor,
}

impl BFlexed for BPopoverPart {
    fn flex(&self, ctx: &BRenderCtx) -> BFlex {
        match *self {
            Self::Panel { two_column } => {
                let wide = ctx.style.dimensions.panel_width * 2.0;
                let panel = BFlex::column()
                    .fill(BFill::Tone(BSurfaceTone::Card))
                    .ink()
                    .border(BEdge::All)
                    .radius(BRadius::All(compatible_inner_radius(
                        ctx.style.radius_for(BStyleRole::Popover),
                        ctx.style.strokes.border,
                    )))
                    // Every dropdown is at least one panel wide, so the menus
                    // in a strip share a width instead of each sizing to its
                    // own longest label; content may grow it further.
                    .min_width(BLen::Dim(BDim::PanelWidth))
                    // As tall as its rows need, or as the window allows,
                    // whichever is less — and it scrolls past that.
                    //
                    // The cap used to be `panel_width * 2`, 480px, which the
                    // File menu cannot meet: it wants about 650. What hid that
                    // was the rows being squashed to fit, until the caps began
                    // keeping their size (`BMenuPart::Cap`) and the menu
                    // started overflowing by 239px instead. Twice a panel's
                    // WIDTH was never a statement about height.
                    //
                    // The window's height, from state, for the same reason the
                    // tree reads its width there: this view is built from state
                    // alone, and a popover is deferred over the whole window,
                    // so a share of its own parent is a share of the anchor and
                    // clips every menu. The fallback is the old cap, for a
                    // render that has not been through a frame yet.
                    .max_height(BLen::Measured(
                        ctx.state
                            .get(crate::v2::state::AppKey::WindowHeight)
                            .and_then(|h| h.parse::<f32>().ok())
                            .map_or(wide, |h| h * 0.92),
                    ))
                    .overflow(BOverflow::Clip)
                    .occludes()
                    .elevated(BElevation::Overlay);
                // The host switcher shows two columns when there is room and
                // one panel's width when there is not, so a narrow area still
                // gets a picker that fits inside it.
                if two_column { panel.max_width(BLen::Measured(wide)) } else { panel }
            }
            Self::Content { empty } => {
                let column = BFlex::column()
                    .width(BLen::Full)
                    .pad(BSpace::PadXs)
                    .gap(BSpace::GapXs);
                // An empty body would collapse a flex column to no height at
                // all, which reads as a menu that did not open.
                if empty {
                    column.min_height(BLen::Dim(BDim::RowHeight))
                } else {
                    column.min_height(BLen::Zero)
                }
            }
            Self::SwitcherRow => BFlex::row().width(BLen::Full).align(BAlign::Stretch),
            Self::Anchor => BFlex::cell().place(BPlace::Anchor),
            Self::SwitcherColumn { leading } => {
                let column = BFlex::column().fills().min_width(BLen::Zero).gap(BSpace::GapXs);
                if leading { column.pad_right(BSpace::PadXs) } else { column.pad_left(BSpace::PadXs) }
            }
        }
    }
}

pub struct BPopover<'a> {
    pub anchor: &'a BView,
    pub body: &'a [BView],
    pub open: bool,
    pub close: &'a BAction,
    pub side: BPopoverSide,
}

impl BPopover<'_> {
    /// Where this popover opens. The side it was declared with is only a
    /// preference: `place::BPlacement` turns it into a point on the anchor and
    /// a panel corner, and flips that corner when the window has no room —
    /// which is why a trigger at a host's right edge no longer clips.
    fn placement(&self, ctx: &BRenderCtx) -> place::BPlacement {
        place::BPlacement::of(self.side, ctx)
    }

    /// Host-switcher popover body: two columns, categories then hosts.
    fn switcher_columns(&self, ctx: &BRenderCtx) -> Option<Div> {
        let [BView::Row { list }] = self.body else {
            return None;
        };
        let [BView::Column { list: categories }, BView::Column { list: hosts }] = list.as_slice() else {
            return None;
        };
        let column = |items: &[BView], leading: bool| {
            let part = BPopoverPart::SwitcherColumn { leading };
            items.iter().fold(part.flex(ctx).div(ctx), |column, item| column.child(walk(item, ctx)))
        };
        Some(
            BPopoverPart::SwitcherRow
                .flex(ctx)
                .div(ctx)
                .child(column(categories, true))
                .child(column(hosts, false)),
        )
    }
}

/// Public compatibility wrapper — renders a popover anchored in flow.
pub fn view(
    anchor: &BView,
    body: &[BView],
    open: bool,
    close: &BAction,
    side: BPopoverSide,
    ctx: &BRenderCtx,
) -> AnyElement {
    BPopover { anchor, body, open, close, side }.render(ctx)
}

impl BPopover<'_> {
    pub fn render(&self, ctx: &BRenderCtx) -> AnyElement {
        if !self.open {
            return walk(self.anchor, ctx);
        }
        // `BelowEnd` is the host switcher, the one popover with its own
        // two-column picker; every other side adopts the panel width so a
        // dropdown reads as a continuation of the button that opened it.
        let two_column = self.side == BPopoverSide::BelowEnd;
        let mut content = BPopoverPart::Content { empty: self.body.is_empty() }
            .flex(ctx)
            .scroller(gpui::SharedString::from("popover-content"), ctx);
        match two_column.then(|| self.switcher_columns(ctx)).flatten() {
            Some(columns) => content = content.child(columns),
            None => {
                for item in self.body {
                    content = content.child(walk(item, ctx));
                }
            }
        }
        let panel = BPopoverPart::Panel { two_column }
            .flex(ctx)
            .div(ctx)
            .id(gpui::SharedString::from("popover-panel"))
            .debug_selector(|| "popover-panel".to_string())
            .child(content);
        // Blender closes a menu block once the pointer strays far enough from
        // it (`BLOCK_MOVEMOUSE_QUIT`). `BLeave` records where the panel and
        // anchor were painted; its backdrop asks on every click and move, and
        // sits BEHIND the panel, which occludes so its own clicks never reach
        // it. That is also what gives every enum picker "select item → the
        // popover closes" without a compound action.
        let leave = leave::BLeave::default();
        let panel = panel.child(leave.panel_probe());
        let backdrop = leave.backdrop(&ctx.handler, self.close);
        let placement = self.placement(ctx);
        let opened = placement.origin().child(placement.anchor(panel.into_any_element()));
        BPopoverPart::Anchor
            .flex(ctx)
            .div(ctx)
            .child(walk(self.anchor, ctx))
            .child(leave.anchor_probe())
            .child(deferred(backdrop))
            .child(deferred(opened))
            .into_any_element()
    }
}
