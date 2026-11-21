// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/scene/svm.cpp` (`SVMCompiler::compile`,
//! lines 1174-1230) and the `compile` of the nodes a gate 1 shader is made
//! of, `scene/shader_nodes.cpp`: `BsdfNode::compile` (2189-2215),
//! `DiffuseBsdfNode::compile` (2613), `EmissionNode::compile` (3072) and
//! `BackgroundNode::compile` (3133), each for inputs that are constants.
//! The words are those of `kernel/svm/node_types.h`: a node is its number
//! (`data/svm.tsv`) and the words of its struct, a float as its bits and an
//! unused stack offset as `SVM_STACK_INVALID` in the low byte.
//!
//! A shader starts with a jump node whose three offsets, surface, volume
//! and displacement, are where each begins in the shader's own words; each
//! of the three ends with `NODE_END`. The shader graph itself
//! (`scene/shader/graph`) is not compiled yet, so a [`Surface`] stands in
//! for the graph of one node.
use crate::kernel::{ShaderNodeType, constant, svm_node};
use crate::scene::Surface;

/// `1 + sizeof(SVMNodeShaderJump) / sizeof(int)`.
pub const JUMP_NODE_SIZE: usize = 4;

/// `SVM_STACK_INVALID`, in the first byte of a word of padded bytes.
const STACK_INVALID: u32 = 255;

fn bits(value: f32) -> u32 {
    value.to_bits()
}

/// `NODE_CLOSURE_SET_WEIGHT`: the weight the next closure node is given.
fn set_weight(out: &mut Vec<u32>, rgb: [f32; 3]) {
    out.extend([svm_node(ShaderNodeType::ClosureSetWeight), bits(rgb[0]), bits(rgb[1]), bits(rgb[2])]);
}

/// What a surface emits as `Shader::estimate_emission` finds it for a
/// graph of one node: the colour of an emission, zero for a diffuse.
pub fn emission_estimate(surface: &Surface) -> [f32; 3] {
    match surface {
        Surface::Emission { color, strength } => color.map(|channel| channel * strength),
        Surface::Background { color, strength } => color.map(|channel| channel * strength),
        Surface::Diffuse { .. } => [0.0; 3],
    }
}

/// The surface block: what `compile_type(SHADER_TYPE_SURFACE)` makes of the
/// one node, then `NODE_END`.
fn surface_block(surface: &Surface) -> Vec<u32> {
    let mut out = Vec::new();
    match surface {
        Surface::Diffuse { color } => {
            set_weight(&mut out, *color);
            let closure = constant("CLOSURE_BSDF_DIFFUSE_ID").unwrap_or(0) as u32;
            // `SVMNodeClosureBsdf`, then `SVMNodeDiffuseBsdfData`: the colour
            // (a constant, so the three floats), the roughness and no normal.
            out.extend([svm_node(ShaderNodeType::ClosureBsdf), closure, STACK_INVALID]);
            out.extend([bits(color[0]), bits(color[1]), bits(color[2]), bits(0.0), STACK_INVALID]);
        }
        Surface::Emission { color, strength } => {
            set_weight(&mut out, color.map(|channel| channel * strength));
            out.extend([svm_node(ShaderNodeType::ClosureEmission), STACK_INVALID]);
        }
        Surface::Background { color, strength } => {
            set_weight(&mut out, color.map(|channel| channel * strength));
            out.extend([svm_node(ShaderNodeType::ClosureBackground), STACK_INVALID]);
        }
    }
    out.push(svm_node(ShaderNodeType::End));
    out
}

/// `SVMCompiler::compile`: the words of one shader, jump node first. The
/// volume and displacement blocks of a surface with none are `NODE_END`.
pub fn compile(surface: &Surface) -> Vec<u32> {
    let end = svm_node(ShaderNodeType::End);
    let block = surface_block(surface);
    let volume = JUMP_NODE_SIZE + block.len();
    let displacement = volume + 1;
    let mut out = vec![svm_node(ShaderNodeType::ShaderJump), JUMP_NODE_SIZE as u32, volume as u32, displacement as u32];
    out.extend(block);
    out.extend([end, end]);
    out
}
