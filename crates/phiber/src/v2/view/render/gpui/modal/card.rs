//! Centered card-modal form on a dismissible dim backdrop.

use super::super::{
    color::{resolved_gradient, to_hsla},
    ctx::BRenderCtx,
    walk,
};
use super::{close_action, is_maximized, title::BTitle};
use crate::v2::modal::BModalType;
use crate::v2::theme::{BShadowStyle, BStyleRole, BSurfaceTone};
use crate::v2::view::BView;
use gpui::prelude::*;
use gpui::{AnyElement, ClickEvent, SharedString, deferred, div, px};
use std::rc::Rc;

pub struct BCard<'a> {
    pub kind: BModalType,
    pub body: &'a BView,
}

impl BCard<'_> {
    pub fn render(&self, c: &BRenderCtx) -> AnyElement {
        let spec = self.kind.spec();
        let maximized = is_maximized(c);
        let mut card = div()
            .flex()
            .flex_col()
            .bg(resolved_gradient(c.style.surface(BSurfaceTone::Modal)))
            .text_color(to_hsla(c.style.colors.text))
            .border(px(c.style.strokes.border))
            .border_color(to_hsla(c.style.colors.border))
            .rounded(px(c.style.radius_for(BStyleRole::Modal)))
            .occlude()
            .child(
                BTitle {
                    kind: self.kind,
                    maximized,
                    rounded: true,
                }
                .render(c),
            );
        if c.style.surface_shadow() == BShadowStyle::ExtraLarge {
            card = card.shadow_xl();
        }
        card = if maximized {
            card.flex_1().size_full()
        } else {
            card.w(px(spec.width)).h(px(spec.height))
        };
        // File/Asset browser bodies compose their own edge-to-edge region
        // stack (header / path bar / [sidebar | files] / execute), so the
        // card must not add an outer gutter — the content already reaches
        // the modal edges the way a host does inside a pane. Every other
        // modal (settings, prefs, profile, …) keeps its pad_lg inner gutter
        // so form content doesn't butt against the card border.
        let inner_pad = if matches!(self.kind, BModalType::FileBrowser) {
            0.0
        } else {
            c.style.spacing.pad_lg
        };
        card = card.child(
            div()
                .flex_1()
                .min_h_0()
                .overflow_hidden()
                .p(px(inner_pad))
                .child(walk(self.body, c)),
        );

        let close_h = c.h.clone();
        let close_a = close_action();
        let backdrop = div()
            .absolute()
            .inset_0()
            .size_full()
            .bg(to_hsla(c.style.colors.bg_modal_dim))
            .id(SharedString::from(format!(
                "modal-backdrop-{:?}",
                self.kind
            )))
            .on_click(
                move |_e: &ClickEvent, w: &mut gpui::Window, cx: &mut gpui::App| {
                    (Rc::clone(&close_h))(&close_a, w, cx);
                },
            );
        let wrap = div()
            .absolute()
            .inset_0()
            .flex()
            .items_center()
            .justify_center()
            .p(px(c.style.spacing.pad_lg))
            .child(card);
        div()
            .absolute()
            .inset_0()
            .child(deferred(backdrop))
            .child(deferred(wrap))
            .into_any_element()
    }
}

use super::super::prim::render_contract::BFocusedRenderer;

impl BFocusedRenderer for BCard<'_> {
    fn render(&self, c: &super::super::ctx::BRenderCtx) -> gpui::AnyElement {
        BCard::render(self, c)
    }
}
