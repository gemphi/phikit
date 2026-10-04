//! A link drag in progress, and what releasing it commits.
//!
//! Blender's `NODE_OT_link` is a modal operator: `node_link_init` on the press,
//! `node_link_find_socket` on every move, `add_dragged_links_to_tree` on the
//! release, `node_link_cancel` on escape. The state it keeps between those is
//! `bNodeLinkDrag`, and [`BLinking`] is that state — anchored end, cursor, and
//! the link being moved if the grab picked one up.
//!
//! It lives in the store rather than in a GPUI drag payload because a socket is
//! PAINTED, not an element: there is no div under a socket to hang a drag on.
//! Driving it from the canvas's own mouse events instead is both the only way
//! and the closer parity — Blender's is a modal operator reading raw events, not
//! a drag-and-drop payload.
//!
//! ## Releasing over nothing
//!
//! `node_link_cancel` restores pre-existing links — a picked-up link comes back
//! on escape, and *"Dragged from output socket (to create a new link), nothing
//! to restore"*. A plain release over empty space is NOT that cancel in Blender:
//! it opens the link-drag search (`link_drag_search.cc`) so you can add a node
//! right there, and the link is only dropped if you dismiss that menu.
//!
//! > **Recorded divergence.** phiber has no Add menu yet, so a release over
//! > nothing is treated as a cancel: the graph goes back to how it was. The
//! > alternative — committing the removal of a moved link and adding nothing —
//! > would make a slipped drag destroy a link with no way to see it coming. A
//! > picked-up link is therefore never removed at the press, only at a
//! > successful drop, so cancelling is not a restore but a no-op, and nothing
//! > can be lost by a drag that goes wrong.

use super::{BLinkEdits, BLinkEnd, allowed, links_to_remove};
use crate::phiphx::blend::shade::graph::{BlendNodeGraph, BlendNodeLink};
use crate::v2::view::draw::canvas::node::socket::BSocketSide;

/// The drag between the press and the release.
#[derive(Clone, Debug, PartialEq)]
pub struct BLinking {
    /// The end that stays put. The free end searches for the opposite side.
    pub anchor: BLinkEnd,
    /// Whether `anchor` is an output — so the free end is looking for an input.
    pub from_output: bool,
    /// The cursor, in canvas fractions, so the preview survives a resize.
    pub at: (f32, f32),
    /// The link this grab picked up off a connected input, if it did. It is
    /// still in the graph: it is removed when a drop succeeds, never at the
    /// press, which is what makes a cancelled drag cost nothing.
    pub moving: Option<(BLinkEnd, BLinkEnd)>,
}

impl BLinking {
    /// Which side a drop is looking for.
    pub fn wants(&self) -> BSocketSide {
        if self.from_output { BSocketSide::Input } else { BSocketSide::Output }
    }

    /// The link being previewed, anchor first — which end that is depends on
    /// which end is anchored, since a link always runs output → input.
    pub fn preview(&self, cursor: (f32, f32), anchor: (f32, f32)) -> ((f32, f32), (f32, f32)) {
        if self.from_output { (anchor, cursor) } else { (cursor, anchor) }
    }
}

/// The edits a release over `target` produces, or `None` if it changes nothing.
///
/// `target` is the socket under the cursor on the side [`BLinking::wants`] asked
/// for, already resolved by the hit-test. `None` for a release over empty space,
/// which is the recorded divergence above: nothing happens.
pub fn commit(
    graph: &BlendNodeGraph,
    wired: &[BlendNodeLink],
    drag: &BLinking,
    target: Option<&BLinkEnd>,
    edits: &BLinkEdits,
) -> Option<BLinkEdits> {
    let target = target?;
    // A link runs output → input whichever end was grabbed.
    let (from, to) = if drag.from_output {
        (drag.anchor.clone(), target.clone())
    } else {
        (target.clone(), drag.anchor.clone())
    };
    if !allowed(graph, wired, &from, &to) {
        return None;
    }
    // The socket being dropped on decides how many links it will keep.
    let socket = graph
        .nodes
        .iter()
        .find(|n| n.name == to.node)?
        .inputs
        .iter()
        .find(|s| s.identifier == to.socket)?;

    let mut next = edits.clone();
    // The moved link goes now, not at the press — so a drag that comes to
    // nothing leaves the graph alone.
    if let Some((was_from, was_to)) = &drag.moving {
        next.removed.push((was_from.clone(), was_to.clone()));
        next.added.retain(|pair| pair != &(was_from.clone(), was_to.clone()));
    }
    // Then whatever has to come out of the target to make room.
    for link in links_to_remove(wired, socket, &from, &to) {
        let pair = (
            BLinkEnd::new(&link.from, &link.from_socket),
            BLinkEnd::new(&link.to, &link.to_socket),
        );
        next.added.retain(|added| added != &pair);
        if !next.removed.contains(&pair) {
            next.removed.push(pair);
        }
    }
    next.added.push((from, to));
    Some(next)
}

/// `anchor|o|x,y` or `anchor|i|x,y|was_from>was_to`.
const FIELD: char = '|';

pub fn encode(drag: &BLinking) -> String {
    let mut out = String::new();
    super::store::write_end(&drag.anchor, &mut out);
    out.push(FIELD);
    out.push(if drag.from_output { 'o' } else { 'i' });
    out.push(FIELD);
    out.push_str(&format!("{},{}", drag.at.0, drag.at.1));
    if let Some((from, to)) = &drag.moving {
        out.push(FIELD);
        super::store::write_end(from, &mut out);
        out.push('>');
        super::store::write_end(to, &mut out);
    }
    out
}

pub fn decode(text: &str) -> Option<BLinking> {
    let mut parts = text.split(FIELD);
    let anchor = super::store::read_end(parts.next()?)?;
    let from_output = match parts.next()? {
        "o" => true,
        "i" => false,
        _ => return None,
    };
    let (x, y) = parts.next()?.split_once(',')?;
    let at = (x.parse().ok()?, y.parse().ok()?);
    let moving = match parts.next() {
        Some(rest) => {
            let (from, to) = rest.split_once('>')?;
            Some((super::store::read_end(from)?, super::store::read_end(to)?))
        }
        None => None,
    };
    Some(BLinking { anchor, from_output, at, moving })
}

/// The drag in progress for this host, if there is one.
pub fn linking_of(
    state: &crate::v2::state::BState,
    host: crate::v2::host::BHostType,
) -> Option<BLinking> {
    use crate::v2::state::{BKey as _, NodeKey};
    decode(state.get(&NodeKey::Linking(host).path())?)
}

#[cfg(test)]
mod tests;
