// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/kernel/globals.h` (`KernelGlobals`) and
//! `kernel/device/cpu/globals.h` (`KernelGlobalsCPU`): what a CPU kernel
//! reads of the scene. Cycles points each `kernel_data_fetch` array at the
//! device's memory; here the arrays stay the `DeviceScene` the managers
//! filled, shared by every thread, and `kernel_data` is read once from its
//! words into the typed structs below, so a kernel does not look a field
//! up by name for each ray.
use std::sync::Arc;

use super::{KernelData, constant};
use super::sample::{SamplerData, SamplingPattern};
use crate::scene::DeviceScene;
use crate::util::{ProjectionTransform, Transform};

/// `kernel_data.cam`, the members the perspective and orthographic
/// cameras use (`kernel/types.h` `KernelCamera`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct KernelCamera {
    pub kind: i32,
    pub aperturesize: f32,
    pub blades: f32,
    pub bladesrotation: f32,
    pub focaldistance: f32,
    pub shuttertime: f32,
    pub num_motion_steps: i32,
    pub have_perspective_motion: bool,
    pub cameratoworld: Transform,
    pub rastertocamera: ProjectionTransform,
    pub dx: [f32; 3],
    pub dy: [f32; 3],
    pub differential_scale: f32,
    pub nearclip: f32,
    pub cliplength: f32,
    pub width: f32,
    pub height: f32,
    pub inv_aperture_ratio: f32,
    pub is_inside_volume: bool,
}

/// `kernel_data.film`: where the combined pass is, and the stride.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct KernelFilm {
    pub pass_stride: i32,
    pub pass_combined: i32,
    pub pass_sample_count: i32,
    pub exposure: f32,
}

/// `kernel_data.integrator`, the members the path and the sampler read.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct KernelIntegrator {
    pub min_bounce: i32,
    pub max_bounce: i32,
    pub max_diffuse_bounce: i32,
    pub max_glossy_bounce: i32,
    pub max_transmission_bounce: i32,
    pub transparent_min_bounce: i32,
    pub transparent_max_bounce: i32,
    pub filter_glossy: f32,
    pub sample_clamp_direct: f32,
    pub sample_clamp_indirect: f32,
    pub filter_closures: i32,
    pub pixel_jitter: [f32; 2],
    pub sampler: SamplerData,
}

/// `kernel_data.background`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct KernelBackground {
    pub surface_shader: i32,
    pub transparent: bool,
    pub use_mis: bool,
}

/// One object, as `kernel_data_fetch(objects, i)` gives it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct KernelObject {
    pub tfm: Transform,
    pub itfm: Transform,
    pub position_offset: i32,
    pub normal_offset: i32,
}

/// One shader, as `kernel_data_fetch(shaders, i)` gives it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct KernelShader {
    pub flags: u32,
    pub constant_emission: [f32; 3],
}

/// `KernelGlobals`.
#[derive(Debug, Clone)]
pub struct KernelGlobals {
    pub scene: Arc<DeviceScene>,
    pub cam: KernelCamera,
    pub film: KernelFilm,
    pub integrator: KernelIntegrator,
    pub background: KernelBackground,
    pub tables_filter_offset: i32,
    pub bvh_root: i32,
    pub kernel_features: u32,
    pub object: Vec<KernelObject>,
    pub shader: Vec<KernelShader>,
}

fn transform(word: Vec<f32>) -> Transform {
    Transform { x: [word[0], word[1], word[2], word[3]], y: [word[4], word[5], word[6], word[7]], z: [word[8], word[9], word[10], word[11]] }
}

fn projection(word: Vec<f32>) -> ProjectionTransform {
    let row = |r: usize| [word[r * 4], word[r * 4 + 1], word[r * 4 + 2], word[r * 4 + 3]];
    ProjectionTransform { x: row(0), y: row(1), z: row(2), w: row(3) }
}

fn float3(word: Vec<f32>) -> [f32; 3] {
    [word[0], word[1], word[2]]
}

fn camera(data: &KernelData) -> KernelCamera {
    let f = |name: &str| data.float("cam", name);
    let i = |name: &str| data.int("cam", name);
    KernelCamera {
        kind: i("type"),
        aperturesize: f("aperturesize"),
        blades: f("blades"),
        bladesrotation: f("bladesrotation"),
        focaldistance: f("focaldistance"),
        shuttertime: f("shuttertime"),
        num_motion_steps: i("num_motion_steps"),
        have_perspective_motion: i("have_perspective_motion") != 0,
        cameratoworld: transform(data.float_list("cam", "cameratoworld")),
        rastertocamera: projection(data.float_list("cam", "rastertocamera")),
        dx: float3(data.float_list("cam", "dx")),
        dy: float3(data.float_list("cam", "dy")),
        differential_scale: f("differential_scale"),
        nearclip: f("nearclip"),
        cliplength: f("cliplength"),
        width: f("width"),
        height: f("height"),
        inv_aperture_ratio: f("inv_aperture_ratio"),
        is_inside_volume: i("is_inside_volume") != 0,
    }
}

