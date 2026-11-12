// SPDX-FileCopyrightText: 2009-2010 NVIDIA Corporation
// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/bvh/split.h` and `split.cpp`
//! (`BVHSpatialSplit`: its constructor and `split`), adapted by Cycles
//! from code of NVIDIA Corporation: the box of a run of references cut by
//! a plane. A reference the plane goes through is put on one side whole,
//! or on both, each with its part of the box, whichever costs a ray
//! least. The plane is the best of the cuts between a fixed number of
//! slabs along each axis.

use crate::bvh::{BvhBuild, BvhLimit, BvhRange, BvhReference, BvhSpatialBin, BvhSpatialStorage};
use crate::util::BoundBox;

/// `BVHSpatialSplit`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BvhSpatialSplit {
    /// What the split costs; `f32::MAX` when none was looked for.
    pub sah: f32,
    /// The axis the plane is across.
    pub dim: usize,
    /// Where on that axis the plane is.
    pub pos: f32,
}

impl Default for BvhSpatialSplit {
    fn default() -> Self {
        BvhSpatialSplit { sah: f32::MAX, dim: 0, pos: 0.0 }
    }
}

impl BvhSpatialSplit {
    /// `BVHSpatialSplit(builder, storage, range, references, nodeSAH)`:
    /// chop every reference into the slabs it reaches into, then take the
    /// cut between slabs that costs least.
    pub fn new(build: &BvhBuild, storage: &mut BvhSpatialStorage, range: &BvhRange, reference: &[BvhReference], node_sah: f32) -> BvhSpatialSplit {
        let bin_count = BvhLimit::get().spatial_bin_count;
        let origin = range.bound.min;
        let bin_size = range.bound.size().map(|side| side * (1.0 / bin_count as f32));
        let per_size = bin_size.map(|size| match size != 0.0 {
            true => 1.0 / size,
            false => 0.0,
        });
        storage.bin.iter_mut().for_each(|bin| bin.fill(BvhSpatialBin::default()));

        for reference in &reference[range.start..range.end()] {
            for dim in 0..3 {
                let slab = |at: f32| ((at - origin[dim]) * per_size[dim]) as i32;
                let first = slab(reference.bound.min[dim]).clamp(0, bin_count as i32 - 1);
                let last = slab(reference.bound.max[dim]).clamp(first, bin_count as i32 - 1);
                let (first, last) = (first as usize, last as usize);
                // The part left of each plane goes into that slab; what
                // is left over goes into the last.
                let mut rest = BvhReference::new(reference.bound, reference.prim_index, reference.prim_object, reference.prim_type);
                for bin in first..last {
                    let (left, right) = BvhSpatialSplit::split_reference(build, &rest, dim, origin[dim] + bin_size[dim] * (bin + 1) as f32);
                    storage.bin[dim][bin].bound.grow_box(&left.bound);
                    rest = right;
                }
                storage.bin[dim][last].bound.grow_box(&rest.bound);
                storage.bin[dim][first].enter += 1;
                storage.bin[dim][last].exit += 1;
            }
        }

        let mut split = BvhSpatialSplit::default();
        storage.right_bound.resize(bin_count, BoundBox::EMPTY);
        for dim in 0..3 {
            let mut right_bound = BoundBox::EMPTY;
            for bin in (1..bin_count).rev() {
                right_bound.grow_box(&storage.bin[dim][bin].bound);
                storage.right_bound[bin - 1] = right_bound;
            }
            let mut left_bound = BoundBox::EMPTY;
            let (mut left_count, mut right_count) = (0, range.size);
            for bin in 1..bin_count {
                left_bound.grow_box(&storage.bin[dim][bin - 1].bound);
                left_count += storage.bin[dim][bin - 1].enter;
                right_count -= storage.bin[dim][bin - 1].exit;
                let sah = node_sah + left_bound.safe_area() * build.param.primitive_cost(left_count) + storage.right_bound[bin - 1].safe_area() * build.param.primitive_cost(right_count);
                if sah < split.sah {
                    split = BvhSpatialSplit { sah, dim, pos: origin[dim] + bin_size[dim] * bin as f32 };
                }
            }
        }
        split
    }

    /// `split`: sort the range's references into those left of the plane,
    /// those right of it and those it goes through; then put each of the
    /// last on the left, on the right, or cut in two on both. The right
    /// halves of those cut are added after the range, so the list grows.
    pub fn split(&self, build: &BvhBuild, storage: &mut BvhSpatialStorage, reference: &mut Vec<BvhReference>, range: &BvhRange) -> (BvhRange, BvhRange) {
        let left_start = range.start;
        let (mut left_end, mut right_start, mut right_end) = (left_start, range.end(), range.end());
        let (mut left_bound, mut right_bound) = (BoundBox::EMPTY, BoundBox::EMPTY);

        let mut at = left_end;
        while at < right_start {
            let bound = reference[at].bound;
            match (bound.max[self.dim] <= self.pos, bound.min[self.dim] >= self.pos) {
                (true, _) => {
                    left_bound.grow_box(&bound);
                    reference.swap(at, left_end);
                    left_end += 1;
                    at += 1;
                }
                // What is swapped in from the right is looked at next.
                (false, true) => {
                    right_bound.grow_box(&bound);
                    right_start -= 1;
                    reference.swap(at, right_start);
                }
                (false, false) => at += 1,
            }
        }

        storage.new_reference.clear();
        while left_end < right_start {
            let whole = reference[left_end];
            let whole = BvhReference::new(whole.bound, whole.prim_index, whole.prim_object, whole.prim_type);
            let (left, right) = BvhSpatialSplit::split_reference(build, &whole, self.dim, self.pos);
            // The boxes when it goes left whole, right whole, or is cut.
            let (to_left, to_right) = (left_bound.merge(&whole.bound), right_bound.merge(&whole.bound));
            let (cut_left, cut_right) = (left_bound.merge(&left.bound), right_bound.merge(&right.bound));
            let cost = |count: usize| build.param.primitive_cost(count);
            let (left_count, right_count) = (left_end - left_start, right_end - right_start);
            let unsplit_left = to_left.safe_area() * cost(left_count + 1) + right_bound.safe_area() * cost(right_count);
            let unsplit_right = left_bound.safe_area() * cost(left_count) + to_right.safe_area() * cost(right_count + 1);
            let duplicate = cut_left.safe_area() * cost(left_count + 1) + cut_right.safe_area() * cost(right_count + 1);
            let least = unsplit_left.min(unsplit_right).min(duplicate);
            match (least == unsplit_left, least == unsplit_right) {
                (true, _) => {
                    left_bound = to_left;
                    left_end += 1;
                }
                (false, true) => {
                    right_bound = to_right;
                    right_start -= 1;
                    reference.swap(left_end, right_start);
                }
                (false, false) => {
                    (left_bound, right_bound) = (cut_left, cut_right);
                    reference[left_end] = left;
                    left_end += 1;
                    storage.new_reference.push(right);
                    right_end += 1;
                }
            }
        }
        let after = right_end - storage.new_reference.len();
        reference.splice(after..after, storage.new_reference.drain(..));
        (BvhRange::new(left_bound, left_start, left_end - left_start), BvhRange::new(right_bound, right_start, right_end - right_start))
    }
}

#[cfg(test)]
mod test;
