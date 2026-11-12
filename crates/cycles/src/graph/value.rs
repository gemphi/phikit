// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! What a socket holds. Cycles stores a socket's value in a member of the
//! node's C++ class (`graph/node.h`, `Node::set`, `Node::get_*`); here a
//! node keeps one [`Value`] for each socket.

use super::SocketKind;
use crate::util::Transform;

/// The value of one socket.
#[derive(Debug, Clone, PartialEq, Default)]
pub enum Value {
    /// A socket that holds nothing here yet: a closure, a link to another
    /// node.
    #[default]
    None,
    Boolean(bool),
    Int(i32),
    Uint(u32),
    Uint64(u64),
    Float(f32),
    /// A point in two dimensions.
    Float2([f32; 2]),
    /// A colour, vector, point or normal.
    Float3([f32; 3]),
    /// An enum, by its number.
    Enum(i32),
    String(String),
    BooleanArray(Vec<bool>),
    IntArray(Vec<i32>),
    FloatArray(Vec<f32>),
    Float3Array(Vec<[f32; 3]>),
    Transform(Transform),
    TransformArray(Vec<Transform>),
}

impl Value {
    /// The value a socket of `kind` starts with: the default `data/node.tsv`
    /// gives as numbers, or the kind's zero when it gives none.
    pub fn default_of(kind: SocketKind, number: &str) -> Value {
        let part: Vec<f64> = number.split(' ').filter_map(|part| part.parse().ok()).collect();
        let first = part.first().copied().unwrap_or(0.0);
        let at = |index: usize| part.get(index).copied().unwrap_or(0.0) as f32;
        match kind {
            SocketKind::Boolean => Value::Boolean(first != 0.0),
            SocketKind::Int => Value::Int(first as i32),
            SocketKind::Uint => Value::Uint(first as u32),
            // `~uint64_t(0)` is written -1: every bit set.
            SocketKind::Uint64 => Value::Uint64(number.split(' ').next().and_then(|part| part.parse::<i64>().ok()).unwrap_or(0) as u64),
            SocketKind::Float => Value::Float(first as f32),
            SocketKind::Enum => Value::Enum(first as i32),
            SocketKind::Point2 => Value::Float2([at(0), at(1)]),
            SocketKind::Color | SocketKind::Vector | SocketKind::Point | SocketKind::Normal => Value::Float3([at(0), at(1), at(2)]),
            SocketKind::String => Value::String(String::new()),
            SocketKind::BooleanArray => Value::BooleanArray(Vec::new()),
            SocketKind::IntArray => Value::IntArray(Vec::new()),
            SocketKind::FloatArray => Value::FloatArray(Vec::new()),
            SocketKind::ColorArray | SocketKind::VectorArray | SocketKind::PointArray | SocketKind::NormalArray => Value::Float3Array(Vec::new()),
            SocketKind::Transform => Value::Transform(Transform::IDENTITY),
            SocketKind::TransformArray => Value::TransformArray(Vec::new()),
            _ => Value::None,
        }
    }

    pub fn as_transform(&self) -> Option<Transform> {
        match self {
            Value::Transform(value) => Some(*value),
            _ => None,
        }
    }

    pub fn as_transform_array(&self) -> Option<&[Transform]> {
        match self {
            Value::TransformArray(value) => Some(value),
            _ => None,
        }
    }

    pub fn as_boolean(&self) -> Option<bool> {
        match self {
            Value::Boolean(value) => Some(*value),
            _ => None,
        }
    }

    /// An int, or the number of an enum.
    pub fn as_int(&self) -> Option<i32> {
        match self {
            Value::Int(value) | Value::Enum(value) => Some(*value),
            Value::Uint(value) => Some(*value as i32),
            _ => None,
        }
    }

    pub fn as_float(&self) -> Option<f32> {
        match self {
            Value::Float(value) => Some(*value),
            _ => None,
        }
    }

    pub fn as_float_array(&self) -> Option<&[f32]> {
        match self {
            Value::FloatArray(value) => Some(value),
            _ => None,
        }
    }

    pub fn as_uint(&self) -> Option<u32> {
        match self {
            Value::Uint(value) => Some(*value),
            _ => None,
        }
    }

    pub fn as_uint64(&self) -> Option<u64> {
        match self {
            Value::Uint64(value) => Some(*value),
            _ => None,
        }
    }

    pub fn as_float2(&self) -> Option<[f32; 2]> {
        match self {
            Value::Float2(value) => Some(*value),
            _ => None,
        }
    }

    pub fn as_float3(&self) -> Option<[f32; 3]> {
        match self {
            Value::Float3(value) => Some(*value),
            _ => None,
        }
    }

    pub fn as_string(&self) -> Option<&str> {
        match self {
            Value::String(value) => Some(value),
            _ => None,
        }
    }

    pub fn as_int_array(&self) -> Option<&[i32]> {
        match self {
            Value::IntArray(value) => Some(value),
            _ => None,
        }
    }

    pub fn as_boolean_array(&self) -> Option<&[bool]> {
        match self {
            Value::BooleanArray(value) => Some(value),
            _ => None,
        }
    }
}

#[cfg(test)]
mod test;
