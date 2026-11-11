// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/graph/node_type.h`, `node_type.cpp` and
//! `node_enum.h` (`NodeType`, `NodeEnum`). The types themselves are
//! `data/node.tsv`, read once.

use std::sync::OnceLock;

use super::{SocketDirection, SocketKind, SocketType, Value};
use crate::util::row_list;

/// `NodeType::Type`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NodeKind {
    #[default]
    None,
    /// A node of a shader graph.
    Shader,
}

/// `NodeEnum`: the choices of an enum socket, each a key and its number.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct NodeEnum {
    pub name: String,
    pub entry: Vec<(String, i32)>,
}

impl NodeEnum {
    /// The number of `key`.
    pub fn number(&self, key: &str) -> Option<i32> {
        self.entry.iter().find(|(name, _)| name == key).map(|(_, number)| *number)
    }

    /// The key of `number`.
    pub fn key(&self, number: i32) -> Option<&str> {
        self.entry.iter().find(|(_, value)| *value == number).map(|(name, _)| name.as_str())
    }
}

/// `NodeType`: a kind of node and the sockets it has.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct NodeType {
    pub name: String,
    /// The C++ class, such as `Integrator`.
    pub class: String,
    /// The Cycles file the type is defined in.
    pub file: String,
    pub kind: NodeKind,
    /// The class of the type this one extends, or empty.
    pub base: String,
    /// The sockets the type declares itself; [`Self::socket_list`] has its
    /// base's too.
    pub socket: Vec<SocketType>,
    pub enum_table: Vec<NodeEnum>,
}

impl NodeType {
    /// `NodeType::find`.
    pub fn find(name: &str) -> Option<&'static NodeType> {
        registry().iter().find(|node| node.name == name)
    }

    /// The type of a C++ class.
    pub fn of_class(class: &str) -> Option<&'static NodeType> {
        registry().iter().find(|node| node.class == class)
    }

    /// Every node type, in the order of `data/node.tsv`.
    pub fn all() -> &'static [NodeType] {
        registry()
    }

    /// The type this one extends.
    pub fn base_type(&self) -> Option<&'static NodeType> {
        NodeType::of_class(&self.base).filter(|_| !self.base.is_empty())
    }

    /// Every socket of the type: its base's first, as Cycles copies them
    /// into a derived type.
    pub fn socket_list(&self) -> Vec<&SocketType> {
        let mut list = self.base_type().map(NodeType::socket_list).unwrap_or_default();
        list.extend(&self.socket);
        list
    }

    /// The enum table of this type or its base named so.
    pub fn enum_named(&self, name: &str) -> Option<&NodeEnum> {
        let own = self.enum_table.iter().find(|table| table.name == name);
        own.or_else(|| self.base_type().and_then(|base| base.enum_named(name)))
    }
}

fn registry() -> &'static [NodeType] {
    static TABLE: OnceLock<Vec<NodeType>> = OnceLock::new();
    TABLE.get_or_init(|| row_list(include_str!("../../data/node.tsv")).fold(Vec::new(), read_row))
}

/// One row of `data/node.tsv` added to the types read so far. A `class`
/// row opens a type; the rows after it fill it in.
fn read_row(mut list: Vec<NodeType>, row: Vec<&'static str>) -> Vec<NodeType> {
    let cell = |index: usize| row.get(index).copied().unwrap_or("");
    if cell(0) == "class" {
        list.push(NodeType { name: cell(1).to_string(), class: cell(2).to_string(), file: cell(3).to_string(), ..NodeType::default() });
        return list;
    }
    match (cell(0), list.last_mut()) {
        ("node", Some(node)) => {
            node.kind = match cell(2) {
                "shader" => NodeKind::Shader,
                _ => NodeKind::None,
            };
            node.base = cell(3).to_string();
        }
        ("enum", Some(node)) => {
            let entry = (cell(3).to_string(), cell(5).parse().unwrap_or(0));
            match node.enum_table.iter_mut().find(|table| table.name == cell(2)) {
                Some(table) => table.entry.push(entry),
                None => node.enum_table.push(NodeEnum { name: cell(2).to_string(), entry: vec![entry] }),
            }
        }
        ("socket", Some(node)) => {
            let kind = SocketKind::named(cell(3));
            node.socket.push(SocketType {
                name: cell(4).to_string(),
                kind,
                direction: match cell(2) {
                    "in" => SocketDirection::Input,
                    "out" => SocketDirection::Output,
                    _ => SocketDirection::Setting,
                },
                label: cell(5).to_string(),
                default_text: cell(6).to_string(),
                default: Value::default_of(kind, cell(7)),
                target: cell(8).to_string(),
                flag: cell(9).parse().unwrap_or(0),
            });
        }
        _ => {}
    }
    list
}
