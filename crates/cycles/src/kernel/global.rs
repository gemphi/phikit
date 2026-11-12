// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/kernel/globals.h` (`KernelGlobal`) and
//! `kernel/device/cpu/globals.h` (`KernelGlobalsCPU`): what every kernel is
//! handed to find the scene with, Cycles' `kg`. Cycles points each
//! `kernel_data_fetch` array at the device's memory; here the arrays stay
//! the `DeviceScene` the managers filled, shared by every thread.
//!
//! Cycles reads `kernel_data.cam.aperturesize` as a member of a struct.
//! Here the constant data is words laid out by `data/kernel.tsv`, so the
//! members a kernel reads for every ray are taken out of the words once,
//! when the globals are made, into the typed structs below.
use std::sync::Arc;
use super::KernelData;
use super::bvh::KernelBvh;
use super::camera::KernelCamera;
use super::flag;
use super::sample::{SamplerData, SamplingPattern};
use crate::scene::DeviceScene;
use crate::util::Transform;
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
    /// A fixed place in the pixel, or `f32::MAX` for the pixel filter's.
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
/// `KernelGlobal`.
#[derive(Debug, Clone)]
pub struct KernelGlobal {
    /// The arrays, `kernel_data_fetch(name, index)`.
    pub scene: Arc<DeviceScene>,
    /// `kernel_data.cam`.
    pub cam: KernelCamera,
    /// `kernel_data.bvh`, and the numbers a walk compares with.
    pub bvh: KernelBvh,
    pub film: KernelFilm,
    pub integrator: KernelIntegrator,
    pub background: KernelBackground,
    /// `kernel_data.tables.filter_table_offset`; a table never added is
    /// read from the start.
    pub filter_table_offset: usize,
    /// `FILTER_TABLE_SIZE`.
    pub filter_table_size: usize,
    /// `SHUTTER_TABLE_SIZE`.
    pub shutter_table_size: usize,
    pub kernel_features: u32,
    pub object: Vec<KernelObject>,
    pub shader: Vec<KernelShader>,
}
fn transform(word: Vec<f32>) -> Transform {
    Transform { x: [word[0], word[1], word[2], word[3]], y: [word[4], word[5], word[6], word[7]], z: [word[8], word[9], word[10], word[11]] }
}
fn float3(word: Vec<f32>) -> [f32; 3] {
    [word[0], word[1], word[2]]
}
fn integrator(data: &KernelData) -> KernelIntegrator {
    let i = |name: &str| data.int("integrator", name);
    let f = |name: &str| data.float("integrator", name);
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
        pixel_jitter: data.float2("integrator", "pixel_jitter"),
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
impl KernelGlobal {
    /// The globals of a scene the managers have written.
    pub fn new(scene: Arc<DeviceScene>) -> KernelGlobal {
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
        KernelGlobal {
            cam: KernelCamera::of(data),
            bvh: KernelBvh::of(data),
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
            filter_table_offset: data.int("tables", "filter_table_offset").max(0) as usize,
            filter_table_size: flag("FILTER_TABLE_SIZE") as usize,
            shutter_table_size: flag("SHUTTER_TABLE_SIZE") as usize,
            kernel_features: data.uint("data", "kernel_features"),
            object: objects,
            shader: shaders,
            scene,
        }
    }
    /// The globals of a copy of `scene`, for a caller that keeps its own.
    pub fn of(scene: &DeviceScene) -> KernelGlobal {
        KernelGlobal::new(Arc::new(scene.clone()))
    }
    /// `kernel_data_fetch(object_flag, object)`: the `SD_OBJECT_*` bits.
    pub fn object_flag(&self, object: i32) -> u32 {
        self.scene.object_flag[object as usize]
    }
}
