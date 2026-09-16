//! Focused centered body layout for the viewport-filling Phiber Welcome form.

use super::super::{ctx::BRenderCtx, walk};
use crate::v2::modal::BModalType;
use crate::v2::view::BView;
use gpui::prelude::*;
use gpui::{AnyElement, div, px};

pub struct BWelcome<'a>(pub &'a BView);

impl BWelcome<'_> {
    fn action_columns(&self, view: &BView, c: &BRenderCtx) -> AnyElement {
        let BView::Row { children } = view else {
            return walk(view, c);
        };
        let mut row = div()
            .flex()
            .flex_row()
            .items_stretch()
            .w_full()
            .min_w_0()
            .gap(px(c.style.spacing.gap_lg));
        for child in children {
            row = row.child(div().flex().flex_1().min_w_0().child(walk(child, c)));
        }
        row.into_any_element()
    }

    pub fn render(&self, c: &BRenderCtx) -> AnyElement {
        let children = match self.0 {
            BView::Column { children } => children.as_slice(),
            _ => std::slice::from_ref(self.0),
        };
        let mut column = div()
            .id("welcome-content-column")
            .debug_selector(|| "welcome-content-column".to_string())
            .flex()
            .flex_col()
            .w_full()
            .max_w(px(BModalType::Splash.spec().width))
            .gap(px(c.style.spacing.gap_lg));
        for (index, child) in children.iter().enumerate() {
            let rendered = if index == 2 {
                self.action_columns(child, c)
            } else {
                walk(child, c)
            };
            column = column.child(rendered);
        }
        div()
            .id("welcome-centered-layout")
            .debug_selector(|| "welcome-centered-layout".to_string())
            .flex()
            .flex_1()
            .size_full()
            .min_h_0()
            .items_center()
            .justify_center()
            .p(px(c.style.spacing.pad_lg))
            .child(column)
            .into_any_element()
    }
}
