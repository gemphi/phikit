//! Making and rewiring links: which socket the cursor is on, whether a link is
//! allowed, and what a drop does to the ones already there.
//!
//! Ported from Blender 5.2.0, `editors/space_node/node_relationships.cc`
//! (`NODE_OT_link`: `node_link_init`, `node_link_find_socket`,
//! `node_remove_existing_links_if_needened`) and `node_edit.cc:791`
//! (`node_find_indicated_socket`).
//!
//! Everything here is pure. The gesture that calls it is in `drag.rs`, the
//! curves it produces are drawn by `paint/`, and the edits it decides are stored
//! by [`BLinkEdits`] — so each rule can be checked against Blender's arithmetic
//! rather than against a screenshot.
//!
//! ## Where the edits live, and why
//!
//! phiber reads the node tree out of the `.blend` on every frame; there is no
//! editable tree in state. So a made or broken link is stored as an EDIT and
//! applied over the file's own link list, exactly as a dragged node's
//! displacement is applied over its fitted position (`drag.rs`). Two
//! consequences, both deliberate:
//!
//! * the file is never modified, so nothing can be lost by editing a graph;
//! * the edits do not survive the session, because nothing writes a node tree
//!   back yet. When `phiphx`'s writer grows node-tree support this is the layer
//!   that feeds it, and the overlay becomes the undo buffer rather than the
//!   store of record.
//!
//! ## What Blender allows
//!
//! `node_link_find_socket` enforces direction by searching only for the
//! opposite kind — drag from an output and it looks for an input, and the other
//! way round — so an output-to-output link cannot be expressed, let alone
//! rejected. On top of that it skips two cases explicitly:
//!
//! * *"Skip if socket is on the same node as the fromsock"* — no self-link;
//! * *"Skip if tsock is already linked with this output"* — no second link
//!   between the same pair of sockets.
//!
//! Notably there is **no type check**. A float into a colour is a legal link in
//! Blender and is converted on evaluation, so refusing it here would be
//! inventing a rule. Cycles are not checked at this point either; Blender tags
//! the tree and resolves them in `ntreeUpdateTree`.

use crate::phiphx::blend::shade::graph::{BlendNode, BlendNodeGraph, BlendNodeLink};
use crate::phiphx::blend::shade::socket::BlendSocket;

/// One end of a link, by the names a link is resolved with.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct BLinkEnd {
    pub node: String,
    pub socket: String,
}

impl BLinkEnd {
    pub fn new(node: &str, socket: &str) -> Self {
        Self { node: node.to_string(), socket: socket.to_string() }
    }
}

/// The identifiers of `node`'s input sockets a drawn wire arrives at — what the
/// socket layout needs to know which inputs are connected (and so how tall each
/// is, and whether it draws its value). Read off `wired`, the file's links with
/// this host's edits applied, so it is the overlay's truth rather than the
/// file's stale `SOCK_IS_LINKED`.
pub fn linked_inputs<'a>(wired: &'a [BlendNodeLink], node: &str) -> Vec<&'a str> {
    wired.iter().filter(|l| l.to == node).map(|l| l.to_socket.as_str()).collect()
}

/// Whether a link from `from` to `to` is one Blender would let you make.
///
/// `graph` is consulted for the two rules that depend on what is already there.
/// The direction rule is not checked because it cannot be violated: the caller
/// searched for the opposite kind of socket, which is how Blender enforces it.
pub fn allowed(graph: &BlendNodeGraph, links: &[BlendNodeLink], from: &BLinkEnd, to: &BLinkEnd) -> bool {
    // "Skip if socket is on the same node as the fromsock."
    if from.node == to.node {
        return false;
    }
    // Both ends have to exist and be drawn; a link to a socket nobody can see
    // is a link nobody can break.
    let visible = |end: &BLinkEnd, outputs: bool| {
        graph.nodes.iter().find(|n| n.name == end.node).is_some_and(|n| {
            let list = if outputs { &n.outputs } else { &n.inputs };
            list.iter().any(|s| s.identifier == end.socket && s.visible())
        })
    };
    if !visible(from, true) || !visible(to, false) {
        return false;
    }
    // "Skip if tsock is already linked with this output."
    !links.iter().any(|l| {
        l.from == from.node && l.from_socket == from.socket && l.to == to.node && l.to_socket == to.socket
    })
}

