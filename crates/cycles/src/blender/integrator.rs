// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/blender/sync.cpp`
//! (`BlenderSync::sync_integrator`, lines 355-545; `sync_film`, 609-668)
//! and `blender/addon/properties.py` (the defaults of the settings): the
//! scene's Cycles settings into the integrator and the film. The file holds
//! only what differs from the default, so a setting that is not in
//! `SceneSource::cycles` has the default given here. Denoising, adaptive
//! sampling, guiding, the light tree, volumes and the fast GI approximation
//! are not synced: none changes what gate 1 draws, only its noise.
use super::source::SceneSource;
use crate::graph::Value;
use crate::kernel::constant;
use crate::scene::Scene;

/// The setting `name` of `scene["cycles"]`, or its default.
pub fn setting(source: &SceneSource, name: &str, default: f64) -> f64 {
    source.cycles.get(name).copied().unwrap_or(default)
}

/// `scene.cycles.samples`: the render's sample count (4096 by default).
pub fn sample_count(source: &SceneSource) -> i32 {
    setting(source, "samples", 4096.0) as i32
}

/// The sampling pattern as `sync_integrator` picks it for a background
/// render (lines 435-460): Automatic is `BLUE_NOISE_PURE`, Classic
/// (tabulated Sobol) and Blue-Noise stay, and without the developer UI
/// anything else is `BLUE_NOISE_PURE`.
fn sampling_pattern(requested: i32) -> &'static str {
    let is = |name: &str| constant(name) == Some(i64::from(requested));
    match () {
        _ if is("SAMPLING_PATTERN_TABULATED_SOBOL") => "tabulated_sobol",
        _ => "blue_noise_pure",
    }
}

/// `sync_integrator`.
pub fn sync_integrator(source: &SceneSource, scene: &mut Scene) {
    let int = |name: &str, default: f64| Value::Int(setting(source, name, default) as i32);
    let node = &mut scene.integrator.node;
    node.set("min_bounce", int("min_light_bounces", 0.0));
    node.set("max_bounce", int("max_bounces", 12.0));
    node.set("max_diffuse_bounce", int("diffuse_bounces", 4.0));
    node.set("max_glossy_bounce", int("glossy_bounces", 4.0));
    node.set("max_transmission_bounce", int("transmission_bounces", 12.0));
    node.set("max_volume_bounce", int("volume_bounces", 0.0));
    node.set("transparent_min_bounce", int("min_transparent_bounces", 0.0));
    node.set("transparent_max_bounce", int("transparent_max_bounces", 8.0));
    node.set("caustics_reflective", Value::Boolean(setting(source, "caustics_reflective", 1.0) != 0.0));
    node.set("caustics_refractive", Value::Boolean(setting(source, "caustics_refractive", 1.0) != 0.0));
    node.set("filter_glossy", Value::Float(setting(source, "blur_glossy", 1.0) as f32));
    node.set("seed", int("seed", 0.0));
    node.set("sample_clamp_direct", Value::Float(setting(source, "sample_clamp_direct", 0.0) as f32));
    node.set("sample_clamp_indirect", Value::Float(setting(source, "sample_clamp_indirect", 0.0) as f32));
    node.set("aa_samples", Value::Int(sample_count(source)));
    node.set_enum("sampling_pattern", sampling_pattern(setting(source, "sampling_pattern", 5.0) as i32));
    // Not the scrambling distance of the classic pattern: 1.
    node.set("scrambling_distance", Value::Float(1.0));
    node.set("use_adaptive_sampling", Value::Boolean(false));
}

/// `sync_film`: the exposure, and the pixel filter (Blackman-Harris of 1.5
/// pixels unless the scene says otherwise; a box filter is 1).
pub fn sync_film(source: &SceneSource, scene: &mut Scene) {
    let node = &mut scene.film.node;
    node.set("exposure", Value::Float(setting(source, "film_exposure", 1.0) as f32));
    let kind = match setting(source, "pixel_filter_type", 2.0) as i32 {
        0 => "box",
        1 => "gaussian",
        _ => "blackman_harris",
    };
    node.set_enum("filter_type", kind);
    let width = match kind {
        "box" => 1.0,
        _ => setting(source, "filter_width", 1.5) as f32,
    };
    node.set("filter_width", Value::Float(width));
    scene.background.transparent = source.film_transparent;
}
