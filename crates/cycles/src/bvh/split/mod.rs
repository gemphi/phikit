// SPDX-FileCopyrightText: 2009-2010 NVIDIA Corporation
// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/bvh/split.h` (`BVHMixedSplit`), adapted by
//! Cycles from code of NVIDIA Corporation: the choice, for a run of
//! references, between a leaf, sorting them into two runs (`object.rs`)
//! and cutting their box in two (`spatial.rs`).
//!
//! Cycles' splits keep pointers to the storage and the references; here
//! both are handed to each call. Not ported: splitting in the aligned
//! space of a curve.

mod object;
mod primitive;
mod spatial;

pub use object::BvhObjectSplit;
pub use spatial::BvhSpatialSplit;

use super::{BvhBuild, BvhLimit, BvhRange, BvhReference, BvhSpatialStorage};
use crate::util::BoundBox;

/// `BVHMixedSplit`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BvhMixedSplit {
    pub object: BvhObjectSplit,
    pub spatial: BvhSpatialSplit,
    /// `leafSAH`: what a leaf of the whole range costs.
    pub leaf_sah: f32,
    /// `nodeSAH`: what walking into a node of two costs.
    pub node_sah: f32,
    /// `minSAH`: the least of the leaf's cost and the two splits'.
    pub min_sah: f32,
    /// A leaf costs least, and the range fits in one.
    pub no_split: bool,
    /// `bounds`.
    pub bound: BoundBox,
}

impl BvhMixedSplit {
    /// `BVHMixedSplit(builder, storage, range, references, level)`: find
    /// both splits. The spatial one is tried only when the halves of the
    /// object split overlap enough, and not too deep in the tree.
    pub fn new(build: &BvhBuild, storage: &mut BvhSpatialStorage, range: &BvhRange, reference: &mut [BvhReference], level: usize) -> BvhMixedSplit {
        let bound = range.bound;
        let area = bound.safe_area();
        let leaf_sah = area * build.param.primitive_cost(range.size);
        let node_sah = area * build.param.node_cost(2);
        let object = BvhObjectSplit::new(&build.param, storage, range, reference, node_sah);
        let mut spatial = BvhSpatialSplit::default();
        if build.param.use_spatial_split && level < BvhLimit::get().max_spatial_depth {
            let mut overlap = object.left_bound;
            overlap.intersect(&object.right_bound);
            if overlap.safe_area() >= build.spatial_min_overlap {
                spatial = BvhSpatialSplit::new(build, storage, range, reference, node_sah);
            }
        }
        let min_sah = leaf_sah.min(object.sah).min(spatial.sah);
        let no_split = min_sah == leaf_sah && build.range_within_max_leaf_size(range, reference);
        BvhMixedSplit { object, spatial, leaf_sah, node_sah, min_sah, no_split, bound }
    }

    /// `split`: the two sides of the split that costs least. A spatial
    /// split that leaves a side empty gives way to the object split.
    pub fn split(&self, build: &BvhBuild, storage: &mut BvhSpatialStorage, reference: &mut Vec<BvhReference>, range: &BvhRange) -> (BvhRange, BvhRange) {
        let mut side = (BvhRange::default(), BvhRange::default());
        if build.param.use_spatial_split && self.min_sah == self.spatial.sah {
            side = self.spatial.split(build, storage, reference, range);
        }
        if side.0.size == 0 || side.1.size == 0 {
            side = self.object.split(reference, range);
        }
        side
    }
}