/// The links a drop on `to` has to remove to make room, in Blender's order.
///
/// `node_remove_existing_links_if_needed` does it in two passes, and the order
/// matters because the first pass makes room for the second:
///
/// 1. every DUPLICATE of the link being dropped comes out, each one decrementing
///    the running count;
/// 2. then, while the count is still at or over the socket's limit, existing
///    links come out until there is room.
///
/// With an input's limit of 1, step 2 is what makes a drop on a connected input
/// *replace* rather than stack. Blender walks the socket's own link list in
/// order, so it is the earliest links that go — an incidental consequence of
/// iteration order rather than a stated policy, and it is matched here so the
/// same link disappears.
pub fn links_to_remove(
    links: &[BlendNodeLink],
    socket: &BlendSocket,
    from: &BLinkEnd,
    to: &BLinkEnd,
) -> Vec<BlendNodeLink> {
    let at_socket: Vec<&BlendNodeLink> =
        links.iter().filter(|l| l.to == to.node && l.to_socket == to.socket).collect();
    let mut count = at_socket.len() as i32;
    let limit = socket.link_limit();
    let mut out: Vec<BlendNodeLink> = Vec::new();

    // Pass one: duplicates of what is being dropped.
    for link in &at_socket {
        if link.from == from.node && link.from_socket == from.socket {
            out.push((*link).clone());
            count -= 1;
        }
    }
    // Pass two: whatever else has to go for the new link to fit.
    for link in &at_socket {
        if out.contains(link) {
            continue;
        }
        if !(count < limit) {
            out.push((*link).clone());
            count -= 1;
        }
    }
    out
}

/// What grabbing `socket` starts: a new link, or the one already attached.
///
/// `node_link_init`'s two halves, which is the behaviour that makes rewiring
/// feel like rewiring:
///
/// * from an **output**, a new link, fixed at the output end. (Blender detaches
///   instead when the output is at its limit or Ctrl is held; an output's limit
///   is 4095, so in practice only the modifier does it, and phiber has no
///   detach modifier yet.)
/// * from an **input that already has a link**, that link is PICKED UP —
///   detached and dragged by its output end — so dragging off a connected input
///   moves the wire rather than making a second one. An input's limit is 1, so a
///   second one could not exist anyway.
/// * from an **empty input**, a new link, fixed at the input end.
#[derive(Clone, Debug, PartialEq)]
pub enum BLinkGrab {
    /// A new link, anchored at this end, with the other end on the cursor.
    Fresh(BLinkEnd),
    /// An existing link, detached: `anchor` stays put and the other end follows
    /// the cursor. `was` is the link to remove, since it is being moved.
    Moved { anchor: BLinkEnd, was: (BLinkEnd, BLinkEnd) },
}

pub fn grab(links: &[BlendNodeLink], node: &BlendNode, socket: &BlendSocket, output: bool) -> BLinkGrab {
    let here = BLinkEnd::new(&node.name, &socket.identifier);
    if output {
        return BLinkGrab::Fresh(here);
    }
    // Blender takes the LAST matching link in list order (`link_to_pick` is
    // overwritten as the loop runs), and never picks one up off a multi-input.
    let picked = (!socket.multi())
        .then(|| links.iter().filter(|l| l.to == node.name && l.to_socket == socket.identifier).next_back())
        .flatten();
    match picked {
        Some(link) => BLinkGrab::Moved {
            anchor: BLinkEnd::new(&link.from, &link.from_socket),
            was: (
                BLinkEnd::new(&link.from, &link.from_socket),
                BLinkEnd::new(&link.to, &link.to_socket),
            ),
        },
        None => BLinkGrab::Fresh(here),
    }
}

/// How the edits are spelled in the store, and read back.
pub mod store;
/// The drag between the press and the release.
pub mod live;

pub use live::{BLinking, commit, linking_of};
pub use store::{BLinkEdits, decode, edits_of, encode};

#[cfg(test)]
mod tests;
