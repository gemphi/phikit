// SPDX-FileCopyrightText: 2009-2010 NVIDIA Corporation
// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/bvh/params.h` (`BVHReference`, `BVHRange`,
//! `BVHSpatialBin`, `BVHSpatialStorage`), adapted by Cycles from code of
//! NVIDIA Corporation: what the builder sorts and splits.
//!
//! Cycles hides a reference's primitive and object, and a range's start
//! and size, in the unused fourth parts of their boxes; here they are
//! fields.

use super::BvhLimit;
use crate::util::BoundBox;

/// `BVHReference`: a primitive, or an instanced object, and the box it is
/// in. A spatial split leaves two references to one primitive, each with
/// its part of the box.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BvhReference {
    /// `bounds`.
    pub bound: BoundBox,
    /// The primitive within its geometry; -1 for an object.
    pub prim_index: i32,
    /// Its object's place in the list the tree is built over.
    pub prim_object: i32,
    /// `PRIMITIVE_TRIANGLE` and the like; 0 for an object.
    pub prim_type: i32,
    /// The span of the shutter the box holds for.
    pub time_from: f32,
    pub time_to: f32,
}

impl BvhReference {
    /// A reference that holds for the whole shutter.
    pub fn new(bound: BoundBox, prim_index: i32, prim_object: i32, prim_type: i32) -> BvhReference {
        BvhReference { bound, prim_index, prim_object, prim_type, time_from: 0.0, time_to: 1.0 }
    }

    /// Its object's place, to look the object up by.
    pub fn object(&self) -> usize {
        usize::try_from(self.prim_object).unwrap_or(0)
    }
}

/// `BVHRange`: a run of references, the box around them, and the box
/// around their centres.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct BvhRange {
    /// `bounds`.
    pub bound: BoundBox,
    /// `cent_bounds`: around twice each reference's centre. Only the
    /// binning builder reads it.
    pub cent_bound: BoundBox,
    pub start: usize,
    pub size: usize,
}

impl BvhRange {
    /// A range with no box around its centres.
    pub fn new(bound: BoundBox, start: usize, size: usize) -> BvhRange {
        BvhRange { bound, cent_bound: BoundBox::EMPTY, start, size }
    }

    /// `end`.
    pub fn end(&self) -> usize {
        self.start + self.size
    }
}

/// `BVHSpatialBin`: a slab of a node's box, the box of what is in it, and
/// how many references start and end in it.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct BvhSpatialBin {
    /// `bounds`.
    pub bound: BoundBox,
    pub enter: usize,
    pub exit: usize,
}

/// `BVHSpatialStorage`: room the splitters work in, kept from one split
/// to the next.
#[derive(Debug, Clone, PartialEq)]
pub struct BvhSpatialStorage {
    /// `right_bounds`: the boxes gathered sweeping from the right.
    pub right_bound: Vec<BoundBox>,
    /// `bins`: for each axis, `BVHParams::NUM_SPATIAL_BINS` of them.
    pub bin: [Vec<BvhSpatialBin>; 3],
    /// `new_references`: the right halves of the references a split cut.
    pub new_reference: Vec<BvhReference>,
}

impl Default for BvhSpatialStorage {
    fn default() -> Self {
        Self::new()
    }
}

impl BvhSpatialStorage {
    /// Storage with its bins made.
    pub fn new() -> BvhSpatialStorage {
        let bin = || vec![BvhSpatialBin::default(); BvhLimit::get().spatial_bin_count];
        BvhSpatialStorage { right_bound: Vec::new(), bin: [bin(), bin(), bin()], new_reference: Vec::new() }
    }
}
