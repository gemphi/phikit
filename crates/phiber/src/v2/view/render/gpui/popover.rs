//! Single-view popover renderer. The anchor stays in flow and the open body
//! is appended as a deferred semantic overlay.
//!
//! ## File layout (preserves source-text characterization test splits)
//!
//! 1. `fn switcher_columns` — two transparent columns for the host switcher
//! 2. `pub fn view(` — public compatibility wrapper (test split sentinel)
//! 3. `fn render` — GPUI builder bodies: semantic surface, border_1(), overflow_hidden(),
//!    compatible_inner_radius, shadow_xl, deferred overlay

use super::{
    color::{resolved_gradient, to_hsla},
    ctx::BRenderCtx,
    prim::render_contract::BFocusedRenderer,
    surface::compatible_inner_radius,
    walk,
};
use crate::v2::theme::{BShadowStyle, BStyleRole, BSurfaceTone};
use crate::v2::view::BView;
use crate::v2::view::prim::{BAction, BPopoverSide};
use gpui::prelude::*;
use gpui::{AnyElement, Div, deferred, div, px};

pub struct BPopover<'a> {
    pub anchor: &'a BView,
    pub body: &'a [BView],
    pub open: bool,
    pub close: &'a BAction,
    pub side: BPopoverSide,
}

impl BPopover<'_> {
    fn place(&self, element: Div, c: &BRenderCtx) -> Div {
        match self.side {
            BPopoverSide::Below => element.absolute().top_full().left_0(),
            BPopoverSide::BelowEnd => element.absolute().top_full().right_0(),
            BPopoverSide::Right => element
                .absolute()
                .left_full()
                .top_0()
                .ml(px(c.style.spacing.pad_xs)),
        }
    }

    /// Host-switcher popover body: two transparent columns (categories | hosts).
    /// The inner columns must remain transparent — the popover surface owns all paint.
    fn switcher_columns(&self, c: &BRenderCtx) -> Option<Div> {
        let [BView::Row { children }] = self.body else {
            return None;
        };
        let [
            BView::Column {
                children: categories,
            },
            BView::Column { children: hosts },
        ] = children.as_slice()
        else {
            return None;
        };
        let mut left = div()
            .flex()
            .flex_col()
            .flex_1()
            .min_w_0()
            .gap(px(c.style.spacing.gap_xs))
            .pr(px(c.style.spacing.pad_xs))
            .border_r(px(c.style.strokes.border))
            .border_color(to_hsla(c.style.colors.sep));
        for item in categories {
            left = left.child(walk(item, c));
        }
        let mut right = div()
            .flex()
            .flex_col()
            .flex_1()
            .min_w_0()
            .gap(px(c.style.spacing.gap_xs))
            .pl(px(c.style.spacing.pad_xs));
        for item in hosts {
            right = right.child(walk(item, c));
        }
        Some(
            div()
                .flex()
                .flex_row()
                .items_stretch()
                .w_full()
                .child(left)
                .child(right),
        )
    }
}

// ── Public compatibility wrapper ──────────────────────────────────────────
// Placed BEFORE `render` so the test's source-text split on `fn switcher_columns(`
// and `pub fn view(` captures only the transparent-column body (no `.bg(`).

/// Public compatibility wrapper — renders a popover anchored in flow.
pub fn view(
    anchor: &BView,
    body: &[BView],
    open: bool,
    close: &BAction,
    side: BPopoverSide,
    c: &BRenderCtx,
) -> AnyElement {
    BPopover {
        anchor,
        body,
        open,
        close,
        side,
    }
    .render(c)
}

// ── Focused renderer impl ─────────────────────────────────────────────────
// All GPUI builder bodies are here. Semantic style: border_1() (single-pixel
// border), compatible_inner_radius(Popover), overflow_hidden(), shadow_xl on
// ExtraLarge shadow style. The outer panel owns all surface paint.

