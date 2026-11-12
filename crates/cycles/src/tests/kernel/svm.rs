// SPDX-License-Identifier: Apache-2.0
//! Tests of `kernel/svm/`: the node numbers and the interpreter, over the
//! words `scene/shader/compile.rs` writes.
use crate::kernel::svm::*;
use crate::kernel::types::*;
use crate::kernel::{KernelGlobals, constant};
use crate::scene::{DeviceScene, ShaderManager, Surface};
use std::sync::Arc;

fn globals(surface: &[Surface]) -> KernelGlobals {
    let mut dscene = DeviceScene::new();
    ShaderManager::device_update(&mut dscene, surface);
    KernelGlobals::new(Arc::new(dscene))
}

fn run(kg: &KernelGlobals, shader: i32) -> ShaderData {
    let mut sd = ShaderData { shader, ..ShaderData::default() };
    svm_eval_nodes(kg, &mut sd);
    sd
}

#[test]
fn the_node_types_have_the_numbers_of_the_template() {
    let numbers: Vec<u32> = ShaderNodeType::ALL.iter().map(|node| svm_node(*node)).collect();
    assert_eq!(numbers, [0, 1, 2, 3, 4, 5]);
    for node in ShaderNodeType::ALL {
        assert_eq!(node_of(svm_node(node)), Some(node));
    }
    assert_eq!(node_of(99), None);
}

#[test]
fn a_diffuse_shader_makes_one_bsdf_of_its_colour() {
    let kg = globals(&[Surface::Diffuse { color: [0.2, 0.4, 0.6] }]);
    let mut sd = ShaderData { shader: 0, n: [0.0, 0.0, 1.0], ..ShaderData::default() };
    svm_eval_nodes(&kg, &mut sd);
    assert_eq!((sd.num_closure, sd.flag & (SD_BSDF | SD_EMISSION)), (1, SD_BSDF));
    assert_eq!(sd.closure[0].weight, [0.2, 0.4, 0.6]);
    assert_eq!(sd.closure[0].kind, constant("CLOSURE_BSDF_DIFFUSE_ID").unwrap() as i32);
    // No normal on the stack: the shading normal.
    assert_eq!(sd.closure[0].n, [0.0, 0.0, 1.0]);
}

#[test]
fn an_emission_shader_emits_colour_times_strength_and_has_no_bsdf() {
    let kg = globals(&[Surface::Emission { color: [1.0, 0.5, 0.25], strength: 4.0 }]);
    let sd = run(&kg, 0);
    assert_eq!(sd.closure_emission_background, [4.0, 2.0, 1.0]);
    assert_eq!((sd.flag & SD_EMISSION, sd.flag & SD_BSDF, sd.num_closure), (SD_EMISSION, 0, 0));
}

#[test]
fn a_background_shader_leaves_its_colour_as_the_emission() {
    let kg = globals(&[Surface::Background { color: [0.1, 0.2, 0.3], strength: 2.0 }]);
    let sd = run(&kg, 0);
    assert_eq!(sd.closure_emission_background, [0.1f32 * 2.0, 0.2 * 2.0, 0.3 * 2.0]);
}

#[test]
fn each_shader_is_found_by_its_place_in_the_jump_table() {
    let kg = globals(&[Surface::Diffuse { color: [0.5; 3] }, Surface::Emission { color: [1.0; 3], strength: 2.0 }, Surface::Diffuse { color: [0.25; 3] }]);
    assert_eq!(run(&kg, 0).closure[0].weight, [0.5; 3]);
    assert_eq!(run(&kg, 1).closure_emission_background, [2.0; 3]);
    assert_eq!(run(&kg, 2).closure[0].weight, [0.25; 3]);
    // The bits that only flag a shader are masked off.
    assert_eq!(run(&kg, 2 | SHADER_SMOOTH_NORMAL as i32).closure[0].weight, [0.25; 3]);
    // The table has one jump node of four words for each shader.
    assert_eq!(kg.scene.svm_node[0], svm_node(ShaderNodeType::ShaderJump));
    assert_eq!(kg.scene.svm_node[4], svm_node(ShaderNodeType::ShaderJump));
}

#[test]
fn a_shader_ends_with_the_end_node_and_its_other_blocks_are_empty() {
    let kg = globals(&[Surface::Diffuse { color: [0.5; 3] }]);
    let words = &kg.scene.svm_node;
    let end = svm_node(ShaderNodeType::End);
    // surface, volume and displacement offsets of the jump node.
    let (surface, volume, displacement) = (words[1] as usize, words[2] as usize, words[3] as usize);
    assert_eq!(surface, 4);
    assert_eq!((words[volume], words[displacement]), (end, end));
    assert_eq!(words[volume - 1], end);
}

#[test]
fn a_shader_run_only_for_its_emission_allocates_no_closure() {
    let kg = globals(&[Surface::Diffuse { color: [0.5; 3] }]);
    let mut sd = ShaderData { shader: 0, num_closure_left: 0, ..ShaderData::default() };
    svm_eval_nodes(&kg, &mut sd);
    assert_eq!((sd.num_closure, sd.flag & SD_BSDF), (0, 0));
}

#[test]
fn the_stack_offset_of_a_padded_word_is_its_first_byte() {
    assert_eq!(StackOffset::of(255).0, 255);
    assert!(!StackOffset::of(255).valid() && StackOffset::of(3).valid());
    assert_eq!(StackOffset::of(0x1234_5603).0, 3);
}
