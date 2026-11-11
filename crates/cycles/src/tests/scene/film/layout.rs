// SPDX-License-Identifier: Apache-2.0
use super::*;

fn offset(layout: &PassLayout) -> Vec<(&str, i32)> {
    layout.offset.iter().map(|(member, offset)| (member.as_str(), *offset)).collect()
}

#[test]
fn passes_follow_one_another_in_the_order_given() {
    let layout = Film::pass_layout(&[noisy("PASS_COMBINED"), noisy("PASS_DEPTH"), noisy("PASS_NORMAL")], false);
    assert_eq!(offset(&layout), [("pass_combined", 0), ("pass_depth", 4), ("pass_normal", 5)]);
    assert_eq!(layout.stride, 8);
}

#[test]
fn each_category_of_pass_has_its_flags() {
    let layout = Film::pass_layout(&[noisy("PASS_COMBINED"), noisy("PASS_DEPTH"), noisy("PASS_NORMAL"), noisy("PASS_DENOISING_ALBEDO")], false);
    // The combined pass is number 1; depth 32 and normal 34; albedo 64.
    assert_eq!((layout.light_pass_flag, layout.pass_flag, layout.denoising_pass_flag), (1 << 1, 1 | 1 << 2, 1));
}

#[test]
fn a_pass_composed_from_others_takes_no_floats() {
    let layout = Film::pass_layout(&[noisy("PASS_COMBINED"), noisy("PASS_DIFFUSE")], false);
    assert_eq!((offset(&layout), layout.stride), (vec![("pass_combined", 0)], 4));
}

#[test]
fn a_denoised_pass_takes_floats_but_the_kernel_is_not_told_where() {
    let layout = Film::pass_layout(&[noisy("PASS_COMBINED"), denoised("PASS_COMBINED")], false);
    assert_eq!((offset(&layout), layout.stride), (vec![("pass_combined", 0)], 8));
    // The denoised volume guiding passes are the exception: they are read.
    let volume = Film::pass_layout(&[noisy("PASS_VOLUME_SCATTER"), denoised("PASS_VOLUME_SCATTER")], false);
    assert_eq!(offset(&volume), [("pass_volume_scatter", 0), ("pass_volume_scatter_denoised", 3)]);
}

#[test]
fn a_motion_pass_needs_motion_vectors() {
    let pass = [noisy("PASS_COMBINED"), noisy("PASS_MOTION")];
    let without = Film::pass_layout(&pass, false);
    assert_eq!((offset(&without), without.stride, without.pass_flag), (vec![("pass_combined", 0)], 8, 0));
    let with = Film::pass_layout(&pass, true);
    assert_eq!((offset(&with), with.pass_flag), (vec![("pass_combined", 0), ("pass_motion", 4)], 1 << 7));
}

#[test]
fn the_kernel_is_told_where_the_first_light_group_is() {
    let group = |name: &str| Pass { lightgroup: name.to_string(), ..noisy("PASS_COMBINED") };
    let layout = Film::pass_layout(&[noisy("PASS_COMBINED"), group("key"), group("fill")], false);
    assert_eq!(offset(&layout), [("pass_combined", 0), ("pass_lightgroup", 4)]);
    // A light group's combined pass has no alpha.
    assert_eq!(layout.stride, 10);
}

#[test]
fn of_several_aovs_and_cryptomatte_passes_the_first_is_the_one_told() {
    let aov = Film::pass_layout(&[named("PASS_AOV_COLOR", "a"), named("PASS_AOV_COLOR", "b"), named("PASS_AOV_VALUE", "c")], false);
    assert_eq!((offset(&aov), aov.stride), (vec![("pass_aov_color", 0), ("pass_aov_value", 8)], 9));
    let crypto = Film::pass_layout(&[noisy("PASS_COMBINED"), noisy("PASS_CRYPTOMATTE"), noisy("PASS_CRYPTOMATTE")], false);
    assert_eq!(offset(&crypto), [("pass_combined", 0), ("pass_cryptomatte", 4)]);
}

#[test]
fn no_pass_is_no_floats() {
    assert_eq!(Film::pass_layout(&[], false), PassLayout::default());
    assert_eq!(Film::pass_layout(&[Pass::default()], false).stride, 0);
}
