// SPDX-License-Identifier: Apache-2.0
//! Tests of `src/scene/integrator`, one file for each file there. This
//! file tests `mod.rs` and holds what the others update an integrator with.

mod halton;
mod param;
mod sampling;
mod tabulated;

use crate::graph::Value;
use crate::kernel::flag;
use crate::scene::*;

/// An integrator changed by `setup`, written to a new device scene.
fn updated(setup: impl FnOnce(&mut Integrator), input: IntegratorInput) -> (Integrator, DeviceScene) {
    let mut integrator = Integrator::new();
    setup(&mut integrator);
    let mut dscene = DeviceScene::new();
    integrator.device_update(&mut dscene, &input);
    (integrator, dscene)
}

/// The same with nothing handed in from the rest of the scene.
fn plain(setup: impl FnOnce(&mut Integrator)) -> DeviceScene {
    updated(setup, IntegratorInput::default()).1
}

fn int(dscene: &DeviceScene, name: &str) -> i32 {
    dscene.data.int("integrator", name)
}

fn float(dscene: &DeviceScene, name: &str) -> f32 {
    dscene.data.float("integrator", name)
}

#[test]
fn the_sample_limit_leaves_room_to_add_an_offset_to_a_count() {
    // The scheduler adds a subset's offset to its length as 32-bit numbers.
    assert_eq!(MAX_SAMPLE, 16_777_216);
    assert!(MAX_SAMPLE.checked_add(MAX_SAMPLE).is_some());
}

#[test]
fn the_kernel_gets_one_bounce_more_than_set_but_not_for_transparency() {
    let dscene = plain(|_| {});
    let bounce = ["min_bounce", "max_bounce", "max_diffuse_bounce", "max_glossy_bounce", "max_transmission_bounce", "max_volume_bounce"].map(|name| int(&dscene, name));
    assert_eq!(bounce, [1, 8, 8, 8, 8, 8]);
    assert_eq!((int(&dscene, "transparent_min_bounce"), int(&dscene, "transparent_max_bounce")), (1, 7));
}

#[test]
fn without_indirect_light_a_path_is_one_bounce() {
    let dscene = plain(|integrator| {
        integrator.node.set("use_indirect_light", Value::Boolean(false));
    });
    assert_eq!((int(&dscene, "min_bounce"), int(&dscene, "max_bounce"), int(&dscene, "max_diffuse_bounce")), (1, 1, 8));
}

#[test]
fn a_limit_of_zero_reaches_the_kernel_as_no_limit() {
    let dscene = plain(|_| {});
    assert_eq!((float(&dscene, "sample_clamp_direct"), float(&dscene, "sample_clamp_indirect")), (f32::MAX, 30.0));
    assert_eq!((float(&dscene, "filter_glossy"), float(&dscene, "ao_bounces_distance")), (f32::MAX, f32::MAX));
    let blurred = plain(|integrator| {
        integrator.node.set("filter_glossy", Value::Float(4.0));
    });
    assert_eq!(float(&blurred, "filter_glossy"), 0.25);
}

#[test]
fn ao_bounces_count_only_with_an_ao_factor() {
    let set = |factor: f32| {
        plain(move |integrator| {
            integrator.node.set("ao_bounces", Value::Int(3));
            integrator.node.set("ao_factor", Value::Float(factor));
        })
    };
    assert_eq!((int(&set(0.0), "ao_bounces"), int(&set(0.5), "ao_bounces")), (0, 3));
}

#[test]
fn what_is_switched_off_is_filtered_out_of_the_closures() {
    assert_eq!(int(&plain(|_| {}), "filter_closures"), 0);
    let (_, dscene) = updated(
        |integrator| {
            integrator.node.set("use_diffuse", Value::Boolean(false));
            integrator.node.set("use_direct_light", Value::Boolean(false));
        },
        IntegratorInput { baking: true, ..IntegratorInput::default() },
    );
    let expect = flag("FILTER_CLOSURE_DIFFUSE") | flag("FILTER_CLOSURE_DIRECT_LIGHT") | flag("FILTER_CLOSURE_TRANSPARENT");
    assert_eq!(int(&dscene, "filter_closures") as u32, expect);
}

#[test]
fn what_the_scene_hands_in_reaches_the_kernel() {
    let input = IntegratorInput { transparent_shadow: true, has_shadow_catcher: true, exposure: 2.0, ..IntegratorInput::default() };
    let (_, dscene) = updated(
        |integrator| {
            integrator.node.set("light_sampling_threshold", Value::Float(0.5));
        },
        input,
    );
    assert_eq!((int(&dscene, "transparent_shadows"), int(&dscene, "has_shadow_catcher")), (1, 1));
    assert_eq!(float(&dscene, "light_inv_rr_threshold"), 4.0);
    assert_eq!(int(&plain(|_| {}), "transparent_shadows"), 0);
}

#[test]
fn guiding_needs_a_device_that_has_it() {
    let on = |integrator: &mut Integrator| {
        integrator.node.set("use_guiding", Value::Boolean(true));
    };
    let without = plain(on);
    assert_eq!((int(&without, "use_guiding"), int(&without, "use_surface_guiding")), (0, 0));
    let (_, with) = updated(on, IntegratorInput { has_guiding: true, ..IntegratorInput::default() });
    assert_eq!((int(&with, "use_guiding"), int(&with, "train_guiding"), int(&with, "use_surface_guiding"), int(&with, "use_volume_guiding")), (1, 1, 1, 1));
    assert_eq!(float(&with, "guiding_roughness_threshold"), 0.05 * 0.05);
}

#[test]
fn an_update_clears_the_changes_and_an_unchanged_integrator_writes_nothing() {
    let (mut integrator, mut dscene) = updated(|_| {}, IntegratorInput::default());
    assert!(!integrator.is_modified());
    dscene.data.set_int("integrator", "max_bounce", 99);
    integrator.device_update(&mut dscene, &IntegratorInput::default());
    assert_eq!(int(&dscene, "max_bounce"), 99);
    integrator.tag_object_update();
    assert!(integrator.is_modified());
    integrator.device_update(&mut dscene, &IntegratorInput::default());
    assert_eq!(int(&dscene, "max_bounce"), 8);
}

#[test]
fn the_kernel_features_follow_the_light_tree_and_additive_ao() {
    let mut integrator = Integrator::new();
    assert_eq!(integrator.kernel_features(), flag("KERNEL_FEATURE_LIGHT_TREE"));
    integrator.node.set("use_light_tree", Value::Boolean(false));
    integrator.node.set("ao_additive_factor", Value::Float(0.5));
    assert_eq!(integrator.kernel_features(), flag("KERNEL_FEATURE_AO_ADDITIVE"));
}
