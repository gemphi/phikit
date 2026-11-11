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

/// `create_mesh`: a `Mesh` for `source`, each triangle on the shader
/// `shader[slot]`. A slot with no shader is the first one, as Blender
/// draws a face whose slot is gone with the first material; with no
/// shaders at all the triangles use shader 0.
pub fn sync_mesh(source: &MeshSource, shader: &[usize]) -> Mesh {
    let mut mesh = Mesh::new();
    let triangle_count = source.triangle.len();
    mesh.resize_mesh(source.vertex.len(), triangle_count);
    mesh.geometry.position_mut().copy_from_slice(&source.vertex);
    let corner: Vec<i32> = source.triangle.iter().flatten().map(|&vertex| vertex as i32).collect();
    let on_shader = |slot: u16| shader.get(usize::from(slot)).or_else(|| shader.first()).copied().unwrap_or(0);
    let triangle_shader: Vec<i32> = (0..triangle_count).map(|index| on_shader(source.slot.get(index).copied().unwrap_or(0)) as i32).collect();
    let smooth: Vec<bool> = (0..triangle_count).map(|index| source.smooth.get(index).copied().unwrap_or(false)).collect();
    let mut used: Vec<usize> = triangle_shader.iter().map(|&shader| shader as usize).collect();
    used.sort_unstable();
    used.dedup();
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
