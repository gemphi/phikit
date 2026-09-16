//! `BNodeGraph` serialize / deserialize + `remove_edge`.
//!
//! Feature: phiber-blender-space-parity
//! Requirements: 22.10, 52.5, 52.15
//!
//! Serialization uses `serde_json`. The graph derives `Serialize` /
//! `Deserialize` via `serde`; we emit through `serde_json::to_string`
//! (compact) so canonical form is deterministic modulo field order
//! within each struct — which serde-generated code emits in
//! declaration order, so `parse(print(g)) == g` and
//! `print(parse(s)) == s` both hold on any `g` produced by the
//! public constructors.
//!
//! Note this file is named `serde_ops` (not `serde`) to avoid shadowing
//! the imported `serde` crate name inside this module.

use super::types::*;

impl BNodeGraph {
    /// Remove the matching edge; no-op if absent.
    pub fn remove_edge(&mut self, e: BEdge) {
        self.edges.retain(|x| x != &e);
    }

    /// Canonical JSON encoding for on-disk persistence.
    pub fn serialize(&self) -> String {
        // `to_string` (not `to_string_pretty`) — compact, whitespace-free,
        // deterministic ordering matches struct declaration order.
        serde_json::to_string(self).expect("BNodeGraph is always serializable")
    }

    /// Parse a graph from canonical JSON.
    pub fn deserialize(s: &str) -> Result<Self, BNodeParseError> {
        serde_json::from_str(s).map_err(|e| BNodeParseError::Json(e.to_string()))
    }
}
