//! What a Geometry Nodes modifier feeds its group: the input values saved
//! on the modifier, the group interface's defaults behind them, and the
//! mesh's own `sharp_edge` attribute the Is Edge Smooth node reads.
//!
//! Where the values live moved in 5.1. Before, `NodesModifierSettings
//! .properties` is a flat `IDP_GROUP` keyed by socket identifier —
//! `Input_1` the value, `Input_1_use_attribute` whether an attribute
//! named by `Input_1_attribute_name` replaces it
//! (`MOD_nodes.cc`, `id_property_create_from_socket`). 5.1 on, Blender
//! keeps them as RNA system properties on `ModifierData.system_properties`:
//! an `inputs` group of one group per identifier with `value`, `type`
//! (`GeometryNodesInputType`: 1 value, 2 attribute) and `attribute_name`
//! (`geometry_nodes_execute.cc`, `init_socket_cpp_value`). A file written
//! by 5.2 still carries the old group as `settings_legacy`, read only when
//! the new one is absent, as `MOD_nodes.cc`'s `blend_read` does.
//!
//! An input absent from both takes the interface socket's own default
//! (`bNodeTreeInterfaceSocket.socket_data`, a `bNodeSocketValueFloat` /
//! `Boolean` / `Int`), which is what Blender falls back to
//! (`GeometryNodesInputType::Fallback`).

use super::nodes_field::BInput;
use crate::blend::file::types::*;
use crate::blend::idprop::{BIdGroup, BIdValue};
use std::collections::HashMap;

/// `GeometryNodesInputType::Attribute`.
const INPUT_ATTRIBUTE: i32 = 2;
/// `NODE_INTERFACE_SOCKET_INPUT`.
const INTERFACE_INPUT: u32 = 1;
/// `NodeTreeInterfaceItemType::Socket`.
const ITEM_SOCKET: u8 = 1;
/// Edge domain (`AttrDomain::Edge`).
const DOMAIN_EDGE: i8 = 1;

impl BlendFile {
    /// The group's input values by socket identifier, from the modifier's
    /// settings over the interface defaults. `None` when an input is fed
    /// by an attribute, which the evaluator does not model.
    pub(crate) fn nodes_inputs(&self, md: &BlendBlock, tree: u64, dna: &DnaType) -> Option<HashMap<String, BInput>> {
        let mut inputs = self.interface_defaults(tree, dna);
        let system = self.ptr(md, self.off("ModifierData", "*system_properties", dna));
        match self.id_group_at(system, dna, 0).and_then(|g| match g.get("inputs") {
            Some(BIdValue::Group(inputs)) => Some(inputs.clone()),
            _ => None,
        }) {
            Some(group) => {
                for name in group.names() {
                    let Some(BIdValue::Group(one)) = group.get(name) else { continue };
                    if one.get("type").and_then(BIdValue::as_int) == Some(INPUT_ATTRIBUTE) {
                        return None;
                    }
                    if let Some(v) = one.get("value").and_then(|v| input_value(v, inputs.get(name))) {
                        inputs.insert(name.clone(), v);
                    }
                }
            }
            None => {
                let settings = self.off("NodesModifierData", "settings_legacy", dna).or_else(|| self.off("NodesModifierData", "settings", dna))?;
                let props = self.ptr(md, Some(settings + self.off("NodesModifierSettings", "*properties", dna)?));
                let group = self.id_group_at(props, dna, 0).unwrap_or_default();
                legacy_inputs(&group, &mut inputs)?;
            }
        }
        Some(inputs)
    }

    /// `bNodeTree.tree_interface`'s input sockets with a stored default,
    /// panels walked (`bNodeTreeInterfacePanel.items_array`).
    fn interface_defaults(&self, tree: u64, dna: &DnaType) -> HashMap<String, BInput> {
        let mut out = HashMap::new();
        let Some(nt) = self.block_at(tree) else { return out };
        let Some(root) = self.off("bNodeTree", "tree_interface", dna).zip(self.off("bNodeTreeInterface", "root_panel", dna)).map(|(a, b)| a + b) else {
            return out;
        };
        self.interface_panel(&nt.data, root, dna, &mut out, 0);
        out
    }

