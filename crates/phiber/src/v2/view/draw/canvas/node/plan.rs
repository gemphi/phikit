//! Where everything is, in canvas pixels.
//!
//! The fit works in fractions and the canvas's size is not known until the paint
//! pass, so this is the one place that turns one into the other. It exists as
//! its own file rather than a private helper in `set.rs` because three things
//! need the same answer and must not disagree about it: the painter drawing the
//! sockets, the hit-test deciding which socket the cursor is on (`pick/`), and
//! the cards themselves.

use super::{paint, socket};
use crate::phiphx::blend::shade::graph::{BlendNodeGraph, BlendNodeLink};

/// Every card and every link, in canvas pixels, for a canvas of `(w, h)`.
///
/// Positions reach here as fractions, because the fit works in fractions and the
/// canvas's size is not known until the paint pass. One function so the divs and
/// the paint pass cannot disagree about where a card is.
pub fn plan(
    graph: &BlendNodeGraph,
    wired: &[BlendNodeLink],
    placed: &[(String, (f32, f32))],
    width: f32,
    height: f32,
    header: f32,
    unit: f32,
    card_width: f32,
) -> (Vec<paint::BNodeCard>, Vec<((f32, f32), (f32, f32))>) {
    let at = |name: &str| placed.iter().find(|(n, _)| n == name).map(|(_, p)| *p);
    let cards: Vec<paint::BNodeCard> = graph
        .nodes
        .iter()
        .filter_map(|node| {
            let (fx, fy) = at(&node.name)?;
            Some(paint::BNodeCard {
                name: node.name.clone(),
                x: fx * width,
                y: fy * height,
                width: card_width,
                header,
                unit,
                // The card's own height, which is Blender's socket layout — the
                // same number the body div is given, so the painted sockets and
                // the drawn card cannot disagree.
                height: header + socket::body_height(node) * unit,
                sockets: socket::rows(node),
            })
        })
        .collect();
    // A link joins two SOCKETS, so each end is looked up by its identifier;
    // a link to a socket that is not drawn is not drawn either.
    let point_of = |node_name: &str, identifier: &str, side_fallback| {
        let node = graph.nodes.iter().find(|n| n.name == node_name)?;
        let card = cards.iter().zip(graph.nodes.iter()).find(|(_, n)| n.name == node_name)?.0;
        let row = socket::row_of(node, identifier)
            // An older file, or a link whose socket identifier did not
            // resolve: fall back to the first socket on the side the link
            // needs, which is where a two-node shader graph's one link goes
            // anyway.
            .or_else(|| socket::rows(node).into_iter().find(|r| r.side == side_fallback))?;
        Some(card.socket_at(&row))
    };
    let links = wired
        .iter()
        .filter_map(|link| {
            let from = point_of(&link.from, &link.from_socket, socket::BSocketSide::Output)?;
            let to = point_of(&link.to, &link.to_socket, socket::BSocketSide::Input)?;
            Some((from, to))
        })
        .collect();
    (cards, links)
}
