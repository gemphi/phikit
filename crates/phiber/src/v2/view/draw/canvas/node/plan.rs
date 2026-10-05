//! Where everything is, in canvas pixels.
//!
//! Positions reach here as FRACTIONS of the canvas and leave as pixels, because
//! the canvas's size is not known until the paint pass and a card's div has to
//! be placed before that. `view::BNodeView::to_fraction` is what makes a
//! fraction expressible without the pixel size; this is the other half, where
//! the pixels finally arrive.
//!
//! It exists as its own file rather than a private helper in `set.rs` because
//! three things need the same answer and must not disagree about it: the painter
//! drawing the sockets, the hit-test deciding which socket the cursor is on
//! (`pick/`), and the cards themselves.

use super::view::BNodeView;
use super::{paint, socket};
use crate::phiphx::blend::shade::graph::{BlendNode, BlendNodeGraph, BlendNodeLink};

/// A card's size in view units — the theme's pixels, read as node space.
///
/// Blender's node space IS pixels at zoom 1: `NODE_DY` is `U.widget_unit` and a
/// default node is 140 wide, which is 140 screen pixels unzoomed. So the theme's
/// own card width and row height are the honest view-unit sizes, and the only
/// thing zoom does to them is what it does to everything else.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BCardSize {
    pub header: f32,
    pub unit: f32,
    pub width: f32,
}

impl BCardSize {
    /// The same sizes in canvas pixels at this view's zoom.
    pub fn scaled(self, zoom: f32) -> Self {
        Self { header: self.header * zoom, unit: self.unit * zoom, width: self.width * zoom }
    }

    /// One node's card height — Blender's socket layout, which IS the height
    /// calculation (`node_update_basis`), in whatever units `self` is in.
    pub fn height_of(self, node: &BlendNode) -> f32 {
        // A reroute is `loc ± NODE_SOCKSIZE`: a square the size of its dot, with
        // no header and no body (`reroute_node_prepare_for_draw`). The side is
        // twice the socket radius, which is what `node_bounds` halves back to
        // the radius to centre the box on the location.
        if node.reroute() {
            return socket::DOT * 2.0 * self.unit;
        }
        // A collapsed node is one pill with no header band, so there is no
        // header to add — `body_height` answers with the whole thing.
        if node.collapsed() {
            return socket::body_height(node) * self.unit;
        }
        self.header + socket::body_height(node) * self.unit
    }
}

/// A card's size as fractions of the canvas — what a div can be given.
///
/// The other half of `BCardSize`: the painter scales by the view's zoom because
/// it knows the pixels, and a div divides by the view's extent because it does
/// not. Both come off the same view, which is what keeps them agreeing.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BCardFraction {
    pub width: f32,
    pub header: f32,
    pub body: f32,
}

impl BCardFraction {
    pub fn of(size: BCardSize, view: &BNodeView, node: &BlendNode) -> Self {
        Self {
            width: size.width / view.width(),
            header: size.header / view.height(),
            body: socket::body_height(node) * size.unit / view.height(),
        }
    }
}

/// Every card and every link, in canvas pixels, for a canvas of `(w, h)`.
///
/// The card sizes are scaled by the view's zoom, so zooming out shrinks the
/// cards rather than only sliding them together. The divs do the same thing a
/// different way — a fraction of the canvas, since they have no pixels to scale
/// by — and both come off the same view, which is what keeps a painted socket on
/// the drawn card's border.
pub fn plan(
    graph: &BlendNodeGraph,
    wired: &[BlendNodeLink],
    placed: &[(String, (f32, f32))],
    width: f32,
    height: f32,
    size: BCardSize,
    view: &BNodeView,
) -> (Vec<paint::BNodeCard>, Vec<((f32, f32), (f32, f32))>) {
    let on_screen = size.scaled(view.zoom(width));
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
                // A reroute's two sockets coincide at its location: the card is
                // zero-wide so `socket_at` puts both on the same edge, which is
                // the one dot a reroute draws (`reroute_node_prepare_for_draw`).
                width: if node.reroute() { 0.0 } else { on_screen.width },
                // A collapsed node's rows are measured from the pill's own top,
                // and a reroute has no header either — both start at their own
                // top edge (`socket::collapsed_rows`, `socket::reroute_rows`).
                header: if node.collapsed() || node.reroute() { 0.0 } else { on_screen.header },
                unit: on_screen.unit,
                // The card's own height, which is Blender's socket layout — the
                // same number the body div is given, so the painted sockets and
                // the drawn card cannot disagree.
                height: on_screen.height_of(node),
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

#[cfg(test)]
mod tests {
    use super::*;

    fn size() -> BCardSize {
        BCardSize { header: 30.0, unit: 36.0, width: 140.0 }
    }

    #[test]
    fn a_reroutes_card_height_is_its_dot_with_no_header() {
        // `reroute_node_prepare_for_draw`: a square `radius * 2` on a side, no
        // header band. Every other node adds its header to the socket walk; a
        // reader that did the same to a reroute would give it a title bar's
        // worth of empty height above a dot.
        let reroute = BlendNode { idname: "NodeReroute".into(), ..Default::default() };
        assert_eq!(size().height_of(&reroute), socket::DOT * 2.0 * 36.0, "the dot, no header");
        // The header IS added for an ordinary node of the same (empty) sockets.
        let ordinary = BlendNode::default();
        assert_eq!(size().height_of(&ordinary), 30.0 + socket::body_height(&ordinary) * 36.0);
    }
}

/// Where every node sits, three ways.
///
/// The drag needs all three: the node's own `bNode.location` out of the file,
/// which a displacement is measured from; the displacement itself; and the two
/// added and turned into a fraction of the canvas, which is where the card is
/// drawn.
///
/// The first two are in VIEW UNITS now — Blender's own coordinates. They used to
/// be fractions of a fitted canvas, and the fit had to be computed from file
/// positions alone so that dragging a node did not reframe the graph under the
/// pointer. With a real view there is nothing to feed back into: the framing is
/// the view's, the view changes only when a pan or a zoom changes it, and a
/// dragged node moves in the same units the file stores.
pub(super) fn positions(
    graph: &BlendNodeGraph,
    view: &BNodeView,
    state: &crate::v2::state::BState,
    host: crate::v2::host::BHostType,
) -> (Vec<(String, (f32, f32))>, Vec<(String, (f32, f32))>, Vec<(String, (f32, f32))>) {
    let located: Vec<(String, (f32, f32))> =
        graph.nodes.iter().map(|n| (n.name.clone(), (n.x, n.y))).collect();
    let offsets: Vec<(String, (f32, f32))> = located
        .iter()
        .map(|(name, _)| (name.clone(), super::drag::offset_of(state, host, name)))
        .collect();
    let placed: Vec<(String, (f32, f32))> = located
        .iter()
        .zip(offsets.iter())
        .map(|((name, base), (_, from))| {
            (name.clone(), view.to_fraction(super::drag::placed(*base, *from)))
        })
        .collect();
    (located, offsets, placed)
}
