//! Focused tree family: indent, guide, disclosure, and row renderers.
//!
//! A tree is a table with indentation: a row's depth is a run of indent
//! units before its arrow slot, which every row keeps (a leaf's is blank,
//! as Blender reserves it), and the hierarchy lines that run down those
//! units are drawn once behind the whole column (`guide::guides_backdrop`),
//! so they reach the column's foot and no row draws a line of its own.

use super::atom;
use super::color::to_hsla;
use super::ctx::BRenderCtx;
use super::walk;
use crate::v2::theme::BStyleRole;
use crate::v2::view::BView;
use crate::v2::view::prim::{BAction, BRowShade};
use gpui::prelude::*;
use gpui::{AnyElement, div, px};

mod disclosure;
use self::disclosure::BTreeDisclosure;

mod drag;
mod guide;
pub use self::guide::guides_backdrop;

pub struct BTreeIndent(pub u16);

pub struct BTreeRow<'a> {
    pub depth: u16,
    pub expanded: Option<bool>,
    pub toggle: Option<&'a BAction>,
    /// Between the disclosure and the icon (a collection's checkbox).
    pub lead: &'a [BView],
    pub icon: &'a str,
    pub list: &'a [BView],
    pub trail: &'a [BView],
    pub shade: BRowShade,
    /// Dragged and dropped on (`drag.rs`).
    pub drag: Option<&'a crate::v2::view::prim::BTreeDrag>,
}

impl BTreeIndent {
    pub fn render(&self, ctx: &BRenderCtx) -> AnyElement {
        self.render_with_unit(ctx.style.dimensions.indent_unit)
    }

    /// Render with the design-system default indent unit for callers that do
    /// not have a `BRenderCtx` (e.g. snapshot helpers). Uses the same value
    /// as `DEFAULT_DIMENSIONS.indent_unit` without importing legacy defaults.
    pub fn render_default(&self) -> AnyElement {
        // 20.0 matches DEFAULT_DIMENSIONS.indent_unit; kept as a named
        // constant here so grep finds it if the default ever changes.
        const DEFAULT_INDENT_UNIT: f32 = 20.0;
        self.render_with_unit(DEFAULT_INDENT_UNIT)
    }

    fn render_with_unit(&self, unit: f32) -> AnyElement {
        div()
            .flex_shrink_0()
            .w(px(unit * self.0 as f32))
            .into_any_element()
    }
}

impl BTreeRow<'_> {
    pub fn render(&self, ctx: &BRenderCtx) -> AnyElement {
        // A tree row takes the same fills as every other selectable control
        // in the app — the strip item fill when banded, the active gradient
        // when selected. Palette-only `row_alt` / `row_selected` entries let
        // the outliner drift to shades no other region used.
        let selected = matches!(self.shade, BRowShade::Selected | BRowShade::Active);
        let unit = ctx.style.dimensions.indent_unit;
        let mut element = div()
            .flex()
            .flex_row()
            .items_center()
            .w_full()
            .relative()
            .h(px(ctx.style.dimensions.row_height))
            // A row keeps its height: when the host is short the list scrolls
            // (`column`) rather than squashing every row to fit.
            .flex_shrink_0()
            .rounded(px(ctx.style.radius_for(BStyleRole::MenuItem)))
            .pr(px(ctx.style.spacing.pad_md))
            .overflow_hidden()
            .text_color(to_hsla(ctx.style.colors.text))
            .text_size(px(ctx.style.typography.body));
        // At rest a row is its editor's body (`host_body_surface`), the
        // surface the list sits on, so the Outliner's content and the region
        // round it are one ground — lifted rows read as strips on a darker
        // parent. Only the pointer and the selection mark a row: the
        // selection its flat tone, the active row a step lighter.
        element = match self.shade {
            BRowShade::Selected => element.bg(to_hsla(ctx.style.colors.row_selected)),
            BRowShade::Active => element.bg(to_hsla(ctx.style.colors.row_active)),
            _ => element.bg(super::color::resolved_gradient(ctx.style.host_body_surface(ctx.host))),
        };
        if !selected {
            let hover = to_hsla(ctx.style.host_item_hover(ctx.host));
            element = element.hover(move |style| style.bg(hover));
        }

        element = element.child(BTreeIndent(self.depth).render(ctx));
        element = element.child(
            BTreeDisclosure {
                expanded: self.expanded,
                toggle: self.toggle,
            }
            .render(ctx),
        );
        for child in self.lead {
            element = element.child(walk(child, ctx));
        }
        element = element.child(
            div()
                .flex_shrink_0()
                .w(px(unit))
                .h_full()
                .flex()
                .items_center()
                .justify_center()
                .child(atom::icon(self.icon, ctx)),
        );

        // The row is the control. Blender selects an outliner item by clicking
        // anywhere on its row and lights the whole row on hover; a labelled
        // button as the name cell gave the name its own hover fill inside the
        // row's, so only the text looked hovered. The name's action lifts to
        // the row and the name draws as plain text.
        let (select, cell_list) = match self.list.split_first() {
            Some((BView::Button(ctl), rest)) if ctl.icon.is_none() && ctl.action.is_some() => {
                (ctl.action.clone(), std::iter::once(BView::label(ctl.label.clone().unwrap_or_default())).chain(rest.iter().cloned()).collect::<Vec<_>>())
            }
            _ => (None, self.list.to_vec()),
        };
        let mut main = div()
            .flex()
            .flex_row()
            .items_center()
            .flex_1()
            .min_w_0()
            .gap(px(ctx.style.spacing.gap_xs))
            .overflow_hidden();
        for child in &cell_list {
            main = main.child(walk(child, ctx));
        }
        element = element.child(main);

        if !self.trail.is_empty() {
            let mut trail = div()
                .flex()
                .flex_row()
                .items_center()
                .flex_shrink_0()
                .gap(px(ctx.style.spacing.gap_xs));
            for child in self.trail {
                trail = trail.child(walk(child, ctx));
            }
            element = element.child(trail);
        }
        let id = match (&select, self.drag) {
            (_, Some(drag)) => format!("outliner-row-{}", drag.payload),
            (Some(action), None) => super::button::element_id("tree-row", None, action),
            (None, None) => return element.into_any_element(),
        };
        let mut element = element.id(gpui::SharedString::from(id));
        if let Some(action) = select {
            let handler = ctx.handler.clone();
            element = element.cursor_pointer().on_click(move |_event, window, app| {
                (std::rc::Rc::clone(&handler))(&action, window, app);
            });
        }
        match self.drag {
            Some(drag) => self::drag::BRowDrag::wire(element, drag, self.icon, ctx).into_any_element(),
            None => element.into_any_element(),
        }
    }
}
