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
use super::{BNode, add, drag, draw_order, frame, header_color, link, paint, reroute, select, title_ink, wire};
use super::plan::{BCardFraction, BCardSize, positions};
use super::view::store as view_store;

use crate::phiphx::blend::shade::graph::BlendNode;
use crate::v2::theme::BStyleRole;
use crate::v2::view::prim::BAction;

use crate::v2::state::{BKey as _, NodeKey};
use crate::v2::view::draw::flex::{BFlex, BPlace};
use gpui::prelude::*;
use gpui::{AnyElement, ClickEvent, SharedString, div, px};
use std::rc::Rc;

/// Below this share of the canvas's width a card stops showing its title.
///
/// Blender's `node_draw_basis` skips a node's label when it is too small to read
/// rather than shrinking the font with the card. The threshold there is a zoom;
/// here it is the card's own fraction of the canvas, which is the one measure of
/// "how small on screen" available before the canvas is measured. A card narrower
/// than a twentieth of the view has no room for a name anyway.
const LABEL_FLOOR: f32 = 0.05;

/// One node's card, at the place it is actually drawn.
///
/// `from` is the displacement already stored for this node, carried into the
/// gesture so a second drag continues from where the first left off instead of
/// snapping back.
///
/// Both the position AND the size are fractions of the canvas, because that is
/// the only thing expressible before the canvas is measured (`to_fraction`) —
/// and it is what makes the zoom real rather than cosmetic: zooming out shrinks
/// the card instead of only sliding it toward its neighbours.
///
/// What does NOT scale is the text, because `text_size` wants pixels and there
/// are none here. Blender stops drawing a node's label below a zoom threshold
/// rather than shrinking it to nothing, so this does the same, with the card's
/// own share of the canvas standing in for the zoom it cannot see
/// ([`LABEL_FLOOR`]).
fn card(
    node: &BlendNode,
    at: (f32, f32),
    size: BCardFraction,
    from: (f32, f32),
    selected: bool,
    ctx: &BRenderCtx,
) -> AnyElement {
    let id = node.name.clone();
    let mut card = div()
        .flex()
        .flex_col()
        .w(gpui::relative(size.width))
        .rounded(px(ctx.style.radius_for(BStyleRole::Card)))
        .bg(to_hsla(ctx.style.colors.bg_panel))
        .border(px(ctx.style.strokes.border))
        .border_color(to_hsla(ctx.style.colors.border))
        .overflow_hidden()
        .child(
            div()
                .h(gpui::relative(size.header))
                .px(px(ctx.style.spacing.pad_sm))
                .flex()
                .items_center()
                .bg(to_hsla(header_color(node, ctx)))
                // A muted node's title fades with its header (`BUT_INACTIVE`).
                .text_color(to_hsla(title_ink(node, ctx)))
                .text_size(px(ctx.style.typography.header))
                .when(size.width >= LABEL_FLOOR, |header| {
                    header.child(SharedString::from(node.title().to_string()))
                }),
        )
        // The body is empty on purpose: everything in it — the socket dots and
        // their names — is painted over the canvas, because a socket sits ON
        // the card's border and a div cannot be drawn half outside its parent.
        // What the body contributes is its HEIGHT, which is Blender's.
        .child(div().h(gpui::relative(size.body)));
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
    let selection = select::selection_of(ctx.state, ctx.host);
    let grid = ctx.style.node_grid_stroke();
    // The canvas with the theme's sweep; the grid at fractions of it.
    let bg = super::super::super::color::resolved_gradient(ctx.style.viewport_gradient());
    let mut canvas_div = super::super::base(ctx).bg(bg);
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
    // A card's size in view units, which is the theme's pixels read as node
    // space (`BCardSize`), and the rectangle this editor is looking at.
    let size = BCardSize {
        header: ctx.style.dimensions.header_height,
        unit: ctx.style.dimensions.row_height,
        width: ctx.style.dimensions.channels_width,
    };
    let view = view_store::view_of(ctx.state, ctx.host, &graph.nodes, |n| size.height_of(n), size.width);
    // Where every node sits: the file's own location, what it has been dragged
    // by, and the two added and framed by the view (`plan::positions`).
    let (located, offsets, placed) = positions(&graph, &view, ctx.state, ctx.host);
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
    // The file's links with this host's edits applied — a made or broken link
    // is an overlay, not a change to the file (`link/mod.rs`).
    let edits = link::edits_of(ctx.state, ctx.host);
    let carrying = link::linking_of(ctx.state, ctx.host);
    let mut wired = edits.apply(&graph.links);
    // A link-drag search's wire rides on its added node (`add::induced_links`).
    wired.extend(add::additions_of(ctx.state, ctx.host).induced_links(&graph.nodes));
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
    // Frames first, behind the links and cards: a frame is a backdrop for the
    // nodes parented into it (`frame::layer`, Blender's `node_draw_nodetree`).
    canvas_div = canvas_div.children(frame::layer(&graph.nodes, &view, size, &offsets, ctx));
    let (painter, painted_bounds) = paint::canvas::element(
        paint::canvas::BNodePainting {
            graph: graph.clone(),
            wired: wired.clone(),
            placed: placed.clone(),
            carrying: carrying.clone(),
            view,
            size,
            ink,
        },
        ctx,
    );
    canvas_div =
        canvas_div.child(BFlex::column().place(BPlace::Overlay).styled(painter, ctx));

    // Selected last, so the one being dragged is over its neighbours
    // (`draw_order`, Blender's `compare_node_depth`). A frame is not in this
    // pass at all: it was drawn first, behind the links (`frame::layer`).
    for node in draw_order(&graph.nodes, &selection) {
        if node.frame() { continue; }
        let Some((_, at)) = placed.iter().find(|(n, _)| *n == node.name) else { continue };
        let from = offsets.iter().find(|(n, _)| *n == node.name).map(|(_, o)| *o).unwrap_or((0.0, 0.0));
        // The editor's selection wins; the file's `NODE_SELECT` is what it opens
        // on. A reroute is no card — just a dot and a square to grab it by.
        let is = select::rank(node, &selection) > 0;
        let el = if node.reroute() {
            reroute::handle(node, *at, size, &view, from, is, ctx)
        } else {
            card(node, *at, BCardFraction::of(size, &view, node), from, is, ctx)
        };
        canvas_div = canvas_div.child(el);
    }
    let (wire, moving) = wire::for_frame(
        wire::BFrame {
            graph: &graph,
            wired,
            placed,
            edits,
            carrying,
            selection: &selection,
            located: &located,
            offsets: &offsets,
            bounds: painted_bounds,
            size,
            view,
        },
        ctx,
    );
    wire::listen(drag::follow(canvas_div, moving, view, ctx), Some(wire), ctx).into_any_element()
}
