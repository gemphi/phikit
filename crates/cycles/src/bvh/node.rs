// SPDX-FileCopyrightText: 2009-2010 NVIDIA Corporation
// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/bvh/node.h` and `node.cpp` (`BVHNode`,
//! `InnerNode`, `LeafNode`, `BVH_STAT`), adapted by Cycles from code of
//! NVIDIA Corporation: the tree as the builder makes it, before it is
//! packed for a device.
//!
//! An inner node has two children: only the binary tree is ported. Not
//! ported: the aligned space of a node (for curves), and `print` and
//! `dump_graph`.

use super::BvhParam;
use crate::util::BoundBox;

/// `BVH_STAT`: what `subtree_size` counts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BvhStat {
    /// `BVH_STAT_NODE_COUNT`.
    NodeCount,
    /// `BVH_STAT_INNER_COUNT`.
    InnerCount,
    /// `BVH_STAT_LEAF_COUNT`.
    LeafCount,
    /// `BVH_STAT_TRIANGLE_COUNT`: the primitives in the leaves.
    TriangleCount,
    /// `BVH_STAT_CHILDNODE_COUNT`.
    ChildNodeCount,
    /// `BVH_STAT_DEPTH`: the nodes on the longest way down.
    Depth,
}

/// What a node is.
#[derive(Debug, Clone, PartialEq)]
pub enum BvhNodeKind {
    /// `InnerNode`.
    Inner { child: Box<[BvhNode; 2]> },
    /// `LeafNode`: the primitives from `lo` up to `hi` in the packed
    /// arrays.
    Leaf { lo: usize, hi: usize },
}

/// `BVHNode`.
#[derive(Debug, Clone, PartialEq)]
pub struct BvhNode {
    /// `bounds`.
    pub bound: BoundBox,
    /// The rays that see something under it.
    pub visibility: u32,
    /// The span of the shutter what is under it holds for.
    pub time_from: f32,
    pub time_to: f32,
    pub kind: BvhNodeKind,
}

impl BvhNode {
    /// `LeafNode(bounds, visibility, lo, hi)`.
    pub fn leaf(bound: BoundBox, visibility: u32, lo: usize, hi: usize) -> BvhNode {
        BvhNode { bound, visibility, time_from: 0.0, time_to: 1.0, kind: BvhNodeKind::Leaf { lo, hi } }
    }

    /// `InnerNode(bounds, child0, child1)`: seen by the rays that see
    /// either child.
    pub fn inner(bound: BoundBox, first: BvhNode, second: BvhNode) -> BvhNode {
        let visibility = first.visibility | second.visibility;
        BvhNode { bound, visibility, time_from: 0.0, time_to: 1.0, kind: BvhNodeKind::Inner { child: Box::new([first, second]) } }
    }

    /// `is_leaf`.
    pub fn is_leaf(&self) -> bool {
        matches!(self.kind, BvhNodeKind::Leaf { .. })
    }

    /// `get_child`, each of them: none for a leaf.
    pub fn child(&self) -> &[BvhNode] {
        match &self.kind {
            BvhNodeKind::Inner { child } => &child[..],
            BvhNodeKind::Leaf { .. } => &[],
        }
    }

    /// `num_triangles`: the primitives of a leaf.
    pub fn triangle_count(&self) -> usize {
        match self.kind {
            BvhNodeKind::Leaf { lo, hi } => hi - lo,
            BvhNodeKind::Inner { .. } => 0,
        }
    }

    /// `getSubtreeSize`.
    pub fn subtree_size(&self, stat: BvhStat) -> usize {
        let below = self.child().iter().map(|child| child.subtree_size(stat));
        match stat {
            BvhStat::Depth => 1 + below.max().unwrap_or(0),
            BvhStat::NodeCount => 1 + below.sum::<usize>(),
            BvhStat::InnerCount => usize::from(!self.is_leaf()) + below.sum::<usize>(),
            BvhStat::LeafCount => usize::from(self.is_leaf()) + below.sum::<usize>(),
            BvhStat::TriangleCount => self.triangle_count() + below.sum::<usize>(),
            BvhStat::ChildNodeCount => self.child().len() + below.sum::<usize>(),
        }
    }

    /// `computeSubtreeSAHCost`: what a ray is expected to pay under this
    /// node, when it reaches it with `probability`. A child is reached as
    /// often as its box's area is of this one's.
    pub fn subtree_sah_cost(&self, param: &BvhParam, probability: f32) -> f32 {
        let own = probability * param.cost(self.child().len(), self.triangle_count());
        let below = self.child().iter().map(|child| child.subtree_sah_cost(param, probability * child.bound.safe_area() / self.bound.safe_area()));
        own + below.sum::<f32>()
    }

    /// `update_visibility`: an inner node with none takes its children's.
    pub fn update_visibility(&mut self) -> u32 {
        match (&mut self.kind, self.visibility) {
            (BvhNodeKind::Inner { child }, 0) => self.visibility = child.iter_mut().fold(0, |seen, child| seen | child.update_visibility()),
            _ => {}
        }
        self.visibility
    }

    /// `update_time`: an inner node holds for as long as its children do.
    pub fn update_time(&mut self) {
        match &mut self.kind {
            BvhNodeKind::Inner { child } => {
                child.iter_mut().for_each(BvhNode::update_time);
                self.time_from = child[0].time_from.min(child[1].time_from);
                self.time_to = child[0].time_to.max(child[1].time_to);
            }
            BvhNodeKind::Leaf { .. } => {}
        }
    }
}

#[cfg(test)]
mod test;
