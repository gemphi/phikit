// SPDX-FileCopyrightText: 2009-2010 NVIDIA Corporation
// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/bvh/split.h` and `split.cpp`
//! (`BVHObjectSplit`), adapted by Cycles from code of NVIDIA Corporation:
//! the references sorted along an axis and cut into two runs where the
//! two boxes cost a ray least. Every cut along every axis is tried.

use crate::bvh::{BvhParam, BvhRange, BvhReference, BvhSpatialStorage, bvh_reference_sort};
use crate::util::BoundBox;

/// `BVHObjectSplit`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BvhObjectSplit {
    /// What the split costs; `f32::MAX` when the range cannot be cut.
    pub sah: f32,
    /// The axis the references are sorted along.
    pub dim: usize,
    /// `num_left`: how many references go left.
    pub left_count: usize,
    /// `left_bounds`.
    pub left_bound: BoundBox,
    /// `right_bounds`.
    pub right_bound: BoundBox,
}

impl BvhObjectSplit {
    /// `BVHObjectSplit(builder, storage, range, references, nodeSAH)`.
    /// The references of the range are left sorted along the last axis.
    pub fn new(param: &BvhParam, storage: &mut BvhSpatialStorage, range: &BvhRange, reference: &mut [BvhReference], node_sah: f32) -> BvhObjectSplit {
        let mut split = BvhObjectSplit { sah: f32::MAX, dim: 0, left_count: 0, left_bound: BoundBox::EMPTY, right_bound: BoundBox::EMPTY };
        let reference = &mut reference[range.start..range.end()];
        for dim in 0..3 {
            bvh_reference_sort(reference, dim);
            // From the right: the box of what is right of each cut.
            storage.right_bound.resize(range.size, BoundBox::EMPTY);
            let mut right_bound = BoundBox::EMPTY;
            for at in (1..range.size).rev() {
                right_bound.grow_box(&reference[at].bound);
                storage.right_bound[at - 1] = right_bound;
            }
            // From the left: the cost of each cut.
            let mut left_bound = BoundBox::EMPTY;
            for at in 1..range.size {
                left_bound.grow_box(&reference[at - 1].bound);
                let right_bound = storage.right_bound[at - 1];
                let sah = node_sah + left_bound.safe_area() * param.primitive_cost(at) + right_bound.safe_area() * param.primitive_cost(range.size - at);
                if sah < split.sah {
                    split = BvhObjectSplit { sah, dim, left_count: at, left_bound, right_bound };
                }
            }
        }
        split
    }

    /// `split`: sort the range along the split's axis and say what its
    /// two sides are.
    pub fn split(&self, reference: &mut [BvhReference], range: &BvhRange) -> (BvhRange, BvhRange) {
        bvh_reference_sort(&mut reference[range.start..range.end()], self.dim);
        let left = BvhRange::new(self.left_bound, range.start, self.left_count);
        let right = BvhRange::new(self.right_bound, left.end(), range.size - self.left_count);
        (left, right)
    }
}

#[cfg(test)]
mod test;
