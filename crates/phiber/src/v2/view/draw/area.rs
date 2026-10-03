//! Area walker — fixed side-column widths come from `BAreaLayout::size()`;
//! the enclosing Zone owns variable-axis fractions and height allocation.
//! Left/Right areas scroll vertically and all hosts fill their assigned slot.

use gpui::prelude::*;
use gpui::{AnyElement, SharedString};
use crate::v2::area::BAreaType;
use crate::v2::host::BHostType;
use crate::v2::theme::BStyleRole;
use crate::v2::view::BView;
use crate::v2::view::layout::BAreaLayout;
use super::flex::{BEdge, BFill, BFlex, BFlexed, BLen, BRadius, BOverflow};
use super::ctx::BRenderCtx;
use super::surface::BCorners;
use super::walk;

fn child_host(v: &BView) -> BHostType {
    if let BView::Host { kind, .. } = v {
        *kind
    } else {
        BHostType::Empty
    }
}

/// The card an area draws its host on: the host's body shade — the panel
/// shade for an editor, the chrome shade for a bar or rail — with the area's
/// outline and rounding. Every area takes the same card, whatever it holds:
/// a canvas paints over its fill wall to wall, Properties stacks its own
/// cards on it, and the window shade shows only in the seams between cards.
fn card(kind: BAreaType, flex: BFlex, ctx: &BRenderCtx) -> BFlex {
    let flex = flex.fill(BFill::HostBody);
    if kind == BAreaType::Top {
        // The top strip is the window's title bar: it meets the window's top
        // edge flush, with no rounded corners up there, and rounds only where
        // it meets the workspace below. Its one border is the line along that
        // bottom edge — the same stroke every other header closes with — not
        // a box, which would draw a seam up the window's own sides.
        let r = ctx.style.radius_for(BStyleRole::Area);
        return flex
            .border(BEdge::Bottom)
            .radius(BRadius::Corners { tl: false, tr: false, bl: true, br: true, radius: r })
            .overflow(BOverflow::Auto);
    }
    // The area's fill is the window shade behind its host. It rounds, so it
    // has to clip: without this the host's own strips paint square over the
    // corner and the radius is decorative — the timeline's background
    // bleeding past its card is this. A popover escapes anyway, painting in
    // the window-root overlay layer rather than in the area.
    flex.border(BEdge::All).radius(BRadius::Role(BStyleRole::Area)).overflow(BOverflow::Auto)
}

/// The parts of an area — one enum for the level, as every other level
/// states its parts (`BCardPart`, `BModalPart`, `BTablePart`).
enum BAreaPart {
    /// An area as drawn: its kind, and the host it holds.
    Frame { kind: BAreaType, host: BHostType },
    /// The scroller a side area wraps a stacking host in.
    Scroll,
}

impl BFlexed for BAreaPart {
    fn flex(&self, ctx: &BRenderCtx) -> BFlex {
        match *self {
            Self::Scroll => BFlex::column().fills().full().unclamped(),
            // The Activity rail is a fixed strip, one rail wide and never
            // dragged, but still a zone of the workspace, so it carries the
            // same card as every other area. One rail wide normally; rail
            // plus a panel while a primary icon's secondary strip is open.
            Self::Frame { kind, host } if host == BHostType::Activity => {
                let width = ctx.style.dimensions.activity_width
                    + match crate::v2::part::activity::option::open_group(ctx.state) {
                        Some(_) => ctx.style.dimensions.panel_width,
                        None => 0.0,
                    };
                card(
                    kind,
                    BFlex::row()
                        .width(BLen::Measured(width))
                        .height(BLen::Full)
                        .min_height(BLen::Zero)
                        .keeps_size(),
                    ctx,
                )
            }
            // The zone owns the variable-axis split; an area only fixes a
            // side column's width. Applying Top/Bottom heights here made
            // nested hosts overflow or collapse instead of filling their
            // allotted slot.
            Self::Frame { kind, .. } => {
                let flex = BFlex::column().fills().min_height(BLen::Zero);
                let flex = match kind.size().width {
                    Some(w) => flex.width(BLen::Measured(w)),
                    None => flex.width(BLen::Full).min_width(BLen::Zero),
                };
                card(kind, flex, ctx)
            }
        }
    }
}

pub fn view(kind: BAreaType, child: &BView, outer: &BRenderCtx) -> AnyElement {
    let host = child_host(child);

    let frame = BAreaPart::Frame { kind, host };
    // The card is the host's: its fill and its flushness follow the host it
    // holds, so the area is drawn in that host's context.
    let ctx = &outer.with_host(host);
    if host == BHostType::Activity {
        // No clipping: an area's opaque child paints its own matching corners
        // (`semantic_hierarchy_surfaces`).
        return frame.flex(ctx).div(ctx).child(walk(child, ctx)).into_any_element();
    }
    let element = frame.flex(ctx).div(ctx);

    // The host is the opaque descendant that reaches every area corner. Give
    // it explicit ownership so it can paint a compatible inner arc; clipping
    // with `overflow_hidden` is intentionally avoided because GPUI's mask is
    // rectangular rather than radius-aware.
    let child_context = ctx.with_corners(BCorners::ALL);
    // A side area scrolls only when its host *stacks* content that can run
    // taller than the slot — properties, an outliner, a file list. A
    // content-fill host is an editor canvas: it fills the area. Scrolling one
    // let its canvas take its natural height and left a dead band between the
    // canvas and the area's bottom edge, which is the gap under the Image
    // Editor in the Texture Paint workspace.
    if kind.is_scrollable() && !host.fills_content() {
        let id = SharedString::from(format!("area-{:?}-{}", kind, host.slug()));
        let scroller = BAreaPart::Scroll
            .flex(ctx)
            .scroller(id, ctx)
            .child(walk(child, &child_context));
        return element.child(scroller).into_any_element();
    }
    // A 3D Viewport draws its own view (`scene::view_slot`): every view key
    // read while its host draws is this area's, and the pointer over its
    // canvas makes it the one the view keys and menus act on.
    let slot = crate::v2::scene::view_slot::BViewSlot::new(crate::v2::space::BSpaceList::active(ctx.state).slug(), kind.label());
    element.child(slot.with(|| walk(child, &child_context))).into_any_element()
}
