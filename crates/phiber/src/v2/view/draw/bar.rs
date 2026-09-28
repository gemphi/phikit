//! The strip card (`BCardKind::Strip`) — a header, footer or tool bar
//! region: one row of controls, a header-tone surface, the surface gutter at
//! either end (Blender's `RGN_TYPE_HEADER` / `RGN_TYPE_FOOTER` /
//! `RGN_TYPE_TOOL_HEADER`). Replaces the `Header`, `Footer` and `Toolbar`
//! panel roles; the painted ids stay `panel-header` / `panel-footer` /
//! `panel-toolbar`.

use super::flex::{BAlign, BDim, BFill, BFlex, BFlexed, BLen, BRadius, BSpace};
use super::ctx::BRenderCtx;
use super::walk;
use crate::v2::theme::{BStyleRole, BSurfaceTone};
use crate::v2::view::prim::{BCard, BStrip};
use gpui::prelude::*;
use gpui::{AnyElement, SharedString};

struct BBarPart(BStrip);

impl BFlexed for BBarPart {
    fn flex(&self, ctx: &BRenderCtx) -> BFlex {
        let row = BFlex::row()
            .width(BLen::Full)
            .align(BAlign::Center)
            .pad_x(BSpace::PadMd)
            .gap(BSpace::GapXs)
            // Top and bottom strips are the same chrome, so a dialog reads
            // as one surface with a strip at each end.
            .fill(BFill::Tone(BSurfaceTone::Header));
        match self.0 {
            // A header keeps its height (the column it heads never trades
            // it for its cards') and rounds its top to the surface above.
            BStrip::Header => row
                .height(BLen::Dim(BDim::HeaderHeight))
                .keeps_size()
                .radius(BRadius::Top(ctx.style.radius_for(BStyleRole::Panel))),
            BStrip::Footer => row.height(BLen::Dim(BDim::FooterHeight)),
            BStrip::Toolbar => row.height(BLen::Dim(BDim::HeaderHeight)),
        }
    }
}

fn name(kind: BStrip) -> &'static str {
    match kind {
        BStrip::Header => "panel-header",
        BStrip::Footer => "panel-footer",
        BStrip::Toolbar => "panel-toolbar",
    }
}

/// The focused renderer for a strip card.
pub struct BBar<'a>(pub BStrip, pub &'a BCard);

impl BBar<'_> {
    pub fn render(&self, ctx: &BRenderCtx) -> AnyElement {
        let id = name(self.0);
        let mut element = BBarPart(self.0).flex(ctx).div(ctx).id(SharedString::from(id)).debug_selector(move || id.to_string());
        // A strip's controls follow the menu treatment whether the strip is
        // a region in a host or a panel in a modal (`chrome`).
        let ctx = &ctx.in_strip();
        for child in &self.1.body {
            element = element.child(walk(child, ctx));
        }
        element.into_any_element()
    }
}
