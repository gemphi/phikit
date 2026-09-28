//! Centered card-modal form on a dismissible dim backdrop.

use super::super::flex::BFlexed;
use super::super::ctx::BRenderCtx;
use super::super::walk;
use super::{BModalPart, close_action, is_maximized};
use super::title::BTitle;
use crate::v2::modal::{BModalShape, BModalType};
use crate::v2::view::BView;
use gpui::prelude::*;
use gpui::{AnyElement, ClickEvent, SharedString, deferred};
use std::rc::Rc;

pub struct BCard<'a> {
    pub kind: BModalType,
    pub body: &'a BView,
}

impl BCard<'_> {
    pub fn render(&self, ctx: &BRenderCtx) -> AnyElement {
        let spec = self.kind.spec();
        let maximized = is_maximized(ctx);
        let id = format!("modal-card-{:?}", self.kind);
        // A modal that raises editors (`spec.areas`) takes the window's
        // height, as an editor does; a form keeps its declared height. Both
        // are half the window wide.
        let elevates_a_host = !spec.areas.is_empty();
        let part = match spec.shape {
            BModalShape::Sheet { share } if !maximized => BModalPart::Sheet { share },
            _ => BModalPart::Card { fills: maximized, tall: elevates_a_host, height: spec.height },
        };
        let mut card = part
        .flex(ctx)
        .div(ctx)
        .id(gpui::SharedString::from(id.clone()))
        .debug_selector(move || id.clone())
        .child(
                BTitle {
                    kind: self.kind,
                    maximized,
                    rounded: true,
                }
                .render(ctx),
            );
        // File/Asset browser bodies compose their own edge-to-edge region
        // stack (header / path bar / [sidebar | files] / execute), so the
        // card must not add an outer gutter — the content already reaches
        // the modal edges the way a host does inside a pane. Every other
        // modal (settings, pref, profile, …) keeps its pad_lg inner gutter
        // so form content doesn't butt against the card border.
        let flush = !self.kind.spec().areas.is_empty();
        card = card.child(
            BModalPart::CardBody { flush, bounded: matches!(spec.shape, BModalShape::Sheet { .. }) }
                .flex(ctx)
                .div(ctx)
                .child(walk(&without_own_header(self.body), ctx)),
        );

        let close_h = ctx.handler.clone();
        let close_a = close_action();
        let backdrop = BModalPart::Backdrop
            .flex(ctx)
            .div(ctx)
            .id(SharedString::from(format!(
                "modal-backdrop-{:?}",
                self.kind
            )))
            .on_click(
                move |_e: &ClickEvent, w: &mut gpui::Window, cx: &mut gpui::App| {
                    (Rc::clone(&close_h))(&close_a, w, cx);
                },
            );
        let wrap = BModalPart::Centerer.flex(ctx).div(ctx).child(card);
        BModalPart::Layer
            .flex(ctx)
            .div(ctx)
            .child(deferred(backdrop))
            .child(deferred(wrap))
            .into_any_element()
    }
}

/// A modal that elevates a whole host (Preferences is the Prefs editor in a
/// dialog) already names it in the title bar, so the host's own header would
/// be a second one directly beneath the first. Every other region is kept, so
/// the host is otherwise the same editor it is in a space.
fn without_own_header(body: &BView) -> BView {
    // A dialog's editors arrive as a zone tree (`BModalSpec::areas`); the
    // rule applies to every host in it.
    match body {
        BView::Zone { kind, list } => {
            return BView::Zone { kind: *kind, list: list.iter().map(without_own_header).collect() };
        }
        BView::Area { kind, child } => {
            return BView::Area { kind: *kind, child: Box::new(without_own_header(child)) };
        }
        BView::Column { list } => {
            return BView::column(list.iter().map(without_own_header).collect());
        }
        _ => {}
    }
    let BView::Host { kind, list } = body else {
        return body.clone();
    };
    if kind.keeps_header_in_dialog() {
        return body.clone();
    }
    BView::Host {
        kind: *kind,
        list: list
            .iter()
            .filter(|child| {
                !matches!(
                    child,
                    BView::Region {
                        kind: crate::v2::region::BRegionType::Header,
                        ..
                    }
                )
            })
            .cloned()
            .collect(),
    }
}
