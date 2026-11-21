// SPDX-FileCopyrightText: 2009-2010 NVIDIA Corporation
// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/bvh/params.h` and `bvh.cpp` (`BVHLayout`,
//! `BVHType`, `BVHParams`, `BVHParams::best_bvh_layout`), adapted by Cycles
//! from code of NVIDIA Corporation: how a tree is to be built.
//!
//! What each setting starts as is `data/param.tsv`; the limits are
//! `data/constant.tsv`. Not ported: `bvh_layout_name`, which only words a
//! log line.

use std::sync::OnceLock;

use crate::kernel::constant;
use crate::util::param_default;

/// `BVHLayout`, the kernel's `KernelBVHLayout`: one bit for each kind of
/// tree a device can walk.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct BvhLayout(pub i32);

impl BvhLayout {
    /// `BVH_LAYOUT_NONE`.
    pub const NONE: BvhLayout = BvhLayout(0);

    /// The layout Cycles names `BVH_LAYOUT_` and `short`, such as `BVH2`.
    pub fn of(short: &str) -> BvhLayout {
        let found = constant(&format!("BVH_LAYOUT_{short}"));
        debug_assert!(found.is_some(), "data/constant.tsv has no BVH_LAYOUT_{short}");
        BvhLayout(found.unwrap_or(0) as i32)
    }

    /// The layouts whose top level holds only instances, so that every
    /// geometry has a tree of its own: those `Geometry::need_build_bvh`
    /// of `scene/geometry.cpp` names.
    pub fn instance_only(self) -> bool {
        const OWN: [&str; 12] = ["OPTIX", "MULTI_OPTIX", "METAL", "MULTI_OPTIX_EMBREE", "MULTI_METAL", "MULTI_METAL_EMBREE", "HIPRT", "MULTI_HIPRT", "MULTI_HIPRT_EMBREE", "EMBREEGPU", "MULTI_EMBREEGPU", "MULTI_EMBREEGPU_EMBREE"];
        static MASK: OnceLock<i32> = OnceLock::new();
        self.0 & MASK.get_or_init(|| OWN.iter().fold(0, |mask, short| mask | BvhLayout::of(short).0)) != 0
    }

    /// `BVHParams::best_bvh_layout`: the layout asked for when the device
    /// has it, else the widest the device has below it, else the widest
    /// the device has. `supported` is the device's layouts, as bits.
    pub fn best(requested: BvhLayout, supported: i32) -> BvhLayout {
        let below = supported & requested.0.wrapping_sub(1);
        let allowed = match below == 0 {
            true => supported,
            false => below,
        };
        match (supported & requested.0 != 0, allowed as u32) {
            (true, _) => requested,
            (false, 0) => BvhLayout::NONE,
            (false, allowed) => BvhLayout(1 << (31 - allowed.leading_zeros())),
        }
    }
}

/// `BVHType`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BvhType {
    /// `BVH_TYPE_DYNAMIC`: a changed mesh has its tree refitted. Quicker
    /// to update, slower to render.
    #[default]
    Dynamic,
    /// `BVH_TYPE_STATIC`: a change builds the tree again.
    Static,
}

impl BvhType {
    /// The type Cycles gives this number.
    pub fn of(number: i64) -> BvhType {
        match Some(number) == constant("BVH_TYPE_STATIC") {
            true => BvhType::Static,
            false => BvhType::Dynamic,
        }
    }
}

/// The sizes Cycles fixes: `BVHParams::MAX_DEPTH` and the others, the
/// bins of `BVHObjectBinning`, and the size of a packed node.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BvhLimit {
    /// `BVHParams::MAX_DEPTH`.
    pub max_depth: usize,
    /// `BVHParams::MAX_SPATIAL_DEPTH`.
    pub max_spatial_depth: usize,
    /// `BVHParams::NUM_SPATIAL_BINS`.
    pub spatial_bin_count: usize,
    /// `BVHObjectBinning::MAX_BINS`.
    pub max_bin: usize,
    /// `BVHObjectBinning::LOG_BLOCK_SIZE`.
    pub log_block_size: u32,
    /// `BVH_NODE_SIZE`: the words of four an inner node takes.
    pub node_size: usize,
    /// `BVH_NODE_LEAF_SIZE`.
    pub leaf_node_size: usize,
}

