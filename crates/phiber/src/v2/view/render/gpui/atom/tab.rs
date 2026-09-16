//! Tab atom — strip button that blends with the surrounding pane on
//! active/hover, producing Blender's "paper tab" continuation effect on the
//! Properties navbar, the view3d sidebar and the top space bar.
//!
//! Geometry: a tab is a cap cut into the strip, so only its *leading* corners
//! are rounded and the trailing edge runs flush into the pane —
//!   • horizontal (labeled) tabs round the TOP corners (`rounded_t`),
//!   • vertical icon-only navbar tabs round the LEFT corners (`rounded_l`).
//! Active tabs always receive the stronger cap. Inactive editor tabs remain
//! transparent until hover; inactive Topbar workspace tabs rest on the light
//! host-matching fill and promote to the stronger active background on hover.

use super::super::{
    color::{resolved_gradient, to_hsla},
    ctx::BRenderCtx,
};
use super::icon_path;
use crate::v2::host::BHostType;
use crate::v2::theme::BStyleRole;
use crate::v2::view::prim::{BAction, BControl};
use gpui::prelude::*;
use gpui::{AnyElement, ClickEvent, Context, Hsla, SharedString, Window, div, px, svg};
use std::rc::Rc;

/// Minimal tooltip view. Style values are captured from `BRenderCtx` at the
/// call site so no hardcoded design literals appear in this file.
struct BTabTooltip {
    text: SharedString,
    pad_x: f32,
    pad_y: f32,
    radius: f32,
    bg: Hsla,
    fg: Hsla,
    size: f32,
}

impl Render for BTabTooltip {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .px(px(self.pad_x))
            .py(px(self.pad_y))
            .rounded(px(self.radius))
            .bg(self.bg)
            .text_color(self.fg)
            .text_size(px(self.size))
            .child(self.text.clone())
    }
}

