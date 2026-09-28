//! The body column (`BColumn::Body`) — a region's scrolling content.
//!
//! Any leading `BView::TableHeader` children are rendered **outside** the
//! scroll container (sticky, inside the outer surface div) so the column
//! heading stays visible while file/content rows scroll beneath it.
//! All other children go into the bounded inner scroll div.
//!
//! This is the outer-surface / bounded-scroll separation required by task
//! 4.3: the outer div owns `bg + border + radius`; the inner div owns
//! `flex_1 min_h_0 min_w_0 overflow_y_scroll`.

use super::color::to_hsla;
use super::ctx::BRenderCtx;
use super::walk;
use super::scroller::{scroll_id, scroller};
use super::flex::BFill;
use crate::v2::theme::BSurfaceTone;
use crate::v2::theme::BStyleRole;
use crate::v2::view::BView;
use gpui::prelude::*;
use gpui::{AnyElement, div, px};

pub struct BBody<'a>(pub &'a [BView]);

impl BBody<'_> {
    pub fn render(&self, ctx: &BRenderCtx) -> AnyElement {
        // Split: TableHeader rows are sticky (outside the scroll viewport);
        // everything else scrolls inside the bounded inner container.
        let sticky_end = self
            .0
            .iter()
            .take_while(|view| matches!(view, BView::TableHeader { .. }))
            .count();
        let sticky = &self.0[..sticky_end];
        let scrollable = &self.0[sticky_end..];

        // A body that holds panels is a frame, not a surface: it gives its
        // children a bounded height to scroll in and paints nothing itself.
        // The file browser's [sidebar | file list] band used to sit inside a
        // painted body, which put a second surface with its own gutter behind
        // two panels that each already carry one — the gap around the file
        // list's column header was that gutter.
        let frame = is_frame(scrollable);
        // A body with a column heading is a section with a title — the same
        // shape as a Properties card, so it takes the card's radius.
        let headed = !sticky.is_empty();

        // Outer surface owns paint, border, radius, and size constraints.
        // The bounded inner child owns only viewport motion and content spacing.
        let outer_padding = ctx.style.spacing.pad_sm;
        let outer_padding = if frame { 0.0 } else { outer_padding };
        let outer = div()
            .id(gpui::SharedString::from("panel-body-surface"))
            .debug_selector(|| "panel-body-surface".to_string())
            .flex()
            .flex_col()
            .flex_1()
            .w_full()
            .min_h_0()
            .min_w_0();
        let outer = if frame {
            outer
        } else {
            let outer = outer
                .bg(BFill::Tone(if headed { BSurfaceTone::Card } else { BSurfaceTone::Body }).background(ctx))
                .border(px(ctx.style.strokes.border))
                .border_color(to_hsla(ctx.style.colors.border));
            if headed {
                outer.rounded(px(ctx.style.radius_for(BStyleRole::Card)))
            } else {
                outer.rounded(px(ctx.style.radius_for(BStyleRole::Panel)))
            }
        };

        // Sticky headers sit inside the outer surface div but above the scroll
        // container, so the column geometry aligns with the scrolling rows.
        // Build the outer with sticky headers chained before the inner scroll div.
        let outer = sticky
            .iter()
            .fold(outer, |acc, header| acc.child(walk(header, ctx)));

        // Bounded scroll container: content spacing and overflow only, and
        // Blender's drawn scroller over its edge (`prim/scroller.rs`).
        let id = scroll_id("panel-body", scrollable);
        let mut inner = div()
            .id(id.clone())
            .flex()
            .flex_col()
            .flex_1()
            .w_full()
            .min_h_0()
            .min_w_0()
            .p(px(outer_padding))
            .overflow_y_scroll()
            .gap(px(ctx.style.spacing.gap_xs));
        for child in scrollable {
            inner = inner.child(walk(child, ctx));
        }
        outer.child(scroller(inner, &id, ctx)).into_any_element()
    }
}

/// One row whose children are all panels: the panels paint, the body frames.
fn is_frame(list: &[BView]) -> bool {
    match list {
        [BView::Row { list }] => {
            !list.is_empty() && list.iter().all(|view| matches!(view, BView::Card(_)))
        }
        _ => false,
    }
}
