//! Viewport-filling Splash/Welcome and About modal form. It arrives filling
//! the window; Restore turns it into a centred window over the dimmed
//! workspace, and Maximise fills the window again.

use super::super::flex::BFlexed;
use super::super::ctx::BRenderCtx;
use super::super::walk;
use super::{BModalPart, close_action, is_fullscreen_maximized};
use super::title::BTitle;
use super::welcome::BWelcome;
use crate::v2::modal::BModalType;
use crate::v2::view::BView;
use gpui::prelude::*;
use gpui::{AnyElement, ClickEvent, SharedString};
use std::rc::Rc;

pub struct BFullscreen<'a> {
    pub kind: BModalType,
    pub body: &'a BView,
}

impl BFullscreen<'_> {
    fn content(&self, maximized: bool, ctx: &BRenderCtx) -> [AnyElement; 2] {
        [
            BTitle {
                kind: self.kind,
                maximized,
                rounded: !maximized,
            }
            .render(ctx),
            if self.kind == BModalType::Splash {
                BWelcome(self.body).render(ctx)
            } else {
                BModalPart::FullscreenBody
                    .flex(ctx)
                    .div(ctx)
                    .child(walk(self.body, ctx))
                    .into_any_element()
            },
        ]
    }

    pub fn render(&self, ctx: &BRenderCtx) -> AnyElement {
        let maximized = is_fullscreen_maximized(ctx);
        // The Welcome backdrop (the mark, covering the window) is the ground
        // the form stands on: the whole window when maximised, the whole
        // screen behind it when restored.
        let tunnel = (self.kind == BModalType::Splash).then(super::super::brand::backdrop);
        if maximized {
            return BModalPart::Fullscreen
                .flex(ctx)
                .div(ctx)
                .children(tunnel)
                .children(self.content(true, ctx))
                .into_any_element();
        }
        let id = format!("modal-window-{:?}", self.kind);
        let window = BModalPart::FullscreenWindow
            .flex(ctx)
            .div(ctx)
            .id(SharedString::from(id.clone()))
            .debug_selector(move || id.clone())
            .children(self.content(false, ctx));
        let close_h = ctx.handler.clone();
        let close_a = close_action();
        let backdrop = BModalPart::Backdrop
            .flex(ctx)
            .div(ctx)
            .id(SharedString::from(format!("modal-backdrop-{:?}", self.kind)))
            .children(tunnel)
            .on_click(move |_e: &ClickEvent, w: &mut gpui::Window, cx: &mut gpui::App| {
                (Rc::clone(&close_h))(&close_a, w, cx);
            });
        BModalPart::Layer
            .flex(ctx)
            .div(ctx)
            .child(backdrop)
            .child(BModalPart::Centerer.flex(ctx).div(ctx).child(window))
            .into_any_element()
    }
}
