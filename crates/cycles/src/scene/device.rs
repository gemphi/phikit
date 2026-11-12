// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/scene/devicescene.h` (`DeviceScene`): the
//! scene as a device holds it, flat arrays and the constant data. The
//! scene's managers fill it in `Scene::device_update`.
//!
//! The arrays Cycles has are `data/array.tsv`, written by `tool/kernel.py`
//! from `kernel/data_arrays.h`. An array becomes a field here when the
//! manager that fills it is ported; so far the integrator's, the lookup
//! tables', the camera's, the objects', the meshes', the tree's and the shaders'. A field has Cycles'
//! name made singular: `objects` is `object`.

use crate::kernel::{KernelData, RecordArray};
use crate::util::{DecomposedTransform, PackedNormal, Transform, row_list};

/// `DeviceScene`.
#[derive(Debug, Clone, PartialEq)]
pub struct DeviceScene {
    /// `bvh_nodes`: the inner nodes of the scene's tree, then those of
    /// each geometry's own. See `bvh::PackedBvh` for these eight.
    pub bvh_node: Vec<[i32; 4]>,
    /// `bvh_leaf_nodes`.
    pub bvh_leaf_node: Vec<[i32; 4]>,
    /// `object_node`: where each object's geometry has its tree.
    pub object_node: Vec<i32>,
    pub prim_type: Vec<i32>,
    pub prim_visibility: Vec<u32>,
    pub prim_index: Vec<i32>,
    pub prim_object: Vec<i32>,
    pub prim_time: Vec<[f32; 2]>,
    /// `objects`: a `KernelObject` for each object.
    pub object: RecordArray,
    /// `object_motion_pass`: for motion vectors, each object's
    /// transform before and after the frame.
    pub object_motion_pass: Vec<Transform>,
    /// `object_motion`: for motion blur, the steps of every object that
    /// moves, one object's after another's.
    pub object_motion: Vec<DecomposedTransform>,
    /// `camera_motion`: for motion blur, the places a moving camera has
    /// over the shutter.
    pub camera_motion: Vec<DecomposedTransform>,
    /// `object_flag`: the `SD_OBJECT_*` bits of each object.
    pub object_flag: Vec<u32>,
    /// `object_prim_offset`: each object's first primitive.
    pub object_prim_offset: Vec<u32>,
    /// `tri_shader`: each triangle's shader and whether it is smooth.
    pub tri_shader: Vec<u32>,
    /// `tri_vindex`: each triangle's three vertices, within its mesh.
    pub tri_vindex: Vec<[u32; 3]>,
    /// `tri_verts`: every mesh's positions.
    pub tri_vert: Vec<[f32; 3]>,
    /// `attributes_map`: an `AttributeMap` entry for each attribute of
    /// each geometry.
    pub attribute_map: RecordArray,
    /// `attributes_float` and the others: the attributes' values, by type.
    pub attribute_float: Vec<f32>,
    pub attribute_float2: Vec<[f32; 2]>,
    pub attribute_float3: Vec<[f32; 3]>,
    pub attribute_float4: Vec<[f32; 4]>,
    pub attribute_uchar4: Vec<[u8; 4]>,
    pub attribute_normal: Vec<PackedNormal>,
    /// `svm_nodes`: the jump table of the shaders and then their nodes.
    pub svm_node: Vec<u32>,
    /// `shaders`: a `KernelShader` for each shader.
    pub shader: RecordArray,
    /// `lookup_table`: every lookup table, one after another.
    pub lookup_table: Vec<f32>,
    /// `sample_pattern_lut`: the tabulated Sobol samples.
    pub sample_pattern_lut: Vec<f32>,
    /// `data`: the constant data.
    pub data: KernelData,
}

impl Default for DeviceScene {
    fn default() -> Self {
        Self::new()
    }
}

impl DeviceScene {
    /// A device scene with nothing in its arrays.
    pub fn new() -> DeviceScene {
        DeviceScene {
            bvh_node: Vec::new(),
            bvh_leaf_node: Vec::new(),
            object_node: Vec::new(),
            prim_type: Vec::new(),
            prim_visibility: Vec::new(),
            prim_index: Vec::new(),
            prim_object: Vec::new(),
            prim_time: Vec::new(),
            object: RecordArray::of("KernelObject").expect("data/record.tsv has KernelObject"),
            object_motion_pass: Vec::new(),
            object_motion: Vec::new(),
            camera_motion: Vec::new(),
            object_flag: Vec::new(),
            object_prim_offset: Vec::new(),
            tri_shader: Vec::new(),
            tri_vindex: Vec::new(),
            tri_vert: Vec::new(),
            attribute_map: RecordArray::of("AttributeMap").expect("data/record.tsv has AttributeMap"),
            attribute_float: Vec::new(),
            attribute_float2: Vec::new(),
            attribute_float3: Vec::new(),
            attribute_float4: Vec::new(),
            attribute_uchar4: Vec::new(),
            attribute_normal: Vec::new(),
            svm_node: Vec::new(),
            shader: RecordArray::of("KernelShader").expect("data/record.tsv has KernelShader"),
            lookup_table: Vec::new(),
            sample_pattern_lut: Vec::new(),
            data: KernelData::new(),
        }
    }

    /// Every array of Cycles' device scene, in its order: the name, the
    /// element's C++ type, and whether a kernel may write it.
    pub fn array_list() -> Vec<(&'static str, &'static str, bool)> {
        let row = row_list(include_str!("../../data/array.tsv"));
        row.filter_map(|row| Some((*row.first()?, *row.get(1)?, row.get(2) == Some(&"1")))).collect()
    }
}

#[cfg(test)]
mod test;
