//! A socket by the identifier a link names it with.
//!
//! Split from `mod.rs`, which owns the layout and had outgrown the 240-line
//! read budget. It is a subject of its own either way: the layout answers
//! *where do this node's sockets sit*, and this answers *which of them is
//! the one this link means* — the lookup that turns a `bNodeLink`'s two
//! ends into two points, asked by `plan`, by the hit-test and by the
//! spatial layout.

use super::{BSocketRow, rows};
use crate::phiphx::blend::shade::graph::BlendNode;

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
pub fn row_of_card(card: &super::super::paint::BNodeCard, identifier: &str) -> Option<BSocketRow> {
    card.sockets.iter().find(|r| r.identifier == identifier).cloned()
}
