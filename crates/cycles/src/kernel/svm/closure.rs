// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/kernel/svm/closure.h`: `svm_node_closure_bsdf`
//! (line 41, the `CLOSURE_BSDF_DIFFUSE_ID` case at 525), `svm_node_closure_
//! emission` (1406), `svm_node_closure_background` (1432) and
//! `svm_node_closure_set_weight` (1477), for constant inputs.
use super::eval::StackOffset;
use crate::kernel::closure::{background_setup, bsdf_diffuse_setup, emission_setup};
use crate::kernel::constant;
use crate::kernel::ShaderData;

/// The number of `CLOSURE_BSDF_DIFFUSE_ID`.
fn diffuse_id() -> u32 {
    constant("CLOSURE_BSDF_DIFFUSE_ID").unwrap_or(-1) as u32
}

/// `stack_load_float_default` for a mix weight: one on the stack, else 1.
fn mix_weight(stack: &[f32], word: u32) -> f32 {
    let offset = StackOffset::of(word);
    match offset.valid() {
        true => stack[offset.0 as usize],
        false => 1.0,
    }
}

/// `svm_node_closure_set_weight`: `rgb_to_spectrum(node.rgb)`.
pub fn closure_set_weight(node: &[u32]) -> [f32; 3] {
    [f32::from_bits(node[0]), f32::from_bits(node[1]), f32::from_bits(node[2])]
}

/// `svm_node_closure_bsdf` for a diffuse BSDF with a constant colour. The
/// words after the node are `SVMNodeDiffuseBsdfData`: the colour, the
/// roughness and the normal's stack offset. Returns the words read, or
/// `None` for a closure type that is not ported. A roughness other than
/// zero is `bsdf_oren_nayar_setup`, which is not ported.
pub fn closure_bsdf(sd: &mut ShaderData, stack: &[f32], closure_weight: [f32; 3], node: &[u32]) -> Option<usize> {
    let (closure_type, mix) = (node[0], mix_weight(stack, node[1]));
    if closure_type != diffuse_id() {
        return None;
    }
    let data = &node[2..7];
    let roughness = f32::from_bits(data[3]);
    let normal = StackOffset::of(data[4]);
    let n = match normal.valid() {
        true => [stack[normal.0 as usize], stack[normal.0 as usize + 1], stack[normal.0 as usize + 2]],
        false => sd.n,
    };
    if mix != 0.0 && roughness.abs() < 1e-4 {
        bsdf_diffuse_setup(sd, n, closure_weight.map(|channel| channel * mix));
    }
    Some(7)
}

/// `svm_node_closure_emission`.
pub fn closure_emission(sd: &mut ShaderData, stack: &[f32], closure_weight: [f32; 3], node: &[u32]) {
    let mix = mix_weight(stack, node[0]);
    if mix != 0.0 {
        emission_setup(sd, closure_weight.map(|channel| channel * mix));
    }
}

/// `svm_node_closure_background`.
pub fn closure_background(sd: &mut ShaderData, stack: &[f32], closure_weight: [f32; 3], node: &[u32]) {
    let mix = mix_weight(stack, node[0]);
    if mix != 0.0 {
        background_setup(sd, closure_weight.map(|channel| channel * mix));
    }
}
