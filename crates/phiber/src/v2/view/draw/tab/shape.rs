//! What kind of cap a tab is — the variant the renderer used to recompute
//! from four booleans every time it needed a number.
//!
//! `tab/mod.rs` carried `icon_only`, `square`, `free_button`, `tall`,
//! `strip_tab` and `properties_rail_tab` alongside each other, and every
//! geometry decision re-derived its answer from some combination of them.
//! They are not six independent facts: they are one question — which of six
//! shapes is this cap — asked six times. Naming it once means each shape
//! states its own geometry, and a new shape cannot be half-added.

use super::cap::BCapSide;
use super::super::ctx::BRenderCtx;
use super::super::flex::{BAlign, BFlex, BJustify, BLen, BSelf, BSpace};
use crate::v2::host::BHostType;

/// One of the six caps the renderer draws.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BCapShape {
    /// A labelled cap cut into a horizontal strip: full strip height, so its
    /// bottom edge is the strip's and the pane below continues the surface.
    StripTab,
    /// An icon-only cap in a horizontal strip: as wide as it is tall, so it
    /// reads as a button rather than a vertical pill.
    StripIcon,
    /// A free button in a rail — the activity bar's: one module square,
    /// centred, capping into no pane.
    RailButton,
    /// An icon-only cap spanning a docked rail's width.
    RailIcon,
    /// A Properties rail tab. It keeps a full-width allocation and carries
    /// its inset in a separate paint layer, so selecting one never moves the
    /// icon (`tests/layout_gutters.rs`).
    PropertiesTab,
    /// An expanded rail entry: the icon leads the label across the rail's
    /// width, so the column reads as a list of commands rather than a stack
    /// of centred pills.
    RailEntry,
}

impl BCapShape {
    /// Which shape a cap is. Order matters: a Properties rail tab is an
    /// icon-only rail cap that paints differently, so it is asked about
    /// first.
    pub fn of(side: BCapSide, labelled: bool, host: BHostType) -> Self {
        match (labelled, side) {
            (false, _) if host == BHostType::Buts => Self::PropertiesTab,
            (false, BCapSide::Below) => Self::StripIcon,
            (false, BCapSide::Free) => Self::RailButton,
            (false, _) => Self::RailIcon,
            (true, BCapSide::Below) => Self::StripTab,
            (true, _) => Self::RailEntry,
        }
    }

    /// Whether the cap shows an icon and no label.
    pub fn icon_only(self) -> bool {
        !matches!(self, Self::StripTab | Self::RailEntry)
    }

    /// Whether this cap paints on its own absolute layer rather than on the
    /// row itself.
    pub fn paints_separately(self) -> bool {
        matches!(self, Self::PropertiesTab)
    }

    /// A labelled cap in a strip is a menu, and `chrome` is where that one
    /// treatment lives — no per-host tab styling.
    pub fn is_strip_tab(self) -> bool {
        matches!(self, Self::StripTab)
    }

    /// A cap's own height: a labelled strip cap fills the strip less the gap
    /// above it; everything else is the card-head module, so expanding a
    /// toolbar widens it without making every button taller.
    pub fn height(self, ctx: &BRenderCtx) -> f32 {
        let d = &ctx.style.dimensions;
        match self {
            // A HORIZONTAL strip's caps hang from it with the menu gap above,
            // which is the topbar's tab shape and deliberate
            // (`chrome_geometry`). The Properties rail's tab is
            // `PropertiesTab` and falls to the arm below, so it is a card
            // head — and a card head is the module now, which is what lines
            // it up with the panel headings beside it.
            Self::StripTab | Self::StripIcon => d.header_height - d.menu_top_gap,
            _ => d.card_head_height(),
        }
    }

