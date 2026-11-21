// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! What `blender/shader.cpp` reads of a material's node tree through RNA
//! (`bNodeTree`, `bNode`, `bNodeSocket`, `bNodeLink`), handed over as
//! plain data. `model.rs` fills it from phiphx; `shader.rs` builds a
//! [`ShaderGraph`](crate::scene::ShaderGraph) from it.
//!
//! A node's `storage` (the settings Blender keeps in a struct of the
//! node's own type) is [`NodeStorage`]; phiphx gives only its address, so
//! a source read from a file has [`NodeStorage::None`] and `shader.rs`
//! leaves out a node that needs one.

/// `bNodeSocket::type`, as `convert_socket_type` (`shader.cpp:201`) sees it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SocketClass {
    Float,
    /// `SOCK_BOOLEAN` and `SOCK_INT` are both an integer to Cycles.
    Boolean,
    Int,
    Vector,
    Color,
    String,
    /// `SOCK_SHADER`.
    Closure,
    /// A type Cycles has no socket for.
    #[default]
    Undefined,
}

/// `bNodeSocket::default_value`, by the struct it is read from.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SocketValue {
    Float(f32),
    Boolean(bool),
    Int(i32),
    Vector([f32; 3]),
    /// The colour without its alpha.
    Color([f32; 3]),
}

/// A `bNodeSocket`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct SocketSource {
    pub identifier: String,
    pub class: SocketClass,
    /// `is_available()`: an unavailable socket is left out.
    pub available: bool,
    pub value: Option<SocketValue>,
}

/// The settings a node keeps in its `storage` that `add_node` reads.
#[derive(Debug, Clone, PartialEq, Default)]
pub enum NodeStorage {
    #[default]
    None,
    /// `NodeShaderMix`.
    Mix { data_type: SocketClass, blend_type: i32, factor_mode: i32, clamp_factor: bool, clamp_result: bool },
    /// `NodeMapRange`; `vector` is `data_type == CD_PROP_FLOAT3`.
    MapRange { vector: bool, interpolation_type: i32, clamp: bool },
    /// `ColorBand` after `colorramp_to_array`: the table of colours and
    /// alphas, and whether it is interpolated.
    ColorRamp { color: Vec<[f32; 3]>, alpha: Vec<f32>, interpolate: bool },
}

/// A `bNode`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct NodeSource {
    pub idname: String,
    /// `bNode::name`, unique in its tree.
    pub name: String,
    pub custom1: i32,
    pub custom2: i32,
    pub muted: bool,
    pub reroute: bool,
    /// `NODE_DO_OUTPUT`.
    pub active_output: bool,
    /// `bNode::id` of a group node, the tree it instances; `None` for a
    /// group whose tree is missing.
    pub group: Option<Box<GraphSource>>,
    pub storage: NodeStorage,
    pub input: Vec<SocketSource>,
    pub output: Vec<SocketSource>,
}

/// A `bNodeLink`, by the names of its nodes and the sockets' identifiers.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct LinkSource {
    pub from: String,
    pub from_socket: String,
    pub to: String,
    pub to_socket: String,
}

/// A `bNodeTree`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct GraphSource {
    pub node: Vec<NodeSource>,
    pub link: Vec<LinkSource>,
    /// The name of the node `ntreeShaderOutputNode(SHD_OUTPUT_CYCLES)` chose.
    pub output: Option<String>,
}

impl GraphSource {
    pub fn is_empty(&self) -> bool {
        self.node.is_empty()
    }
}
