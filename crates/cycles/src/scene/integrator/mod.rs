// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/scene/integrator.h` and `integrator.cpp`
//! (`Integrator`, `Integrator::device_update`, `get_kernel_features`): the
//! settings of the path tracer, and how they reach the kernel's constant
//! data. The sockets are the `integrator` type of `data/node.tsv`.
//!
//! The tabulated Sobol samples themselves (`tabulated_sobol_generate_4D`)
//! are made by `tabulated::fill_sample_pattern`, which the scene's update
//! calls; the integrator sets the size of the table.

mod halton;
mod param;
mod sampling;
mod tabulated;

pub use halton::HaltonSequence;
pub use param::GuidingParam;
pub use tabulated::{fill_sample_pattern, tabulated_sobol_generate_4d};

use super::DeviceScene;
use crate::graph::Node;
use crate::kernel::{KernelData, constant, flag};
use crate::util::{next_power_of_two, reverse_integer_bits};

/// `Integrator::MAX_SAMPLES`: the most samples a render may ask for.
pub const MAX_SAMPLE: i32 = 1 << 24;

/// What `device_update` reads from the rest of the scene. Each comes from
/// a part of the scene that is not ported yet, so it is handed in.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct IntegratorInput {
    /// A shader in use has a transparent surface that casts transparent
    /// shadows, or a volume (`integrator.cpp:250-262`).
    pub transparent_shadow: bool,
    /// `scene->bake_manager->get_baking()`.
    pub baking: bool,
    /// `device->info.has_guiding`.
    pub has_guiding: bool,
    /// `scene->film->get_exposure()`.
    pub exposure: f32,
    /// `scene->has_shadow_catcher()`.
    pub has_shadow_catcher: bool,
}

impl Default for IntegratorInput {
    fn default() -> Self {
        Self { transparent_shadow: false, baking: false, has_guiding: false, exposure: 1.0, has_shadow_catcher: false }
    }
}

/// `Integrator`.
#[derive(Debug, Clone, PartialEq)]
pub struct Integrator {
    /// The settings, by socket name.
    pub node: Node,
    pub pixel_jitter_state: HaltonSequence,
    pub pixel_jitter_frame: i32,
    shadow_catcher_needs_recalc: bool,
}

impl Default for Integrator {
    fn default() -> Self {
        Self::new()
    }
}

/// The integrator's struct of the kernel data, to write to by name.
struct Out<'a>(&'a mut KernelData);

impl Out<'_> {
    fn int(&mut self, name: &str, value: i32) {
        self.0.set_int("integrator", name, value);
    }

    fn float(&mut self, name: &str, value: f32) {
        self.0.set_float("integrator", name, value);
    }

    fn boolean(&mut self, name: &str, value: bool) {
        self.0.set_int("integrator", name, i32::from(value));
    }
}

/// Zero means "no limit" to the user; the kernel wants the limit itself.
fn limit(value: f32, scale: f32) -> f32 {
    match value == 0.0 {
        true => f32::MAX,
        false => value * scale,
    }
}

impl Integrator {
    /// An integrator with every socket at Cycles' default.
    pub fn new() -> Integrator {
        let node = Node::new("integrator").expect("data/node.tsv has the integrator");
        Integrator { node, pixel_jitter_state: HaltonSequence::default(), pixel_jitter_frame: 0, shadow_catcher_needs_recalc: true }
    }

    /// `is_modified`.
    pub fn is_modified(&self) -> bool {
        self.node.is_modified() || self.shadow_catcher_needs_recalc
    }

    /// `clear_modified`.
    pub fn clear_modified(&mut self) {
        self.node.clear_modified();
        self.shadow_catcher_needs_recalc = false;
    }

    /// `tag_update` with `OBJECT_MANAGER`: the objects changed, so whether
    /// there is a shadow catcher has to be worked out again.
    pub fn tag_object_update(&mut self) {
        self.shadow_catcher_needs_recalc = true;
    }

    /// `get_kernel_features`.
    pub fn kernel_features(&self) -> u32 {
        let additive = match self.node.float("ao_additive_factor") != 0.0 {
            true => flag("KERNEL_FEATURE_AO_ADDITIVE"),
            false => 0,
        };
        let light_tree = match self.node.boolean("use_light_tree") {
            true => flag("KERNEL_FEATURE_LIGHT_TREE"),
            false => 0,
        };
        additive | light_tree
    }

