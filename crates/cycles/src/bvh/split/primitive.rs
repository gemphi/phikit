// SPDX-FileCopyrightText: 2009-2010 NVIDIA Corporation
// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/bvh/split.cpp` (`BVHSpatialSplit`:
//! `split_reference`, `split_triangle_primitive`,
//! `split_triangle_reference`, `split_object_reference`), adapted by
//! Cycles from code of NVIDIA Corporation: the boxes of the two parts a
//! plane cuts a primitive into.
//!
//! Not ported: `split_curve_primitive` and `split_point_primitive`, which
//! wait for hair and point clouds.

use super::BvhSpatialSplit;
use crate::bvh::{BvhBuild, BvhReference};
use crate::scene::Mesh;
use crate::util::vector::mix;
use crate::util::{BoundBox, Transform};

/// `split_triangle_primitive`: grow `left` and `right` by the parts of a
/// triangle on each side of the plane at `pos` across `dim`. A corner on
/// the plane is in both; an edge through the plane adds the point where
/// it crosses to both.
fn split_triangle_primitive(mesh: &Mesh, transform: Option<&Transform>, prim_index: usize, dim: usize, pos: f32, left: &mut BoundBox, right: &mut BoundBox) {
    let placed = |point: [f32; 3]| transform.map(|transform| transform.point(point)).unwrap_or(point);
    let corner = mesh.triangle(prim_index).corner(mesh.geometry.position()).map(placed);
    for edge in 0..3 {
        let (from, to) = (corner[(edge + 2) % 3], corner[edge]);
        let (from_at, to_at) = (from[dim], to[dim]);
        if from_at <= pos {
            left.grow(from);
        }
        if from_at >= pos {
            right.grow(from);
        }
        if (from_at < pos && to_at > pos) || (from_at > pos && to_at < pos) {
            let crossing = mix(from, to, ((pos - from_at) / (to_at - from_at)).clamp(0.0, 1.0));
            left.grow(crossing);
            right.grow(crossing);
        }
    }
}

impl BvhSpatialSplit {
    /// `split_reference`: the two references a plane cuts one into. Each
    /// keeps the primitive, with the box of its part, which is no larger
    /// than the reference's own box on that side of the plane. An object
    /// is cut as all its triangles, placed by its transform.
    pub fn split_reference(build: &BvhBuild, reference: &BvhReference, dim: usize, pos: f32) -> (BvhReference, BvhReference) {
        let (mut left, mut right) = (BoundBox::EMPTY, BoundBox::EMPTY);
        let object = build.object.get(reference.object());
        let mesh = build.mesh_of(reference.object());
        match (mesh, reference.prim_type & build.bit.triangle != 0) {
            (Some(mesh), true) => split_triangle_primitive(mesh, None, reference.prim_index as usize, dim, pos, &mut left, &mut right),
            // `split_object_reference`.
            (Some(mesh), false) => {
                let transform = object.map(|object| object.transform());
                (0..mesh.triangle_count()).for_each(|index| split_triangle_primitive(mesh, transform.as_ref(), index, dim, pos, &mut left, &mut right));
            }
            (None, _) => {}
        }
        left.max[dim] = pos;
        right.min[dim] = pos;
        left.intersect(&reference.bound);
        right.intersect(&reference.bound);
        let part = |bound: BoundBox| BvhReference::new(bound, reference.prim_index, reference.prim_object, reference.prim_type);
        (part(left), part(right))
    }
}

#[cfg(test)]
mod test;
