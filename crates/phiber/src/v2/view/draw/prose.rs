//! Prose: wrapping text for reading, by role (`BView::Prose`).
//!
//! A label is a line of chrome; prose is a paragraph. Every role is a
//! `BFlex` decision — size, ink, the air above a heading, the marker before
//! an item — so the reader's page is set from the theme like everything else.

use super::ctx::BRenderCtx;
use super::flex::{BAlign, BFlex, BFlexed, BInk, BLen, BSpace, BType};
use crate::v2::view::prim::BProse;
use gpui::prelude::*;
use gpui::{AnyElement, SharedString};

/// The mark before a list item.
const BULLET: &str = "•";

enum BProsePart {
    /// The block: a column of one text run, the width of the page.
    Block(BProse),
    /// An item's marker cell.
    Marker,
    /// An item's text, beside its marker.
    ItemText,
}

impl BFlexed for BProsePart {
    fn flex(&self, _c: &BRenderCtx) -> BFlex {
        let text = BFlex::column().width(BLen::Full).min_width(BLen::Zero);
        match self {
            Self::Block(BProse::Title) => text.type_size(BType::Header).ink().pad_bottom(BSpace::GapXs),
            Self::Block(BProse::Heading) => text.type_size(BType::Header).ink().pad_top(BSpace::GapLg),
            Self::Block(BProse::Para) => text.ink().pad_bottom(BSpace::GapSm),
            Self::Block(BProse::Term) => text.ink().pad_bottom(BSpace::GapXs),
            Self::Block(BProse::Note) => text.ink_tone(BInk::Dim).type_size(BType::Caption).pad_bottom(BSpace::GapSm),
            Self::Block(BProse::Item) => BFlex::row()
                .width(BLen::Full)
                .min_width(BLen::Zero)
                .align(BAlign::Start)
                .gap(BSpace::GapSm)
                .ink()
                .pad_bottom(BSpace::GapXs),
            Self::Marker => BFlex::cell().keeps_size().ink_tone(BInk::Dim),
            Self::ItemText => BFlex::cell().fills().min_width(BLen::Zero),
        }
    }
}

pub fn view(role: BProse, text: &str, ctx: &BRenderCtx) -> AnyElement {
    let block = BProsePart::Block(role).flex(ctx).div(ctx);
    let text = SharedString::from(text.to_string());
    match role {
        BProse::Item => block
            .child(BProsePart::Marker.flex(ctx).div(ctx).child(SharedString::from(BULLET)))
            .child(BProsePart::ItemText.flex(ctx).div(ctx).child(text))
            .into_any_element(),
        _ => block.child(text).into_any_element(),
    }
}