    /// `device_update`: write the settings into `kernel_data.integrator`.
    /// Nothing when no setting changed.
    pub fn device_update(&mut self, dscene: &mut DeviceScene, input: &IntegratorInput) {
        if !self.is_modified() {
            return;
        }
        let node = &self.node;
        let use_light_tree = dscene.data.int("integrator", "use_light_tree") != 0;
        let mut out = Out(&mut dscene.data);

        // One more than set, so a bounce of 0 is direct light only.
        let (min_bounce, max_bounce) = match node.boolean("use_indirect_light") {
            true => (node.int("min_bounce") + 1, node.int("max_bounce") + 1),
            false => (1, 1),
        };
        out.int("min_bounce", min_bounce);
        out.int("max_bounce", max_bounce);
        out.int("max_diffuse_bounce", node.int("max_diffuse_bounce") + 1);
        out.int("max_glossy_bounce", node.int("max_glossy_bounce") + 1);
        out.int("max_transmission_bounce", node.int("max_transmission_bounce") + 1);
        out.int("max_volume_bounce", node.int("max_volume_bounce") + 1);
        out.int("transparent_min_bounce", node.int("transparent_min_bounce") + 1);
        // Not this one: 0 transparent bounces is none at all.
        out.int("transparent_max_bounce", node.int("transparent_max_bounce"));

        let ao_factor = node.float("ao_factor");
        out.int("ao_bounces", i32::from(ao_factor != 0.0) * node.int("ao_bounces"));
        out.float("ao_bounces_distance", node.float("ao_distance"));
        out.float("ao_bounces_factor", ao_factor);
        out.float("ao_additive_factor", node.float("ao_additive_factor"));
        out.int("direct_light_sampling_type", flag("DIRECT_LIGHT_SAMPLING_MIS") as i32);

        // Only when a shader needs them: they cost time.
        out.boolean("transparent_shadows", input.transparent_shadow);
        out.boolean("volume_ray_marching", node.boolean("volume_ray_marching"));
        out.int("volume_max_steps", node.int("volume_max_steps"));
        out.boolean("caustics_reflective", node.boolean("caustics_reflective"));
        out.boolean("caustics_refractive", node.boolean("caustics_refractive"));
        let blur = node.float("filter_glossy");
        let filter_glossy = match blur == 0.0 {
            true => f32::MAX,
            false => 1.0 / blur,
        };
        out.float("filter_glossy", filter_glossy);
        out.int("filter_closures", self.filter_closure(input.baking) as i32);

        let guiding = self.guiding_param(input.has_guiding);
        out.boolean("use_guiding", guiding.enabled);
        out.boolean("train_guiding", guiding.enabled);
        out.boolean("use_surface_guiding", guiding.use_surface_guiding);
        out.boolean("use_volume_guiding", guiding.use_volume_guiding);
        out.float("surface_guiding_probability", node.float("surface_guiding_probability"));
        out.float("volume_guiding_probability", node.float("volume_guiding_probability"));
        out.boolean("use_guiding_direct_light", node.boolean("use_guiding_direct_light"));
        out.boolean("use_guiding_mis_weights", node.boolean("use_guiding_mis_weights"));
        out.int("guiding_distribution_type", guiding.distribution);
        out.int("guiding_directional_sampling_type", guiding.sampling_type);
        out.float("guiding_roughness_threshold", guiding.roughness_threshold);

        out.float("sample_clamp_direct", limit(node.float("sample_clamp_direct"), 3.0));
        out.float("sample_clamp_indirect", limit(node.float("sample_clamp_indirect"), 3.0));

        let sample_count = node.int("aa_samples").min(MAX_SAMPLE);
        let sequence = next_power_of_two((sample_count - 1) as u32);
        let (pattern, blue_noise_length) = self.sampling_pattern(sample_count);
        out.int("sampling_pattern", pattern);
        out.float("scrambling_distance", node.float("scrambling_distance"));
        out.int("sobol_index_mask", reverse_integer_bits(sequence.wrapping_sub(1)) as i32);
        out.int("blue_noise_sequence_length", blue_noise_length);
        out.int("seed", self.seed(pattern) as i32);

        // The light manager sets `use_light_tree`, and is tagged when the
        // socket changes.
        let threshold = node.float("light_sampling_threshold");
        let inv_rr_threshold = match threshold > 0.0 && !use_light_tree {
            true => input.exposure / threshold,
            false => 0.0,
        };
        out.float("light_inv_rr_threshold", inv_rr_threshold);

        // The table of tabulated Sobol samples: its size is set here; the
        // samples are generated with the samplers.
        let least = constant("MIN_TAB_SOBOL_SAMPLES").unwrap_or(0);
        let most = constant("MAX_TAB_SOBOL_SAMPLES").unwrap_or(i64::MAX);
        let sequence_size = i64::from(sequence).clamp(least, most);
        let table_size = sequence_size * constant("NUM_TAB_SOBOL_PATTERNS").unwrap_or(0) * constant("NUM_TAB_SOBOL_DIMENSIONS").unwrap_or(0);
        if pattern == flag("SAMPLING_PATTERN_TABULATED_SOBOL") as i32 && dscene.sample_pattern_lut.len() as i64 != table_size {
            out.int("tabulated_sobol_sequence_size", sequence_size as i32);
        }
        out.boolean("has_shadow_catcher", input.has_shadow_catcher);

        let jitter = self.next_pixel_jitter();
        dscene.data.set_float2("integrator", "pixel_jitter", jitter);
        self.clear_modified();
    }

    /// The closures the kernel leaves out (`filter_closures`).
    fn filter_closure(&self, baking: bool) -> u32 {
        let node = &self.node;
        // Baking is of the object itself, not of what shows through it.
        let off = [
            (!node.boolean("use_direct_light"), "FILTER_CLOSURE_DIRECT_LIGHT"),
            (!node.boolean("use_diffuse"), "FILTER_CLOSURE_DIFFUSE"),
            (!node.boolean("use_glossy"), "FILTER_CLOSURE_GLOSSY"),
            (!node.boolean("use_transmission"), "FILTER_CLOSURE_TRANSMISSION"),
            (!node.boolean("use_emission"), "FILTER_CLOSURE_EMISSION"),
            (baking, "FILTER_CLOSURE_TRANSPARENT"),
        ];
        off.into_iter().filter(|(left_out, _)| *left_out).fold(0, |mask, (_, name)| mask | flag(name))
    }
}

#[cfg(test)]
mod test;
