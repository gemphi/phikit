//! Single-view popover renderer. The anchor stays in flow and the open body
//! is appended as a deferred semantic overlay.
//!
//! ## File layout (preserves source-text characterization test splits)
//!
//! 1. `fn switcher_columns` — two transparent columns for the host switcher
//! 2. `pub fn view(` — public compatibility wrapper (test split sentinel)
//! 3. `fn render` — GPUI builder bodies: semantic surface, border_1(), overflow_hidden(),
//!    compatible_inner_radius, shadow_xl, deferred overlay

use super::color::{resolved_gradient, to_hsla};
use super::ctx::BRenderCtx;
use super::surface::compatible_inner_radius;
use super::walk;
pub mod place;

use crate::v2::theme::{BElevation, BStyleRole, BSurfaceTone};
use crate::v2::view::draw::flex::shadowed;
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
    /// Where this popover opens. The side it was declared with is only a
    /// preference: `place::BPlacement` turns it into a point on the anchor and
    /// a panel corner, and flips that corner when the window has no room —
    /// which is why a trigger at a host's right edge no longer clips.
    fn placement(&self, ctx: &BRenderCtx) -> place::BPlacement {
        place::BPlacement::of(self.side, ctx)
    }

    /// Host-switcher popover body: two transparent columns (categories | hosts).
    /// The inner columns must remain transparent — the popover surface owns all paint.
    fn switcher_columns(&self, ctx: &BRenderCtx) -> Option<Div> {
        let [BView::Row { list }] = self.body else {
            return None;
        };
        let [
            BView::Column {
                list: categories,
            },
            BView::Column { list: hosts },
        ] = list.as_slice()
        else {
            return None;
        };
        let mut left = div()
            .flex()
            .flex_col()
            .flex_1()
            .min_w_0()
            .gap(px(ctx.style.spacing.gap_xs))
            .pr(px(ctx.style.spacing.pad_xs))
            .border_r(px(ctx.style.strokes.border))
            .border_color(to_hsla(ctx.style.colors.sep));
        for item in categories {
            left = left.child(walk(item, ctx));
        }
        let mut right = div()
            .flex()
            .flex_col()
            .flex_1()
            .min_w_0()
            .gap(px(ctx.style.spacing.gap_xs))
            .pl(px(ctx.style.spacing.pad_xs));
        for item in hosts {
            right = right.child(walk(item, ctx));
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
    ctx: &BRenderCtx,
) -> AnyElement {
    BPopover {
        anchor,
        body,
        open,
        close,
        side,
    }
    .render(ctx)
}

// ── Focused renderer impl ─────────────────────────────────────────────────
// All GPUI builder bodies are here. Semantic style: border_1() (single-pixel
// border), compatible_inner_radius(Popover), overflow_hidden(), the theme's
// surface shadow. The outer panel owns all surface paint.

impl BPopover<'_> {
    pub fn render(&self, ctx: &BRenderCtx) -> AnyElement {
        if !self.open {
            return walk(self.anchor, ctx);
        }
        // BelowEnd (host switcher) needs its explicit two-panel-wide picker;
        // Below / Right adopt the anchor width so the dropdown reads as a
        // continuation of the button that opened it (card/section behavior).
        // Every popover frames its rows the same way the toolbar and sidebar
        // frame theirs: one gutter on all four sides, so a row's hover fill
        // stops short of the panel edge instead of running into the border
        // radius. The two-column host switcher (BelowEnd) only differs in the
        // gap it leaves between rows.
        let two_column = self.side == BPopoverSide::BelowEnd;
        let frame = ctx.style.spacing.pad_xs;
        let row_gap = if two_column {
            ctx.style.spacing.gap_xs
        } else {
            ctx.style.spacing.gap_none
        };
        let mut content = div()
            .id(gpui::SharedString::from("popover-content"))
            .flex()
            .flex_col()
            .w_full()
            .min_h_0()
            .overflow_y_scroll()
            .p(px(frame))
            .gap(px(row_gap));
        if two_column {
            if let Some(columns) = self.switcher_columns(ctx) {
                content = content.child(columns);
            } else {
                for item in self.body {
                    content = content.child(walk(item, ctx));
                }
            }
        } else if self.body.is_empty() {
            content = content.min_h(px(ctx.style.dimensions.row_height));
        } else {
            for item in self.body {
                content = content.child(walk(item, ctx));
            }
        }
        let panel_base = div().flex().flex_col();
        let panel_sized = match self.side {
            BPopoverSide::BelowEnd => {
                // Host switcher: two columns when there is room for them, one
                // panel wide when there is not, so a narrow area still gets a
                // picker that fits inside it.
                panel_base
                    .min_w(px(ctx.style.dimensions.panel_width))
                    .max_w(px(ctx.style.dimensions.panel_width * 2.0))
            }
            _ => {
                // Every dropdown is at least one panel wide, so the menus in a
                // strip share a width instead of each one sizing to its own
                // longest label. It still adopts the anchor's width when that
                // is wider, and content may grow it further.
                panel_base
                    .min_w_full()
                    .min_w(px(ctx.style.dimensions.panel_width))
            }
        };
        let panel = panel_sized
            .id(gpui::SharedString::from("popover-panel"))
            .debug_selector(|| "popover-panel".to_string())
            .bg(resolved_gradient(ctx.style.surface(BSurfaceTone::Card)))
            .text_color(to_hsla(ctx.style.colors.text))
            .border(px(ctx.style.strokes.border))
            .border_color(to_hsla(ctx.style.colors.border))
            .rounded(px(compatible_inner_radius(
                ctx.style.radius_for(BStyleRole::Popover),
                ctx.style.strokes.border,
            )))
            // A picker taller than the area scrolls inside its own panel
            // instead of running past the bottom of the host.
            .max_h(px(ctx.style.dimensions.panel_width * 2.0))
            .overflow_hidden()
            .occlude()
            .child(content);
        let panel = shadowed(panel, ctx.style.shadow_for(BElevation::Overlay));
        // Click-outside-to-close backdrop. A full-viewport transparent layer
        // sits BEHIND the popover panel (via `deferred()` ordering); the panel
        // uses `.occlude()` so pointer events on it never reach the backdrop.
        // Clicking anywhere else dispatches `self.close`, which clears the
        // popover's open-state key. Also gives every enum picker built via
        // `prop_view` free "select item → popover closes on outside click"
        // semantics without needing compound BActions.
        let handler = ctx.handler.clone();
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
        let placement = self.placement(ctx);
        let opened = placement
            .origin()
            .child(placement.anchor(panel.into_any_element()));
        div()
            .relative()
            .child(walk(self.anchor, ctx))
            .child(deferred(backdrop))
            .child(deferred(opened))
            .into_any_element()
    }
}
