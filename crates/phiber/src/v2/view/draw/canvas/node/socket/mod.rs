//! Where a node's sockets sit on its card, and how tall that makes the card.
//!
//! Ported from Blender 5.2.0, `editors/space_node/node_draw.cc`:
//! `node_update_basis_from_socket_lists` walks the two lists and
//! `node_update_basis_socket` places each one, with the metrics from
//! `node_intern.hh:334`:
//!
//! | Blender | value | here |
//! |---|---|---|
//! | `NODE_DY` | `U.widget_unit` | [`ROW`] — one socket's row |
//! | `NODE_DYS` | `U.widget_unit / 2` | [`HALF`] — the socket sits at the row's middle |
//! | `NODE_ITEM_SPACING_Y` | `0.1 * U.widget_unit` | [`SPACING`] |
//! | `NODE_SOCKSIZE` | `0.25 * U.widget_unit` | [`DOT`] — the socket's radius |
//!
//! Everything here is in widget units, and the renderer multiplies by the
//! theme's `row_height` — which is phiber's `U.widget_unit`: the height of one
//! control row, which is exactly what `NODE_DY` is in Blender.
//!
//! The layout, in the order Blender does it:
//!
//! 1. a quarter unit of space under the header (`NODE_DYS / 2`);
//! 2. the **outputs**, top-down, each taking a full row, the socket itself at the
//!    row's middle, on the card's RIGHT edge;
//! 3. a quarter unit (`NODE_DY / 4`) if there were any outputs;
//! 4. the node's buttons — phiber draws none yet, so nothing is reserved;
//! 5. the **inputs**, each a full row, on the card's LEFT edge;
//! 6. a quarter unit of padding if there were any inputs.
//!
//! Outputs above inputs is the fact worth stating, because it is the opposite of
//! what a dataflow diagram usually does and it is what makes a Blender graph
//! readable: a node's result leaves from the top right and arrives at the next
//! node's top left, so a chain of nodes reads as a descending staircase.
//!
//! One Blender quirk is ported rather than tidied: between two outputs the gap is
//! added only when the NEXT output is available, and between two inputs whenever
//! there is a next socket at all, available or not
//! (`socket->next && socket->next->is_available()` against a bare
//! `socket->next`). So a node whose last input is followed by a disabled one is a
//! tenth of a unit taller than it looks like it should be. It is two lines apart
//! in the same function, so it reads as an oversight — but it is what Blender
//! draws, and a card a tenth of a unit shorter than Blender's would put every
//! link below it a hair out.
//!
//! What is NOT here: a multi-input socket's extra height.
//! `node_update_basis_socket` adds `max(total_inputs - 2, 0) *
//! NODE_MULTI_INPUT_LINK_GAP` for a `SOCK_MULTI_INPUT`, where `total_inputs` is
//! how many links actually arrive at it — a runtime count. No shader node has a
//! multi-input socket (they are a Geometry Nodes thing, Join Geometry and the
//! like), so the shader editor is unaffected; a geometry graph will need the link
//! count threaded in.

use crate::phiphx::blend::shade::graph::BlendNode;
use crate::phiphx::blend::shade::socket::BlendSocket;

/// `NODE_DY` — one socket's row.
pub const ROW: f32 = 1.0;
/// `NODE_DYS` — half a row, which is where the socket sits within its row.
pub const HALF: f32 = ROW / 2.0;
/// `NODE_ITEM_SPACING_Y` — between two sockets of the same list.
pub const SPACING: f32 = 0.1;
/// `NODE_SOCKSIZE` — the socket's radius.
pub const DOT: f32 = 0.25;
/// `NODE_DYS / 2`, the space under the header and the padding at the bottom.
const PAD: f32 = HALF / 2.0;
/// `NODE_DY / 4`, between the outputs and whatever follows them.
const AFTER_OUTPUTS: f32 = ROW / 4.0;

/// Which edge of the card a socket sits on.
///
/// Blender puts an input at `locx` and an output at `locx + NODE_WIDTH(node)`
/// (`node_update_basis_socket`) — the card's two vertical edges, with the socket
/// centred ON the edge rather than inside it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BSocketSide {
    Input,
    Output,
}

/// One socket, placed.
#[derive(Clone, Debug, PartialEq)]
pub struct BSocketRow {
    /// `bNodeSocket.identifier` — what a link names this end by.
    pub identifier: String,
    /// What the row prints.
    pub name: String,
    /// `eNodeSocketDatatype`, for `theme::socket_color`.
    pub kind: i32,
    pub side: BSocketSide,
    /// Units below the bottom of the header, at the socket's own centre.
    pub y: f32,
}

