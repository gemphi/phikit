//! Where a popover opens — the one rule, for every popover in the app.
//!
//! A declaring site says which side it *prefers* (`BPopoverSide`); it never
//! says which side the panel actually lands on. That is decided here, from
//! the room the window has left, so a trigger at the right edge of a host
//! opens leftwards instead of running off the screen.
//!
//! The mechanism is two nested boxes:
//!   • an **origin** — a zero-size point pinned to one edge of the anchor
//!     (below it for a drop-down, beside it for a submenu), and
//!   • an **anchored** panel whose named corner meets that point. GPUI's
//!     `anchored` measures the panel at prepaint; if the panel would cross a
//!     window edge it swaps to the opposite corner, and if even the swap does
//!     not fit it slides the panel back inside.
//!
//! Nothing here knows about menus, switchers or property enums — it knows
//! about a point and a corner, which is why every popover can share it.

use super::super::ctx::BRenderCtx;
use crate::v2::theme::BResolvedUiStyle;
use crate::v2::view::prim::BPopoverSide;
use gpui::prelude::*;
use gpui::{Anchor, Anchored, AnyElement, Div, anchored, div, px};

/// How one preferred side resolves into a point on the anchor and the corner
/// of the panel that meets it.
pub struct BPlacement {
    pub corner: Anchor,
    /// Gap between the anchor's edge and the panel, along the opening axis.
    pub gap: f32,
    /// True when the panel drops below the anchor rather than beside it.
    vertical: bool,
    /// True when the panel's TRAILING edge meets the anchor's trailing edge
    /// rather than its leading one: a header popover grows back across the
    /// header it was opened from.
    trailing: bool,
    /// Along the other axis: a submenu rises by its own frame so its first
    /// row stands level with the row that opened it.
    lift: f32,
}

impl BPlacement {
    pub fn of(side: BPopoverSide, ctx: &BRenderCtx) -> Self {
        Self::from_style(side, ctx.style)
    }

    /// The corner a side resolves to, before any flip. Exposed so the rule can
    /// be asserted without standing up a window.
    pub fn corner_for(side: BPopoverSide, style: &BResolvedUiStyle) -> Anchor {
        Self::from_style(side, style).corner
    }

    fn from_style(side: BPopoverSide, style: &BResolvedUiStyle) -> Self {
        match side {
            // A drop-down hangs from the anchor's bottom-left corner. The
            // host switcher opens from the trigger's leading edge too, so a
            // wide picker grows into its area rather than out of it; the flip
            // below still rescues it at a window edge.
            BPopoverSide::Below | BPopoverSide::Switcher => Self {
                corner: Anchor::TopLeft,
                gap: 0.0,
                vertical: true,
                lift: 0.0,
                trailing: false,
            },
            // A header popover hangs from the anchor's bottom-RIGHT corner.
            // Its trigger is an icon at the header's right end, so leading-
            // edge placement sent the panel off the edge of the host and the
            // flip then slid it back to a position that lined up with
            // nothing. Trailing-edge placement grows it back across the
            // header, and at one side zone wide its leading edge lands on
            // the header's first icon.
            BPopoverSide::BelowEnd => Self {
                corner: Anchor::TopRight,
                gap: 0.0,
                vertical: true,
                lift: 0.0,
                trailing: true,
            },
            // A submenu opens beside its row, clear of the parent panel by
            // one gap: past the parent's frame (its pad and border) and
            // `gap_xs` beyond, its first row level with the row it came from.
            BPopoverSide::Right => {
                let frame = style.spacing.pad_xs + style.strokes.border;
                Self { corner: Anchor::TopLeft, gap: frame + style.spacing.gap_xs, vertical: false, lift: frame, trailing: false }
            }
        }
    }

    /// The zero-size point on the anchor that the panel's corner meets.
    pub fn origin(&self) -> Div {
        let element = div().absolute();
        match (self.vertical, self.trailing) {
            (true, false) => element.top_full().left_0(),
            (true, true) => element.top_full().right_0(),
            (false, _) => element.left_full().top_0(),
        }
    }

    /// Wrap the panel so it opens from that point and stays on screen.
    pub fn anchor(&self, panel: AnyElement) -> Anchored {
        // The gap opens along one axis only; the other stays flush with the
        // anchor's edge.
        let (zero, gap) = (gpui::Pixels::ZERO, px(self.gap));
        let offset = if self.vertical {
            gpui::point(zero, gap)
        } else {
            gpui::point(gap, px(-self.lift))
        };
        // The default fit mode is `SwitchAnchor`: flip to the opposite corner
        // when the panel would cross a window edge, then slide it back inside
        // if even the flipped side does not fit.
        anchored()
            .anchor(self.corner)
            .position_mode(gpui::AnchoredPositionMode::Local)
            .offset(offset)
            .child(panel)
    }
}
