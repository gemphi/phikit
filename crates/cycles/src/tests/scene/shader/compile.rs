// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Tests of `scene/shader/compile.rs` and `manager.rs`: the words a shader
//! compiles to, in the layout of `kernel/svm/node_types.h`.
use crate::kernel::bit::{SD_HAS_CONSTANT_EMISSION, SD_HAS_EMISSION, SD_MIS_BACK, SD_MIS_FRONT};
use crate::kernel::{ShaderNodeType, constant, svm_node};
use crate::scene::{DeviceScene, JUMP_NODE_SIZE, ShaderManager, Surface, compile, emission_estimate};

fn node(kind: ShaderNodeType) -> u32 {
    svm_node(kind)
}

#[test]
fn a_diffuse_bsdf_compiles_to_a_weight_a_closure_and_its_data() {
    let words = compile(&Surface::Diffuse { color: [0.25, 0.5, 0.75] });
    let diffuse = constant("CLOSURE_BSDF_DIFFUSE_ID").unwrap() as u32;
    let expect = [
        // The jump node: surface, volume, displacement.
        node(ShaderNodeType::ShaderJump), 4, 17, 18,
        // NODE_CLOSURE_SET_WEIGHT and its rgb.
        node(ShaderNodeType::ClosureSetWeight), 0.25f32.to_bits(), 0.5f32.to_bits(), 0.75f32.to_bits(),
        // NODE_CLOSURE_BSDF, the closure type, no mix weight (SVM_STACK_INVALID).
        node(ShaderNodeType::ClosureBsdf), diffuse, 255,
        // SVMNodeDiffuseBsdfData: the colour, the roughness, no normal.
        0.25f32.to_bits(), 0.5f32.to_bits(), 0.75f32.to_bits(), 0.0f32.to_bits(), 255,
        node(ShaderNodeType::End),
        // The volume and displacement blocks.
        node(ShaderNodeType::End), node(ShaderNodeType::End),
    ];
    assert_eq!(words, expect);
}

#[test]
fn an_emission_compiles_to_the_weight_of_colour_times_strength() {
    let words = compile(&Surface::Emission { color: [1.0, 0.5, 0.25], strength: 4.0 });
    assert_eq!(&words[4..10], &[node(ShaderNodeType::ClosureSetWeight), 4.0f32.to_bits(), 2.0f32.to_bits(), 1.0f32.to_bits(), node(ShaderNodeType::ClosureEmission), 255]);
    assert_eq!(words.len(), 4 + 6 + 1 + 2);
}

#[test]
fn a_background_compiles_to_the_background_closure() {
    let words = compile(&Surface::Background { color: [0.5; 3], strength: 2.0 });
    assert_eq!(words[8], node(ShaderNodeType::ClosureBackground));
    assert_eq!(&words[5..8], &[1.0f32.to_bits(); 3]);
}

#[test]
fn the_jump_node_gives_where_each_block_begins() {
    for surface in [Surface::Diffuse { color: [0.5; 3] }, Surface::Emission { color: [1.0; 3], strength: 1.0 }] {
        let words = compile(&surface);
        let (surface_at, volume_at, displacement_at) = (words[1] as usize, words[2] as usize, words[3] as usize);
        assert_eq!(surface_at, JUMP_NODE_SIZE);
        assert_eq!(words[volume_at - 1], node(ShaderNodeType::End), "the surface block ends");
        assert_eq!((words[volume_at], words[displacement_at], displacement_at + 1), (node(ShaderNodeType::End), node(ShaderNodeType::End), words.len()));
    }
}

#[test]
fn the_manager_gives_a_jump_node_to_each_shader_then_their_nodes() {
    let surface = [Surface::Diffuse { color: [0.5; 3] }, Surface::Emission { color: [1.0; 3], strength: 2.0 }];
    let mut dscene = DeviceScene::new();
    ShaderManager::device_update(&mut dscene, &surface);
    let words = &dscene.svm_node;
    let (first, second) = (compile(&surface[0]), compile(&surface[1]));
    assert_eq!(words.len(), 2 * JUMP_NODE_SIZE + (first.len() - JUMP_NODE_SIZE) + (second.len() - JUMP_NODE_SIZE));
    // The first shader's surface block starts after the table.
    assert_eq!(words[1] as usize, 2 * JUMP_NODE_SIZE);
    assert_eq!(words[words[1] as usize], node(ShaderNodeType::ClosureSetWeight));
    // The second's follows the first's three blocks.
    assert_eq!(words[JUMP_NODE_SIZE + 1] as usize, 2 * JUMP_NODE_SIZE + first.len() - JUMP_NODE_SIZE);
    assert_eq!(dscene.shader.len(), 2);
}

#[test]
fn a_shader_that_emits_is_flagged_and_a_strong_one_is_sampled_as_a_light() {
    let flags = |surface: Surface| ShaderManager::flags(&surface);
    assert_eq!(flags(Surface::Diffuse { color: [0.5; 3] }), SD_HAS_CONSTANT_EMISSION);
    // Under 0.5: emits, but is not sampled as a light.
    assert_eq!(flags(Surface::Emission { color: [0.4; 3], strength: 1.0 }), SD_HAS_CONSTANT_EMISSION | SD_HAS_EMISSION);
    assert_eq!(flags(Surface::Emission { color: [0.4; 3], strength: 2.0 }), SD_HAS_CONSTANT_EMISSION | SD_HAS_EMISSION | SD_MIS_FRONT | SD_MIS_BACK);
    assert_eq!(emission_estimate(&Surface::Emission { color: [0.5, 1.0, 0.0], strength: 2.0 }), [1.0, 2.0, 0.0]);
    assert_eq!(emission_estimate(&Surface::Diffuse { color: [1.0; 3] }), [0.0; 3]);
}

#[test]
fn the_manager_writes_the_kernel_shader_of_each() {
    let mut dscene = DeviceScene::new();
    ShaderManager::device_update(&mut dscene, &[Surface::Diffuse { color: [0.5; 3] }, Surface::Emission { color: [1.0, 0.0, 0.5], strength: 3.0 }]);
    assert_eq!(dscene.shader.uint(0, "flags"), SD_HAS_CONSTANT_EMISSION);
    assert_eq!(dscene.shader.float_list(1, "constant_emission"), vec![3.0, 0.0, 1.5]);
    assert_eq!(dscene.shader.uint(1, "flags") & SD_HAS_EMISSION, SD_HAS_EMISSION);
}
