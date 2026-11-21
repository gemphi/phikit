// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Nodes and their sockets: `intern/cycles/graph`. Everything a scene holds,
//! its integrator, film, camera, objects, lights and every shader node, is
//! a [`Node`] of a [`NodeType`], and a node's settings are its sockets.
//!
//! Cycles declares the sockets in `NODE_DEFINE` blocks; here they are
//! `data/node.tsv`, which `tool/node.py` writes from those blocks.

mod hash;
mod node;
mod schema;
mod socket;
mod value;

pub use node::Node;
pub use schema::{NodeEnum, NodeKind, NodeType};
pub use socket::{SocketDirection, SocketKind, SocketType};
pub use value::Value;
