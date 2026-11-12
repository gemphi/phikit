// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/kernel/svm/svm.h` (`svm_eval_nodes`, lines
//! 90-250): the interpreter loop of the surface shader. The shader starts
//! at its jump node in the table; the node sets the closure weight or
//! allocates a closure from it.
use super::closure::{closure_background, closure_bsdf, closure_emission, closure_set_weight};
use super::{ShaderNodeType, node_of};
use crate::kernel::globals::KernelGlobals;
use crate::kernel::types::{SHADER_MASK, SVM_STACK_SIZE, ShaderData};

/// `SVMStackOffset`: a byte; 255 is `SVM_STACK_INVALID`, not on the stack.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StackOffset(pub u8);

impl StackOffset {
    /// The offset in the first byte of a word of padded bytes.
    pub fn of(word: u32) -> StackOffset {
        StackOffset((word & 0xff) as u8)
    }
    pub fn valid(self) -> bool {
        self.0 != 255
    }
}

/// `svm_eval_nodes` for `SHADER_TYPE_SURFACE`: run the shader of
/// `sd.shader` and leave its closures in `sd`. A node that is not ported
/// ends the shader, with no closure from it.
pub fn svm_eval_nodes(kg: &KernelGlobals, sd: &mut ShaderData) {
    let node = &kg.scene.svm_node;
    let stack = [0.0f32; SVM_STACK_SIZE];
    let mut closure_weight = [0.0f32; 3];
    // The jump node of the shader: one per shader, `1 + 3` words each.
    let jump = ((sd.shader as u32 & SHADER_MASK) as usize) * 4;
    let mut offset = node[jump + 1] as usize;
    while let Some(kind) = node.get(offset).copied().and_then(node_of) {
        offset += 1;
        match kind {
            ShaderNodeType::End | ShaderNodeType::ShaderJump => return,
            ShaderNodeType::ClosureSetWeight => {
                closure_weight = closure_set_weight(&node[offset..]);
                offset += 3;
            }
            ShaderNodeType::ClosureBsdf => match closure_bsdf(sd, &stack, closure_weight, &node[offset..]) {
                Some(words) => offset += words,
                None => return,
            },
            ShaderNodeType::ClosureEmission => {
                closure_emission(sd, &stack, closure_weight, &node[offset..]);
                offset += 1;
            }
            ShaderNodeType::ClosureBackground => {
                closure_background(sd, &stack, closure_weight, &node[offset..]);
                offset += 1;
            }
        }
    }
}
