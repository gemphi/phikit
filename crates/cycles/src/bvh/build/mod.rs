// SPDX-FileCopyrightText: 2009-2010 NVIDIA Corporation
// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/bvh/build.h` and `build.cpp` (`BVHBuild`:
//! `run` and the two `build_node`), adapted by Cycles from code of NVIDIA
//! Corporation: the builder of the tree. It gathers a reference for each
//! primitive, or for each instanced object at the top level
//! (`reference.rs`), splits them over and over (`../binning.rs`, or
//! `../split` when primitives may be cut), and ends each branch in leaves
//! (`leaf.rs`) that say where its primitives are in the packed arrays.
//!
//! Cycles builds the two halves of a large node on separate threads; here
//! one thread builds the whole tree. Not ported: progress and cancelling,
//! unaligned nodes (for curves), and `rotate`, which Cycles has switched
//! off.

mod leaf;
mod reference;

use super::split::BvhMixedSplit;
use super::{BvhNode, BvhObjectBinning, BvhParam, BvhRange, BvhReference, BvhSpatialStorage, PackedBvh};
use crate::kernel::flag;
use crate::scene::{Mesh, Object};

/// The `PRIMITIVE_*` bits the builder tells primitives apart by.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PrimitiveBit {
    pub triangle: i32,
    pub curve: i32,
    pub point: i32,
    pub motion: i32,
    /// `PRIMITIVE_ALL`.
    pub all: i32,
}

impl PrimitiveBit {
    /// The bits, by Cycles' names.
    pub fn new() -> PrimitiveBit {
        let bit = |name: &str| flag(name) as i32;
        PrimitiveBit { triangle: bit("PRIMITIVE_TRIANGLE"), curve: bit("PRIMITIVE_CURVE"), point: bit("PRIMITIVE_POINT"), motion: bit("PRIMITIVE_MOTION"), all: bit("PRIMITIVE_ALL") }
    }

    /// `PRIMITIVE_INDEX`: a type's place among the kinds of primitive,
    /// each with and without motion.
    pub fn index(&self, prim_type: i32) -> usize {
        let kind = prim_type & self.all;
        kind.trailing_zeros() as usize * 2 + usize::from(kind & self.motion != 0)
    }
}

impl Default for PrimitiveBit {
    fn default() -> Self {
        Self::new()
    }
}

/// `BVHBuild`.
#[derive(Debug)]
pub struct BvhBuild<'a> {
    /// `objects`: what the tree is built over.
    pub object: &'a [Object],
    /// The scene's meshes, which the objects' `geometry` are places in.
    pub mesh: &'a [Mesh],
    /// `prim_type`, `prim_index`, `prim_object` and `prim_time`, which
    /// the leaves are written into.
    pub pack: &'a mut PackedBvh,
    /// `params`; the builder switches spatial splits off where they do
    /// not work.
    pub param: BvhParam,
    pub bit: PrimitiveBit,
    /// Leaves hold for spans of the shutter.
    pub need_prim_time: bool,
    /// `spatial_min_overlap`: the area two halves must overlap by before
    /// a spatial split is tried.
    pub spatial_min_overlap: f32,
    /// `spatial_free_index`: the first place in the packed arrays no leaf
    /// has taken, when primitives may be cut.
    pub spatial_free_index: usize,
}

impl<'a> BvhBuild<'a> {
    /// `BVHBuild(objects, prim_type, prim_index, prim_object, prim_time,
    /// params, progress)`.
    pub fn new(object: &'a [Object], mesh: &'a [Mesh], pack: &'a mut PackedBvh, param: BvhParam) -> BvhBuild<'a> {
        BvhBuild { object, mesh, pack, param, bit: PrimitiveBit::new(), need_prim_time: false, spatial_min_overlap: 0.0, spatial_free_index: 0 }
    }

    /// The mesh of the object a reference is of.
    pub fn mesh_of(&self, object: usize) -> Option<&'a Mesh> {
        self.mesh.get(self.object.get(object)?.geometry)
    }

    /// `run`: build the tree and say what its root is.
    pub fn run(&mut self) -> BvhNode {
        // A spatial split does not cut the positions of a motion step.
        let blurred = self.object.iter().any(|object| self.mesh.get(object.geometry).is_some_and(Mesh::has_motion_blur));
        if !self.param.top_level && blurred {
            self.param.use_spatial_split = false;
        }
        let (root, mut reference) = self.add_reference();
        // Not worth its time at the top level, Cycles found.
        if self.param.top_level {
            self.param.use_spatial_split = false;
        }
        self.spatial_min_overlap = root.bound.safe_area() * self.param.spatial_split_alpha;
        self.spatial_free_index = 0;
        self.need_prim_time = self.param.use_motion_step();

        let count = reference.len();
        self.pack.prim_type.resize(count, 0);
        self.pack.prim_index.resize(count, 0);
        self.pack.prim_object.resize(count, 0);
        self.pack.prim_time.resize(usize::from(self.need_prim_time) * count, [0.0; 2]);

        let mut node = match self.param.use_spatial_split {
            true => self.build_spatial_node(root, &mut reference, 0, &mut BvhSpatialStorage::new()),
            false => self.build_node(&BvhObjectBinning::new(root, &reference), &mut reference, 0),
        };
        node.update_visibility();
        node.update_time();
        node
    }

    /// The top level has an inner node at its root even over one object,
    /// since an instance is not tested against the rays that see it.
    fn must_split(&self, size: usize, level: usize) -> bool {
        size > 0 && self.param.top_level && level == 0
    }

    /// `build_node` of the binning builder: a leaf when the range is
    /// small or a leaf costs less, else the best cut and a node of the
    /// two sides.
    fn build_node(&mut self, range: &BvhObjectBinning, reference: &mut [BvhReference], level: usize) -> BvhNode {
        let size = range.range.size;
        let leaf_sah = self.param.sah_primitive_cost * range.leaf_sah;
        let split_sah = self.param.sah_node_cost * range.range.bound.half_area() + self.param.sah_primitive_cost * range.split_sah;
        let small = self.param.small_enough_for_leaf(size, level);
        if !self.must_split(size, level) && (small || (self.range_within_max_leaf_size(&range.range, reference) && leaf_sah < split_sah)) {
            return self.create_leaf_node(&range.range, reference);
        }
        let (left, right) = range.split(reference);
        let first = self.build_node(&left, reference, level + 1);
        let second = self.build_node(&right, reference, level + 1);
        BvhNode::inner(range.range.bound, first, second)
    }

    /// `build_node` of the spatial split builder: as above, with the
    /// choice between sorting the references into two runs and cutting
    /// the box in two, which may leave a primitive on both sides. The
    /// right side's references are copied out, since building the left
    /// may add to the list.
    fn build_spatial_node(&mut self, range: BvhRange, reference: &mut Vec<BvhReference>, level: usize, storage: &mut BvhSpatialStorage) -> BvhNode {
        let must_split = self.must_split(range.size, level);
        if !must_split && self.param.small_enough_for_leaf(range.size, level) {
            return self.create_leaf_node(&range, reference);
        }
        let split = BvhMixedSplit::new(self, storage, &range, reference, level);
        if !must_split && split.no_split {
            return self.create_leaf_node(&range, reference);
        }
        let (left, mut right) = split.split(self, storage, reference, &range);
        let mut right_reference = reference[right.start..right.end()].to_vec();
        right.start = 0;
        let first = self.build_spatial_node(left, reference, level + 1, storage);
        let second = self.build_spatial_node(right, &mut right_reference, level + 1, storage);
        BvhNode::inner(range.bound, first, second)
    }
}

#[cfg(test)]
mod test;
