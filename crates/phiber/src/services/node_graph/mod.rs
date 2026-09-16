//! Shared node graph service used by Shader / Compositor / GeoNode hosts.
//!
//! Kit-agnostic — every UI kit consumes these types via
//! `apple::services::node_graph::…`.
//!
//! Requirements: 22.1, 22.5–22.10, 23.1, 24.1, 25.1, 52.5–52.7, 52.15

#![allow(dead_code)]

pub mod types;
pub mod mutate;
pub mod add_edge;
pub mod serde_ops;

pub use types::{
    BEdge, BEdgeError, BNode, BNodeGraph, BNodeKind, BNodeParseError,
    BNodeTreeKind, BSocket, BSocketDir, BSocketType,
};

/// Default socket layout for a given [`BNodeKind`].
///
/// Currently returns `(vec![], vec![])` for every kind — [`BNodeGraph::add_node`]
/// creates nodes with empty socket vectors, and [`BNodeGraph::add_edge`]
/// treats empty socket vecs as "direction unspecified" (skips the
/// [`BEdgeError::SocketDir`] check). Follow-up work can populate the
/// per-kind socket tables against the Blender reference without breaking
/// the graph API — the structural invariants (no self-edges, no cycles)
/// hold regardless.
pub fn sockets_for(_kind: BNodeKind) -> (Vec<BSocket>, Vec<BSocket>) {
    (Vec::new(), Vec::new())
}
