//! `BNodeGraph::add_edge` — validated edge insertion with cycle detection.
//!
//! Feature: phiber-blender-space-parity
//! Requirements: 22.6, 22.7
//!
//! Rejection order (matches the design's contract):
//! 1. [`BEdgeError::SelfEdge`] when `e.from.0 == e.to.0`.
//! 2. [`BEdgeError::UnknownNode`] when either endpoint id is absent.
//! 3. [`BEdgeError::SocketDir`] when the source socket is not an `Output`
//!    or the destination socket is not an `Input`. Skipped when either
//!    node has empty socket vectors — those callers accept the
//!    "sockets unspecified" case (integration tests, minimal fixtures).
//! 4. [`BEdgeError::Cycle`] when adding the edge would introduce a cycle.
//!    Detected by DFS from `e.from.0` on the pre-insert edge set looking
//!    for `e.to.0`.

use super::types::*;
use std::collections::HashSet;

impl BNodeGraph {
    /// Attempt to insert `e`. Returns `Ok(())` on success, an [`BEdgeError`]
    /// on rejection. The graph is left untouched on any error path.
    pub fn add_edge(&mut self, e: BEdge) -> Result<(), BEdgeError> {
        if e.from.0 == e.to.0 {
            return Err(BEdgeError::SelfEdge);
        }
        let src = self.nodes.iter().find(|n| n.id == e.from.0).ok_or(BEdgeError::UnknownNode)?;
        let dst = self.nodes.iter().find(|n| n.id == e.to.0).ok_or(BEdgeError::UnknownNode)?;

        // Direction check — skipped when the node has no declared sockets
        // (test fixtures use empty vecs).
        if !src.outputs.is_empty() {
            match src.outputs.get(e.from.1 as usize) {
                Some(s) if s.dir == BSocketDir::Output => {}
                _ => return Err(BEdgeError::SocketDir),
            }
        }
        if !dst.inputs.is_empty() {
            match dst.inputs.get(e.to.1 as usize) {
                Some(s) if s.dir == BSocketDir::Input => {}
                _ => return Err(BEdgeError::SocketDir),
            }
        }

        // Cycle check: if there's already a path from e.to.0 back to
        // e.from.0 in the existing edge set, adding e would close a cycle.
        if reaches(&self.edges, e.to.0, e.from.0) {
            return Err(BEdgeError::Cycle);
        }

        self.edges.push(e);
        Ok(())
    }
}

/// True iff there is a directed path from `src` to `dst` in `edges`,
/// following `from → to`. Iterative DFS over the edge set.
fn reaches(edges: &[BEdge], src: u32, dst: u32) -> bool {
    if src == dst {
        return true;
    }
    let mut seen: HashSet<u32> = HashSet::new();
    let mut stack: Vec<u32> = vec![src];
    while let Some(n) = stack.pop() {
        if !seen.insert(n) {
            continue;
        }
        for e in edges.iter().filter(|e| e.from.0 == n) {
            if e.to.0 == dst {
                return true;
            }
            stack.push(e.to.0);
        }
    }
    false
}
