// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/blender/mesh.cpp` (`mesh_triangle`, `
//! create_mesh`, `attr_create_uv_map`): the triangles of a Blender mesh
//! into a Cycles `Mesh`: positions, triangles with their shader and
//! smooth flag, the UV map, and the vertex normals Cycles works out
//! (`Mesh::add_vertex_normals`). Subdivision, vertex colours, generated
//! coordinates and the motion steps are not ported yet.
use super::source::MeshSource;
use crate::graph::Value;
use crate::scene::{AttributeData, AttributeStandard, Mesh};

/// `create_mesh`: a `Mesh` for `source` drawn with `shader`, the
/// shaders of its material slots in order (`find_used_shaders`,
/// `blender/geometry.cpp` 63-95), which become its `used_shaders`. Each
/// triangle keeps its slot, clamped to that list (`mesh.cpp` 713,
/// `clamp(material_index, 0, used_shaders.size() - 1)`); the device
/// finds the shader through `used_shaders` (`Mesh::pack_shaders`).
pub fn sync_mesh(source: &MeshSource, shader: &[usize]) -> Mesh {
    let mut mesh = Mesh::new();
    let triangle_count = source.triangle.len();
    mesh.resize_mesh(source.vertex.len(), triangle_count);
    mesh.geometry.position_mut().copy_from_slice(&source.vertex);
    let corner: Vec<i32> = source.triangle.iter().flatten().map(|&vertex| vertex as i32).collect();
    let last = shader.len().saturating_sub(1) as i32;
    let triangle_shader: Vec<i32> = (0..triangle_count).map(|index| i32::from(source.slot.get(index).copied().unwrap_or(0)).clamp(0, last)).collect();
    let smooth: Vec<bool> = (0..triangle_count).map(|index| source.smooth.get(index).copied().unwrap_or(false)).collect();
    let used = shader.to_vec();
    let node = &mut mesh.geometry.node;
    node.set("triangles", Value::IntArray(corner));
    node.set("shader", Value::IntArray(triangle_shader));
    node.set("smooth", Value::BooleanArray(smooth));
    mesh.geometry.set_used_shader(used);
    if source.uv.len() == source.vertex.len() && !source.uv.is_empty() {
        // `attr_create_uv_map`: a UV for each corner of each triangle.
        let uv: Vec<[f32; 2]> = source.triangle.iter().flatten().map(|&vertex| source.uv[vertex as usize]).collect();
        mesh.add_attribute(AttributeStandard::of("UV")).center = AttributeData::Float2(uv);
    }
    mesh.add_vertex_normal();
    mesh.compute_bound();
    mesh
}