fn integrator(data: &KernelData) -> KernelIntegrator {
    let i = |name: &str| data.int("integrator", name);
    let f = |name: &str| data.float("integrator", name);
    let jitter = data.float2("integrator", "pixel_jitter");
    KernelIntegrator {
        min_bounce: i("min_bounce"),
        max_bounce: i("max_bounce"),
        max_diffuse_bounce: i("max_diffuse_bounce"),
        max_glossy_bounce: i("max_glossy_bounce"),
        max_transmission_bounce: i("max_transmission_bounce"),
        transparent_min_bounce: i("transparent_min_bounce"),
        transparent_max_bounce: i("transparent_max_bounce"),
        filter_glossy: f("filter_glossy"),
        sample_clamp_direct: f("sample_clamp_direct"),
        sample_clamp_indirect: f("sample_clamp_indirect"),
        filter_closures: i("filter_closures"),
        pixel_jitter: jitter,
        sampler: SamplerData {
            pattern: SamplingPattern::of(i("sampling_pattern")),
            seed: i("seed") as u32,
            sequence_length: i("blue_noise_sequence_length") as u32,
            index_mask: i("sobol_index_mask") as u32,
            sequence_size: i("tabulated_sobol_sequence_size") as u32,
            scrambling_distance: f("scrambling_distance"),
        },
    }
}

impl KernelGlobals {
    /// The scene as a kernel reads it.
    pub fn new(scene: Arc<DeviceScene>) -> KernelGlobals {
        let data = &scene.data;
        let object = &scene.object;
        let objects = (0..object.len())
            .map(|index| KernelObject {
                tfm: transform(object.float_list(index, "tfm")),
                itfm: transform(object.float_list(index, "itfm")),
                position_offset: object.int(index, "position_offset"),
                normal_offset: object.int(index, "normal_offset"),
            })
            .collect();
        let shaders = (0..scene.shader.len())
            .map(|index| KernelShader { flags: scene.shader.uint(index, "flags"), constant_emission: float3(scene.shader.float_list(index, "constant_emission")) })
            .collect();
        KernelGlobals {
            cam: camera(data),
            film: KernelFilm {
                pass_stride: data.int("film", "pass_stride"),
                pass_combined: data.int("film", "pass_combined"),
                pass_sample_count: data.int("film", "pass_sample_count"),
                exposure: data.float("film", "exposure"),
            },
            integrator: integrator(data),
            background: KernelBackground {
                surface_shader: data.int("background", "surface_shader"),
                transparent: data.int("background", "transparent") != 0,
                use_mis: data.int("background", "use_mis") != 0,
            },
            tables_filter_offset: data.int("tables", "filter_table_offset"),
            bvh_root: data.int("bvh", "root"),
            kernel_features: data.uint("data", "kernel_features"),
            object: objects,
            shader: shaders,
            scene,
        }
    }

    /// `kernel/util/lookup_table.h` `lookup_table_read`: the table at
    /// `offset`, `size` long, read at `x` in 0..1 with linear interpolation.
    pub fn lookup_table_read(&self, x: f32, offset: i32, size: i32) -> f32 {
        let x = x.clamp(0.0, 1.0) * (size - 1) as f32;
        let index = (x as i32).min(size - 1);
        let next = (index + 1).min(size - 1);
        let t = x - index as f32;
        let table = &self.scene.lookup_table;
        let data0 = table[(index + offset) as usize];
        match t == 0.0 {
            true => data0,
            false => (1.0 - t) * data0 + t * table[(next + offset) as usize],
        }
    }

    /// `kernel_data_fetch(object_flag, object)`: the `SD_OBJECT_*` bits.
    pub fn object_flag(&self, object: i32) -> u32 {
        self.scene.object_flag[object as usize]
    }

    /// `CAMERA_PERSPECTIVE`, `CAMERA_ORTHOGRAPHIC`, by the table.
    pub fn camera_kind(name: &str) -> i32 {
        constant(name).unwrap_or(-1) as i32
    }
}
