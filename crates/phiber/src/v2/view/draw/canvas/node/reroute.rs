//! A reroute's one element: the square it is clicked and dragged by.
//!
//! Ported from Blender 5.2.0 `reroute_node_draw`. A reroute has no card, no
//! header, no title and no body — `reroute_node_prepare_for_draw` shrinks it to
//! a `NODE_SOCKSIZE`-radius box centred on the node's location, and the only
//! thing painted is a single socket dot (`reroute_node_draw_body`).
//!
//! That dot is not drawn here. A reroute's one input and one output are both at
//! its location (`socket::reroute_rows`), so the socket pass paints the dot for
//! free, the same way it paints every other socket. What a reroute still needs,
//! and a card would have given it, is somewhere to CLICK: the card is the
//! element that carries select-on-click and the drag, and a reroute has none.
//! This is that element — a transparent square over the dot, centred on the
//! location, so a reroute selects and drags exactly as a card does.
//!
//! The square is the dot's own size (`loc ± DOT`), which is the reroute's whole
//! `draw_bounds`: a reroute is a small target in Blender too.

use super::super::super::ctx::BRenderCtx;
use super::plan::BCardSize;
use super::view::BNodeView;
use super::{drag, socket};

use crate::phiphx::blend::shade::graph::BlendNode;
use crate::v2::state::{BKey as _, NodeKey};
use crate::v2::view::draw::color::to_hsla;
use crate::v2::view::prim::BAction;
use gpui::prelude::*;
use gpui::{AnyElement, ClickEvent, SharedString, div, px};
use std::rc::Rc;

/// The transparent hit square a reroute is selected and dragged by, centred on
/// its location.
///
/// `at` is the location as a fraction of the canvas (where the dot is painted);
/// the square's half-extent is the dot's radius in those same fractions —
/// `DOT` widget units scaled to view units by `size.unit` and divided by the
/// view's extent, which is what turns a view-unit size into the fraction a div
/// is placed with.
pub fn handle(
    node: &BlendNode,
    at: (f32, f32),
    size: BCardSize,
    view: &BNodeView,
    from: (f32, f32),
    selected: bool,
    ctx: &BRenderCtx,
) -> AnyElement {
    let id = node.name.clone();
    let half = (
        socket::DOT * size.unit / view.width(),
        socket::DOT * size.unit / view.height(),
    );
    let mut square = div()
        .id(SharedString::from(format!("node-{id}")))
        .absolute()
        .left(gpui::relative(at.0 - half.0))
        .top(gpui::relative(at.1 - half.1))
        .w(gpui::relative(half.0 * 2.0))
        .h(gpui::relative(half.1 * 2.0))
        .rounded_full();
    if selected {
        // Blender draws a selected reroute's socket with the selection outline;
        // a ring in the accent round the dot is the readable stand-in here.
        square = square
            .border(px(ctx.style.strokes.border * 2.0))
            .border_color(to_hsla(ctx.style.colors.accent));
    }
    let handler = ctx.handler.clone();
    let action = BAction::Set(NodeKey::Selected(ctx.host).path(), id.clone());
    let clickable = square.on_click(move |_e: &ClickEvent, w, cx| {
        (Rc::clone(&handler))(&action, w, cx);
    });
    drag::grab(clickable, drag::BNodeDrag::new(crate::v2::state::intern(&id), from)).into_any_element()
}
