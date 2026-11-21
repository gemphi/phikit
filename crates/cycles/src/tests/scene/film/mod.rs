// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Tests of `src/scene/film`, one file for each file there. This file
//! tests `mod.rs` and holds what the others make passes with.

mod filter;
mod layout;
mod pass;

use crate::graph::Value;
use crate::kernel::flag;
use crate::scene::*;

/// A noisy pass of the type Cycles names so, such as `PASS_DEPTH`.
fn noisy(name: &str) -> Pass {
    Pass::new(PassType::named(name).expect("in data/pass.tsv"), PassMode::Noisy)
}

fn denoised(name: &str) -> Pass {
    Pass { mode: PassMode::Denoised, ..noisy(name) }
}

fn named(name: &str, label: &str) -> Pass {
    Pass { name: label.to_string(), ..noisy(name) }
}

/// A new film written to a new device scene with `pass`.
fn updated(pass: &[Pass]) -> (Film, DeviceScene, LookupTableSet) {
    let (mut film, mut dscene, mut lookup) = (Film::new(), DeviceScene::new(), LookupTableSet::new());
    film.device_update(&mut dscene, &mut lookup, pass, false);
    (film, dscene, lookup)
}

fn int(dscene: &DeviceScene, name: &str) -> i32 {
    dscene.data.int("film", name)
}

#[test]
fn a_film_s_settings_reach_the_kernel() {
    let (_, dscene, _) = updated(&[Film::default_pass()]);
    assert_eq!((dscene.data.float("film", "exposure"), dscene.data.float("film", "pass_alpha_threshold")), (1.0, 0.5));
    // Mist over 100 units, kept as one over the depth.
    assert_eq!((dscene.data.float("film", "mist_inv_depth"), dscene.data.float("film", "mist_falloff")), (0.01, 1.0));
    assert_eq!((int(&dscene, "cryptomatte_passes"), int(&dscene, "use_approximate_shadow_catcher")), (0, 0));
    assert_eq!(int(&dscene, "denoising_pass_options_flag") as u32, flag("DENOISING_PASS_FOLLOW_REFLECTIONS") | flag("DENOISING_PASS_USE_ALBEDO_ROUGHNESS_WEIGHTING"));
}

#[test]
fn a_pass_there_is_gets_its_offset_and_one_there_is_not_is_unused() {
    let (_, dscene, _) = updated(&[Film::default_pass(), noisy("PASS_DEPTH")]);
    assert_eq!((int(&dscene, "pass_combined"), int(&dscene, "pass_depth"), int(&dscene, "pass_stride")), (0, 4, 5));
    assert_eq!((int(&dscene, "pass_normal"), int(&dscene, "pass_sample_count")), (-1, -1));
    // Cycles does not mark the mist pass unused: it stays as it was.
    assert_eq!(int(&dscene, "pass_mist"), 0);
}

#[test]
fn the_pixel_filter_goes_into_the_lookup_tables() {
    let (mut film, dscene, mut lookup) = updated(&[Film::default_pass()]);
    assert_eq!((lookup.table_count(), dscene.lookup_table.len()), (1, 1024));
    assert_eq!(dscene.data.int("tables", "filter_table_offset"), 0);
    assert_eq!(dscene.lookup_table, filter_table("box", 1.0));
    film.device_free(&mut lookup);
    assert_eq!(lookup.table_count(), 0);
}

#[test]
fn an_unchanged_film_writes_nothing_and_a_changed_one_replaces_its_table() {
    let (mut film, mut dscene, mut lookup) = updated(&[Film::default_pass()]);
    dscene.data.set_int("film", "pass_stride", 99);
    film.device_update(&mut dscene, &mut lookup, &[Film::default_pass()], false);
    assert_eq!(int(&dscene, "pass_stride"), 99);
    film.node.set("exposure", Value::Float(2.0));
    film.node.set_enum("filter_type", "gaussian");
    film.device_update(&mut dscene, &mut lookup, &[Film::default_pass()], false);
    assert_eq!((int(&dscene, "pass_stride"), dscene.data.float("film", "exposure")), (4, 2.0));
    assert_eq!(lookup.table_count(), 1);
    assert_eq!(dscene.lookup_table, filter_table("gaussian", 1.0));
}

#[test]
fn mist_of_no_depth_is_no_mist() {
    let (mut film, mut dscene, mut lookup) = updated(&[Film::default_pass()]);
    film.node.set("mist_depth", Value::Float(0.0));
    film.device_update(&mut dscene, &mut lookup, &[Film::default_pass()], false);
    assert_eq!(dscene.data.float("film", "mist_inv_depth"), 0.0);
}

#[test]
fn the_display_pass_is_the_combined_pass_unless_set() {
    let mut film = Film::new();
    assert_eq!(film.display_pass(), PassType::named("PASS_COMBINED").expect("in the table"));
    film.node.set_enum("display_pass", "depth");
    assert_eq!(film.display_pass(), PassType::named("PASS_DEPTH").expect("in the table"));
}

#[test]
fn an_aov_is_found_among_the_aovs_of_its_kind() {
    let pass = [named("PASS_AOV_COLOR", "a"), named("PASS_AOV_VALUE", "b"), named("PASS_AOV_COLOR", "c"), named("PASS_AOV_VALUE", "d")];
    assert_eq!(Film::aov_offset(&pass, "c"), Some((4, true)));
    assert_eq!(Film::aov_offset(&pass, "d"), Some((1, false)));
    assert_eq!(Film::aov_offset(&pass, "a"), Some((0, true)));
    assert_eq!(Film::aov_offset(&pass, "no such aov"), None);
}

#[test]
fn light_groups_are_numbered_in_the_order_the_passes_name_them() {
    let group = |name: &str| Pass { lightgroup: name.to_string(), ..noisy("PASS_COMBINED") };
    assert_eq!(Film::lightgroup_list(&[noisy("PASS_COMBINED"), group("key"), group("fill"), group("key")]), ["key", "fill"]);
}

#[test]
fn the_passes_tell_the_kernel_what_it_has_to_be_able_to_do() {
    assert_eq!(Film::kernel_features(&[noisy("PASS_COMBINED")]), 0);
    assert_eq!(Film::kernel_features(&[noisy("PASS_DIFFUSE_DIRECT")]), flag("KERNEL_FEATURE_LIGHT_PASSES"));
    assert_eq!(Film::kernel_features(&[noisy("PASS_AO")]), flag("KERNEL_FEATURE_AO_PASS"));
    assert_eq!(Film::kernel_features(&[denoised("PASS_COMBINED")]), flag("KERNEL_FEATURE_DENOISING"));
    assert_eq!(Film::kernel_features(&[noisy("PASS_DENOISING_NORMAL")]), flag("KERNEL_FEATURE_DENOISING"));
    // A pass composed from others is not written, so it asks for nothing.
    assert_eq!(Film::kernel_features(&[noisy("PASS_DIFFUSE")]), 0);
}
