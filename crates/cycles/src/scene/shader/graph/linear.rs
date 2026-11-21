// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/scene/shader_nodes.cpp`
//! (`is_linear_operation` of `MixNode`, `MixColorNode`, `MixFloatNode`,
//! `MixVectorNode`, `MixVectorNonUniformNode`, `MapRangeNode`,
//! `VectorMapRangeNode`, `MathNode`, `VectorMathNode`): whether a node
//! only multiplies by, or adds, constants. A volume's attribute may be
//! sampled at random when everything between it and the output is.
//!
//! The classes that say yes outright are in `data/shader.tsv`.

use super::ShaderNode;
use crate::kernel::constant;

/// An enum socket has one of the values Cycles names so.
fn is_one_of(node: &ShaderNode, socket: &str, name: &[&str]) -> bool {
    name.iter().any(|name| constant(name) == Some(i64::from(node.node.int(socket))))
}

/// The inputs named so have no link.
fn unlinked(node: &ShaderNode, name: &[&str]) -> bool {
    name.iter().all(|name| node.input_named(name).and_then(|at| node.input[at].link).is_none())
}

/// No more than one input of the node is linked.
fn one_variable(node: &ShaderNode) -> bool {
    node.input.iter().filter(|input| input.link.is_some()).count() <= 1
}

impl ShaderNode {
    /// `is_linear_operation`.
    pub fn is_linear_operation(&self) -> bool {
        let node = self;
        let blend = ["NODE_MIX_BLEND", "NODE_MIX_ADD", "NODE_MIX_MUL", "NODE_MIX_SUB"];
        let unclamped = |socket: &str| !node.node.boolean(socket);
        match (node.says("is_linear_operation"), node.type_name()) {
            (Some("true"), _) => true,
            (Some("code"), "mix") => is_one_of(node, "mix_type", &blend) && unclamped("use_clamp") && unlinked(node, &["Factor"]),
            (Some("code"), "mix_color") => is_one_of(node, "blend_type", &blend) && unclamped("use_clamp") && unclamped("use_clamp_result") && unlinked(node, &["Factor"]),
            (Some("code"), "mix_float" | "mix_vector" | "mix_vector_non_uniform") => unclamped("use_clamp") && unlinked(node, &["Factor"]),
            (Some("code"), "map_range") => is_one_of(node, "range_type", &["NODE_MAP_RANGE_LINEAR"]) && unlinked(node, &["To Min", "To Max"]),
            (Some("code"), "vector_map_range") => is_one_of(node, "range_type", &["NODE_MAP_RANGE_LINEAR"]) && unlinked(node, &["From_Min_FLOAT3", "From_Max_FLOAT3", "To_Min_FLOAT3", "To_Max_FLOAT3"]),
            (Some("code"), "math") => match (is_one_of(node, "math_type", &["NODE_MATH_ADD", "NODE_MATH_SUBTRACT", "NODE_MATH_MULTIPLY", "NODE_MATH_MULTIPLY_ADD"]), is_one_of(node, "math_type", &["NODE_MATH_DIVIDE"])) {
                (true, _) => one_variable(node),
                (false, true) => unlinked(node, &["Value2"]),
                (false, false) => false,
            },
            (Some("code"), "vector_math") => match (is_one_of(node, "math_type", &["NODE_VECTOR_MATH_ADD", "NODE_VECTOR_MATH_SUBTRACT", "NODE_VECTOR_MATH_MULTIPLY", "NODE_VECTOR_MATH_MULTIPLY_ADD"]), is_one_of(node, "math_type", &["NODE_VECTOR_MATH_DIVIDE"])) {
                (true, _) => one_variable(node),
                (false, true) => unlinked(node, &["Vector2"]),
                (false, false) => false,
            },
            _ => false,
        }
    }
}
