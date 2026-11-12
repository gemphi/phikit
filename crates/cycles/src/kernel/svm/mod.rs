// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/kernel/svm/`: the shader virtual machine.
//! A shader is a list of nodes read one after the other (`svm.h`); each is
//! its number and the words of its struct (`node_types.h`).
//!
//! | File | Ported from |
//! |---|---|
//! | `eval.rs` | `svm.h` `svm_eval_nodes` |
//! | `closure.rs` | `closure.h`: the closure, emission and background nodes |
//!
//! Only the nodes of a gate 1 shader run: the shader jump, the closure
//! weight, a diffuse BSDF, an emission, a background, and the end. A node
//! of any other number ends the shader (`eval.rs`).
mod closure;
mod eval;
pub use eval::{StackOffset, svm_eval_nodes};

use std::collections::HashMap;
use std::sync::OnceLock;

use crate::util::row_list;

/// The `ShaderNodeType`s gate 1 uses, named as Cycles names them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShaderNodeType {
    End,
    ShaderJump,
    ClosureBsdf,
    ClosureEmission,
    ClosureBackground,
    ClosureSetWeight,
}

impl ShaderNodeType {
    pub const ALL: [ShaderNodeType; 6] = [Self::End, Self::ShaderJump, Self::ClosureBsdf, Self::ClosureEmission, Self::ClosureBackground, Self::ClosureSetWeight];

    /// Cycles' name, as `data/svm.tsv` has it.
    pub fn name(self) -> &'static str {
        match self {
            Self::End => "NODE_END",
            Self::ShaderJump => "NODE_SHADER_JUMP",
            Self::ClosureBsdf => "NODE_CLOSURE_BSDF",
            Self::ClosureEmission => "NODE_CLOSURE_EMISSION",
            Self::ClosureBackground => "NODE_CLOSURE_BACKGROUND",
            Self::ClosureSetWeight => "NODE_CLOSURE_SET_WEIGHT",
        }
    }
}

fn table() -> &'static HashMap<&'static str, u32> {
    static TABLE: OnceLock<HashMap<&'static str, u32>> = OnceLock::new();
    TABLE.get_or_init(|| row_list(include_str!("../../../data/svm.tsv")).filter_map(|row| Some((*row.first()?, row.get(1)?.parse().ok()?))).collect())
}

/// The number of a node type, from `data/svm.tsv`.
pub fn svm_node(node: ShaderNodeType) -> u32 {
    table().get(node.name()).copied().unwrap_or(u32::MAX)
}

/// The node a number stands for, among the gate 1 types.
pub fn node_of(number: u32) -> Option<ShaderNodeType> {
    ShaderNodeType::ALL.into_iter().find(|node| svm_node(*node) == number)
}

#[cfg(test)]
mod test;
