// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Triangles: `intern/cycles/kernel/geom/triangle*.h`. This file is
//! `triangle.h` (`triangle_normal` line 29, `triangle_vertices` 89,
//! `triangle_smooth_normal` 129): the corners and normals of a hit, from
//! the device scene's `tri_vindex`, `tri_verts` and `attributes_normal`.
//! `intersect.rs` is `triangle_intersect.h`.
mod intersect;
pub use intersect::triangle_intersect;
use crate::kernel::KernelGlobals;
use crate::kernel::types::{SD_OBJECT_HAS_CORNER_NORMALS, SD_OBJECT_NEGATIVE_SCALE, SD_OBJECT_TRANSFORM_APPLIED};
use crate::util::vector::{cross, is_finite, normalize, sub};
/// `triangle_vertices`: the three positions of triangle `prim` of `object`.
pub fn triangle_vertices(kg: &KernelGlobals, object: i32, prim: i32) -> [[f32; 3]; 3] {
    let scene = &*kg.scene;
    let offset = kg.object[object as usize].position_offset as usize;
    let index = scene.tri_vindex[prim as usize];
    index.map(|vertex| scene.tri_vert[offset + vertex as usize])
}
/// `triangle_normal`: the face normal in object space; the other way
/// round for an object of negative scale whose transform is applied.
pub fn triangle_normal(kg: &KernelGlobals, object: i32, prim: i32, object_flag: u32) -> [f32; 3] {
    let [v0, v1, v2] = triangle_vertices(kg, object, prim);
    let negative = object_flag & SD_OBJECT_NEGATIVE_SCALE != 0 && object_flag & SD_OBJECT_TRANSFORM_APPLIED != 0;
    match negative {
        true => normalize(cross(sub(v2, v0), sub(v1, v0))),
        false => normalize(cross(sub(v1, v0), sub(v2, v0))),
    }
}
/// `triangle_smooth_normal`: the vertex normals of the triangle,
/// interpolated at `u` and `v`; the face normal `ng` when they cancel.
pub fn triangle_smooth_normal(kg: &KernelGlobals, ng: [f32; 3], object: i32, object_flag: u32, prim: i32, u: f32, v: f32) -> [f32; 3] {
    let scene = &*kg.scene;
    let offset = kg.object[object as usize].normal_offset as usize;
    let index = scene.tri_vindex[prim as usize];
    let corner = match object_flag & SD_OBJECT_HAS_CORNER_NORMALS != 0 {
        true => [prim as usize * 3, prim as usize * 3 + 1, prim as usize * 3 + 2],
        false => index.map(|vertex| vertex as usize),
    };
    let n = corner.map(|place| scene.attribute_normal[offset + place].decode());
    let w = 1.0 - u - v;
    let sum = [0, 1, 2].map(|axis| w * n[0][axis] + u * n[1][axis] + v * n[2][axis]);
    let length = (sum[0] * sum[0] + sum[1] * sum[1] + sum[2] * sum[2]).sqrt();
    // `safe_normalize`; a zero normal is the face normal.
    match length > 0.0 && is_finite(sum) {
        true => sum.map(|part| part / length),
        false => ng,
    }
}