impl BvhLimit {
    /// The limits, read once.
    pub fn get() -> &'static BvhLimit {
        static LIMIT: OnceLock<BvhLimit> = OnceLock::new();
        let number = |name: &str| {
            let found = constant(name);
            debug_assert!(found.is_some(), "data/constant.tsv has no {name}");
            found.unwrap_or(1) as usize
        };
        LIMIT.get_or_init(|| BvhLimit {
            max_depth: number("BVHParams::MAX_DEPTH"),
            max_spatial_depth: number("BVHParams::MAX_SPATIAL_DEPTH"),
            spatial_bin_count: number("BVHParams::NUM_SPATIAL_BINS"),
            max_bin: number("BVHObjectBinning::MAX_BINS"),
            log_block_size: number("BVHObjectBinning::LOG_BLOCK_SIZE") as u32,
            node_size: number("BVH_NODE_SIZE"),
            leaf_node_size: number("BVH_NODE_LEAF_SIZE"),
        })
    }
}

/// `BVHParams`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BvhParam {
    /// A primitive may be cut in two where that makes tighter boxes.
    pub use_spatial_split: bool,
    /// How much two halves must overlap, as a share of the whole tree's
    /// area, before a cut is tried.
    pub spatial_split_alpha: f32,
    pub unaligned_split_threshold: f32,
    /// What walking into a node costs, against testing a primitive.
    pub sah_node_cost: f32,
    pub sah_primitive_cost: f32,
    pub min_leaf_size: usize,
    pub max_triangle_leaf_size: usize,
    pub max_motion_triangle_leaf_size: usize,
    pub max_curve_leaf_size: usize,
    pub max_motion_curve_leaf_size: usize,
    pub max_point_leaf_size: usize,
    pub max_motion_point_leaf_size: usize,
    /// The tree over the scene's objects, not over one geometry.
    pub top_level: bool,
    pub bvh_layout: BvhLayout,
    /// `use_unaligned_nodes`: for curves. Not ported.
    pub use_unaligned_node: bool,
    pub use_compact_structure: bool,
    /// `num_motion_triangle_steps`: a moving triangle gets a leaf for each
    /// of twice this many spans of the shutter.
    pub motion_triangle_step_count: usize,
    /// `num_motion_curve_steps`.
    pub motion_curve_step_count: usize,
    /// `num_motion_point_steps`.
    pub motion_point_step_count: usize,
    pub bvh_type: BvhType,
    /// `curve_subdivisions`.
    pub curve_subdivision: i32,
}

impl Default for BvhParam {
    fn default() -> Self {
        let number = |name: &str| param_default("BVHParams", name);
        let (flag, size) = (|name: &str| number(name) != 0.0, |name: &str| number(name) as usize);
        BvhParam {
            use_spatial_split: flag("use_spatial_split"),
            spatial_split_alpha: number("spatial_split_alpha") as f32,
            unaligned_split_threshold: number("unaligned_split_threshold") as f32,
            sah_node_cost: number("sah_node_cost") as f32,
            sah_primitive_cost: number("sah_primitive_cost") as f32,
            min_leaf_size: size("min_leaf_size"),
            max_triangle_leaf_size: size("max_triangle_leaf_size"),
            max_motion_triangle_leaf_size: size("max_motion_triangle_leaf_size"),
            max_curve_leaf_size: size("max_curve_leaf_size"),
            max_motion_curve_leaf_size: size("max_motion_curve_leaf_size"),
            max_point_leaf_size: size("max_point_leaf_size"),
            max_motion_point_leaf_size: size("max_motion_point_leaf_size"),
            top_level: flag("top_level"),
            bvh_layout: BvhLayout(number("bvh_layout") as i32),
            use_unaligned_node: flag("use_unaligned_nodes"),
            use_compact_structure: flag("use_compact_structure"),
            motion_triangle_step_count: size("num_motion_triangle_steps"),
            motion_curve_step_count: size("num_motion_curve_steps"),
            motion_point_step_count: size("num_motion_point_steps"),
            bvh_type: BvhType::of(number("bvh_type") as i64),
            curve_subdivision: number("curve_subdivisions") as i32,
        }
    }
}

impl BvhParam {
    /// `cost`: what a node with this many children and primitives costs.
    pub fn cost(&self, node_count: usize, primitive_count: usize) -> f32 {
        self.node_cost(node_count) + self.primitive_cost(primitive_count)
    }

    /// `primitive_cost`.
    pub fn primitive_cost(&self, count: usize) -> f32 {
        count as f32 * self.sah_primitive_cost
    }

    /// `node_cost`.
    pub fn node_cost(&self, count: usize) -> f32 {
        count as f32 * self.sah_node_cost
    }

    /// `small_enough_for_leaf`: too few primitives to split, or too deep.
    pub fn small_enough_for_leaf(&self, size: usize, level: usize) -> bool {
        size <= self.min_leaf_size || level >= BvhLimit::get().max_depth
    }

    /// `use_motion_steps`: leaves are made for spans of the shutter.
    pub fn use_motion_step(&self) -> bool {
        self.motion_curve_step_count > 0 || self.motion_triangle_step_count > 0 || self.motion_point_step_count > 0
    }
}
