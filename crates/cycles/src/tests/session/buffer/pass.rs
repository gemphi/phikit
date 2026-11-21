// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
use crate::scene::{Pass, PassMode, PassType};
use crate::session::*;

fn pass(name: &str, mode: PassMode) -> Pass {
    Pass { name: name.to_string(), ..Pass::new(PassType::named(name).expect("in data/pass.tsv"), mode) }
}

fn laid_out(scene_pass: &[Pass]) -> BufferParam {
    let mut param = BufferParam::full(64, 32);
    param.update_pass_from(scene_pass);
    param
}

#[test]
fn passes_sit_one_after_another_in_a_pixel() {
    let param = laid_out(&[
        pass("PASS_COMBINED", PassMode::Noisy),
        pass("PASS_DEPTH", PassMode::Noisy),
        pass("PASS_NORMAL", PassMode::Noisy),
    ]);
    let offset: Vec<i32> = param.pass.iter().map(|pass| pass.offset).collect();
    assert_eq!(offset, [0, 4, 5]);
    assert_eq!(param.pass_stride, 8, "four floats, one, three");
    let depth = PassType::named("PASS_DEPTH").expect("depth");
    assert_eq!(param.pass_offset(depth, PassMode::Noisy), 4);
    assert_eq!(param.pass_offset(depth, PassMode::Denoised), PASS_UNUSED);
    assert_eq!(param.pass_offset(PassType::NONE, PassMode::Noisy), PASS_UNUSED);
}

#[test]
fn a_pass_composed_from_others_takes_no_floats() {
    let param = laid_out(&[
        pass("PASS_DIFFUSE", PassMode::Noisy),
        pass("PASS_DIFFUSE_DIRECT", PassMode::Noisy),
        pass("PASS_DIFFUSE_INDIRECT", PassMode::Noisy),
    ]);
    assert_eq!(param.pass[0].offset, PASS_UNUSED);
    assert_eq!((param.pass[1].offset, param.pass[2].offset, param.pass_stride), (0, 3, 6));
    let diffuse = PassType::named("PASS_DIFFUSE").expect("diffuse");
    assert_eq!(param.pass_offset(diffuse, PassMode::Noisy), PASS_UNUSED);
}

#[test]
fn noisy_and_denoised_passes_of_one_type_have_their_own_offsets() {
    let param = laid_out(&[pass("PASS_COMBINED", PassMode::Noisy), pass("PASS_COMBINED", PassMode::Denoised)]);
    let combined = PassType::named("PASS_COMBINED").expect("combined");
    assert_eq!(param.pass_offset(combined, PassMode::Noisy), 0);
    assert_eq!(param.pass_offset(combined, PassMode::Denoised), 4);
    assert!(param.find_pass(combined, PassMode::Denoised).is_some());
    assert!(param.find_pass_named("PASS_COMBINED").is_some() && param.find_pass_named("nothing").is_none());
}

#[test]
fn the_combined_pass_shows_the_shadow_catcher_matte_when_there_is_one() {
    let combined = PassType::named("PASS_COMBINED").expect("combined");
    let matte = PassType::named("PASS_SHADOW_CATCHER_MATTE").expect("matte");
    let plain = laid_out(&[pass("PASS_COMBINED", PassMode::Noisy)]);
    assert_eq!(plain.actual_display_pass(combined, PassMode::Noisy).map(|pass| pass.pass_type), Some(combined));
    let caught = laid_out(&[pass("PASS_COMBINED", PassMode::Noisy), pass("PASS_SHADOW_CATCHER_MATTE", PassMode::Noisy)]);
    assert_eq!(caught.actual_display_pass(combined, PassMode::Noisy).map(|pass| pass.pass_type), Some(matte));
    assert!(plain.actual_display_pass(matte, PassMode::Noisy).is_none());
}

#[test]
fn a_change_of_passes_or_size_is_a_modified_buffer_and_more_samples_is_not() {
    let base = laid_out(&[pass("PASS_COMBINED", PassMode::Noisy)]);
    assert!(!base.modified(&BufferParam { sample_count: 64, ..base.clone() }));
    assert!(base.modified(&laid_out(&[pass("PASS_COMBINED", PassMode::Noisy), pass("PASS_DEPTH", PassMode::Noisy)])));
    assert!(base.modified(&BufferParam { exposure: 2.0, ..base.clone() }));
}
