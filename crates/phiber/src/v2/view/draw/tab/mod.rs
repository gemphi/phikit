//! Tab atom — strip button that blends with the surrounding pane on
//! active/hover, producing Blender's "paper tab" continuation effect on the
//! Properties navbar, the view3d sidebar and the top space bar.
//!
//! Geometry: a horizontal tab is a cap cut into its strip, rounded on the
//! TOP corners (`rounded_t`) and running flush into the pane below. A rail
//! entry is not a cap: in every rail — the toolbar, the sidebar, the
//! Properties and Preferences navbars, docked or floating — it is the same
//! square tile on its resting ground, lit with the accent when active
//! (`rail`). One kind, so a change to it lands in every host at once.

use super::color::{resolved_gradient, to_hsla};
use super::ctx::BRenderCtx;
use super::chrome;
use super::atom::icon_path;
use super::flex::{BFlex, BFlexed, BLen, BOverflow};

pub mod cap;
mod shape;
use self::shape::BCapShape;
use self::cap::{BCapSide, BTabTooltip};
use crate::v2::theme::{BControlState, BStyleRole};
use crate::v2::view::prim::BControl;
use gpui::prelude::*;
use gpui::{AnyElement, ClickEvent, SharedString, px, svg};
use std::rc::Rc;

/// One tab renderer over the shared control record: `tooltip` (when set)
/// rides the strip as a hover tooltip — used by icon-only rail tabs and the
/// activity bar where the inline label is hidden.
/// A horizontal tab, whose pane continues below it.
pub fn view(ctl: &BControl, ctx: &BRenderCtx) -> AnyElement {
    cap_view(ctl, BCapSide::Below, ctx)
}

/// A rail entry: the square tile every rail draws, docked or floating. It
/// always rests on its tile (`chrome::rests`, through the floating paint), so
/// an icon never sits on nothing — the viewport's strips are the reference.
pub fn rail(ctl: &BControl, ctx: &BRenderCtx) -> AnyElement {
    cap_view(ctl, BCapSide::Free, &ctx.floating())
}

fn cap_view(ctl: &BControl, side: BCapSide, ctx: &BRenderCtx) -> AnyElement {
    let ic = ctl.icon.as_deref().unwrap_or("");
    let lbl = ctl.label.as_deref();
    let active = ctl.active;
    // Which of the six caps this is, asked once (`shape.rs`) instead of
    // re-derived from four booleans at every geometry decision.
    let shape = BCapShape::of(side, lbl.is_some());
    let strip_tab = chrome::in_strip(ctx) && shape.is_strip_tab();
    let text = if active { ctx.style.colors.text } else { ctx.style.colors.text_dim };
    let hover_fill = to_hsla(ctx.style.tab_hover(ctx.host));
    let r = px(ctx.style.radius_for(BStyleRole::Tab));

    let mut row = shape.row(ctx).div(ctx).text_color(to_hsla(text)).text_size(px(ctx.style.typography.body));

    if active {
        // An icon-only rail entry (the activity bar, the toolbar, a navbar)
        // marks its active state the way every control does: the accent at
        // the active opacity, one hue for the whole family of controls
        // (`style.control_fill`). A labelled tab keeps its filled cap, which
        // is what joins it to the pane beneath.
        if shape.icon_only() {
            let on = ctx
                .style
                .control_fill(BControlState::Active, ctx.floating)
                .unwrap_or(ctx.style.colors.accent);
            row = row.bg(to_hsla(on));
        } else {
            let active_fill = ctx.style.tab_active_gradient(ctx.host).along(side.gradient_axis());
            row = row.bg(resolved_gradient(active_fill));
        }
        // Every tab uses fill-only selection — no accent border strip. The
        // active cap and the surface beneath (popover / pane body) form one
        // continuous filled surface, so a hard line at the seam would break
        // the paper-tab illusion.
        row = side.round(row, r);
    } else if strip_tab {
        // Resting fill and hover lift are applied to the cap below, so the
        // paint stops at the rounded corners instead of over-painting them.
        row = side.round(row, r);
    }

    if !ic.is_empty() {
        let icon_id_text = format!("tab-icon-{}-{}", ic, lbl.unwrap_or(ic));
        row = row.child(
            BCapPart::Icon
                .flex(ctx)
                .div(ctx)
                .id(SharedString::from(icon_id_text.clone()))
                .debug_selector(move || icon_id_text)
                .child(
                    svg()
                        .path(icon_path(ic))
                        .size(px(ctx.style.dimensions.icon_size))
                        // An icon takes the icon shade, not the text's.
                        .text_color(to_hsla(ctx.style.icon_ink(active))),
                ),
        );
    }
    if let Some(t) = lbl {
        row = row.child(SharedString::from(t.to_string()));
    }

    let id_text = format!("tab-{}-{}", ic, lbl.unwrap_or(ic));
    let content: AnyElement = row.into_any_element();

    let Some(a) = ctl.action.as_ref() else {
        return content;
    };
    let handler = ctx.handler.clone();
    let action = a.clone();
    let id = SharedString::from(id_text.clone());
    // The outer clickable owns the round-cap shape and hard-clips its
    // children so a hover fill can never bleed past the corners. Without the
    // clip the hover paint drew a full rectangle on top of the rounded cap
    // ("squarish on hover").
    let mut element = shape
        .seat(ctx, BFlex::row().overflow(BOverflow::Clip))
        .div(ctx)
        .id(id)
        .debug_selector(|| id_text)
        .cursor_pointer()
        .child(content);
    // The gap sits on the cap's far edge, so its pane-facing edge is the
    // strip's and the pane beyond continues the same surface.
    element = side.gap(element);
    element = side.round(element, r);
    if !active {
        // Every control's rest / hover fill comes from one mapping
        // (`chrome::paint`): the accent at an opacity. Docked rails paint
        // nothing until hovered; floating chrome rests on the faint step so
        // it stays legible over the canvas.
        element = if strip_tab || ctx.floating || shape.icon_only() {
            chrome::paint(element, false, ctx)
        } else {
            element.hover(move |s| s.bg(hover_fill))
        };
    }
    // Tooltip must be attached before on_click (which consumes el by value).
    if let Some(tip) = ctl.tooltip.as_deref() {
        let tip = tip.to_string();
        let style = *ctx.style;
        element = element.tooltip(move |_, cx| cx.new(|_| BTabTooltip::of(&tip, &style)).into());
    }
    element
        .on_click(move |_e: &ClickEvent, w: &mut gpui::Window, cx: &mut gpui::App| {
            (Rc::clone(&handler))(&action, w, cx);
        })
        .into_any_element()
}

/// The parts a cap is built from — one enum for the level, as every other
/// level states its parts (`BCardPart`, `BModalPart`, `BTablePart`). Which
/// *kind* of cap this is, is `BCapShape` (`tab/shape.rs`).
enum BCapPart {
    /// The icon cell: one icon size, never shrinking.
    Icon,
}

impl BFlexed for BCapPart {
    fn flex(&self, ctx: &BRenderCtx) -> BFlex {
        match self {
            Self::Icon => {
                let size = BLen::Measured(ctx.style.dimensions.icon_size);
                BFlex::row().width(size).height(size).keeps_size()
            }
        }
    }
}
