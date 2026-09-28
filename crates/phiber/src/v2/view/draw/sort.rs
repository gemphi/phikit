//! A sortable column heading — Blender's file-list header: the label, free
//! space, and the sort arrow at the far end.
//!
//! The whole heading is the control, so its fill spans the column and its
//! text keeps the inset the rows below it use. Every decision here is a
//! `BFlex` token (`tests/style_budget.rs`): the fills are `BFill`, and the
//! hover is the one every selectable thing outside a header strip answers
//! with, rather than a `.hover(|s| s.bg(…))` written out here.

use super::atom::icon_path;
use super::color::to_hsla;
use super::ctx::BRenderCtx;
use super::flex::{BAlign, BFill, BFlex, BFlexed, BJustify, BLen, BOverflow, BRadius, BSpace};
use crate::v2::theme::BStyleRole;
use crate::v2::view::{BAction, BColAlign, BSortDirection};
use gpui::prelude::*;
use gpui::{AnyElement, ClickEvent, SharedString, px, svg};
use std::rc::Rc;

pub struct BSort<'a> {
    pub label: &'a str,
    pub direction: Option<BSortDirection>,
    pub align: BColAlign,
    pub action: &'a BAction,
}

/// The parts of a sortable column heading — one enum for the level, as
/// every other level states its parts (`BCardPart`, `BModalPart`).
enum BSortPart {
    /// The heading itself. A sortable heading is a tab: the same rounding as
    /// the header's view-mode tabs, and the column the list is ordered by
    /// wears the active shade at rest, as the chosen tab does.
    Heading { sorted: bool },
    /// The arrow slot, always reserved so every heading lines up with its
    /// column whether or not it is the sorted one.
    Indicator,
    /// The label and the arrow, pushed to opposite ends.
    Content,
}

impl BFlexed for BSortPart {
    fn flex(&self, ctx: &BRenderCtx) -> BFlex {
        match *self {
            Self::Heading { sorted } => {
                let heading = BFlex::row()
                    .width(BLen::Full)
                    .height(BLen::Full)
                    .align(BAlign::Center)
                    // The text lands in the column the rows below it start in.
                    .pad_x(BSpace::PadSm)
                    .radius(BRadius::Role(BStyleRole::Tab))
                    .overflow(BOverflow::Clip)
                    .hovered(BFill::ItemHover);
                if sorted { heading.fill(BFill::ControlActive) } else { heading }
            }
            Self::Indicator => {
                let size = BLen::Measured(ctx.style.dimensions.icon_size_sm);
                BFlex::row()
                    .width(size)
                    .height(size)
                    .align(BAlign::Center)
                    .justify(BJustify::Center)
                    .keeps_size()
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
    fn indicator(&self, ctx: &BRenderCtx) -> gpui::Div {
        let size = ctx.style.dimensions.icon_size_sm;
        let slot = BSortPart::Indicator.flex(ctx).div(ctx);
        let Some(direction) = self.direction else {
            return slot;
        };
        let icon = match direction {
            BSortDirection::Ascending => "chevron_up",
            BSortDirection::Descending => "chevron_down",
        };
        // The sorted column's arrow carries the text tone, so the column the
        // list is ordered by is the one that reads; an unsorted column has no
        // arrow to dim.
        slot.child(svg().path(icon_path(icon)).size(px(size)).text_color(to_hsla(ctx.style.colors.text)))
    }

    pub fn render(&self, ctx: &BRenderCtx) -> AnyElement {
        let mut label = BFlex::row().fills().min_width(BLen::Zero).div(ctx).truncate();
        if self.align == BColAlign::End {
            label = label.text_right();
        }
        let content = BSortPart::Content
            .flex(ctx)
            .div(ctx)
            .child(label.child(SharedString::from(self.label.to_string())))
            .child(self.indicator(ctx));

        let handler = ctx.handler.clone();
        let action = self.action.clone();
        let id_text = format!("file-sort-{}", self.label.to_ascii_lowercase().replace(' ', "-"));
        let selector = id_text.clone();
        BSortPart::Heading { sorted: self.direction.is_some() }
            .flex(ctx)
            .div(ctx)
            .id(SharedString::from(id_text))
            .debug_selector(move || selector.clone())
            .cursor_pointer()
            .child(content)
            .on_click(move |_event: &ClickEvent, window, app| {
                (Rc::clone(&handler))(&action, window, app);
            })
            .into_any_element()
    }
}
