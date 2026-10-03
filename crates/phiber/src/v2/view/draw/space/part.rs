//! The space level's layout: the window root, the middle band, and the centre
//! zone's cell — and the one `BFlexed` impl that answers how each is laid out.

use crate::v2::theme::BStyleRole;
use super::super::ctx::BRenderCtx;
use super::super::flex::{BFill, BFlex, BFlexed, BLen, BOverflow, BRadius, BSpace};

/// The pieces the space is built from.
pub(super) enum BSpacePart {
    /// The window: a column of zone bands on the gutter colour. No padding —
    /// zones tile to the window edge as Blender's editors do; the gap between
    /// them is the resize seam.
    Root,
    /// The middle band: the side columns and the centre, side by side.
    Middle,
    /// The centre column of the middle band: the centre zone with the
    /// bottom band — Timeline, Dope Sheet — under it, so the side columns
    /// run past both down to the status bar, as Blender's do.
    Centre { lead: bool },
    /// The activity rail's cell: a fixed strip that hugs its rail.
    Strip { lead: bool },
}

impl BFlexed for BSpacePart {
    fn flex(&self, _c: &BRenderCtx) -> BFlex {
        match *self {
            Self::Root => BFlex::column()
                .full()
                .fill(BFill::Gutter)
                .ink()
                .radius(BRadius::Role(BStyleRole::Workspace))
                // The window shade is rounded, so what sits on it stops at
                // the same corner: an area painting square over it is what
                // made the radius decorative.
                .overflow(BOverflow::Auto)
                .gap(BSpace::Seam),
            Self::Middle => BFlex::row().fills().min_height(BLen::Zero).width(BLen::Full),
            Self::Strip { lead } => {
                let cell = BFlex::cell().keeps_size().height(BLen::Full).min_height(BLen::Zero);
                if lead {
                    cell.margin_left(BSpace::Seam)
                } else {
                    cell
                }
            }
            Self::Centre { lead } => {
                let cell = BFlex::column().fills().height(BLen::Full).min_height(BLen::Zero).unclamped().gap(BSpace::Seam);
                // Every zone boundary keeps the same gutter — that rhythm is
                // what makes the zones read as separate editors.
                if lead {
                    cell.margin_left(BSpace::Seam)
                } else {
                    cell
                }
            }
        }
    }
}

