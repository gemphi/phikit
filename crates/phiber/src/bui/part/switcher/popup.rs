//! Switcher popup overlay — 2-column container component struct.

use super::super::super::bview::{ActionHandler, BAction};
use super::super::super::theme::BPalette;
use gpui::prelude::*;
use gpui::{
    AnyElement, App, ClickEvent, Div, InteractiveElement, IntoElement, SharedString, Stateful,
    StatefulInteractiveElement, Window, deferred, div, px,
};

pub struct SwitcherPopup {
    pub id: String,
}

impl SwitcherPopup {
    pub fn build(
        &self,
        p: &BPalette,
        left_col: Stateful<Div>,
        right_col: Stateful<Div>,
        action: Option<&BAction>,
        on_action: ActionHandler,
    ) -> AnyElement {
        let popup = div()
            .absolute()
            .top_full()
            .right_0()
            .mt(px(p.gap_xs))
            .flex()
            .flex_row()
            .w(px(520.0))
            .h(px(400.0))
            .bg(p.fill_sheen(p.bg_viewport))
            .border_1()
            .border_color(p.border)
            .rounded(px(p.radius_md))
            .occlude()
            .shadow_xl()
            .child(left_col)
            .child(right_col);

        let close_action = action.cloned();
        let close_handler = on_action;
        let backdrop = div()
            .absolute()
            .top_0()
            .left_0()
            .size_full()
            .id(SharedString::from(format!("{}-backdrop", self.id)))
            .on_click(move |_e: &ClickEvent, w: &mut Window, cx: &mut App| {
                if let Some(act) = &close_action {
                    close_handler(act, w, cx);
                }
            });

        div()
            .child(deferred(backdrop))
            .child(deferred(popup))
            .into_any_element()
    }
}
