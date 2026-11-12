// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/graph/node_type.h` (`SocketType`).

use super::Value;

/// `SocketType::Type`: what a socket holds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SocketKind {
    #[default]
    Undefined,
    Boolean,
    Float,
    Int,
    Uint,
    Uint64,
    Color,
    Vector,
    Point,
    Normal,
    Point2,
    Closure,
    String,
    Enum,
    Transform,
    Node,
    BooleanArray,
    FloatArray,
    IntArray,
    ColorArray,
    VectorArray,
    PointArray,
    NormalArray,
    Point2Array,
    StringArray,
    TransformArray,
    NodeArray,
}

impl SocketKind {
    /// The kind `data/node.tsv` names so: the `SOCKET_*` macro's name in
    /// lower case.
    pub fn named(name: &str) -> SocketKind {
        match name {
            "boolean" => SocketKind::Boolean,
            "float" => SocketKind::Float,
            "int" => SocketKind::Int,
            "uint" => SocketKind::Uint,
            "uint64" => SocketKind::Uint64,
            "color" => SocketKind::Color,
            "vector" => SocketKind::Vector,
            "point" => SocketKind::Point,
            "normal" => SocketKind::Normal,
            "point2" => SocketKind::Point2,
            "closure" => SocketKind::Closure,
            "string" => SocketKind::String,
            "enum" => SocketKind::Enum,
            "transform" => SocketKind::Transform,
            "node" => SocketKind::Node,
            "boolean_array" => SocketKind::BooleanArray,
            "float_array" => SocketKind::FloatArray,
            "int_array" => SocketKind::IntArray,
            "color_array" => SocketKind::ColorArray,
            "vector_array" => SocketKind::VectorArray,
            "point_array" => SocketKind::PointArray,
            "normal_array" => SocketKind::NormalArray,
            "point2_array" => SocketKind::Point2Array,
            "string_array" => SocketKind::StringArray,
            "transform_array" => SocketKind::TransformArray,
            "node_array" => SocketKind::NodeArray,
            _ => SocketKind::Undefined,
        }
    }
}

/// Whether a socket is a setting of the node, or an input or output of a
/// shader node that others link to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SocketDirection {
    #[default]
    Setting,
    Input,
    Output,
}

/// `SocketType`: one socket of a node type.
#[derive(Debug, Clone, PartialEq)]
pub struct SocketType {
    pub name: String,
    pub kind: SocketKind,
    pub direction: SocketDirection,
    /// `ui_name`.
    pub label: String,
    /// The default as Cycles writes it, such as `FLT_MAX`.
    pub default_text: String,
    pub default: Value,
    /// The enum table an enum socket chooses from, or the node type a node
    /// socket links to.
    pub target: String,
    /// `flags`: the `SocketType::Flags` bits, such as `LINKABLE`.
    pub flag: u32,
}

#[cfg(test)]
mod test;
