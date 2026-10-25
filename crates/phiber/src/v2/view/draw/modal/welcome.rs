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
        for child in list {
            column = column.child(BWelcomePart::WelcomeLine.flex(ctx).div(ctx).child(walk(child, ctx)));
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
/// These lived in `BModalPart` beside `Layer`, `Backdrop` and
/// `TitleBar`, which every modal uses. Nothing but this screen ever asked
/// for them, and a shared vocabulary carrying one screen's furniture is how
/// that vocabulary grows to sixteen variants.
enum BWelcomePart {
    /// The Welcome form: its content column, centred in the modal.
    WelcomeLayout,
    WelcomeColumn { max_width: f32 },
    /// One line of the column — the logo, its tagline, the tabs, the cards
    /// — as tall as what it holds, so the lines stack rather than share the
    /// window's height out between them.
    WelcomeLine,
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

            Self::WelcomeLine => BFlex::column().width(BLen::Full).keeps_size(),
        }
    }
}
