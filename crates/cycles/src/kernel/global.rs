// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/kernel/device/cpu/globals.h`
//! (`KernelGlobalsCPU`): what every kernel is handed to find the scene
//! with, Cycles' `kg`.
//!
//! Cycles reads `kernel_data.cam.aperturesize` as a member of a struct. Here
//! the constant data is words laid out by `data/kernel.tsv`, so the members
//! a kernel reads for every ray are taken out of the words once, when the
//! globals are made. It grows with the kernels: so far what a camera ray
//! and a walk of the tree need.

use super::bvh::KernelBvh;
use super::camera::KernelCamera;
use super::flag;
use super::geom::ObjectField;
use crate::scene::DeviceScene;

/// `KernelGlobals`.
#[derive(Debug, Clone, PartialEq)]
pub struct KernelGlobal<'a> {
    /// The arrays, `kernel_data_fetch(name, index)`.
    pub scene: &'a DeviceScene,
    /// `kernel_data.cam`.
    pub cam: KernelCamera,
    /// `kernel_data.bvh`.
    pub bvh: KernelBvh,
    /// Where an element of `scene.object` keeps its members.
    pub object: ObjectField,
    /// `kernel_data.integrator.pixel_jitter`.
    pub pixel_jitter: [f32; 2],
    /// `kernel_data.tables.filter_table_offset`.
    pub filter_table_offset: usize,
    /// `FILTER_TABLE_SIZE`.
    pub filter_table_size: usize,
    /// `SHUTTER_TABLE_SIZE`.
    pub shutter_table_size: usize,
}

impl<'a> KernelGlobal<'a> {
    /// The globals of a scene the managers have written.
    pub fn new(scene: &'a DeviceScene) -> KernelGlobal<'a> {
        let data = &scene.data;
        KernelGlobal {
            scene,
            cam: KernelCamera::of(data),
            bvh: KernelBvh::of(data),
            object: ObjectField::new(),
            pixel_jitter: data.float2("integrator", "pixel_jitter"),
            filter_table_offset: data.int("tables", "filter_table_offset").max(0) as usize,
            filter_table_size: flag("FILTER_TABLE_SIZE") as usize,
            shutter_table_size: flag("SHUTTER_TABLE_SIZE") as usize,
        }
    }
}
