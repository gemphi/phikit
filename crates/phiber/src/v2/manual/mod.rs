//! The phiber manual, declared as data.
//!
//! Blender's manual is a tree of `.rst` files whose shape exists only in
//! `index.rst` toctrees and in the directory layout: nothing in Blender can
//! be asked what the manual covers, which page answers to which editor, or
//! which pages are still unwritten. Ours is a tree of [`BManualNode`], in
//! the same shape every other part of phiber is declared in
//! (`part::set`'s `BPartSpec`, `region`'s `BRegionSpec`, `modal::set`'s
//! `BModalSpec`) — a `static` set of nodes, read rather than walked on
//! disk.
//!
//! Declaring it buys three things a directory of prose cannot:
//!
//! 1. **The hierarchy is complete before the prose is.** Every page exists
//!    as a [`BManualState::Stub`] from the first commit, so the gap list is
//!    a query (`stubs()`), not an audit.
//! 2. **Each page names the Blender page it answers to** (`blender`), so
//!    parity is checkable: a node whose `blender` path does not exist under
//!    `docs/blender-manual/manual/` is a page we invented, and a Blender
//!    page no node claims is a page we have not covered.
//! 3. **Each page that needs a screenshot says which screen**
//!    ([`BManualFigure`]), so the shots can be taken in one pass, in one
//!    style, instead of being gathered ad hoc per page.
//!
//! The figure style is the deck: see [`BDeck`].

pub mod deck;
pub mod set;

pub use deck::BDeck;
pub use set::MANUAL;

use crate::v2::host::BHostType;
use crate::v2::space::BSpaceType;

/// How far a page has got. The manual ships with every node present and
/// most of them `Stub`, so "what is left" is a filter rather than a memory.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BManualState {
    /// Declared, with no prose yet.
    Stub,
    /// Written, with no figure taken.
    Drafted,
    /// Written and its figure taken, but not checked against Blender's page.
    Shot,
    /// Written, shot, and read against the Blender page it answers to.
    Done,
}

/// The screen a page's figure shows.
///
/// A figure is not a free-hand screenshot: it is this space, with this host
/// in focus, drawn through [`BDeck`]. Two pages that name the same space and
/// host get the same frame, which is what makes a manual read as one
/// document rather than a folder of captures taken on different days.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BManualFigure {
    pub space: BSpaceType,
    /// The host the figure centres on; the rest of the screen is drawn
    /// behind it at the wave's lower steps.
    pub host: BHostType,
}

/// One page of the manual.
#[derive(Clone, Copy, Debug)]
pub struct BManualNode {
    /// The path segment, and the file stem under `docs/manual/`.
    pub slug: &'static str,
    pub title: &'static str,
    /// The Blender page this answers to, relative to
    /// `docs/blender-manual/manual/`, or `""` where phiber has no
    /// counterpart (a page about phiray, say, which Blender has no engine
    /// for). An empty string is a deliberate divergence, not an omission.
    pub blender: &'static str,
    /// The screen this page shows, when it shows one. A reference page
    /// (a list of shortcuts) has none.
    pub figure: Option<BManualFigure>,
    pub state: BManualState,
    pub children: &'static [BManualNode],
}

impl BManualNode {
    /// Every node under this one, itself first.
    pub fn walk(&'static self) -> Vec<&'static BManualNode> {
        let mut out = vec![self];
        for child in self.children {
            out.extend(child.walk());
        }
        out
    }
}

/// Every node in the manual, depth first.
pub fn every() -> Vec<&'static BManualNode> {
    MANUAL.iter().flat_map(BManualNode::walk).collect()
}

/// The pages still to write — the gap list, as a query.
pub fn stubs() -> Vec<&'static BManualNode> {
    every().into_iter().filter(|n| n.state == BManualState::Stub).collect()
}

/// Every distinct screen the manual has to depict, so the shots can be
/// taken in one pass: each figure once, with the pages that want it.
pub fn figures() -> Vec<(BManualFigure, Vec<&'static BManualNode>)> {
    let mut out: Vec<(BManualFigure, Vec<&'static BManualNode>)> = Vec::new();
    for node in every() {
        let Some(figure) = node.figure else { continue };
        match out.iter_mut().find(|(f, _)| *f == figure) {
            Some((_, pages)) => pages.push(node),
            None => out.push((figure, vec![node])),
        }
    }
    out
}

/// How far along the manual is, as (done, total).
pub fn progress() -> (usize, usize) {
    let all = every();
    (all.iter().filter(|n| n.state == BManualState::Done).count(), all.len())
}

#[cfg(test)]
#[path = "mod_tests.rs"]
mod tests;
