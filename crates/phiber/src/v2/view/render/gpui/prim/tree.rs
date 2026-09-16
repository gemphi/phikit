//! Focused tree family: indent, guide, disclosure, and row renderers.

use super::super::{atom, color::to_hsla, ctx::BRenderCtx, walk};
use crate::v2::theme::BStyleRole;
use crate::v2::view::BView;
use crate::v2::view::prim::{BAction, BRowShade};
use gpui::prelude::*;
use gpui::{AnyElement, div, px, svg};

pub struct BTreeIndent(pub u16);

pub struct BTreeRow<'a> {
    pub depth: u16,
    pub expanded: Option<bool>,
    pub toggle: Option<&'a BAction>,
    pub icon: &'a str,
    pub children: &'a [BView],
    pub trail: &'a [BView],
    pub shade: BRowShade,
}

struct BTreeGuide;

struct BTreeDisclosure<'a> {
    expanded: Option<bool>,
    toggle: Option<&'a BAction>,
}

impl BTreeIndent {
    pub fn render(&self, c: &BRenderCtx) -> AnyElement {
        self.render_with_unit(c.style.dimensions.indent_unit)
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
    pub fn render(&self, c: &BRenderCtx) -> AnyElement {
        let selected = self.shade == BRowShade::Selected;
        let resting = match self.shade {
            BRowShade::Flat => None,
            BRowShade::Alt => Some(c.style.colors.row_alt),
            BRowShade::Selected => Some(c.style.colors.row_selected),
        };
        let unit = c.style.dimensions.indent_unit;
        let mut element = div()
            .flex()
            .flex_row()
            .items_center()
            .w_full()
            .relative()
            .h(px(c.style.dimensions.row_height))
            .rounded(px(c.style.radius_for(BStyleRole::MenuItem)))
            .pr(px(c.style.spacing.pad_md))
            .overflow_hidden()
            .text_color(to_hsla(c.style.colors.text))
            .text_size(px(c.style.typography.body));
        if let Some(fill) = resting {
            element = element.bg(to_hsla(fill));
        }
        if selected {
            element = element.child(
                div()
                    .absolute()
                    .left_0()
                    .top_0()
                    .bottom_0()
                    .w(px(c.style.strokes.border * 2.0))
                    .bg(to_hsla(c.style.colors.accent)),
            );
        }
        let hover = to_hsla(c.style.colors.bg_hover);
        element = element.hover(move |style| style.bg(hover));

        for _ in 0..self.depth {
            element = element.child(BTreeGuide.render(c));
        }
        element = element.child(
            BTreeDisclosure {
                expanded: self.expanded,
                toggle: self.toggle,
            }
            .render(c),
        );
        element = element.child(
            div()
                .flex_shrink_0()
                .w(px(unit))
                .h_full()
                .flex()
                .items_center()
                .justify_center()
                .child(atom::icon(self.icon, c)),
        );

        let mut main = div()
            .flex()
            .flex_row()
            .items_center()
            .flex_1()
            .min_w_0()
            .gap(px(c.style.spacing.gap_xs))
            .overflow_hidden();
        for child in self.children {
            main = main.child(walk(child, c));
        }
        element = element.child(main);

        if !self.trail.is_empty() {
            let mut trail = div()
                .flex()
                .flex_row()
                .items_center()
                .flex_shrink_0()
                .gap(px(c.style.spacing.gap_xs));
            for child in self.trail {
                trail = trail.child(walk(child, c));
            }
            element = element.child(trail);
        }
        element.into_any_element()
    }
}

impl BTreeGuide {
    fn render(&self, c: &BRenderCtx) -> AnyElement {
        let unit = c.style.dimensions.indent_unit;
        let stroke = c.style.strokes.hairline;
        div()
            .flex_shrink_0()
            .relative()
            .w(px(unit))
            .h_full()
            .child(
                div()
                    .absolute()
                    .top_0()
                    .bottom_0()
                    .left(px((unit - stroke) / 2.0))
                    .w(px(stroke))
                    .bg(to_hsla(c.style.colors.sep)),
            )
            .into_any_element()
    }
}

impl BTreeDisclosure<'_> {
    fn render(&self, c: &BRenderCtx) -> AnyElement {
        let unit = c.style.dimensions.indent_unit;
        let cell = div()
            .flex_shrink_0()
            .w(px(unit))
            .h_full()
            .flex()
            .items_center()
            .justify_center();
        let Some(open) = self.expanded else {
            return cell.into_any_element();
        };
        let icon = if open {
            "chevron_down"
        } else {
            "chevron_right"
        };
        let glyph = || {
            svg()
                .path(format!("icons/{icon}.svg"))
                .size(px(c.style.typography.body))
                .flex_shrink_0()
                .text_color(to_hsla(c.style.colors.text_dim))
        };
        let Some(action) = self.toggle else {
            return cell.child(glyph()).into_any_element();
        };
        let handler = c.h.clone();
        let action_copy = action.clone();
        let row_id = match action {
            BAction::Set(path, _)
            | BAction::Toggle(path)
            | BAction::Open(path)
            | BAction::Reset(path) => path.segs.last().copied().unwrap_or("row"),
        };
        let selector = format!("tree-arrow-{row_id}");
        let id = gpui::SharedString::from(atom::button::element_id("tree", None, action));
        cell.id(id)
            .debug_selector(move || selector.clone())
            .cursor_pointer()
            .child(glyph())
            .on_click(move |_event, window, app| {
                (std::rc::Rc::clone(&handler))(&action_copy, window, app);
            })
            .into_any_element()
    }
}

use super::render_contract::BFocusedRenderer;

impl BFocusedRenderer for BTreeIndent {
    fn render(&self, c: &BRenderCtx) -> AnyElement {
        BTreeIndent::render(self, c)
    }
}

impl BFocusedRenderer for BTreeRow<'_> {
    fn render(&self, c: &BRenderCtx) -> AnyElement {
        BTreeRow::render(self, c)
    }
}
