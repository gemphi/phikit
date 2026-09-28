//! Focused centered body layout for the viewport-filling Phiber Welcome form.

use super::super::flex::BFlexed;
use super::super::ctx::BRenderCtx;
use super::super::walk;
use super::BModalPart;
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
        let mut row = BModalPart::WelcomeRow.flex(ctx).div(ctx);
        for child in list {
            row = row.child(BModalPart::WelcomeCell.flex(ctx).div(ctx).child(walk(child, ctx)));
        }
        row.into_any_element()
    }

    pub fn render(&self, ctx: &BRenderCtx) -> AnyElement {
        let list = match self.0 {
            BView::Column { list } => list.as_slice(),
            _ => std::slice::from_ref(self.0),
        };
        let mut column = BModalPart::WelcomeColumn {
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
        BModalPart::WelcomeLayout
            .flex(ctx)
            .div(ctx)
            .id("welcome-centered-layout")
            .debug_selector(|| "welcome-centered-layout".to_string())
            .child(column)
            .into_any_element()
    }
}
