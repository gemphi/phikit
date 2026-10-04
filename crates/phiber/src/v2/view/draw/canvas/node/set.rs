//! What a node graph paints: each node's card, sized the way Blender sizes it,
//! with its sockets and the links between them.
//!
//! `mod.rs` holds the fit maths and where the graph comes from, `socket.rs` the
//! socket layout, `paint.rs` the curves and dots, `drag.rs` what moving a card
//! does.
//!
//! The split between div and paint is not arbitrary. A card is a rectangle with a
//! title, which is what a div is for. A link is a cubic bezier and a socket is a
//! circle straddling the card's border — neither is a box, so both are painted.
//!
//! The card's height is `socket::body_height`, not its content's: in Blender the
//! socket layout IS the height calculation (`node_update_basis`), so a card sized
//! any other way would hold its own sockets only by luck.

use super::super::super::color::to_hsla;
use super::super::super::ctx::BRenderCtx;
use super::{BExtent, BNode, drag, draw_order, header_color, link, paint, plan::plan, socket, wire};

use crate::phiphx::blend::shade::graph::BlendNode;
use crate::v2::state::{BKey as _, NodeKey};
use crate::v2::theme::BStyleRole;
use crate::v2::view::draw::flex::{BFlex, BPlace};
use crate::v2::view::prim::BAction;
use gpui::prelude::*;
use gpui::{AnyElement, ClickEvent, SharedString, canvas, div, px};
use std::rc::Rc;

/// One node's card, at the place it is actually drawn.
///
/// `from` is the displacement already stored for this node, carried into the
/// gesture so a second drag continues from where the first left off instead of
/// snapping back to the fitted position.
fn card(
    node: &BlendNode,
    at: (f32, f32),
    from: (f32, f32),
    selected: bool,
    ctx: &BRenderCtx,
) -> AnyElement {
    let id = node.name.clone();
    let unit = ctx.style.dimensions.row_height;
    let mut card = div()
        .flex()
        .flex_col()
        .w(px(ctx.style.dimensions.channels_width))
        .rounded(px(ctx.style.radius_for(BStyleRole::Card)))
        .bg(to_hsla(ctx.style.colors.bg_panel))
        .border(px(ctx.style.strokes.border))
        .border_color(to_hsla(ctx.style.colors.border))
        .overflow_hidden()
        .child(
            div()
                .h(px(ctx.style.dimensions.header_height))
                .px(px(ctx.style.spacing.pad_sm))
                .flex()
                .items_center()
                .bg(to_hsla(header_color(node, ctx)))
                .text_color(to_hsla(ctx.style.colors.text))
                .text_size(px(ctx.style.typography.header))
                .child(SharedString::from(node.title().to_string())),
        )
        // The body is empty on purpose: everything in it — the socket dots and
        // their names — is painted over the canvas, because a socket sits ON
        // the card's border and a div cannot be drawn half outside its parent.
        // What the body contributes is its HEIGHT, which is Blender's.
        .child(div().h(px(socket::body_height(node) * unit)));
    if selected {
        card = card
            .border(px(ctx.style.strokes.border * 2.0))
            .border_color(to_hsla(ctx.style.colors.accent));
    }
    let handler = ctx.handler.clone();
    let action = BAction::Set(NodeKey::Selected(ctx.host).path(), id.clone());
    let placed = div()
        .id(SharedString::from(format!("node-{id}")))
        .absolute()
        .left(gpui::relative(at.0))
        .top(gpui::relative(at.1))
        .child(card)
        .on_click(move |_e: &ClickEvent, w, cx| {
            (Rc::clone(&handler))(&action, w, cx);
        });
    drag::grab(placed, drag::BNodeDrag::new(crate::v2::state::intern(&id), from)).into_any_element()
}

