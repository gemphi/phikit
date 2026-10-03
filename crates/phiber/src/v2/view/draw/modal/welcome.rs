//! Focused centered body layout for the viewport-filling Phiber Welcome form.

use super::super::flex::{BAlign, BFlex, BFlexed, BJustify, BLen, BSpace};
use super::super::ctx::BRenderCtx;
use super::super::walk;
use crate::v2::modal::BModalType;
use crate::v2::view::BView;
use gpui::prelude::*;
use gpui::AnyElement;

pub struct BWelcome<'a>(pub &'a BView);

impl BWelcome<'_> {
    fn action_columns(&self, view: &BView, ctx: &BRenderCtx) -> AnyElement {
        let BView::Row { list } = view else {
            return walk(view, ctx);
        };
        let mut row = BWelcomePart::WelcomeRow.flex(ctx).div(ctx);
        for child in list {
            row = row.child(BWelcomePart::WelcomeCell.flex(ctx).div(ctx).child(walk(child, ctx)));
        }
        row.into_any_element()
    }

    pub fn render(&self, ctx: &BRenderCtx) -> AnyElement {
        let list = match self.0 {
            BView::Column { list } => list.as_slice(),
            _ => std::slice::from_ref(self.0),
        };
        let mut column = BWelcomePart::WelcomeColumn {
            max_width: BModalType::Splash.spec().width,
        }
        .flex(ctx)
        .div(ctx)
        .id("welcome-content-column")
        .debug_selector(|| "welcome-content-column".to_string());
        for (index, child) in list.iter().enumerate() {
            let rendered = if index == 2 {
                self.action_columns(child, ctx)
            } else {
                walk(child, ctx)
            };
            column = column.child(rendered);
        }
        BWelcomePart::WelcomeLayout
            .flex(ctx)
            .div(ctx)
            .id("welcome-centered-layout")
            .debug_selector(|| "welcome-centered-layout".to_string())
            .child(column)
            .into_any_element()
    }
}

/// The Welcome form's own layout — its shape, not the modal vocabulary's.
///
/// These four lived in `BModalPart` beside `Layer`, `Backdrop` and
/// `TitleBar`, which every modal uses. Nothing but this screen ever asked
/// for them, and a shared vocabulary carrying one screen's furniture is how
/// that vocabulary grows to sixteen variants.
enum BWelcomePart {
    /// The Welcome form: its content column, centred in the modal.
    WelcomeLayout,
    WelcomeColumn { max_width: f32 },
    /// A row of equal cards in the Welcome form, and one of its cells.
    WelcomeRow,
    WelcomeCell,
}

impl BFlexed for BWelcomePart {
    fn flex(&self, _ctx: &BRenderCtx) -> BFlex {
        match *self {
            Self::WelcomeLayout => BFlex::row()
                .fills()
                .full()
                .min_height(BLen::Zero)
                .align(BAlign::Center)
                .justify(BJustify::Center)
                .pad(BSpace::PadLg),

            Self::WelcomeColumn { max_width } => BFlex::column()
                .width(BLen::Full)
                .max_width(BLen::Measured(max_width))
                .gap(BSpace::GapLg),

            Self::WelcomeRow => BFlex::row()
                .align(BAlign::Stretch)
                .width(BLen::Full)
                .min_width(BLen::Zero)
                .gap(BSpace::GapLg),

            Self::WelcomeCell => BFlex::row().fills().min_width(BLen::Zero),
        }
    }
}