    /// The row inside the cap: its size across the strip, how its content
    /// sits, and the inset its text keeps.
    pub fn row(self, ctx: &BRenderCtx) -> BFlex {
        let row = BFlex::row()
            .align(BAlign::Center)
            .gap(BSpace::GapSm)
            .height(BLen::Measured(self.height(ctx)));
        match self {
            Self::StripTab => row.pad_x(BSpace::PadMd),
            Self::StripIcon | Self::RailButton => row
                .width(BLen::Measured(self.height(ctx)))
                .justify(BJustify::Center),
            Self::RailIcon | Self::PropertiesTab => {
                row.width(BLen::Full).justify(BJustify::Center)
            }
            Self::RailEntry => row
                .width(BLen::Full)
                .justify(BJustify::Start)
                .pad_x(BSpace::PadSm),
        }
    }

    /// How the whole cap sits in its strip. A labelled strip cap takes the
    /// strip's own alignment — it is already pinned to the strip's base by
    /// `BCapSide::gap`, which is what puts its whole gap above it and lands
    /// its bottom edge on the header's rule. The rest keep their full height
    /// — a rail with more entries than room must scroll, never squash its
    /// icons to fit.
    pub fn seat(self, ctx: &BRenderCtx, cap: BFlex) -> BFlex {
        match self {
            Self::StripTab => cap,
            Self::StripIcon => cap
                .keeps_size()
                .width(BLen::Measured(self.height(ctx)))
                .self_align(BSelf::End),
            Self::RailButton => cap
                .keeps_size()
                .width(BLen::Measured(self.height(ctx)))
                .self_align(BSelf::Center),
            Self::PropertiesTab => cap.keeps_size().width(BLen::Full),
            Self::RailIcon | Self::RailEntry => cap.keeps_size().self_align(BSelf::Stretch),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_side_and_label_combination_names_exactly_one_shape() {
        use BCapSide::*;
        let cases = [
            // A Properties rail tab is asked about first: it is an icon-only
            // rail cap, so a later arm would swallow it.
            ((Left, false, BHostType::Buts), BCapShape::PropertiesTab),
            ((Right, false, BHostType::Buts), BCapShape::PropertiesTab),
            ((Below, false, BHostType::Empty), BCapShape::StripIcon),
            ((Free, false, BHostType::Empty), BCapShape::RailButton),
            ((Left, false, BHostType::Empty), BCapShape::RailIcon),
            ((Right, false, BHostType::Empty), BCapShape::RailIcon),
            ((Below, true, BHostType::Empty), BCapShape::StripTab),
            ((Left, true, BHostType::Empty), BCapShape::RailEntry),
            ((Free, true, BHostType::Empty), BCapShape::RailEntry),
        ];
        for ((side, labelled, host), want) in cases {
            assert_eq!(BCapShape::of(side, labelled, host), want, "{side:?} labelled={labelled} {host:?}");
        }
    }

    #[test]
    fn a_labelled_cap_is_never_icon_only_and_only_properties_paints_apart() {
        for shape in [
            BCapShape::StripTab,
            BCapShape::StripIcon,
            BCapShape::RailButton,
            BCapShape::RailIcon,
            BCapShape::PropertiesTab,
            BCapShape::RailEntry,
        ] {
            let labelled = matches!(shape, BCapShape::StripTab | BCapShape::RailEntry);
            assert_eq!(shape.icon_only(), !labelled, "{shape:?}");
            assert_eq!(
                shape.paints_separately(),
                shape == BCapShape::PropertiesTab,
                "{shape:?}: only the Properties tab has a paint layer of its own"
            );
        }
    }

    #[test]
    fn only_a_labelled_strip_cap_fills_the_strip() {
        // The rest are the card-head module, so expanding a toolbar widens
        // it instead of making every button taller.
        assert!(BCapShape::StripTab.is_strip_tab());
        for shape in [BCapShape::RailButton, BCapShape::RailIcon, BCapShape::PropertiesTab, BCapShape::RailEntry] {
            assert!(!shape.is_strip_tab(), "{shape:?}");
        }
    }
}
