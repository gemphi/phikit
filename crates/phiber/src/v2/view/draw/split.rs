//! The property-split row (`BView::Split`) — Blender's
//! `uiLayout.use_property_split` (`ui_item_prop_split`): the label in one
//! column, the value in the other, one row tall. Its painted ids keep the
//! `section-…` names the old `Section` panel role had.

use super::ctx::BRenderCtx;
use super::flex::{BAlign, BDim, BFill, BFlex, BFlexed, BJustify, BLen, BSpace};
use super::walk;
use crate::v2::theme::BSurfaceTone;
use crate::v2::view::BView;
use gpui::prelude::*;
use gpui::AnyElement;

pub struct BSplit<'a>(pub &'a [BView]);

impl BSplit<'_> {
    pub fn render(&self, ctx: &BRenderCtx) -> AnyElement {
        let selector = match self.0.first() {
            Some(BView::Label { text }) => format!("section-{text}"),
            _ => "section".to_string(),
        };
        let row_id = selector.clone();
        let mut element = BSplitPart::Row { toned: ctx.host != crate::v2::host::BHostType::Buts }
            .flex(ctx)
            .div(ctx)
            .id(gpui::SharedString::from(selector.clone()))
            .debug_selector(move || selector.clone());
        let last = self.0.len().saturating_sub(1);
        for (n, child) in self.0.iter().enumerate() {
            // The first and last columns are the two a reader lines up: the
            // label under the heading title, the value under its chevron.
            let slot = match n {
                0 => format!("{row_id}-label"),
                n if n == last => format!("{row_id}-value"),
                n => format!("{row_id}-{n}"),
            };
            // The value column hands its whole width to one control, so a
            // dropdown there is as wide as the label beside it rather than as
            // wide as its own longest option.
            let inner = if n == last && last > 0 { ctx.filling() } else { ctx.clone() };
            element = element.child(
                BSplitPart::Column
                    .flex(ctx)
                    .div(ctx)
                    .id(gpui::SharedString::from(slot.clone()))
                    .debug_selector(move || slot.clone())
                    .child(walk(child, &inner)),
            );
        }
        element.into_any_element()
    }
}

/// The parts of a property-split row — one enum for the level, as every
/// other level states its parts (`BCardPart`, `BTablePart`).
enum BSplitPart {
    /// A property row is a card row: the card-head module, so the field or
    /// button in it has room over and under it. Outside Properties it carries
    /// the section tone; inside, the card it sits in already does.
    Row { toned: bool },
    /// One column of the row. The first and last are the two a reader lines
    /// up: the label under the heading title, the value under its chevron.
    Column,
}

impl BFlexed for BSplitPart {
    fn flex(&self, _ctx: &BRenderCtx) -> BFlex {
        let toned = match *self {
            Self::Column => return BFlex::row().fills().min_width(BLen::Zero),
            Self::Row { toned } => toned,
        };
        let row = BFlex::row()
            .align(BAlign::Center)
            .justify(BJustify::Between)
            .gap(BSpace::GapXs)
            .width(BLen::Full)
            .height(BLen::Dim(BDim::CardHead))
            .pad_x(BSpace::PadSm);
        if toned { row.fill(BFill::Tone(BSurfaceTone::Section)) } else { row }
    }
}
