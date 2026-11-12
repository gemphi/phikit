// SPDX-FileCopyrightText: 2009-2010 NVIDIA Corporation
// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/bvh/build.cpp` (`BVHBuild`:
//! `range_within_max_leaf_size`, `create_leaf_node`,
//! `create_object_leaf_nodes`), adapted by Cycles from code of NVIDIA
//! Corporation: the end of a branch. A leaf holds primitives of one type,
//! or one instanced object, so a range of several types ends in a small
//! tree of leaves.

use super::BvhBuild;
use crate::bvh::{BvhNode, BvhRange, BvhReference};
use crate::util::BoundBox;

impl BvhBuild<'_> {
    /// The rays that see the object a reference is of.
    fn visibility(&self, reference: &BvhReference) -> u32 {
        self.object.get(reference.object()).map(|object| object.visibility_for_tracing()).unwrap_or(0)
    }

    /// Write a reference into the packed arrays at `at`.
    fn write(&mut self, at: usize, reference: &BvhReference) {
        self.pack.prim_type[at] = reference.prim_type;
        self.pack.prim_index[at] = reference.prim_index;
        self.pack.prim_object[at] = reference.prim_object;
        if self.need_prim_time {
            self.pack.prim_time[at] = [reference.time_from, reference.time_to];
        }
    }

    /// `range_within_max_leaf_size`: the range has no more primitives of
    /// any kind than a leaf of that kind may.
    pub fn range_within_max_leaf_size(&self, range: &BvhRange, reference: &[BvhReference]) -> bool {
        let param = &self.param;
        let limit = [[param.max_curve_leaf_size, param.max_motion_curve_leaf_size], [param.max_triangle_leaf_size, param.max_motion_triangle_leaf_size], [param.max_point_leaf_size, param.max_motion_point_leaf_size]];
        if range.size > param.max_triangle_leaf_size.max(param.max_curve_leaf_size).max(param.max_point_leaf_size) {
            return false;
        }
        let mut count = [[0usize; 2]; 3];
        for reference in &reference[range.start..range.end()] {
            let kind = [self.bit.curve, self.bit.triangle, self.bit.point].iter().position(|bit| reference.prim_type & bit != 0);
            let moving = usize::from(reference.prim_type & self.bit.motion != 0);
            kind.into_iter().for_each(|kind| count[kind][moving] += 1);
        }
        (0..3).all(|kind| (0..2).all(|moving| count[kind][moving] <= limit[kind][moving]))
    }

    /// `create_object_leaf_nodes`: a leaf for each instanced object, under
    /// a tree that halves them. No objects make an empty leaf.
    fn create_object_leaf_node(&mut self, reference: &[BvhReference], start: usize) -> BvhNode {
        match reference {
            [] => BvhNode::leaf(BoundBox::EMPTY, 0, 0, 0),
            [only] => {
                self.write(start, only);
                BvhNode { time_from: only.time_from, time_to: only.time_to, ..BvhNode::leaf(only.bound, self.visibility(only), start, start + 1) }
            }
            _ => {
                let (first, second) = reference.split_at(reference.len() / 2);
                let (first, second) = (self.create_object_leaf_node(first, start), self.create_object_leaf_node(second, start + first.len()));
                let (time_from, time_to) = (first.time_from.min(second.time_from), first.time_to.max(second.time_to));
                BvhNode { time_from, time_to, ..BvhNode::inner(first.bound.merge(&second.bound), first, second) }
            }
        }
    }

    /// `create_leaf_node`: the references of `range` written into the
    /// packed arrays, one type after another, and a leaf for each type.
    /// Without spatial splits a range's place in the arrays is its place
    /// among the references; with them the leaves take the arrays up in
    /// the order they are made.
    pub fn create_leaf_node(&mut self, range: &BvhRange, reference: &[BvhReference]) -> BvhNode {
        let reference = &reference[range.start..range.end()];
        let (primitive, object): (Vec<BvhReference>, Vec<BvhReference>) = reference.iter().partition(|reference| reference.prim_index != -1);
        let mut by_type: Vec<Vec<BvhReference>> = Vec::new();
        for reference in primitive {
            let at = self.bit.index(reference.prim_type);
            by_type.resize(by_type.len().max(at + 1), Vec::new());
            by_type[at].push(reference);
        }

        let start = match self.param.use_spatial_split {
            true => {
                let start = self.spatial_free_index;
                self.spatial_free_index += range.size;
                let end = self.pack.prim_type.len().max(start + range.size);
                self.pack.prim_type.resize(end, 0);
                self.pack.prim_index.resize(end, 0);
                self.pack.prim_object.resize(end, 0);
                self.pack.prim_time.resize(usize::from(self.need_prim_time) * end, [0.0; 2]);
                start
            }
            false => range.start,
        };

        let mut leaf = Vec::new();
        let mut at = start;
        for group in by_type.iter().filter(|group| !group.is_empty()) {
            let mut node = BvhNode { time_from: 1.0, time_to: 0.0, ..BvhNode::leaf(BoundBox::EMPTY, 0, at, at + group.len()) };
            for reference in group {
                self.write(at, reference);
                node.bound.grow_box(&reference.bound);
                node.visibility |= self.visibility(reference);
                node.time_from = node.time_from.min(reference.time_from);
                node.time_to = node.time_to.max(reference.time_to);
                at += 1;
            }
            leaf.push(node);
        }
        // The objects, or an empty leaf when the range has nothing.
        if leaf.is_empty() || !object.is_empty() {
            leaf.push(self.create_object_leaf_node(&object, at));
        }

        // One leaf is the node; more hang under inner nodes, two by two.
        let pair = |first: BvhNode, second: BvhNode| BvhNode::inner(first.bound.merge(&second.bound), first, second);
        let mut leaf = leaf.into_iter();
        match (leaf.next(), leaf.next(), leaf.next(), leaf.next(), leaf.next()) {
            (Some(a), None, ..) => a,
            (Some(a), Some(b), None, ..) => BvhNode::inner(range.bound, a, b),
            (Some(a), Some(b), Some(c), None, _) => BvhNode::inner(range.bound, a, pair(b, c)),
            (Some(a), Some(b), Some(c), Some(d), None) => pair(pair(a, b), pair(c, d)),
            (Some(a), Some(b), Some(c), Some(d), Some(e)) => BvhNode::inner(range.bound, pair(pair(a, b), pair(c, d)), e),
            _ => BvhNode::leaf(BoundBox::EMPTY, 0, 0, 0),
        }
    }
}

#[cfg(test)]
mod test;