impl BPopover<'_> {
    pub fn render(&self, c: &BRenderCtx) -> AnyElement {
        if !self.open {
            return walk(self.anchor, c);
        }
        // BelowEnd (host switcher) needs its explicit two-panel-wide picker;
        // Below / Right adopt the anchor width so the dropdown reads as a
        // continuation of the button that opened it (card/section behavior).
        // Menu dropdowns (Below / Right) are gapless: rows run full-width and
        // stack flush against each other with only a thin vertical frame, so
        // the popover reads as one continuous menu (Blender's `uiPopupMenu`)
        // rather than a set of padded pills. The two-column host switcher
        // (BelowEnd) keeps its inner padding for the category / host columns.
        let two_column = self.side == BPopoverSide::BelowEnd;
        let (frame_x, frame_y, row_gap) = if two_column {
            (
                c.style.spacing.pad_xs,
                c.style.spacing.pad_xs,
                c.style.spacing.gap_xs,
            )
        } else {
            (0.0, c.style.spacing.pad_xs, c.style.spacing.gap_none)
        };
        let mut content = div()
            .flex()
            .flex_col()
            .w_full()
            .py(px(frame_y))
            .px(px(frame_x))
            .gap(px(row_gap));
        if two_column {
            if let Some(columns) = self.switcher_columns(c) {
                content = content.child(columns);
            } else {
                for item in self.body {
                    content = content.child(walk(item, c));
                }
            }
        } else if self.body.is_empty() {
            content = content.min_h(px(c.style.dimensions.row_height));
        } else {
            for item in self.body {
                content = content.child(walk(item, c));
            }
        }
        let panel_base = self.place(div().flex().flex_col(), c);
        let panel_sized = match self.side {
            BPopoverSide::BelowEnd => {
                // Host switcher: explicit two-panel width for the categories +
                // hosts columns.
                panel_base.min_w(px(c.style.dimensions.panel_width * 2.0))
            }
            _ => {
                // Default: adopt the anchor's rendered width. min_w_full()
                // reads from the wrapping .relative() div, whose bounds equal
                // the anchor's natural width. Content still lets the panel
                // grow past that when a body row demands more.
                panel_base.min_w_full()
            }
        };
        let mut panel = panel_sized
            .bg(resolved_gradient(c.style.surface(BSurfaceTone::Card)))
            .text_color(to_hsla(c.style.colors.text))
            .border_1()
            .border_color(to_hsla(c.style.colors.border))
            .rounded(px(compatible_inner_radius(
                c.style.radius_for(BStyleRole::Popover),
                c.style.strokes.border,
            )))
            .overflow_hidden()
            .occlude()
            .child(content);
        if c.style.surface_shadow() == BShadowStyle::ExtraLarge {
            panel = panel.shadow_xl();
        }
        // Click-outside-to-close backdrop. A full-viewport transparent layer
        // sits BEHIND the popover panel (via `deferred()` ordering); the panel
        // uses `.occlude()` so pointer events on it never reach the backdrop.
        // Clicking anywhere else dispatches `self.close`, which clears the
        // popover's open-state key. Also gives every enum picker built via
        // `prop_view` free "select item → popover closes on outside click"
        // semantics without needing compound BActions.
        let handler = c.h.clone();
        let close_action = self.close.clone();
        // Deferred elements paint into the window-root overlay layer, so
        // `.absolute().inset_0().size_full()` covers the entire viewport
        // without any hardcoded geometry literals. Any pointer-down that
        // lands outside the popover panel (which uses `.occlude()`) hits
        // this backdrop and dispatches `close`.
        let backdrop = div()
            .id("popover-backdrop")
            .absolute()
            .inset_0()
            .size_full()
            .on_mouse_down(gpui::MouseButton::Left, move |_, w, cx| {
                (std::rc::Rc::clone(&handler))(&close_action, w, cx);
            });
        div()
            .relative()
            .child(walk(self.anchor, c))
            .child(deferred(backdrop))
            .child(deferred(panel))
            .into_any_element()
    }
}

impl BFocusedRenderer for BPopover<'_> {
    fn render(&self, c: &BRenderCtx) -> AnyElement {
        BPopover::render(self, c)
    }
}
