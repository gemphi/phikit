// SPDX-FileCopyrightText: 2009-2010 NVIDIA Corporation
// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! The tree a ray walks to find what it hits: `intern/cycles/bvh`, which
//! Cycles adapted from code of NVIDIA Corporation. This file is ported
//! from `bvh.h` and `bvh.cpp` (`PackedBVH`, `BVH::create`).
//!
//! ```text
//!  BvhBuild     references to primitives, split into a tree of BvhNode
//!     │
//!  Bvh2         the tree packed into flat arrays: PackedBvh
//!     │
//!  DeviceScene  the scene's tree, with each instanced geometry's own after it
//! ```
//!
//! Only Cycles' own layout, the binary `BVH2`, is ported. The layouts of
//! Embree, OptiX, Metal and HIPRT are those libraries' trees; `create` has
//! none for them. Cycles' `BVH` also keeps the geometry and the objects it
//! is over; here they are handed to `build` and `refit`.

mod binning;
mod build;
mod bvh2;
mod node;
mod param;
mod reference;
mod sort;
mod split;

pub use binning::BvhObjectBinning;
pub use build::{BvhBuild, PrimitiveBit};
pub use bvh2::Bvh2;
pub use node::{BvhNode, BvhNodeKind, BvhStat};
pub use param::{BvhLayout, BvhLimit, BvhParam, BvhType};
pub use reference::{BvhRange, BvhReference, BvhSpatialBin, BvhSpatialStorage};
pub use sort::{bvh_reference_compare, bvh_reference_sort};
pub use split::{BvhMixedSplit, BvhObjectSplit, BvhSpatialSplit};

/// `PackedBVH`: a tree as a device walks it.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PackedBvh {
    /// `nodes`: `BVH_NODE_SIZE` words of four for each inner node: its
    /// children's visibility and where they are, then their two boxes.
    pub node: Vec<[i32; 4]>,
    /// `leaf_nodes`: a word of four for each leaf: its first primitive,
    /// the one after its last, its visibility and its primitives' type.
    pub leaf_node: Vec<[i32; 4]>,
    /// `object_node`: for each object, the root of its geometry's tree;
    /// a leaf is counted from -1 down.
    pub object_node: Vec<i32>,
    /// `prim_type`: `PRIMITIVE_TRIANGLE` and the like.
    pub prim_type: Vec<i32>,
    /// `prim_visibility`: the rays that see each primitive.
    pub prim_visibility: Vec<u32>,
    /// `prim_index`: the primitive a place in the tree stands for, since
    /// a spatial split has one in several places; -1 for an object.
    pub prim_index: Vec<i32>,
    /// `prim_object`: the object each primitive is of.
    pub prim_object: Vec<i32>,
    /// `prim_time`: the span of the shutter each primitive's leaf holds
    /// for; empty unless the tree keeps time steps.
    pub prim_time: Vec<[f32; 2]>,
    /// `root_index`: 0, or -1 when the tree is one leaf.
    pub root_index: i32,
}

/// `BVH::create`: a tree to be built with these settings. `None` for a
/// layout that is not ported.
pub fn create(param: BvhParam) -> Option<Bvh2> {
    (param.bvh_layout == BvhLayout::of("BVH2")).then(|| Bvh2::new(param))
}