/// One tab renderer over the shared control record: `tooltip` (when set)
/// rides the strip as a hover tooltip — used by icon-only rail tabs and the
/// activity bar where the inline label is hidden.
pub fn view(ctl: &BControl, c: &BRenderCtx) -> AnyElement {
    let ic = ctl.icon.as_deref().unwrap_or("");
    let lbl = ctl.label.as_deref();
    let active = ctl.active;
    let action = ctl.action.as_ref();
    let tooltip = ctl.tooltip.as_deref();
    // Icon-only tabs (no label) are vertical navbar tabs. Properties gives
    // their paint a separate absolute layer so state never moves the icon.
    let icon_only = lbl.is_none();
    let properties_rail_tab = c.host == BHostType::Buts && icon_only;
    // Only labeled workspace tabs in the Topbar use the elevated resting
    // treatment. Editor-host tabs retain their transparent-rest/hover model.
    let topbar_main_tab = c.host == BHostType::Topbar && !icon_only;
    let text = if active {
        to_hsla(c.style.colors.text)
    } else {
        to_hsla(c.style.colors.text_dim)
    };
    let hover_fill = if icon_only && c.host != BHostType::Buts {
        // Activity bar and non-Properties icon rails: lift from the soft strip.
        to_hsla(c.style.rail_tab_hover(c.host))
    } else {
        to_hsla(c.style.tab_hover(c.host))
    };
    let r = px(c.style.radius_for(BStyleRole::Tab));

    let mut row = div()
        .flex()
        .flex_row()
        .items_center()
        .gap(px(c.style.spacing.gap_sm))
        .h(px(c.style.dimensions.button_height))
        .text_color(text)
        .text_size(px(c.style.typography.body));
    row = if icon_only {
        row.w_full().justify_center()
    } else {
        row.px(px(c.style.spacing.pad_md))
    };

    // Non-Properties tabs retain their existing state styling. Properties
    // rail tabs paint on a stable sibling layer assembled below.
    if active && !properties_rail_tab {
        // Icon-only rails (activity bar, non-Buts navbars) use a parent-tonal
        // lift instead of the accent-tinted host_item_active gradient — no
        // "foreign blue" on chrome rails.
        let active_fill = if icon_only {
            c.style.rail_tab_active(c.host)
        } else {
            c.style.tab_active_gradient(c.host)
        };
        row = row.bg(resolved_gradient(active_fill));
        // Every tab uses fill-only selection — no accent border strip.
        // The active cap and the surface beneath (popover / pane body)
        // form one continuous filled surface, so a hard line at the seam
        // would break the paper-tab illusion.
        if icon_only {
            row = row.rounded_l(r);
        } else {
            row = row.rounded_t(r);
        }
    } else if topbar_main_tab {
        let active_fill = resolved_gradient(c.style.tab_active_gradient(c.host));
        row = row
            .bg(hover_fill)
            .rounded_t(r)
            .hover(move |s| s.bg(active_fill.clone()).rounded_t(r));
    }

    if !ic.is_empty() {
        let icon_id_text = format!("tab-icon-{}-{}", ic, lbl.unwrap_or(ic));
        row = row.child(
            div()
                .id(SharedString::from(icon_id_text.clone()))
                .debug_selector(move || icon_id_text.clone())
                .size(px(c.style.typography.header))
                .flex_shrink_0()
                .child(
                    svg()
                        .path(icon_path(ic))
                        .size(px(c.style.typography.header))
                        .text_color(text),
                ),
        );
    }
    if let Some(t) = lbl {
        row = row.child(SharedString::from(t.to_string()));
    }

    let id_text = format!("tab-{}-{}", ic, lbl.unwrap_or(ic));
    let content: AnyElement = if properties_rail_tab {
        let group = SharedString::from(format!("{id_text}-paint-hover"));
        let paint_id_text = format!("{id_text}-background");
        let mut paint = div()
            .id(SharedString::from(paint_id_text.clone()))
            .debug_selector(move || paint_id_text.clone())
            .absolute()
            .top_0()
            .bottom_0()
            .left(px(c.style.properties_tab_background_inset()))
            .right(px(-c.style.properties_tab_overlap()))
            // Round only the left corners — the right edge bleeds into the
            // content region, so it must be flush (no right-side radius).
            .rounded_l(r);
        paint = if active {
            paint.bg(resolved_gradient(c.style.tab_active_gradient(c.host)))
        } else {
            paint.group_hover(group.clone(), move |style| style.bg(hover_fill))
        };
        div()
            .group(group)
            .relative()
            .w_full()
            .h(px(c.style.dimensions.button_height))
            .child(paint)
            .child(row)
            .into_any_element()
    } else {
        row.into_any_element()
    };

    let Some(a) = action else {
        return content;
    };
    let handler = c.h.clone();
    let action = a.clone();
    let id = SharedString::from(id_text.clone());
    // The outer clickable owns the round-cap shape and hard-clips its
    // children so a hover fill can never bleed past the corners. Without
    // overflow_hidden the hover paint drew a full rectangle on top of the
    // rounded cap ("squarish on hover" bug).
    let cap_round = r;
    let mut el = div()
        .id(id)
        .debug_selector(|| id_text)
        .cursor_pointer()
        .overflow_hidden()
        .child(content);
    el = if icon_only {
        el.rounded_l(cap_round)
    } else {
        el.rounded_t(cap_round)
    };
    // Vertical rail tabs span the full rail width. Properties paint may
    // overlap the pane, but tab allocation and centered icon stay unchanged.
    if icon_only {
        el = el.w_full();
    }
    if !properties_rail_tab && !active && !topbar_main_tab {
        el = el.hover(move |s| s.bg(hover_fill));
    }
    // Tooltip must be attached before on_click (which consumes el by value).
    if let Some(tip) = tooltip {
        let tip = SharedString::from(tip.to_string());
        let pad_x = c.style.spacing.pad_sm;
        let pad_y = c.style.spacing.pad_xs;
        let radius = c.style.radius_for(BStyleRole::Button);
        let bg = to_hsla(c.style.colors.bg_app);
        let fg = to_hsla(c.style.colors.text);
        let size = c.style.typography.body;
        el = el.tooltip(move |_, cx| {
            cx.new(|_| BTabTooltip {
                text: tip.clone(),
                pad_x,
                pad_y,
                radius,
                bg,
                fg,
                size,
            })
            .into()
        });
    }
    el.on_click(
        move |_e: &ClickEvent, w: &mut gpui::Window, cx: &mut gpui::App| {
            (Rc::clone(&handler))(&action, w, cx);
        },
    )
    .into_any_element()
}