pub fn render(ctx: &BRenderCtx) -> AnyElement {
    let selected = ctx.state.get(&NodeKey::Selected(ctx.host).path()).unwrap_or("").to_string();
    let grid = ctx.style.node_grid_stroke();
    // The canvas, with the theme's sweep when it has one.
    let bg = super::super::super::color::resolved_gradient(ctx.style.viewport_gradient());
    let mut canvas_div = super::super::base(ctx).bg(bg);
    // Fractions are canvas coordinates, not design-system spacing.
    for fraction in [0.125_f32, 0.25, 0.375, 0.5, 0.625, 0.75, 0.875] {
        canvas_div = canvas_div
            .child(
                div()
                    .absolute()
                    .top_0()
                    .h_full()
                    .w(px(grid.width))
                    .left(gpui::relative(fraction))
                    .bg(to_hsla(grid.color)),
            )
            .child(
                div()
                    .absolute()
                    .left_0()
                    .w_full()
                    .h(px(grid.width))
                    .top(gpui::relative(fraction))
                    .bg(to_hsla(grid.color)),
            );
    }

    // No file, or a material with no tree: the canvas and its grid, and
    // nothing invented on top of them.
    let Some(graph) = BNode::graph(ctx) else {
        return canvas_div.into_any_element();
    };
    let extent = BExtent::of(&graph.nodes);
    // Two tables, because the drag needs both: where the FIT puts each node,
    // which is what a displacement is measured from and what the clamp is
    // relative to, and where it is drawn once its displacement is added. The
    // extent is deliberately computed from the file's positions alone — adding
    // displacements back into it would reframe the graph under the pointer
    // while a card is being dragged (`drag.rs`).
    let fitted: Vec<(String, (f32, f32))> = graph
        .nodes
        .iter()
        .map(|n| {
            let p = extent.place(n);
            (n.name.clone(), (p.x, p.y))
        })
        .collect();
    let offsets: Vec<(String, (f32, f32))> = fitted
        .iter()
        .map(|(name, _)| (name.clone(), drag::offset_of(ctx.state, ctx.host, name)))
        .collect();
    let placed: Vec<(String, (f32, f32))> = fitted
        .iter()
        .zip(offsets.iter())
        .map(|((name, base), (_, from))| (name.clone(), drag::placed(*base, *from)))
        .collect();

    // The sockets and the links, painted under the cards: a socket straddles a
    // card's border, so the card's own fill has to go over it.
    let ink = paint::BNodeInk {
        wire: ctx.style.colors.text_dim,
        // `TH_WIRE` is the node editor's own `back` in Blender's factory
        // theme, so the canvas shade is the faithful outline here.
        wire_shadow: ctx.style.colors.bg_viewport,
        label: ctx.style.colors.text_dim,
        outline: ctx.style.colors.border,
        ring: ctx.style.strokes.border,
    };
    let (header, unit) = (ctx.style.dimensions.header_height, ctx.style.dimensions.row_height);
    let card_width = ctx.style.dimensions.channels_width;
    // The file's links with this host's edits applied — a made or broken link
    // is an overlay, not a change to the file (`link/mod.rs`).
    let edits = link::edits_of(ctx.state, ctx.host);
    let carrying = link::linking_of(ctx.state, ctx.host);
    let mut wired = edits.apply(&graph.links);
    // A link being carried is drawn following the cursor, so its settled
    // version must not also be drawn — otherwise the old wire stays put while
    // its copy moves.
    if let Some(drag) = &carrying {
        if let Some((from, to)) = &drag.moving {
            wired.retain(|l| {
                !(l.from == from.node
                    && l.from_socket == from.socket
                    && l.to == to.node
                    && l.to_socket == to.socket)
            });
        }
    }
    let painted = (graph.clone(), placed.clone(), wired.clone(), carrying.clone());
    // The canvas's own rectangle, published by the paint pass for the gesture
    // to measure the pointer against (`canvas/time/gesture.rs` does the same).
    let painted_bounds: std::rc::Rc<std::cell::Cell<Option<gpui::Bounds<gpui::Pixels>>>> =
        std::rc::Rc::new(std::cell::Cell::new(None));
    let publish = std::rc::Rc::clone(&painted_bounds);
    let painter = canvas(
        |_, _, _| (),
        move |bounds, (), window, cx| {
            publish.set(Some(bounds));
            let (w, h) = (f32::from(bounds.size.width), f32::from(bounds.size.height));
            let (cards, mut links) =
                plan(&painted.0, &painted.2, &painted.1, w, h, header, unit, card_width);
            // The carried link, from its anchored socket to the cursor.
            if let Some(drag) = &painted.3 {
                let anchor = cards
                    .iter()
                    .find(|card| card.name == drag.anchor.node)
                    .and_then(|card| {
                        socket::row_of_card(card, &drag.anchor.socket)
                            .map(|row| card.socket_at(&row))
                    });
                if let Some(anchor) = anchor {
                    links.push(drag.preview((drag.at.0 * w, drag.at.1 * h), anchor));
                }
            }
            paint::paint(&cards, &links, &ink, bounds, window, cx);
        },
    );
    canvas_div =
        canvas_div.child(BFlex::column().place(BPlace::Overlay).styled(painter, ctx));

    // Selected last, so the one being dragged is over its neighbours
    // (`draw_order`, Blender's `compare_node_depth`).
    for node in draw_order(&graph.nodes, &selected) {
        let Some((_, at)) = placed.iter().find(|(n, _)| *n == node.name) else { continue };
        let from = offsets.iter().find(|(n, _)| *n == node.name).map(|(_, o)| *o).unwrap_or((0.0, 0.0));
        // The editor's own selection wins; the file's `NODE_SELECT` is what
        // it opens on.
        let is = if selected.is_empty() { node.selected() } else { selected == node.name };
        canvas_div = canvas_div.child(card(node, *at, from, is, ctx));
    }
    let wire = std::rc::Rc::new(wire::BWire {
        graph: graph.clone(),
        wired,
        placed,
        edits,
        drag: carrying,
        handler: ctx.handler.clone(),
        host: ctx.host,
        bounds: std::rc::Rc::clone(&painted_bounds),
        header,
        unit,
        card_width,
    });
    wire::listen(drag::follow(canvas_div, fitted, ctx), Some(wire), ctx).into_any_element()
}
