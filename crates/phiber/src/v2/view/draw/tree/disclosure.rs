//! The disclosure triangle at the head of a tree row — one indent unit
//! wide, blank for a leaf so names stay column-aligned (Blender reserves the
//! slot too), and the click that opens or closes the row.

use super::BTreePart;
use super::super::color::to_hsla;
use super::super::ctx::BRenderCtx;
use super::super::flex::BFlexed;
use crate::v2::view::prim::BAction;
use gpui::prelude::*;
use gpui::{AnyElement, px, svg};

pub(super) struct BTreeDisclosure<'a> {
    pub(super) expanded: Option<bool>,
    pub(super) toggle: Option<&'a BAction>,
}

impl BTreeDisclosure<'_> {
    pub(super) fn render(&self, ctx: &BRenderCtx) -> AnyElement {
        // The same slot the type icon takes, so the two columns line up.
        let cell = BTreePart::IconSlot.flex(ctx).div(ctx);
        // Every row carries its arrow: a leaf's is dimmed and does nothing,
        // so the arrow column reads down the whole tree.
        let Some(open) = self.expanded else {
            let mut ink = ctx.style.colors.text_dim;
            ink.3 *= 0.35;
            return cell
                .child(svg().path("icons/chevron_right.svg").size(px(ctx.style.dimensions.icon_size_sm)).flex_shrink_0().text_color(to_hsla(ink)))
                .into_any_element();
        };
        let icon = if open {
            "chevron_down"
        } else {
            "chevron_right"
        };
        let glyph = || {
            svg()
                .path(format!("icons/{icon}.svg"))
                .size(px(ctx.style.dimensions.icon_size_sm))
                .flex_shrink_0()
                .text_color(to_hsla(ctx.style.colors.text_dim))
        };
        let Some(action) = self.toggle else {
            return cell.child(glyph()).into_any_element();
        };
        let handler = ctx.handler.clone();
        let action_copy = action.clone();
        let row_id = match action {
            BAction::Set(path, _)
            | BAction::Adjust(path, _)
            | BAction::Toggle(path)
            | BAction::Open(path)
            | BAction::Reset(path) => path.segs.last().copied().unwrap_or("row"),
        };
        let selector = format!("tree-arrow-{row_id}");
        let id = gpui::SharedString::from(super::super::button::element_id("tree", None, action));
        cell.id(id)
            .debug_selector(move || selector)
            .cursor_pointer()
            .child(glyph())
            .on_click(move |_event, window, app| {
                (std::rc::Rc::clone(&handler))(&action_copy, window, app);
            })
            .into_any_element()
    }
}

