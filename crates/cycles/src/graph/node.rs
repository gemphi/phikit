// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/graph/node.h` and `node.cpp` (`Node`): a
//! value for each socket of a node type, and which of them changed since
//! the device was last updated.

use super::{NodeType, SocketType, Value};
use crate::util::Transform;

/// `Node`.
#[derive(Debug, Clone, PartialEq)]
pub struct Node {
    pub name: String,
    node_type: &'static NodeType,
    /// The sockets, the base type's first, each with its value.
    socket: Vec<(&'static SocketType, Value)>,
    /// `socket_modified`: a bit for each socket set since the last
    /// `clear_modified`. A new node has every bit set.
    modified: u64,
}

impl Node {
    /// A node of the type named so, every socket at its default. `None`
    /// when `data/node.tsv` has no such type.
    pub fn new(type_name: &str) -> Option<Node> {
        let node_type = NodeType::find(type_name)?;
        let socket = node_type.socket_list().into_iter().map(|socket| (socket, socket.default.clone())).collect();
        Some(Node { name: String::new(), node_type, socket, modified: u64::MAX })
    }

    pub fn node_type(&self) -> &'static NodeType {
        self.node_type
    }

    fn index(&self, name: &str) -> Option<usize> {
        let found = self.socket.iter().position(|(socket, _)| socket.name == name);
        debug_assert!(found.is_some(), "node type {} has no socket {name}", self.node_type.name);
        found
    }

    /// The bit of the socket at `index`. Past 64 sockets they share the
    /// last bit, as in Cycles.
    fn bit(index: usize) -> u64 {
        1 << index.min(63)
    }

    pub fn has(&self, name: &str) -> bool {
        self.socket.iter().any(|(socket, _)| socket.name == name)
    }

    /// Every socket with its value, the base type's first.
    pub fn socket_list(&self) -> impl Iterator<Item = (&'static SocketType, &Value)> {
        self.socket.iter().map(|(socket, value)| (*socket, value))
    }

    /// `copy_value`: the socket `name` takes the value `other` has in its
    /// socket `other_name`. Nothing is marked modified.
    pub fn copy_value(&mut self, name: &str, other: &Node, other_name: &str) {
        match (self.index(name), other.get(other_name)) {
            (Some(index), Some(value)) => self.socket[index].1 = value.clone(),
            _ => {}
        }
    }

    /// The value of the socket named so.
    pub fn get(&self, name: &str) -> Option<&Value> {
        self.index(name).map(|index| &self.socket[index].1)
    }

    /// `set`: give the socket a value. `false`, and nothing is marked
    /// modified, when it has that value already or there is no such socket.
    pub fn set(&mut self, name: &str, value: Value) -> bool {
        let Some(index) = self.index(name) else {
            return false;
        };
        if self.socket[index].1 == value {
            return false;
        }
        self.socket[index].1 = value;
        self.modified |= Self::bit(index);
        true
    }

    /// Set an enum socket by the key of its choice, such as `sobol_burley`.
    pub fn set_enum(&mut self, name: &str, key: &str) -> bool {
        let table = self.index(name).map(|index| self.socket[index].0.target.as_str());
        let number = table.and_then(|table| self.node_type.enum_named(table)).and_then(|table| table.number(key));
        number.is_some_and(|number| self.set(name, Value::Enum(number)))
    }

    /// A boolean socket's value; `false` when there is none.
    pub fn boolean(&self, name: &str) -> bool {
        self.get(name).and_then(Value::as_boolean).unwrap_or(false)
    }

    /// An int or enum socket's value; zero when there is none.
    pub fn int(&self, name: &str) -> i32 {
        self.get(name).and_then(Value::as_int).unwrap_or(0)
    }

    /// A float socket's value; zero when there is none.
    pub fn float(&self, name: &str) -> f32 {
        self.get(name).and_then(Value::as_float).unwrap_or(0.0)
    }

    /// A uint socket's value; zero when there is none.
    pub fn uint(&self, name: &str) -> u32 {
        self.get(name).and_then(Value::as_uint).unwrap_or(0)
    }

    /// A 64-bit socket's value; zero when there is none.
    pub fn uint64(&self, name: &str) -> u64 {
        self.get(name).and_then(Value::as_uint64).unwrap_or(0)
    }

    /// A two-float socket's value; zeros when there is none.
    pub fn float2(&self, name: &str) -> [f32; 2] {
        self.get(name).and_then(Value::as_float2).unwrap_or_default()
    }

    /// A colour, vector, point or normal socket's value; zeros when there is none.
    pub fn float3(&self, name: &str) -> [f32; 3] {
        self.get(name).and_then(Value::as_float3).unwrap_or_default()
    }

    /// A string socket's value; empty when there is none.
    pub fn string(&self, name: &str) -> &str {
        self.get(name).and_then(Value::as_string).unwrap_or("")
    }

    /// An int array socket's value; empty when there is none.
    pub fn int_array(&self, name: &str) -> &[i32] {
        self.get(name).and_then(Value::as_int_array).unwrap_or(&[])
    }

    /// A boolean array socket's value; empty when there is none.
    pub fn boolean_array(&self, name: &str) -> &[bool] {
        self.get(name).and_then(Value::as_boolean_array).unwrap_or(&[])
    }

    /// A float array socket's value; empty when there is none.
    pub fn float_array(&self, name: &str) -> &[f32] {
        self.get(name).and_then(Value::as_float_array).unwrap_or(&[])
    }

    /// A transform socket's value; the identity when there is none.
    pub fn transform(&self, name: &str) -> Transform {
        self.get(name).and_then(Value::as_transform).unwrap_or_default()
    }

    /// A transform array socket's value; empty when there is none.
    pub fn transform_array(&self, name: &str) -> &[Transform] {
        self.get(name).and_then(Value::as_transform_array).unwrap_or(&[])
    }

    /// `is_modified`: a socket was set since the last `clear_modified`.
    pub fn is_modified(&self) -> bool {
        self.modified != 0
    }

    /// `<socket>_is_modified`.
    pub fn socket_is_modified(&self, name: &str) -> bool {
        self.index(name).is_some_and(|index| self.modified & Self::bit(index) != 0)
    }

    /// `tag_modified`: take every socket as changed.
    pub fn tag_modified(&mut self) {
        self.modified = u64::MAX;
    }

    /// `tag_<socket>_modified`.
    pub fn tag_socket_modified(&mut self, name: &str) {
        self.modified |= self.index(name).map(Self::bit).unwrap_or(0);
    }

    /// `clear_modified`: the device has what the node holds.
    pub fn clear_modified(&mut self) {
        self.modified = 0;
    }
}
