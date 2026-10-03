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

use super::color::{resolved_gradient, to_hsla};
use super::ctx::BRenderCtx;
use super::chrome;
use super::atom::icon_path;
use super::flex::{BFill, BFlex, BFlexed, BLen, BOverflow, BPlace};

pub mod cap;
mod shape;
mod stack;
use self::shape::BCapShape;
pub use self::stack::rail_stack;
use self::cap::{BCapSide, BTabTooltip};
use crate::v2::host::BHostType;
use crate::v2::theme::{BControlState, BStyleRole};
use crate::v2::view::prim::{BControl, BRailSide};
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

/// A rail tab, whose pane continues to the side the rail is pinned opposite.
pub fn rail(ctl: &BControl, side: BRailSide, ctx: &BRenderCtx) -> AnyElement {
    // A floating strip caps into nothing, so its entries are buttons.
    let side = if ctx.floating {
        BCapSide::Free
    } else {
        BCapSide::from(side)
    };
    cap_view(ctl, side, ctx)
}

/// A rail entry that is a free button — the activity bar's: it caps into no
/// pane, rounds fully and centres in its rail, but it is docked chrome, so it
/// rests on nothing and takes the accent only on hover and when active.
pub fn rail_button(ctl: &BControl, ctx: &BRenderCtx) -> AnyElement {
    cap_view(ctl, BCapSide::Free, ctx)
}

fn cap_view(ctl: &BControl, side: BCapSide, ctx: &BRenderCtx) -> AnyElement {
    let ic = ctl.icon.as_deref().unwrap_or("");
    let lbl = ctl.label.as_deref();
    let active = ctl.active;
    // Which of the six caps this is, asked once (`shape.rs`) instead of
    // re-derived from four booleans at every geometry decision.
    let shape = BCapShape::of(side, lbl.is_some(), ctx.host);
    let strip_tab = chrome::in_strip(ctx) && shape.is_strip_tab();
    let text = if active { ctx.style.colors.text } else { ctx.style.colors.text_dim };
    let hover_fill = if shape.icon_only() && ctx.host != BHostType::Buts {
        // Activity bar and non-Properties icon rails: lift from the soft strip.
        to_hsla(ctx.style.rail_tab_hover(ctx.host))
    } else {
        to_hsla(ctx.style.tab_hover(ctx.host))
    };
    let r = px(ctx.style.radius_for(BStyleRole::Tab));

    let mut row = shape.row(ctx).div(ctx).text_color(to_hsla(text)).text_size(px(ctx.style.typography.body));

    // Non-Properties tabs paint on the row. Properties rail tabs paint on a
    // stable sibling layer assembled below.
    if active && !shape.paints_separately() {
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
                .debug_selector(move || icon_id_text.clone())
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
    let content: AnyElement = if shape.paints_separately() {
        properties_layers(row, &id_text, active, side, r, hover_fill, ctx)
    } else {
        row.into_any_element()
    };

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
    // strip's and the pane beyond continues the same surface. Properties rail
    // tabs are the exception: they carry the same inset in their paint layer
    // instead, so selection never moves the icon.
    if !shape.paints_separately() {
        element = side.gap(element, ctx.style.dimensions.menu_top_gap);
    }
    element = side.round(element, r);
    if !shape.paints_separately() && !active {
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

/// A Properties rail tab's two layers: the paint, inset on the far edge and
/// bleeding past the pane-facing one, and the row over it. Keeping them
/// separate is what stops selection from moving the icon.
#[allow(clippy::too_many_arguments)]
fn properties_layers(
    row: gpui::Div,
    id_text: &str,
    active: bool,
    side: BCapSide,
    r: gpui::Pixels,
    hover_fill: gpui::Hsla,
    ctx: &BRenderCtx,
) -> AnyElement {
    let group = SharedString::from(format!("{id_text}-paint-hover"));
    let paint_id_text = format!("{id_text}-background");
    let mut paint = BCapPart::Paint { resting: !active }
        .flex(ctx)
        .div(ctx)
        .id(SharedString::from(paint_id_text.clone()))
        .debug_selector(move || paint_id_text.clone());
    // The far edge is inset and rounded; the pane-facing edge bleeds into the
    // content region, so it stays flush.
    paint = side.paint(
        paint,
        ctx.style.properties_tab_background_inset(),
        ctx.style.properties_tab_overlap(),
    );
    paint = side.round(paint, r);
    paint = if active {
        // The selected Properties tab sweeps into the panel beside it, so the
        // rail and the content read as one area.
        paint.bg(resolved_gradient(ctx.style.tab_active_gradient(ctx.host).along(side.gradient_axis())))
    } else {
        paint.group_hover(group.clone(), move |style| style.bg(hover_fill))
    };
    BCapPart::Layers
        .flex(ctx)
        .div(ctx)
        .group(group)
        .child(paint)
        .child(row)
        .into_any_element()
}

/// The parts a cap is built from — one enum for the level, as every other
/// level states its parts (`BCardPart`, `BModalPart`, `BTablePart`). Which
/// *kind* of cap this is, is `BCapShape` (`tab/shape.rs`).
enum BCapPart {
    /// The icon cell: one icon size, never shrinking.
    Icon,
    /// The paint layer, spanning the cap top to bottom; its left and right
    /// come from `BCapSide::paint`.
    /// The paint layer. `resting` fills it with the tab's chip; the selected
    /// tab paints its own gradient over the top.
    Paint { resting: bool },
    /// The two-layer stack a Properties rail tab is: the paint, then the row.
    Layers,
}

impl BFlexed for BCapPart {
    fn flex(&self, ctx: &BRenderCtx) -> BFlex {
        match self {
            Self::Icon => {
                let size = BLen::Measured(ctx.style.dimensions.icon_size);
                BFlex::row().width(size).height(size).keeps_size()
            }
            Self::Paint { resting } => {
                let flex = BFlex::row().place(BPlace::Overlay);
                if *resting { flex.fill(BFill::RailTabResting) } else { flex }
            }
            Self::Layers => BFlex::row()
                .place(BPlace::Anchor)
                .width(BLen::Full)
                .height(BLen::Measured(ctx.style.dimensions.card_head_height())),
        }
    }
}
