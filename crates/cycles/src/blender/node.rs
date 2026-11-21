// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/blender/shader.cpp` (`add_node`, lines
//! 356-1201, for the nodes this crate compiles): the Cycles node a Blender
//! node stands for, with the settings Blender keeps in `custom1`,
//! `custom2` and `storage`. A node whose settings are in a `storage` the
//! source does not have is left out, as is one Cycles has no class for.
use super::graph::{NodeSource, NodeStorage, SocketClass};
use crate::graph::Value;
use crate::scene::ShaderGraph;

/// `SHD_GLOSSY_*` (`DNA_node_types.h`) to the closure number Cycles uses.
fn glossy_distribution(custom1: i32) -> Option<&'static str> {
    match custom1 {
        0 => Some("beckmann"),
        2 => Some("ggx"),
        3 => Some("ashikhmin_shirley"),
        4 => Some("multi_ggx"),
        _ => None,
    }
}

/// `SHD_MIXRGB_CLAMP`.
const MIXRGB_CLAMP: i32 = 2;

/// `add_node`: make the node `b_node` stands for in `graph`; its id, or
/// `None` when it has no Cycles node here.
pub fn add_node(graph: &mut ShaderGraph, b_node: &NodeSource) -> Option<usize> {
    let make = |graph: &mut ShaderGraph, name: &str| graph.create_node(name);
    let id = match b_node.idname.as_str() {
        "ShaderNodeRGB" => {
            let id = make(graph, "color")?;
            if let Some(super::graph::SocketValue::Color(value)) = b_node.output.first().and_then(|socket| socket.value) {
                set(graph, id, "value", Value::Float3(value));
            }
            id
        }
        "ShaderNodeValue" => {
            let id = make(graph, "value")?;
            if let Some(super::graph::SocketValue::Float(value)) = b_node.output.first().and_then(|socket| socket.value) {
                set(graph, id, "value", Value::Float(value));
            }
            id
        }
        "ShaderNodeMixRGB" => {
            let id = make(graph, "mix")?;
            set(graph, id, "mix_type", Value::Enum(b_node.custom1));
            set(graph, id, "use_clamp", Value::Boolean(b_node.custom2 & MIXRGB_CLAMP != 0));
            id
        }
        "ShaderNodeMix" => match &b_node.storage {
            NodeStorage::Mix { data_type: SocketClass::Vector, clamp_factor, .. } => {
                let id = make(graph, "mix_vector")?;
                set(graph, id, "use_clamp", Value::Boolean(*clamp_factor));
                id
            }
            NodeStorage::Mix { data_type: SocketClass::Color, blend_type, clamp_factor, clamp_result, .. } => {
                let id = make(graph, "mix_color")?;
                set(graph, id, "blend_type", Value::Enum(*blend_type));
                set(graph, id, "use_clamp", Value::Boolean(*clamp_factor));
                set(graph, id, "use_clamp_result", Value::Boolean(*clamp_result));
                id
            }
            NodeStorage::Mix { clamp_factor, .. } => {
                let id = make(graph, "mix_float")?;
                set(graph, id, "use_clamp", Value::Boolean(*clamp_factor));
                id
            }
            _ => return None,
        },
        "ShaderNodeSeparateXYZ" => make(graph, "separate_xyz")?,
        "ShaderNodeCombineXYZ" => make(graph, "combine_xyz")?,
        "ShaderNodeMapRange" => match b_node.storage {
            NodeStorage::MapRange { vector, interpolation_type, clamp } => {
                let id = make(graph, if vector { "vector_map_range" } else { "map_range" })?;
                set(graph, id, if vector { "use_clamp" } else { "clamp" }, Value::Boolean(clamp));
                set(graph, id, "range_type", Value::Enum(interpolation_type));
                id
            }
            _ => return None,
        },
        "ShaderNodeValToRGB" => match &b_node.storage {
            NodeStorage::ColorRamp { color, alpha, interpolate } => {
                let id = make(graph, "rgb_ramp")?;
                set(graph, id, "ramp", Value::Float3Array(color.clone()));
                set(graph, id, "ramp_alpha", Value::FloatArray(alpha.clone()));
                set(graph, id, "interpolate", Value::Boolean(*interpolate));
                id
            }
            _ => return None,
        },
        "ShaderNodeMath" => {
            let id = make(graph, "math")?;
            set(graph, id, "math_type", Value::Enum(b_node.custom1));
            set(graph, id, "use_clamp", Value::Boolean(b_node.custom2 != 0));
            id
        }
        "ShaderNodeVectorMath" => {
            let id = make(graph, "vector_math")?;
            set(graph, id, "math_type", Value::Enum(b_node.custom1));
            id
        }
        "ShaderNodeMapping" => {
            let id = make(graph, "mapping")?;
            set(graph, id, "mapping_type", Value::Enum(b_node.custom1));
            id
        }
        "ShaderNodeAddShader" => make(graph, "add_closure")?,
        "ShaderNodeMixShader" => make(graph, "mix_closure")?,
        "ShaderNodeBackground" => make(graph, "background_shader")?,
        "ShaderNodeBsdfDiffuse" => make(graph, "diffuse_bsdf")?,
        "ShaderNodeBsdfAnisotropic" => {
            let id = make(graph, "glossy_bsdf")?;
            if let Some(name) = glossy_distribution(b_node.custom1) {
                graph.node_mut(id)?.node.set_enum("distribution", name);
            }
            id
        }
        "ShaderNodeBsdfTransparent" => make(graph, "transparent_bsdf")?,
        "ShaderNodeEmission" => make(graph, "emission")?,
        // `get_tex_mapping` reads the texture mapping from the node's
        // `storage`; the identity mapping is Cycles' own default.
        "ShaderNodeTexChecker" => make(graph, "checker_texture")?,
        "ShaderNodeTexCoord" => {
            let id = make(graph, "texture_coordinate")?;
            set(graph, id, "from_dupli", Value::Boolean(b_node.custom1 != 0));
            id
        }
        _ => return None,
    };
    Some(id)
}

fn set(graph: &mut ShaderGraph, id: usize, socket: &str, value: Value) {
    if let Some(node) = graph.node_mut(id) {
        node.node.set(socket, value);
    }
}
