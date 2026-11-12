// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
use crate::scene::*;

fn pass(name: &str) -> PassType {
    PassType::named(name).unwrap_or_else(|| panic!("{name} is missing from data/pass.tsv"))
}

#[test]
fn the_passes_keep_the_numbers_cycles_gives_them() {
    assert_eq!(pass("PASS_COMBINED"), PassType(1));
    assert_eq!(pass("PASS_DEPTH"), PassType(32));
    assert_eq!(pass("PASS_DENOISING_ALBEDO"), PassType(64));
    assert_eq!(pass("PASS_BAKE_PRIMITIVE"), PassType(96));
    assert_eq!(PassType::count(), 129);
    assert_eq!(pass("PASS_MIST").name(), Some("PASS_MIST"));
    assert_eq!(PassType::named("PASS_CATEGORY_LIGHT_END"), None, "a marker is not a pass");
}

#[test]
fn combined_is_four_floats_and_three_for_a_light_group() {
    let whole = PassInfo::of(pass("PASS_COMBINED"), PassMode::Noisy, false, false);
    let group = PassInfo::of(pass("PASS_COMBINED"), PassMode::Noisy, false, true);
    assert_eq!((whole.component_count, whole.use_exposure, whole.support_denoise), (4, true, true));
    assert_eq!((group.component_count, group.support_denoise), (3, false));
}

#[test]
fn diffuse_is_composed_from_its_direct_and_indirect_passes() {
    let diffuse = PassInfo::of(pass("PASS_DIFFUSE"), PassMode::Noisy, false, false);
    assert!(!diffuse.is_written && diffuse.use_compositing);
    assert_eq!(diffuse.direct_type, pass("PASS_DIFFUSE_DIRECT"));
    assert_eq!(diffuse.indirect_type, pass("PASS_DIFFUSE_INDIRECT"));
    assert_eq!(diffuse.divide_type, pass("PASS_DIFFUSE_COLOR"));
    let with_albedo = PassInfo::of(pass("PASS_DIFFUSE"), PassMode::Noisy, true, false);
    assert_eq!(with_albedo.divide_type, PassType::NONE, "with the albedo kept in, nothing is divided out");
}

#[test]
fn a_noisy_volume_scatter_pass_is_wider_than_its_denoised_one() {
    let noisy = PassInfo::of(pass("PASS_VOLUME_SCATTER"), PassMode::Noisy, false, false);
    let denoised = PassInfo::of(pass("PASS_VOLUME_SCATTER"), PassMode::Denoised, false, false);
    assert_eq!((noisy.component_count, denoised.component_count), (3, 1));
    assert!(!noisy.use_filter && noisy.support_denoise);
}

#[test]
fn the_data_passes_are_laid_out_as_cycles_lays_them() {
    let depth = PassInfo::of(pass("PASS_DEPTH"), PassMode::Noisy, false, false);
    assert_eq!((depth.component_count, depth.use_filter), (1, false));
    let motion = PassInfo::of(pass("PASS_MOTION"), PassMode::Noisy, false, false);
    assert_eq!((motion.component_count, motion.divide_type), (4, pass("PASS_MOTION_WEIGHT")));
    assert!(PassInfo::of(pass("PASS_RENDER_TIME"), PassMode::Noisy, false, false).scale_by_timer);
    assert_eq!(PassInfo::of(PassType(1000), PassMode::Noisy, false, false).component_count, 0);
}

#[test]
fn a_scene_pass_knows_whether_it_is_written() {
    assert!(Pass::new(pass("PASS_COMBINED"), PassMode::Noisy).is_written());
    assert!(!Pass::new(pass("PASS_GLOSSY"), PassMode::Noisy).is_written());
}
