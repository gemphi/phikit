// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Tests of `blender/integrator.rs`: the scene's Cycles settings into the
//! integrator and the film, with the defaults of `properties.py` for what
//! the file leaves out.
use crate::blender::*;
use crate::scene::Scene;

fn synced(setting: &[(&str, f64)]) -> Scene {
    let cycles = setting.iter().map(|(name, value)| (name.to_string(), *value)).collect();
    let source = SceneSource { width: 8, height: 8, world_strength: 1.0, cycles, ..SceneSource::default() };
    sync_scene(&source)
}

fn integrator(scene: &mut Scene) -> crate::scene::DeviceScene {
    scene.device_update()
}

#[test]
fn a_file_with_no_settings_has_blenders_defaults() {
    let mut scene = synced(&[]);
    let dscene = integrator(&mut scene);
    let int = |name: &str| dscene.data.int("integrator", name);
    // One over what the property says, so that 0 is direct light only.
    assert_eq!((int("min_bounce"), int("max_bounce"), int("max_diffuse_bounce"), int("max_glossy_bounce"), int("max_transmission_bounce")), (1, 13, 5, 5, 13));
    assert_eq!((int("transparent_min_bounce"), int("transparent_max_bounce")), (1, 8));
    // Filter glossy 1, no clamp.
    assert_eq!(dscene.data.float("integrator", "filter_glossy"), 1.0);
    assert_eq!(dscene.data.float("integrator", "sample_clamp_indirect"), f32::MAX);
    // 4096 samples; Automatic is the pure blue-noise pattern.
    assert_eq!(sample_count(&SceneSource::default()), 4096);
    assert_eq!(int("sampling_pattern"), 2);
    assert_eq!(int("blue_noise_sequence_length"), 4096);
}

#[test]
fn what_the_file_holds_is_used_and_the_classic_pattern_is_kept() {
    let mut scene = synced(&[("samples", 25.0), ("max_bounces", 3.0), ("diffuse_bounces", 1.0), ("sampling_pattern", 1.0), ("seed", 7.0), ("sample_clamp_indirect", 2.0), ("blur_glossy", 0.0)]);
    let dscene = integrator(&mut scene);
    let int = |name: &str| dscene.data.int("integrator", name);
    assert_eq!((int("max_bounce"), int("max_diffuse_bounce"), int("sampling_pattern"), int("seed")), (4, 2, 1, 7));
    // A blur of zero is no filter at all.
    assert_eq!(dscene.data.float("integrator", "filter_glossy"), f32::MAX);
    // The clamp is three times the setting.
    assert_eq!(dscene.data.float("integrator", "sample_clamp_indirect"), 6.0);
    // 25 samples: a sequence of 32, kept at the least of 256 for the table.
    assert_eq!(int("tabulated_sobol_sequence_size"), 256);
    assert_eq!(dscene.sample_pattern_lut.len(), 256 * 256 * 4);
}

#[test]
fn a_pattern_other_than_classic_is_the_blue_noise_one() {
    for (requested, expect) in [(5.0, 2), (2.0, 2), (0.0, 2), (3.0, 2), (4.0, 2)] {
        let mut scene = synced(&[("sampling_pattern", requested)]);
        assert_eq!(integrator(&mut scene).data.int("integrator", "sampling_pattern"), expect, "pattern {requested}");
    }
}

#[test]
fn the_film_has_a_blackman_harris_filter_of_one_and_a_half_pixels() {
    let scene = synced(&[]);
    assert_eq!(scene.film.node.float("filter_width"), 1.5);
    assert_eq!(scene.film.node.int("filter_type"), 2);
    assert_eq!(scene.film.node.float("exposure"), 1.0);
    let mut gaussian = synced(&[("pixel_filter_type", 1.0), ("filter_width", 2.0), ("film_exposure", 2.0)]);
    assert_eq!((gaussian.film.node.int("filter_type"), gaussian.film.node.float("filter_width"), gaussian.film.node.float("exposure")), (1, 2.0, 2.0));
    // A box filter is always a pixel.
    let boxed = synced(&[("pixel_filter_type", 0.0), ("filter_width", 3.0)]);
    assert_eq!(boxed.film.node.float("filter_width"), 1.0);
    let _ = integrator(&mut gaussian);
}

#[test]
fn a_transparent_film_makes_a_transparent_background() {
    let source = SceneSource { width: 8, height: 8, world_strength: 1.0, film_transparent: true, ..SceneSource::default() };
    let mut scene = sync_scene(&source);
    assert!(scene.background.transparent);
    assert_eq!(scene.device_update().data.int("background", "transparent"), 1);
}