/// Lay out one list, advancing `y` (which counts DOWN from the header in units).
///
/// `gap_needs_available` is Blender's asymmetry: `true` for the outputs, which
/// only space when the next socket is available, `false` for the inputs, which
/// space whenever there is a next socket.
fn place(
    sockets: &[BlendSocket],
    side: BSocketSide,
    gap_needs_available: bool,
    y: &mut f32,
    out: &mut Vec<BSocketRow>,
) -> bool {
    let mut any = false;
    for (i, socket) in sockets.iter().enumerate() {
        // `node_update_basis_socket` returns false without touching `locy` for a
        // socket it will not draw, so an unavailable one takes no room at all.
        if !socket.visible() {
            continue;
        }
        let top = *y;
        out.push(BSocketRow {
            identifier: socket.identifier.clone(),
            name: socket.name.clone(),
            kind: socket.kind,
            side,
            y: top + HALF,
        });
        *y = top + ROW;
        any = true;
        let next = sockets.get(i + 1);
        let gap = match (gap_needs_available, next) {
            (_, None) => false,
            (true, Some(n)) => n.available(),
            (false, Some(_)) => true,
        };
        if gap {
            *y += SPACING;
        }
    }
    any
}

/// A reroute's two sockets, both at the dot's centre.
///
/// `reroute_node_prepare_for_draw` puts a reroute's one input and one output at
/// the SAME point — the node's location — so the single dot drawn stands for
/// both ends. Here that is `y = 0`: every node's card is placed with its
/// top-left at the location, the socket sits at that point, and the painter's
/// dot (a circle of `DOT` radius) comes out centred on the location, which is
/// exactly what a reroute is.
///
/// The names are blanked: a reroute paints no socket labels (its only text is
/// an optional node label, which phiber does not draw yet), and a row carrying
/// "Input"/"Output" would paint those words beside the dot. Both rows are kept
/// even though they coincide, because a link names its end by `identifier` and
/// a link leaving the reroute's output must still resolve.
pub fn reroute_rows(node: &BlendNode) -> Vec<BSocketRow> {
    let mut out = Vec::new();
    for (sockets, side) in [(&node.outputs, BSocketSide::Output), (&node.inputs, BSocketSide::Input)] {
        if let Some(socket) = sockets.iter().find(|s| s.visible()) {
            out.push(BSocketRow {
                identifier: socket.identifier.clone(),
                name: String::new(),
                kind: socket.kind,
                side,
                y: 0.0,
            });
        }
    }
    out
}

/// Every socket of `node`, placed, outputs first — which is also top first.
pub fn rows(node: &BlendNode) -> Vec<BSocketRow> {
    if node.reroute() {
        return reroute_rows(node);
    }
    if node.collapsed() {
        return collapsed_rows(node);
    }
    let mut out = Vec::new();
    let mut y = PAD;
    let had_outputs = place(&node.outputs, BSocketSide::Output, true, &mut y, &mut out);
    if had_outputs {
        y += AFTER_OUTPUTS;
    }
    place(&node.inputs, BSocketSide::Input, false, &mut y, &mut out);
    out
}

/// How tall the card's body is, in units — the header is not counted.
///
/// This is `node_update_basis`'s own answer: the layout walk IS the height
/// calculation in Blender, which is why the two are one function here too. A
/// card sized any other way would have its sockets inside it by luck.
pub fn body_height(node: &BlendNode) -> f32 {
    // A reroute has no body at all: the whole node is one painted dot sitting
    // AT the location, so its sockets are placed from a zero-height body and its
    // box size is set apart (`plan::BCardSize::height_of`, `loc ± NODE_SOCKSIZE`).
    if node.reroute() {
        return 0.0;
    }
    // A collapsed node has no body: the pill IS its height, and `BCardSize`
    // adds no header to it (`plan::BCardSize::height_of`).
    if node.collapsed() {
        return collapsed_height(node);
    }
    let mut y = PAD;
    let had_outputs = place(&node.outputs, BSocketSide::Output, true, &mut y, &mut Vec::new());
    if had_outputs {
        y += AFTER_OUTPUTS;
    }
    let had_inputs = place(&node.inputs, BSocketSide::Input, false, &mut y, &mut Vec::new());
    if had_inputs || had_outputs {
        y += PAD;
    }
    y
}

/// Where one named socket of `node` sits, if it has one.
///
/// A link names its ends by `identifier`, so this is the lookup that turns a
/// link into two points. `None` for a socket that is not drawn, which is what
/// makes a link to a hidden socket skippable rather than drawn to the origin.
pub fn row_of(node: &BlendNode, identifier: &str) -> Option<BSocketRow> {
    rows(node).into_iter().find(|r| r.identifier == identifier)
}

/// One row of an already laid-out card, by identifier.
///
/// `row_of` takes the node and lays it out again; a card has its rows already,
/// and the preview needs exactly one of them. Same lookup, no second layout.
pub fn row_of_card(card: &super::paint::BNodeCard, identifier: &str) -> Option<BSocketRow> {
    card.sockets.iter().find(|r| r.identifier == identifier).cloned()
}

#[cfg(test)]
mod tests;

/// `node_update_collapsed` — the layout of a node folded with H.
pub mod collapsed;

pub use collapsed::{COLLAPSED_RAD, collapsed_height, collapsed_rows, collapsed_top};
