use super::super::{
    atom::icon_path,
    color::{resolved_gradient, to_hsla},
    ctx::BRenderCtx,
};
use crate::v2::theme::{BStyleRole, BSurfaceTone};
use crate::v2::view::BAction;
use gpui::prelude::*;
use gpui::{AnyElement, ClickEvent, SharedString, div, px, svg};
use std::rc::Rc;

pub struct BAccordion<'a> {
    pub title: &'a str,
    pub expanded: bool,
    pub action: &'a BAction,
}

impl BAccordion<'_> {
    pub fn render(&self, c: &BRenderCtx) -> AnyElement {
        let id_text = format!("accordion-header-{}", action_key(self.action));
        let group = SharedString::from(format!("{id_text}-hover"));
        let chip_size = c
            .style
            .dimensions
            .button_height
            .min(c.style.dimensions.header_height - 2.0 * c.style.spacing.pad_xs);
        let chip_fill = if self.expanded {
            c.style.colors.bg_active
        } else {
            c.style.colors.bg_hover
        };
        let hover_fill = chip_fill;
        let icon = if self.expanded {
            "chevron_down"
        } else {
            "chevron_right"
        };
        let title_id_text = format!("accordion-title-{}", action_key(self.action));
        let title_el = div()
            .id(SharedString::from(title_id_text.clone()))
            .debug_selector(move || title_id_text.clone())
            .min_w_0()
            .overflow_hidden()
            .child(SharedString::from(self.title.to_string()));
        let chip_id_text = format!("accordion-disclosure-{}", action_key(self.action));
        let chip = div()
            .id(SharedString::from(chip_id_text.clone()))
            .debug_selector(move || chip_id_text.clone())
            .flex()
            .flex_shrink_0()
            .items_center()
            .justify_center()
            .w(px(chip_size))
            .h(px(chip_size))
            .rounded(px(c.style.radius_for(BStyleRole::Button)))
            .bg(to_hsla(chip_fill))
            .group_hover(group.clone(), move |style| style.bg(to_hsla(hover_fill)))
            .child(
                svg()
                    .path(icon_path(icon))
                    .size(px(c.style.typography.body))
                    .text_color(to_hsla(c.style.colors.text)),
            );

        let handler = c.h.clone();
        let action = self.action.clone();
        div()
            .id(SharedString::from(id_text.clone()))
            .debug_selector(move || id_text.clone())
            .group(group)
            .cursor(gpui::CursorStyle::PointingHand)
            .flex()
            .flex_row()
            .items_center()
            .justify_between()
            .w_full()
            .h(px(c.style.dimensions.header_height))
            .px(px(c.style.spacing.pad_md))
            .rounded_t(px(c.style.radius_for(BStyleRole::Card)))
            .bg(resolved_gradient(c.style.surface(BSurfaceTone::Header)))
            .border_b(px(c.style.strokes.border))
            .border_color(to_hsla(c.style.colors.sep))
            .text_color(to_hsla(c.style.colors.text))
            .text_size(px(c.style.typography.body))
            .child(title_el)
            .child(chip)
            .on_click(move |_event: &ClickEvent, window, app| {
                (Rc::clone(&handler))(&action, window, app);
            })
            .into_any_element()
    }
}

fn action_key(action: &BAction) -> String {
    let path = match action {
        BAction::Set(path, _)
        | BAction::Toggle(path)
        | BAction::Open(path)
        | BAction::Reset(path) => path,
    };
    path.encode().replace('.', "-")
}

use super::render_contract::BFocusedRenderer;

impl BFocusedRenderer for BAccordion<'_> {
    fn render(&self, c: &BRenderCtx) -> AnyElement {
        BAccordion::render(self, c)
    }
}
