// SPDX-FileCopyrightText: 2009-2010 NVIDIA Corporation
// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/bvh/bvh2.cpp` (`BVH2::pack_instances`),
//! adapted by Cycles from code of NVIDIA Corporation: the scene's arrays
//! hold the top level's tree, then the tree of each geometry that has its
//! own, once however many objects place it. Each object is told where its
//! geometry's tree starts.

use std::collections::HashMap;

use super::Bvh2;
use crate::bvh::{BvhLayout, BvhLimit};
use crate::scene::{Mesh, Object};

/// The tree of a mesh that has its own.
fn own(mesh: &Mesh, layout: BvhLayout) -> Option<&Bvh2> {
    mesh.geometry.bvh.as_ref().filter(|_| mesh.geometry.need_build_bvh(layout))
}

impl Bvh2 {
    /// `pack_instances`: make room for the top level's `node_size` nodes
    /// and `leaf_node_size` leaves, and copy each geometry's tree in
    /// after them, moved to where it now is.
    pub(super) fn pack_instance(&mut self, node_size: usize, leaf_node_size: usize, object: &[Object], mesh: &[Mesh]) {
        let layout = self.param.bvh_layout;
        // A primitive of geometry with its transform applied is in the
        // top level's tree: count it from the start of every mesh's.
        for (index, at) in self.pack.prim_index.iter_mut().zip(&self.pack.prim_object) {
            let offset = usize::try_from(*at).ok().and_then(|at| mesh.get(object.get(at)?.geometry)).map(|mesh| mesh.geometry.prim_offset);
            match (*index != -1, offset) {
                (true, Some(offset)) => *index += offset as i32,
                _ => {}
            }
        }

        let total = |size: fn(&Bvh2) -> usize| mesh.iter().filter_map(|mesh| own(mesh, layout)).map(size).sum::<usize>();
        let prim_size = self.pack.prim_index.len() + total(|bvh| bvh.pack.prim_index.len());
        let (mut prim_offset, mut node_offset, mut leaf_offset) = (self.pack.prim_index.len(), node_size, leaf_node_size);
        self.pack.prim_index.resize(prim_size, 0);
        self.pack.prim_type.resize(prim_size, 0);
        self.pack.prim_object.resize(prim_size, 0);
        self.pack.prim_visibility.resize(prim_size, 0);
        self.pack.prim_time.resize(usize::from(self.param.use_motion_step()) * prim_size, [0.0; 2]);
        self.pack.node.resize(node_size + total(|bvh| bvh.pack.node.len()), [0; 4]);
        self.pack.leaf_node.resize(leaf_node_size + total(|bvh| bvh.pack.leaf_node.len()), [0; 4]);
        self.pack.object_node = vec![0; object.len()];

        let mut placed: HashMap<usize, i32> = HashMap::new();
        for (index, object) in object.iter().enumerate() {
            // Geometry with no tree of its own is in the top level's.
            let Some(bvh) = mesh.get(object.geometry).and_then(|mesh| own(mesh, layout)) else {
                continue;
            };
            if let Some(at) = placed.get(&object.geometry) {
                self.pack.object_node[index] = *at;
                continue;
            }
            let at = match bvh.pack.root_index == -1 {
                true => -(leaf_offset as i32) - 1,
                false => node_offset as i32,
            };
            self.pack.object_node[index] = at;
            placed.insert(object.geometry, at);

            let geometry_offset = mesh[object.geometry].geometry.prim_offset as i32;
            for (prim, from) in bvh.pack.prim_index.iter().enumerate() {
                let to = prim_offset + prim;
                self.pack.prim_index[to] = from + geometry_offset;
                self.pack.prim_type[to] = bvh.pack.prim_type[prim];
                self.pack.prim_visibility[to] = bvh.pack.prim_visibility[prim];
                // Not read for an instance.
                self.pack.prim_object[to] = 0;
                match (bvh.pack.prim_time.get(prim), self.pack.prim_time.get_mut(to)) {
                    (Some(from), Some(to)) => *to = *from,
                    _ => {}
                }
            }
            // A leaf's primitives are counted from where they now are.
            for (leaf, from) in bvh.pack.leaf_node.iter().enumerate() {
                self.pack.leaf_node[leaf_offset + leaf] = [from[0] + prim_offset as i32, from[1] + prim_offset as i32, from[2], from[3]];
            }
            // A node's children too: inner nodes up, leaves down.
            let moved = |child: i32| match child < 0 {
                true => child - leaf_offset as i32,
                false => child + node_offset as i32,
            };
            let step = BvhLimit::get().node_size;
            for (node, from) in bvh.pack.node.chunks_exact(step).enumerate() {
                let to = node_offset + node * step;
                self.pack.node[to..to + step].copy_from_slice(from);
                self.pack.node[to] = [from[0][0], from[0][1], moved(from[0][2]), moved(from[0][3])];
            }
            node_offset += bvh.pack.node.len();
            leaf_offset += bvh.pack.leaf_node.len();
            prim_offset += bvh.pack.prim_index.len();
        }
    }
}
