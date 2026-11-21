// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/scene/scene.h` (`SceneParams`): what a scene
//! is made with. A change to most of it means the scene is made again.
//!
//! What each setting starts as is `data/param.tsv`.

use crate::bvh::{BvhLayout, BvhType};
use crate::session::ShadingSystem;
use crate::util::param_default;

/// `SceneParams`.
#[derive(Debug, Clone, PartialEq)]
pub struct SceneParam {
    /// `shadingsystem`.
    pub shading_system: ShadingSystem,
    /// The widest tree wanted; the device may only have a narrower one.
    pub bvh_layout: BvhLayout,
    pub bvh_type: BvhType,
    pub use_bvh_spatial_split: bool,
    pub use_bvh_compact_structure: bool,
    /// `use_bvh_unaligned_nodes`.
    pub use_bvh_unaligned_node: bool,
    /// `num_bvh_time_steps`: a moving primitive gets a leaf for each of
    /// twice this many spans of the shutter. Zero gives it one.
    pub bvh_time_step_count: usize,
    /// `hair_subdivisions`.
    pub hair_subdivision: i32,
    /// `hair_shape`: a `CurveShapeType`, by its number.
    pub hair_shape: i32,
    pub texture_resolution: f32,
    pub texture_limit: i32,
    pub use_texture_cache: bool,
    pub auto_texture_cache: bool,
    pub texture_cache_path: String,
    pub background: bool,
}

impl Default for SceneParam {
    fn default() -> Self {
        let number = |name: &str| param_default("SceneParams", name);
        let flag = |name: &str| number(name) != 0.0;
        SceneParam {
            shading_system: ShadingSystem::default(),
            bvh_layout: BvhLayout(number("bvh_layout") as i32),
            bvh_type: BvhType::of(number("bvh_type") as i64),
            use_bvh_spatial_split: flag("use_bvh_spatial_split"),
            use_bvh_compact_structure: flag("use_bvh_compact_structure"),
            use_bvh_unaligned_node: flag("use_bvh_unaligned_nodes"),
            bvh_time_step_count: number("num_bvh_time_steps") as usize,
            hair_subdivision: number("hair_subdivisions") as i32,
            hair_shape: number("hair_shape") as i32,
            texture_resolution: number("texture_resolution") as f32,
            texture_limit: number("texture_limit") as i32,
            use_texture_cache: flag("use_texture_cache"),
            auto_texture_cache: flag("auto_texture_cache"),
            texture_cache_path: String::new(),
            background: flag("background"),
        }
    }
}

impl SceneParam {
    /// `modified`: whether the scene has to be made again. Whether it
    /// renders in the background is not looked at.
    pub fn modified(&self, other: &SceneParam) -> bool {
        *self != SceneParam { background: self.background, ..other.clone() }
    }

    /// `curve_subdivisions`: the pieces a curve is cut into, no more than
    /// Embree would.
    pub fn curve_subdivision(&self) -> i32 {
        (1 << self.hair_subdivision.clamp(0, 30)).clamp(1, 16)
    }
}
