// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/scene/svm.cpp` (`SVMShaderManager::
//! device_update`, lines 60-130) and `scene/shader.cpp`
//! (`ShaderManager::device_update_common`, lines 604-720, and
//! `Shader::estimate_emission`, line 1070): every shader compiled into one
//! list of words, a jump table first, and a `KernelShader` for each.
//!
//! Of a shader's flags these are here: `SD_MIS_FRONT` and `SD_MIS_BACK`
//! (an emission over 0.5 is sampled as a light), `SD_HAS_EMISSION` and
//! `SD_HAS_CONSTANT_EMISSION`. The rest wait for the node graph.
use super::compile::{JUMP_NODE_SIZE, compile, emission_estimate};
use crate::kernel::bit::{SD_HAS_CONSTANT_EMISSION, SD_HAS_EMISSION, SD_MIS_BACK, SD_MIS_FRONT};
use crate::kernel::{ShaderNodeType, svm_node};
use crate::scene::{DeviceScene, Surface};

/// `ShaderManager`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ShaderManager;

impl ShaderManager {
    /// `get_shader_id`'s number for the shader at `place`; the shaders are
    /// numbered as they come.
    pub fn flags(surface: &Surface) -> u32 {
        let estimate = emission_estimate(surface);
        let mut flag = SD_HAS_CONSTANT_EMISSION;
        if estimate != [0.0; 3] {
            flag |= SD_HAS_EMISSION;
            // `EMISSION_SAMPLING_AUTO`: a weak emission is not sampled.
            if estimate.iter().fold(0.0f32, |most, channel| most.max(channel.abs())) > 0.5 {
                flag |= SD_MIS_FRONT | SD_MIS_BACK;
            }
        }
        flag
    }

    /// `device_update`: compile the shaders and write `svm_nodes` and
    /// `shaders` of the device scene.
    pub fn device_update(dscene: &mut DeviceScene, surface: &[Surface]) {
        let shader_node: Vec<Vec<u32>> = surface.iter().map(compile).collect();
        // The global list holds a jump table, one jump node per shader,
        // and then the nodes of every shader without its own jump node.
        let mut node_offset = surface.len() * JUMP_NODE_SIZE;
        let mut table = Vec::new();
        let mut body = Vec::new();
        for local in &shader_node {
            let shift = node_offset as u32 - JUMP_NODE_SIZE as u32;
            table.extend([svm_node(ShaderNodeType::ShaderJump), local[1] + shift, local[2] + shift, local[3] + shift]);
            body.extend_from_slice(&local[JUMP_NODE_SIZE..]);
            node_offset += local.len() - JUMP_NODE_SIZE;
        }
        table.extend(body);
        dscene.svm_node = table;
        dscene.shader.alloc(surface.len());
        for (index, surface) in surface.iter().enumerate() {
            dscene.shader.set_uint(index, "flags", Self::flags(surface));
            dscene.shader.set_float_list(index, "constant_emission", &emission_estimate(surface));
        }
    }
}