    /// One panel's items: a socket's default, or a nested panel's.
    fn interface_panel(&self, data: &[u8], at: usize, dna: &DnaType, out: &mut HashMap<String, BInput>, depth: u32) {
        let ps = self.header.pointer_size as usize;
        let le = self.header.little_endian;
        let (Some(arr_o), Some(num_o)) = (self.off("bNodeTreeInterfacePanel", "**items_array", dna), self.off("bNodeTreeInterfacePanel", "items_num", dna)) else {
            return;
        };
        let Some(count) = data.get(at + num_o..at + num_o + 4).map(|b| self.read_i32(b)) else { return };
        let Some(array) = self.block_at(Self::read_ptr(data, at + arr_o, ps, le)) else { return };
        for i in 0..count.clamp(0, 4096) as usize {
            let Some(item) = self.block_at(Self::read_ptr(&array.data, i * ps, ps, le)) else { continue };
            let kind = self.off("bNodeTreeInterfaceItem", "item_type", dna).and_then(|o| item.data.get(o)).copied();
            match kind {
                Some(ITEM_SOCKET) => {
                    if let Some((id, v)) = self.interface_socket(item, dna) {
                        out.insert(id, v);
                    }
                }
                _ if depth < 16 => self.interface_panel(&item.data, 0, dna, out, depth + 1),
                _ => {}
            }
        }
    }

    /// A `bNodeTreeInterfaceSocket` input's identifier and default.
    fn interface_socket(&self, item: &BlendBlock, dna: &DnaType) -> Option<(String, BInput)> {
        let st = "bNodeTreeInterfaceSocket";
        if self.int_field(item, st, "flag", dna) & INTERFACE_INPUT == 0 {
            return None;
        }
        let id = self.cstring_at(item, st, "*identifier", dna)?;
        let kind = self.cstring_at(item, st, "*socket_type", dna)?;
        let value = self.block_at(self.ptr(item, self.off(st, "*socket_data", dna)))?;
        let v = match kind.as_str() {
            "NodeSocketFloat" => BInput::Float(self.float_field(value, "bNodeSocketValueFloat", "value", dna, 0.0)),
            "NodeSocketBool" => BInput::Bool(self.int_field(value, "bNodeSocketValueBoolean", "value", dna) != 0),
            "NodeSocketInt" => BInput::Int(self.int_field(value, "bNodeSocketValueInt", "value", dna) as i32),
            _ => return None,
        };
        Some((id, v))
    }

    /// The mesh's `sharp_edge` edges as vertex pairs, `a < b` (the edge
    /// domain bool attribute; a 4.x named layer or 5.x attribute storage).
    pub(crate) fn mesh_sharp_edges(&self, me: &BlendBlock, dna: &DnaType, verts: usize) -> Vec<(u32, u32)> {
        self.scoped(me, || {
            let Some(flags) = self.crease_layer(me, dna, "sharp_edge", DOMAIN_EDGE, 1) else { return Vec::new() };
            self.mesh_edges(me, dna)
                .into_iter()
                .zip(flags.iter())
                .filter(|&((a, b), &s)| s != 0 && crate::blend::mesh::crease::edge_verts_are_good(a, b, verts))
                .map(|((a, b), _)| (a.min(b), a.max(b)))
                .collect()
        })
    }
}

/// A saved value as the input's type (the default says which).
fn input_value(v: &BIdValue, like: Option<&BInput>) -> Option<BInput> {
    Some(match (like, v) {
        (Some(BInput::Bool(_)), v) => BInput::Bool(v.as_bool()?),
        (Some(BInput::Int(_)), v) => BInput::Int(v.as_int()?),
        (_, BIdValue::Bool(b)) => BInput::Bool(*b),
        (_, BIdValue::Int(i)) => BInput::Int(*i),
        (_, BIdValue::Float(f)) => BInput::Float(*f as f32),
        _ => return None,
    })
}

/// The flat pre-5.1 group: `<id>`, `<id>_use_attribute`.
fn legacy_inputs(group: &BIdGroup, inputs: &mut HashMap<String, BInput>) -> Option<()> {
    for name in group.names() {
        if name.ends_with("_use_attribute") {
            if group.get(name).and_then(BIdValue::as_bool) == Some(true) {
                return None;
            }
            continue;
        }
        if name.ends_with("_attribute_name") {
            continue;
        }
        if let Some(v) = group.get(name).and_then(|v| input_value(v, inputs.get(name))) {
            inputs.insert(name.clone(), v);
        }
    }
    Some(())
}
