//! The strip card (`BCardKind::Strip`) — a header, footer or tool bar
//! region: one row of controls, a header-tone surface, the surface gutter at
//! either end (Blender's `RGN_TYPE_HEADER` / `RGN_TYPE_FOOTER` /
//! `RGN_TYPE_TOOL_HEADER`). Replaces the `Header`, `Footer` and `Toolbar`
//! panel roles; the painted ids stay `panel-header` / `panel-footer` /
//! `panel-toolbar`.

use super::flex::{BAlign, BDim, BEdge, BFill, BFlex, BFlexed, BJustify, BLen, BRadius, BSpace};
use super::ctx::BRenderCtx;
use super::walk;
use crate::v2::theme::{BStyleRole, BSurfaceTone};
use crate::v2::view::prim::{BCard, BStrip};
use gpui::prelude::*;
use gpui::{AnyElement, SharedString};

enum BBarPart {
    /// The strip itself, by kind.
    Strip(BStrip),
    /// A tab bar's ground: one pill around its tabs, as wide as they are.
    TabGround,
}

impl BFlexed for BBarPart {
    fn flex(&self, ctx: &BRenderCtx) -> BFlex {
        let kind = match self {
            // A step back from whatever it sits on, edged: it reads on the
            // Welcome backdrop and on the see-through palette alike.
            Self::TabGround => {
                return BFlex::row()
                    .keeps_size()
                    .align(BAlign::Center)
                    .pad(BSpace::PadXs)
                    .gap(BSpace::GapXs)
                    .radius(BRadius::Pill)
                    .fill(BFill::Tone(BSurfaceTone::Nested(1)))
                    .border(BEdge::All);
            }
            Self::Strip(kind) => *kind,
        };
        if kind == BStrip::Tabs {
            // Only the ground is painted; the line it sits on is not.
            return BFlex::row().width(BLen::Full).align(BAlign::Center).justify(BJustify::Center).pad_y(BSpace::PadXs);
        }
        let row = BFlex::row()
            .width(BLen::Full)
            .align(BAlign::Center)
            .pad_x(BSpace::PadMd)
            .gap(BSpace::GapXs)
            // Top and bottom strips are the same chrome, so a dialog reads
            // as one surface with a strip at each end.
            .fill(BFill::Tone(BSurfaceTone::Header));
        match kind {
            // A header keeps its height (the column it heads never trades
            // it for its cards') and rounds its top to the surface above.
            // The line under a header is the accordion's own border, so a
            // modal's header and a panel's card heading close the same way
            // against what they head (`BEdge::Bottom` is `BEdge::All`'s
            // stroke and colour, one edge of it).
            BStrip::Header => row
                .height(BLen::Dim(BDim::HeaderHeight))
                .keeps_size()
                .border(BEdge::Bottom)
                .radius(BRadius::Top(ctx.style.radius_for(BStyleRole::Panel))),
            BStrip::Footer => row.height(BLen::Dim(BDim::FooterHeight)),
            BStrip::Toolbar | BStrip::Tabs => row.height(BLen::Dim(BDim::HeaderHeight)),
        }
    }
}

fn name(kind: BStrip) -> &'static str {
    match kind {
        BStrip::Header => "panel-header",
        BStrip::Footer => "panel-footer",
        BStrip::Toolbar => "panel-toolbar",
        BStrip::Tabs => "panel-tabs",
    }
}

/// The focused renderer for a strip card.
pub struct BBar<'a>(pub BStrip, pub &'a BCard);

impl BBar<'_> {
    pub fn render(&self, ctx: &BRenderCtx) -> AnyElement {
        let id = name(self.0);
        let element = BBarPart::Strip(self.0).flex(ctx).div(ctx).id(SharedString::from(id)).debug_selector(move || id.to_string());
        // A strip's controls follow the menu treatment whether the strip is
        // a region in a host or a panel in a modal (`chrome`).
        let ctx = &ctx.in_strip();
        let tabs = self.1.body.iter().map(|child| walk(child, ctx));
        if self.0 == BStrip::Tabs {
            return element.child(BBarPart::TabGround.flex(ctx).div(ctx).children(tabs)).into_any_element();
        }
        element.children(tabs).into_any_element()
    }
}
