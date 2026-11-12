// SPDX-FileCopyrightText: 2009-2010 NVIDIA Corporation
// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/bvh/bvh2.h` and `bvh2.cpp` (`BVH2`: `build`,
//! `refit`, `pack_nodes`, `pack_leaf`, `pack_aligned_node`,
//! `pack_primitives`), adapted by Cycles from code of NVIDIA Corporation:
//! the binary tree, packed into flat arrays. `refit.rs` fits a packed
//! tree to moved vertices; `instance.rs` puts each geometry's tree after
//! the scene's.
//!
//! An inner node is four words of four:
//!
//! ```text
//!  visibility of child 0, of child 1, where child 0 is, where child 1 is
//!  x: least of child 0, least of child 1, most of child 0, most of child 1
//!  y: the same
//!  z: the same
//! ```
//!
//! A child that is a leaf is told from one that is not by where it is:
//! a leaf's place among the leaves is written with every bit turned.
//! Not ported: unaligned nodes (for curves), progress and cancelling.

mod instance;
mod refit;

use super::{BvhBuild, BvhLimit, BvhNode, BvhNodeKind, BvhParam, BvhStat, PackedBvh};
use crate::kernel::flag;
use crate::scene::{Mesh, Object};
use crate::util::BoundBox;

/// `BVH2`.
#[derive(Debug, Clone, PartialEq)]
pub struct Bvh2 {
    /// `params`.
    pub param: BvhParam,
    pub pack: PackedBvh,
}

impl Bvh2 {
    /// A tree to be built with these settings.
    pub fn new(param: BvhParam) -> Bvh2 {
        Bvh2 { param, pack: PackedBvh::default() }
    }

    /// `build`: build the tree over these objects, whose `geometry` are
    /// places in `mesh`, and pack it. At the top level each instanced
    /// geometry's own tree, which must be built already, is packed after
    /// it.
    pub fn build(&mut self, object: &[Object], mesh: &[Mesh]) {
        let root = BvhBuild::new(object, mesh, &mut self.pack, self.param).run();
        self.pack_primitive(object);
        self.pack_node(&root, object, mesh);
    }

    /// `pack_primitives`: the rays that see each primitive. An instanced
    /// object has none here; its visibility is tested as an object.
    pub(super) fn pack_primitive(&mut self, object: &[Object]) {
        let seen = |(index, at): (&i32, &i32)| match *index != -1 {
            true => usize::try_from(*at).ok().and_then(|at| object.get(at)).map(Object::visibility_for_tracing).unwrap_or(0),
            false => 0,
        };
        self.pack.prim_visibility = self.pack.prim_index.iter().zip(&self.pack.prim_object).map(seen).collect();
    }

    /// `pack_leaf`.
    fn pack_leaf(&mut self, at: usize, node: &BvhNode) {
        let (lo, hi) = match node.kind {
            BvhNodeKind::Leaf { lo, hi } => (lo, hi),
            BvhNodeKind::Inner { .. } => (0, 0),
        };
        let is_object = hi - lo == 1 && self.pack.prim_index.get(lo) == Some(&-1);
        let span = match is_object {
            true => [!(lo as i32), 0],
            false => [lo as i32, hi as i32],
        };
        let prim_type = match hi != lo {
            true => self.pack.prim_type.get(lo).copied().unwrap_or(0),
            false => 0,
        };
        self.pack.leaf_node[at] = [span[0], span[1], node.visibility as i32, prim_type];
    }

    /// `pack_aligned_node`: an inner node at `at`, from its children's
    /// boxes, places and visibility.
    pub(super) fn pack_aligned_node(&mut self, at: usize, bound: [&BoundBox; 2], child: [i32; 2], visibility: [u32; 2]) {
        let aligned = !flag("PATH_RAY_VISIBILITY_NODE_UNALIGNED");
        let bit = |value: f32| value.to_bits() as i32;
        self.pack.node[at] = [(visibility[0] & aligned) as i32, (visibility[1] & aligned) as i32, child[0], child[1]];
        for axis in 0..3 {
            self.pack.node[at + 1 + axis] = [bit(bound[0].min[axis]), bit(bound[1].min[axis]), bit(bound[0].max[axis]), bit(bound[1].max[axis])];
        }
    }

    /// `pack_nodes`: the tree under `root` into `nodes` and `leaf_nodes`.
    /// Inner nodes and leaves are each numbered in the order their
    /// parents are packed.
    fn pack_node(&mut self, root: &BvhNode, object: &[Object], mesh: &[Mesh]) {
        let limit = BvhLimit::get();
        let leaf_count = root.subtree_size(BvhStat::LeafCount);
        let node_size = (root.subtree_size(BvhStat::NodeCount) - leaf_count) * limit.node_size;
        self.pack.node.clear();
        self.pack.leaf_node.clear();
        match self.param.top_level {
            // First the other trees, so where they go is known.
            true => self.pack_instance(node_size, leaf_count * limit.leaf_node_size, object, mesh),
            false => {
                self.pack.node.resize(node_size, [0; 4]);
                self.pack.leaf_node.resize(leaf_count * limit.leaf_node_size, [0; 4]);
            }
        }

        let (mut next_node, mut next_leaf) = (0, 0);
        let mut place = |node: &BvhNode| match node.is_leaf() {
            true => {
                next_leaf += limit.leaf_node_size;
                next_leaf - limit.leaf_node_size
            }
            false => {
                next_node += limit.node_size;
                next_node - limit.node_size
            }
        };
        // `encodeIdx`.
        let encode = |node: &BvhNode, at: usize| match node.is_leaf() {
            true => !(at as i32),
            false => at as i32,
        };
        let mut stack = vec![(root, place(root))];
        while let Some((node, at)) = stack.pop() {
            match &node.kind {
                BvhNodeKind::Leaf { .. } => self.pack_leaf(at, node),
                BvhNodeKind::Inner { child } => {
                    let child_at = [place(&child[0]), place(&child[1])];
                    stack.push((&child[0], child_at[0]));
                    stack.push((&child[1], child_at[1]));
                    let encoded = [encode(&child[0], child_at[0]), encode(&child[1], child_at[1])];
                    self.pack_aligned_node(at, [&child[0].bound, &child[1].bound], encoded, [child[0].visibility, child[1].visibility]);
                }
            }
        }
        debug_assert_eq!(node_size, next_node);
        // Where a walk starts; a tree of one leaf has no node to start at.
        self.pack.root_index = match root.is_leaf() {
            true => -1,
            false => 0,
        };
    }
}

#[cfg(test)]
mod test;
