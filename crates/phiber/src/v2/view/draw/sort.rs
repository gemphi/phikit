//! A sortable column heading — Blender's file-list header: the label, free
//! space, and the sort arrow at the far end.
//!
//! The whole heading is the control: its fill runs the column's width as a
//! tile inset from the header's edges, and its text keeps the inset the
//! rows below it use. Every decision here is a
//! `BFlex` token (`tests/style_budget.rs`): the fills are `BFill`, and the
//! hover is the one every selectable thing outside a header strip answers
//! with, rather than a `.hover(|s| s.bg(…))` written out here.

use super::atom::icon_path;
use super::color::to_hsla;
use super::ctx::BRenderCtx;
use super::flex::{BAlign, BFlex, BFlexed, BJustify, BLen, BOverflow, BSelf, BSpace};
use crate::v2::view::{BAction, BColAlign, BSortDirection};
use gpui::prelude::*;
use gpui::{AnyElement, ClickEvent, SharedString, px, svg};
use std::rc::Rc;

/// The hover group a column heading's cell is (`table::HeadCell`): under
/// the pointer, an unsorted column shows the arrow a click would sort by.
pub const HEADING: &str = "sort-heading";

pub struct BSort<'a> {
    pub label: &'a str,
    pub direction: Option<BSortDirection>,
    pub align: BColAlign,
    pub action: &'a BAction,
}

/// The parts of a sortable column heading — one enum for the level, as
/// every other level states its parts (`BCardPart`, `BModalPart`).
enum BSortPart {
    /// The heading itself: its label where the column's values start, the
    /// arrow at the column's end on the one the list is ordered by —
    /// Blender's header, which marks the sort with the arrow alone.
    Heading,
    /// The arrow slot, always reserved so every heading lines up with its
    /// column whether or not it is the sorted one. `hint`: the arrow of an
    /// unsorted column, shown only while its heading is hovered.
    Indicator { hint: bool },
    /// The label and the arrow, pushed to opposite ends.
    Content,
}

impl BFlexed for BSortPart {
    fn flex(&self, ctx: &BRenderCtx) -> BFlex {
        match *self {
            // The cell it stands in takes the hover (`table::HeadCell`), so
            // the whole column lights, the gap before it included.
            Self::Heading => BFlex::row()
                .fills()
                .height(BLen::Full)
                .self_align(BSelf::Center)
                .align(BAlign::Center)
                .overflow(BOverflow::Clip),
            Self::Indicator { hint } => {
                let size = BLen::Measured(ctx.style.dimensions.icon_size_sm);
                let slot = BFlex::row().width(size).height(size).align(BAlign::Center).justify(BJustify::Center).keeps_size();
                if hint { slot.revealed_by(HEADING) } else { slot }
            }
            Self::Content => BFlex::row()
                .width(BLen::Full)
                .min_width(BLen::Zero)
                .align(BAlign::Center)
                .justify(BJustify::Between)
                .gap(BSpace::GapXs),
        }
    }
}

impl BSort<'_> {
    fn justify(&self) -> BJustify {
        match self.align {
            BColAlign::Start => BJustify::Start,
            BColAlign::End => BJustify::End,
        }
    }

    fn indicator(&self, ctx: &BRenderCtx) -> gpui::Div {
        let size = ctx.style.dimensions.icon_size_sm;
        // The sorted column shows its order; any other, hovered, the order a
        // click gives it — a column newly picked sorts ascending
        // (`file::op::BSortRule`), dimmer, as a hint is.
        let (direction, hint) = match self.direction {
            Some(direction) => (direction, false),
            None => (BSortDirection::Ascending, true),
        };
        let icon = match direction {
            BSortDirection::Ascending => "chevron_up",
            BSortDirection::Descending => "chevron_down",
        };
        let ink = if hint { ctx.style.colors.text_dim } else { ctx.style.colors.text };
        BSortPart::Indicator { hint }.flex(ctx).div(ctx).child(svg().path(icon_path(icon)).size(px(size)).text_color(to_hsla(ink)))
    }

    pub fn render(&self, ctx: &BRenderCtx) -> AnyElement {
        // The word where the values are — from the column's start, or for a
        // right-aligned column ending where they end — and the arrow always
        // at the heading's right end.
        let label = BFlex::row().fills().min_width(BLen::Zero).justify(self.justify()).div(ctx).truncate();
        let label = label.child(SharedString::from(self.label.to_string()));
        let content = BSortPart::Content.flex(ctx).div(ctx).child(label).child(self.indicator(ctx));

        let handler = ctx.handler.clone();
        let action = self.action.clone();
        let id_text = format!("file-sort-{}", self.label.to_ascii_lowercase().replace(' ', "-"));
        let selector = id_text.clone();
        BSortPart::Heading
            .flex(ctx)
            .div(ctx)
            .id(SharedString::from(id_text))
            .debug_selector(move || selector)
            .cursor_pointer()
            .child(content)
            .on_click(move |_event: &ClickEvent, window, app| {
                (Rc::clone(&handler))(&action, window, app);
            })
            .into_any_element()
    }
}
