// SPDX-FileCopyrightText: 2009-2010 NVIDIA Corporation
// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/bvh/build.cpp` (`BVHBuild`:
//! `add_reference_triangles`, `add_reference_geometry`,
//! `add_reference_object`, `add_references`), adapted by Cycles from code
//! of NVIDIA Corporation: the references a tree is built from.
//!
//! Not ported: `add_reference_curves` and `add_reference_points`, which
//! wait for hair and point clouds.

use super::BvhBuild;
use crate::bvh::{BvhRange, BvhReference};
use crate::scene::{Mesh, Triangle};
use crate::util::BoundBox;

/// The references gathered so far, the box around them, and the box
/// around their doubled centres.
struct Gathered {
    reference: Vec<BvhReference>,
    bound: BoundBox,
    center: BoundBox,
}

impl Gathered {
    /// Take a reference whose box holds something.
    fn add(&mut self, reference: BvhReference) {
        if reference.bound.valid() {
            self.bound.grow_box(&reference.bound);
            self.center.grow(reference.bound.center2());
            self.reference.push(reference);
        }
    }
}

impl BvhBuild<'_> {
    /// `add_reference_triangles`: a reference for each triangle of
    /// `mesh`, which object `object` places. A triangle that moves has
    /// one box around all its motion steps, or, when the tree keeps time
    /// steps and primitives are not cut, a reference for each span of the
    /// shutter with the box it sweeps in that span.
    fn add_reference_triangle(&self, gathered: &mut Gathered, mesh: &Mesh, object: i32) {
        let Some(position) = mesh.geometry.position_attribute() else {
            return;
        };
        let prim_type = mesh.primitive_type();
        let moving = mesh.has_motion_blur() && position.has_motion();
        let whole = self.param.motion_triangle_step_count == 0 || self.param.use_spatial_split;
        let step_count = mesh.geometry.motion_step_count() as usize;
        let corner = mesh.geometry.node.int_array("triangles");
        for (index, v) in corner.chunks_exact(3).enumerate() {
            let triangle = Triangle { v: [v[0], v[1], v[2]] };
            let index = index as i32;
            let mut bound = BoundBox::EMPTY;
            match (moving, whole) {
                (false, _) => {
                    triangle.bound_grow(position.center.float3(), &mut bound);
                    if triangle.valid(position.center.float3()) {
                        gathered.add(BvhReference::new(bound, index, object, prim_type));
                    }
                }
                (true, true) => {
                    (0..position.motion_step_count()).for_each(|step| triangle.bound_grow(position.data(step).float3(), &mut bound));
                    gathered.add(BvhReference::new(bound, index, object, prim_type));
                }
                (true, false) => {
                    let span_count = self.param.motion_triangle_step_count * 2;
                    let span_size = 1.0 / span_count as f32;
                    let around = |time: f32| {
                        let mut bound = BoundBox::EMPTY;
                        triangle.motion_vert(position, step_count, time).into_iter().for_each(|point| bound.grow(point));
                        bound
                    };
                    let mut before = around(0.0);
                    for span in 1..=span_count {
                        let (time_from, time_to) = ((span - 1) as f32 * span_size, span as f32 * span_size);
                        let now = around(time_to);
                        let mut bound = before;
                        bound.grow_box(&now);
                        gathered.add(BvhReference { time_from, time_to, ..BvhReference::new(bound, index, object, prim_type) });
                        before = now;
                    }
                }
            }
        }
    }

    /// `add_references`: every reference of the tree, and the range of
    /// them all. At the top level an object rays cannot reach is left
    /// out, and an instanced object is one reference; its triangles are
    /// in its geometry's own tree.
    pub fn add_reference(&self) -> (BvhRange, Vec<BvhReference>) {
        let mut gathered = Gathered { reference: Vec::new(), bound: BoundBox::EMPTY, center: BoundBox::EMPTY };
        for (index, object) in self.object.iter().enumerate() {
            let Some(mesh) = self.mesh.get(object.geometry) else {
                continue;
            };
            let index = index as i32;
            match (self.param.top_level, object.is_traceable(&mesh.geometry), mesh.geometry.is_instanced()) {
                (true, false, _) => {}
                // `add_reference_object`. Cycles takes the object's box as it is.
                (true, true, true) => {
                    gathered.bound.grow_box(&object.bound);
                    gathered.center.grow(object.bound.center2());
                    gathered.reference.push(BvhReference::new(object.bound, -1, index, 0));
                }
                _ => self.add_reference_triangle(&mut gathered, mesh, index),
            }
        }
        // An empty mesh is a point at the origin.
        if !gathered.bound.valid() {
            gathered.bound.grow([0.0; 3]);
        }
        let range = BvhRange { bound: gathered.bound, cent_bound: gathered.center, start: 0, size: gathered.reference.len() };
        (range, gathered.reference)
    }
}

#[cfg(test)]
mod test;
