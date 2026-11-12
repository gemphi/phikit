// SPDX-FileCopyrightText: 2009-2010 NVIDIA Corporation
// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/bvh/bvh2.cpp` (`BVH2`: `refit`,
//! `refit_nodes`, `refit_node`, `refit_primitives`), adapted by Cycles
//! from code of NVIDIA Corporation: a packed tree kept as it is, with
//! every box fitted again to vertices that moved. Quicker than building,
//! and right only while the mesh has the triangles it was built with.
//!
//! Not ported: the boxes of curves and points.

use super::Bvh2;
use crate::scene::{Mesh, Object};
use crate::util::BoundBox;

impl Bvh2 {
    /// `refit`: for the tree of one geometry, never the top level.
    pub fn refit(&mut self, object: &[Object], mesh: &[Mesh]) {
        debug_assert!(!self.param.top_level);
        self.pack_primitive(object);
        // `refit_nodes`.
        self.refit_node(0, self.pack.root_index == -1, object, mesh);
    }

    /// `refit_node`: fit the node at `at` and say what box it has now and
    /// which rays see it.
    fn refit_node(&mut self, at: usize, leaf: bool, object: &[Object], mesh: &[Mesh]) -> (BoundBox, u32) {
        match leaf {
            true => {
                let data = self.pack.leaf_node.get(at).copied().unwrap_or([0; 4]);
                let (bound, visibility) = self.refit_primitive(data[0], data[1], object, mesh);
                self.pack.leaf_node.get_mut(at).into_iter().for_each(|node| *node = [data[0], data[1], visibility as i32, data[3]]);
                (bound, visibility)
            }
            false => {
                let data = self.pack.node.get(at).copied().unwrap_or([0; 4]);
                // A leaf's place is written with every bit turned.
                let below = |child: i32| ((child ^ (child >> 31)) as usize, child < 0);
                let (first, second) = (below(data[2]), below(data[3]));
                let first = self.refit_node(first.0, first.1, object, mesh);
                let second = self.refit_node(second.0, second.1, object, mesh);
                self.pack_aligned_node(at, [&first.0, &second.0], [data[2], data[3]], [first.1, second.1]);
                (first.0.merge(&second.0), first.1 | second.1)
            }
        }
    }

    /// `refit_primitives`: the box around the primitives from `start` up
    /// to `end`, at every motion step, and the rays that see them.
    fn refit_primitive(&self, start: i32, end: i32, object: &[Object], mesh: &[Mesh]) -> (BoundBox, u32) {
        let (mut bound, mut visibility) = (BoundBox::EMPTY, 0);
        for prim in usize::try_from(start).unwrap_or(0)..usize::try_from(end).unwrap_or(0) {
            let index = self.pack.prim_index.get(prim).copied().unwrap_or(-1);
            let object = self.pack.prim_object.get(prim).and_then(|at| object.get(usize::try_from(*at).ok()?));
            let Some(object) = object else {
                continue;
            };
            match (index, mesh.get(object.geometry)) {
                // An instanced object.
                (-1, _) => bound.grow_box(&object.bound),
                (index, Some(mesh)) => {
                    let offset = match self.param.top_level {
                        true => mesh.geometry.prim_offset,
                        false => 0,
                    };
                    let triangle = mesh.triangle((index as usize).saturating_sub(offset));
                    let position = mesh.geometry.position_attribute();
                    let step_count = match (mesh.geometry.use_motion_blur(), position) {
                        (true, Some(position)) => position.motion_step_count(),
                        _ => 1,
                    };
                    for step in 0..step_count {
                        position.into_iter().for_each(|position| triangle.bound_grow(position.data(step).float3(), &mut bound));
                    }
                }
                (_, None) => {}
            }
            visibility |= object.visibility_for_tracing();
        }
        (bound, visibility)
    }
}

#[cfg(test)]
mod test;
