// SPDX-FileCopyrightText: 2009-2011 Intel Corporation
// SPDX-FileCopyrightText: 2012-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/bvh/binning.h` and `binning.cpp`
//! (`BVHObjectBinning`), adapted by Cycles from code of NVIDIA
//! Corporation: the split of a run of references that costs a ray least,
//! found by dropping each reference into one of a few bins along each
//! axis by its centre and trying the cuts between bins.
//!
//! Not ported: binning in the aligned space of a curve.

use super::{BvhLimit, BvhRange, BvhReference};
use crate::util::BoundBox;

/// `BVHObjectBinning`: a range, and the best cut found for it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BvhObjectBinning {
    pub range: BvhRange,
    /// `splitSAH`: what the best cut costs.
    pub split_sah: f32,
    /// `leafSAH`: what a leaf of the whole range costs.
    pub leaf_sah: f32,
    /// The axis of the best cut.
    dim: usize,
    /// The bin the best cut is before; -1 when no cut was found.
    pos: i32,
    /// `num_bins`.
    bin_count: usize,
    /// From a doubled centre to a bin.
    scale: [f32; 3],
}

/// `blocks`: primitives are counted in blocks of four, as a leaf is
/// tested.
fn block(count: usize) -> usize {
    let log = BvhLimit::get().log_block_size;
    (count + (1 << log) - 1) >> log
}

impl BvhObjectBinning {
    /// `BVHObjectBinning(job, prims)`: bin the references of `range`,
    /// which are in `prim`, and find the best cut.
    pub fn new(range: BvhRange, prim: &[BvhReference]) -> BvhObjectBinning {
        let bin_count = BvhLimit::get().max_bin.min((4.0 + 0.05 * range.size as f32) as usize);
        let extent = range.cent_bound.size();
        let scale = extent.map(|side| match side != 0.0 {
            true => bin_count as f32 / side,
            false => 0.0,
        });
        let mut binning = BvhObjectBinning { range, split_sah: f32::MAX, leaf_sah: 0.0, dim: 0, pos: 0, bin_count, scale };

        // Each bin's box and count, for each axis.
        let mut bound = vec![[BoundBox::EMPTY; 3]; bin_count];
        let mut count = vec![[0usize; 3]; bin_count];
        for reference in prim.iter().skip(range.start).take(range.size) {
            let bin = binning.bin_of_box(&reference.bound);
            for axis in 0..3 {
                count[bin[axis]][axis] += 1;
                bound[bin[axis]][axis].grow_box(&reference.bound);
            }
        }

        // From the right: the area and the count of what is right of each cut.
        let mut right_area = vec![[0.0f32; 3]; bin_count];
        let mut right_count = vec![[0.0f32; 3]; bin_count];
        let (mut seen, mut merged) = ([0usize; 3], [BoundBox::EMPTY; 3]);
        for bin in (1..bin_count).rev() {
            for axis in 0..3 {
                seen[axis] += count[bin][axis];
                merged[axis].grow_box(&bound[bin][axis]);
                right_count[bin][axis] = block(seen[axis]) as f32;
                right_area[bin][axis] = merged[axis].half_area();
            }
        }

        // From the left: the cost of each cut. A cut with an empty side
        // costs something that is not a number, and never wins.
        let (mut best_sah, mut best_split) = ([f32::MAX; 3], [-1i32; 3]);
        (seen, merged) = ([0; 3], [BoundBox::EMPTY; 3]);
        for bin in 1..bin_count {
            for axis in 0..3 {
                seen[axis] += count[bin - 1][axis];
                merged[axis].grow_box(&bound[bin - 1][axis]);
                let sah = merged[axis].half_area() * block(seen[axis]) as f32 + right_area[bin][axis] * right_count[bin][axis];
                if sah < best_sah[axis] {
                    (best_sah[axis], best_split[axis]) = (sah, bin as i32);
                }
            }
        }
        // An axis the centres do not spread along cannot be cut.
        let best_sah: [f32; 3] = std::array::from_fn(|axis| match extent[axis] <= 0.0 {
            true => f32::MAX,
            false => best_sah[axis],
        });
        let least = best_sah[0].min(best_sah[1].min(best_sah[2]));
        binning.dim = best_sah.iter().position(|sah| *sah == least).unwrap_or(2);
        binning.split_sah = best_sah[binning.dim];
        binning.pos = best_split[binning.dim];
        binning.leaf_sah = range.bound.half_area() * block(range.size) as f32;
        binning
    }

    /// `get_bin` of a point: the bin a doubled centre falls in along each
    /// axis, which may be outside the bins.
    fn bin_of_point(&self, center: [f32; 3]) -> [i32; 3] {
        std::array::from_fn(|axis| ((center[axis] - self.range.cent_bound.min[axis]) * self.scale[axis] - 0.5) as i32)
    }

    /// `get_bin` of a box: the bin of its centre, kept inside the bins.
    fn bin_of_box(&self, bound: &BoundBox) -> [usize; 3] {
        self.bin_of_point(bound.center2()).map(|bin| bin.clamp(0, self.bin_count as i32 - 1) as usize)
    }

    /// `split`: put the references left of the best cut first in `prim`,
    /// and bin each side. When every reference falls on one side, as when
    /// all have one centre, the range is halved as it stands.
    pub fn split(&self, prim: &mut [BvhReference]) -> (BvhObjectBinning, BvhObjectBinning) {
        let (start, size) = (self.range.start, self.range.size);
        // The box of each side, and the box of its centres.
        let mut side = [(BoundBox::EMPTY, BoundBox::EMPTY); 2];
        let (mut left, mut right) = (0, size);
        while left < right {
            let reference = prim[start + left];
            let center = reference.bound.center2();
            let at = usize::from(self.bin_of_point(center)[self.dim] >= self.pos);
            side[at].0.grow_box(&reference.bound);
            side[at].1.grow(center);
            match at {
                0 => left += 1,
                _ => {
                    right -= 1;
                    prim.swap(start + left, start + right);
                }
            }
        }
        let made = |side: (BoundBox, BoundBox), start: usize, size: usize, prim: &[BvhReference]| {
            BvhObjectBinning::new(BvhRange { bound: side.0, cent_bound: side.1, start, size }, prim)
        };
        if left != 0 && size - right != 0 {
            return (made(side[0], start, left, prim), made(side[1], start + left, size - right, prim));
        }
        let half = size / 2;
        let around = |from: usize, to: usize| {
            let mut side = (BoundBox::EMPTY, BoundBox::EMPTY);
            for reference in &prim[start + from..start + to] {
                side.0.grow_box(&reference.bound);
                side.1.grow(reference.bound.center2());
            }
            side
        };
        (made(around(0, half), start, half, prim), made(around(half, size), start + half, size - half, prim))
    }
}
