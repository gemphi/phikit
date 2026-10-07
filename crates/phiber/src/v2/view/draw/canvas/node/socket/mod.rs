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
//! Everything here is in widget units; the renderer multiplies by the theme's
//! `row_height`, phiber's `U.widget_unit` — exactly what `NODE_DY` is in Blender.
//!
//! The layout, in Blender's order: a quarter unit under the header
//! (`NODE_DYS / 2`); the **outputs** top-down on the card's RIGHT edge, each at
//! its row's middle; a quarter unit after them (`NODE_DY / 4`); the **inputs**
//! on the LEFT edge; a quarter unit of padding if there were any. Outputs above
//! inputs is what makes a chain of nodes read as a descending staircase, the
//! opposite of a usual dataflow diagram.
//!
//! A socket is NOT always one row: an unconnected vector input draws a field
//! column and is several rows tall (`value::rows_tall`), so the height is a
//! question about the wiring — which is why `rows`/`body_height` take the
//! linked inputs. A multi-input socket's extra height is still NOT here
//! (`NODE_MULTI_INPUT_LINK_GAP`, a Geometry Nodes thing absent from shaders).
//!
//! One Blender quirk is ported rather than tidied: the gap after an output is
//! added only before an AVAILABLE next socket, but after an input before any
//! next socket at all. So a node whose last input is followed by a disabled one
//! is a tenth of a unit taller than it looks — but it is what Blender draws.

use crate::phiphx::blend::shade::graph::BlendNode;
use crate::phiphx::blend::shade::socket::{BSocketDefault, BlendSocket};

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
    /// `bNodeSocket.default_value`, so the painter can draw it on an unconnected
    /// input (`value::widget`). `None` for an output, a reroute/collapsed row, or
    /// a socket with no stored default.
    pub default: Option<BSocketDefault>,
    /// Whether a drawn wire arrives here — the overlay's truth, not the file's
    /// `SOCK_IS_LINKED`. Set by `place` from the `linked` identifiers, since the
    /// flag also decides the row's height (a linked vector collapses).
    pub linked: bool,
}

/// Lay out one list, advancing `y` (which counts DOWN from the header in units).
///
/// `gap_needs_available` is Blender's asymmetry: `true` for the outputs, which
/// only space before an available next socket, `false` for the inputs, which
/// space before any next socket.
///
/// `linked` is the identifiers of the inputs a drawn wire arrives at — it sets
/// each socket's `linked` flag and, through `value::rows_tall`, its height (an
/// unconnected vector input is taller).
fn place(
    sockets: &[BlendSocket],
    side: BSocketSide,
    gap_needs_available: bool,
    linked: &[&str],
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
        let is_linked = linked.contains(&socket.identifier.as_str());
        // Only an input's default is ever shown (`value::widget`).
        let default = if side == BSocketSide::Input { socket.default.clone() } else { None };
        out.push(BSocketRow {
            identifier: socket.identifier.clone(),
            name: socket.name.clone(),
            kind: socket.kind,
            side,
            // `locy - NODE_DYS` from the block's top, so the dot stays level with
            // the name row even on a tall vector.
            y: top + HALF,
            default: default.clone(),
            linked: is_linked,
        });
        // A vector input adds a field row per component; everything else is one.
        *y = top + value::rows_tall(side, is_linked, default.as_ref());
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
/// the SAME point — the node's location — so the single dot stands for both.
/// Here that is `y = 0`, the card's top-left, so the painter's dot comes out
/// centred on the location, which is exactly what a reroute is.
///
/// The names are blanked — a reroute paints no socket labels — but both rows
/// are kept though they coincide, because a link leaving the output still
/// resolves by `identifier`.
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
                default: None,
                linked: false,
            });
        }
    }
    out
}

/// Every socket of `node`, placed, outputs first — which is also top first.
///
/// `linked` names the inputs a wire arrives at: it sets each input's `linked`
/// flag and, through it, whether a vector input draws its taller field column
/// (`place`). A caller with no wiring in hand passes `&[]`.
pub fn rows(node: &BlendNode, linked: &[&str]) -> Vec<BSocketRow> {
    if node.reroute() {
        return reroute_rows(node);
    }
    if node.collapsed() {
        return collapsed_rows(node);
    }
    let mut out = Vec::new();
    let mut y = PAD;
    let had_outputs = place(&node.outputs, BSocketSide::Output, true, linked, &mut y, &mut out);
    if had_outputs {
        y += AFTER_OUTPUTS;
    }
    place(&node.inputs, BSocketSide::Input, false, linked, &mut y, &mut out);
    out
}

/// How tall the card's body is, in units — the header is not counted.
///
/// This is `node_update_basis`'s own answer: the layout walk IS the height
/// calculation in Blender, which is why the two are one function here too. A
/// card sized any other way would have its sockets inside it by luck.
pub fn body_height(node: &BlendNode, linked: &[&str]) -> f32 {
    // A reroute has no body: the whole node is one painted dot (`height_of`).
    if node.reroute() {
        return 0.0;
    }
    // A collapsed node has no body either: the pill IS its height.
    if node.collapsed() {
        return collapsed_height(node);
    }
    let mut y = PAD;
    let had_outputs = place(&node.outputs, BSocketSide::Output, true, linked, &mut y, &mut Vec::new());
    if had_outputs {
        y += AFTER_OUTPUTS;
    }
    let had_inputs = place(&node.inputs, BSocketSide::Input, false, linked, &mut y, &mut Vec::new());
    if had_inputs || had_outputs {
        y += PAD;
    }
    y
}

/// Where one named socket of `node` sits, if it has one.
///
/// A link names its ends by `identifier`, so this is the lookup that turns a
/// link into two points. `None` for a socket that is not drawn. `linked` matters
/// because a vector input above a socket makes those below it sit lower.
pub fn row_of(node: &BlendNode, identifier: &str, linked: &[&str]) -> Option<BSocketRow> {
    rows(node, linked).into_iter().find(|r| r.identifier == identifier)
}

/// One row of an already laid-out card, by identifier — the same lookup as
/// `row_of` without laying the node out a second time.
pub fn row_of_card(card: &super::paint::BNodeCard, identifier: &str) -> Option<BSocketRow> {
    card.sockets.iter().find(|r| r.identifier == identifier).cloned()
}

#[cfg(test)]
mod tests;

/// `node_update_collapsed` — the layout of a node folded with H.
pub mod collapsed;

/// What an unconnected input socket shows for its default value.
pub mod value;

pub use collapsed::{COLLAPSED_RAD, collapsed_height, collapsed_rows, collapsed_top};
