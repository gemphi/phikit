//! Modal title bar with existing minimize, maximize/restore, and close actions.

use super::super::{
    color::{resolved_gradient, to_hsla},
    ctx::BRenderCtx,
    surface::compatible_inner_radius,
};
use crate::v2::modal::BModalType;
use crate::v2::state::{
    key::{K_MODAL, K_MODAL_MAXIMIZED, K_MODAL_MINIMIZED},
    val::V_EMPTY,
};
use crate::v2::theme::{BStyleRole, BSurfaceTone};
use crate::v2::view::prim::BAction;
use gpui::prelude::*;
use gpui::{AnyElement, ClickEvent, SharedString, div, px, svg};
use std::rc::Rc;

pub struct BTitle {
    pub kind: BModalType,
    pub maximized: bool,
    pub rounded: bool,
}

impl BTitle {
    fn button(
        &self,
        id: &str,
        icon: &str,
        action: BAction,
        c: &BRenderCtx,
    ) -> gpui::Stateful<gpui::Div> {
        let handler = c.h.clone();
        let dispatched = action.clone();
        let button_id = SharedString::from(format!("modal-{id}"));
        div()
            .id(button_id)
            .cursor_pointer()
            .flex()
            .items_center()
            .justify_center()
            .w(px(c.style.dimensions.button_height))
            .h(px(c.style.dimensions.button_height))
            .rounded(px(c.style.radius_for(BStyleRole::Button)))
            .hover(|style| style.bg(to_hsla(c.style.colors.bg_hover)))
            .child(
                svg()
                    .path(format!("icons/{icon}.svg"))
                    .size(px(c.style.typography.header))
                    .flex_shrink_0()
                    .text_color(to_hsla(c.style.colors.text_dim)),
            )
            .on_click(
                move |_e: &ClickEvent, w: &mut gpui::Window, cx: &mut gpui::App| {
                    (Rc::clone(&handler))(&dispatched, w, cx);
                },
            )
    }

    pub fn render(&self, c: &BRenderCtx) -> AnyElement {
        let spec = self.kind.spec();
        let max_icon = if self.maximized {
            "minimize_2"
        } else {
            "maximize_2"
        };
        let title_row = div()
            .flex()
            .flex_row()
            .items_center()
            .gap(px(c.style.spacing.gap_sm))
            .child(
                svg()
                    .path(format!("icons/{}.svg", spec.icon))
                    .size(px(c.style.typography.header))
                    .flex_shrink_0()
                    .text_color(to_hsla(c.style.colors.text)),
            )
            .child(SharedString::from(spec.label.to_string()));
        let mut bar = div()
            .flex()
            .flex_row()
            .items_center()
            .justify_between()
            .h(px(c.style.dimensions.header_height))
            .px(px(c.style.spacing.pad_md))
            .bg(resolved_gradient(c.style.surface(BSurfaceTone::Header)))
            .border_b(px(c.style.strokes.border))
            .border_color(to_hsla(c.style.colors.border))
            .text_size(px(c.style.typography.header))
            .child(title_row)
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap(px(c.style.spacing.gap_xs))
                    .child(self.button("min", "minus", BAction::Toggle(K_MODAL_MINIMIZED), c))
                    .child(self.button("max", max_icon, BAction::Toggle(K_MODAL_MAXIMIZED), c))
                    .child(self.button(
                        "close",
                        "x",
                        BAction::Set(K_MODAL, V_EMPTY.to_string()),
                        c,
                    )),
            );
        if self.rounded {
            bar = bar.rounded_t(px(compatible_inner_radius(
                c.style.radius_for(BStyleRole::Modal),
                c.style.strokes.border,
            )));
        }
        bar.into_any_element()
    }
}
