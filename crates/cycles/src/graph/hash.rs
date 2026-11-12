// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/graph/node.cpp` (`Node::hash`): a node's
//! type and the value of every socket, taken into an MD5 digest, by which
//! a change is told.
//!
//! Cycles appends the bytes of each value as it holds them; here they are
//! the same bytes, least first. A string is its text here, where Cycles
//! appends the address of its interned copy; a closure and a link to
//! another node add nothing.

use super::{Node, SocketDirection, Value};
use crate::util::Md5Hash;

fn append_float(md5: &mut Md5Hash, value: &[f32]) {
    value.iter().for_each(|part| md5.append(&part.to_le_bytes()));
}

impl Node {
    /// `hash`.
    pub fn hash(&self, md5: &mut Md5Hash) {
        md5.append_str(&self.node_type().name);
        for (socket, value) in self.socket_list().filter(|(socket, _)| socket.direction != SocketDirection::Output) {
            md5.append_str(&socket.name);
            match value {
                Value::None => {}
                Value::Boolean(value) => md5.append(&[u8::from(*value)]),
                Value::Int(value) | Value::Enum(value) => md5.append(&value.to_le_bytes()),
                Value::Uint(value) => md5.append(&value.to_le_bytes()),
                Value::Uint64(value) => md5.append(&value.to_le_bytes()),
                Value::Float(value) => append_float(md5, &[*value]),
                Value::Float2(value) => append_float(md5, value),
                Value::Float3(value) => append_float(md5, value),
                Value::String(value) => md5.append_str(value),
                Value::BooleanArray(value) => value.iter().for_each(|part| md5.append(&[u8::from(*part)])),
                Value::IntArray(value) => value.iter().for_each(|part| md5.append(&part.to_le_bytes())),
                Value::FloatArray(value) => append_float(md5, value),
                Value::Float3Array(value) => value.iter().for_each(|part| append_float(md5, part)),
                Value::Transform(value) => append_float(md5, &value.word()),
                Value::TransformArray(value) => value.iter().for_each(|part| append_float(md5, &part.word())),
            }
        }
    }
}

#[cfg(test)]
mod test;
