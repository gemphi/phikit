// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/scene/shader_graph.h` (`ShaderBump`,
//! `ShaderIO`, `ShaderInput`, `ShaderOutput`): the sockets of a node in a
//! shader graph, and the links between them.
//!
//! Cycles links sockets by pointer; here a socket is named by its node's
//! id in the graph and its place among that node's inputs or outputs.


use crate::graph::{SocketKind, SocketType};

/// `ShaderBump`: for bump mapping a node is evaluated three times; this
/// says which sample a copy of it is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ShaderBump {
    /// `SHADER_BUMP_NONE`.
    #[default]
    None,
    /// `SHADER_BUMP_CENTER`.
    Center,
    /// `SHADER_BUMP_DX`.
    Dx,
    /// `SHADER_BUMP_DY`.
    Dy,
}

/// An input of a node in a graph: what Cycles holds as a `ShaderInput *`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InputId {
    /// The node's id.
    pub node: usize,
    /// The input's place among the node's inputs.
    pub socket: usize,
}

/// An output of a node in a graph: what Cycles holds as a
/// `ShaderOutput *`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OutputId {
    /// The node's id.
    pub node: usize,
    /// The output's place among the node's outputs.
    pub socket: usize,
}

/// `ShaderInput`: an input socket of a shader node. Linked to an output,
/// or else it has the value its node holds for it.
#[derive(Debug, Clone, PartialEq)]
pub struct ShaderInput {
    pub socket_type: &'static SocketType,
    /// The output it takes its value from.
    pub link: Option<OutputId>,
    /// A constant was folded into it, so that an empty link is not taken
    /// for an unused input.
    pub constant_folded_in: bool,
    /// `stack_offset`: where the SVM compiler put the value, SVM_STACK_INVALID if nowhere.
    pub stack_offset: u8,
}

impl ShaderInput {
    /// An input for this socket, not linked.
    pub fn new(socket_type: &'static SocketType) -> ShaderInput {
        ShaderInput { socket_type, link: None, constant_folded_in: false, stack_offset: 255 }
    }

    /// `name`: the name it is looked up by, such as `Surface`.
    #[allow(clippy::misnamed_getters, reason = "Cycles' `name()` is the socket's `ui_name`")]
    pub fn name(&self) -> &'static str {
        &self.socket_type.label
    }

    /// The name its node holds its value by, such as `surface`.
    pub fn socket_name(&self) -> &'static str {
        &self.socket_type.name
    }

    /// `type`.
    pub fn kind(&self) -> SocketKind {
        self.socket_type.kind
    }

    /// `flags`: the `SocketType::Flags` bits.
    pub fn flag(&self) -> u32 {
        self.socket_type.flag
    }
}

/// `ShaderOutput`: an output socket of a shader node.
#[derive(Debug, Clone, PartialEq)]
pub struct ShaderOutput {
    pub socket_type: &'static SocketType,
    /// `links`: the inputs that take their value from it.
    pub link: Vec<InputId>,
    /// `stack_offset`.
    pub stack_offset: u8,
}

impl ShaderOutput {
    /// An output for this socket, linked to nothing.
    pub fn new(socket_type: &'static SocketType) -> ShaderOutput {
        ShaderOutput { socket_type, link: Vec::new(), stack_offset: 255 }
    }

    /// `name`.
    #[allow(clippy::misnamed_getters, reason = "Cycles' `name()` is the socket's `ui_name`")]
    pub fn name(&self) -> &'static str {
        &self.socket_type.label
    }

    /// `type`.
    pub fn kind(&self) -> SocketKind {
        self.socket_type.kind
    }
}

#[cfg(test)]
mod test;
