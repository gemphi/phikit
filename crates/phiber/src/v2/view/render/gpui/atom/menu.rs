//! Menu-popover row — flat by default, hover-only highlight, matches
//! Blender's `layout.operator` rows. Optional `hover` action fires when
//! the pointer enters the row (submenu anchors use it to open on hover).

use super::super::{color::to_hsla, ctx::BRenderCtx};
use super::icon_path;
use crate::v2::theme::BStyleRole;
use crate::v2::view::prim::{BAction, BControl};
use gpui::prelude::*;
use gpui::{AnyElement, ClickEvent, SharedString, div, px, svg};
use std::rc::Rc;

pub fn view(ctl: &BControl, c: &BRenderCtx) -> AnyElement {
    let ic = ctl.icon.as_deref().unwrap_or("");
    let lbl = ctl.label.as_deref().unwrap_or("");
    let sc = ctl.shortcut.as_deref().unwrap_or("");
    let action = ctl.action.as_ref();
    let hover = ctl.hover.as_ref();
    // Menu rows are full-width and square (no per-row radius): in a gapless
    // popover they stack flush so the hover highlight spans the entire row
    // edge-to-edge, matching Blender's `uiPopupMenu` rows.
    let mut row = div()
        .flex()
        .flex_row()
        .items_center()
        .w_full()
        .gap(px(c.style.spacing.gap_sm))
        .px(px(c.style.spacing.pad_sm))
        .h(px(c.style.dimensions.row_height))
        .text_color(to_hsla(c.style.colors.text))
        .text_size(px(c.style.typography.body));
    if !ic.is_empty() {
        row = row.child(
            svg()
                .path(icon_path(ic))
                .size(px(c.style.typography.body))
                .flex_shrink_0()
                .text_color(to_hsla(c.style.colors.text_dim)),
        );
    }
    row = row.child(div().flex_1().child(SharedString::from(lbl.to_string())));
    if !sc.is_empty() {
        row = row.child(
            div()
                .text_color(to_hsla(c.style.colors.text_dim))
                .text_size(px(c.style.typography.caption))
                .child(SharedString::from(sc.to_string())),
        );
    }
    let Some(a) = action else {
        return row.into_any_element();
    };
    let ch = c.h.clone();
    let ac = a.clone();
    let id = SharedString::from(format!("menu-{}", lbl));
    // Full-width, square clickable: the hover fill spans the whole row so
    // rows read as one continuous menu column.
    let mut el = div()
        .id(id)
        .cursor_pointer()
        .w_full()
        .child(row)
        // Menu rows share the light host-matched interaction fill used by
        // the anchor that opened this popover.
        .hover(|s| s.bg(to_hsla(c.style.host_item_hover(c.host))))
        .on_click(
            move |_e: &ClickEvent, w: &mut gpui::Window, cx: &mut gpui::App| {
                (Rc::clone(&ch))(&ac, w, cx);
            },
        );
    if let Some(h) = hover {
        let hh = c.h.clone();
        let ha = h.clone();
        el = el.on_hover(
            move |hovered: &bool, w: &mut gpui::Window, cx: &mut gpui::App| {
                if *hovered {
                    (Rc::clone(&hh))(&ha, w, cx);
                }
            },
        );
    }
    el.into_any_element()
}

pub fn sep(c: &BRenderCtx) -> AnyElement {
    div()
        // GPUI mechanics: hairline separator uses the border stroke width as
        // the minimal 1px divider height in the GPUI div layout.
        .h(px(c.style.strokes.hairline))
        .w_full()
        .my(px(c.style.spacing.gap_xs))
        .bg(to_hsla(c.style.colors.sep))
        .into_any_element()
}
