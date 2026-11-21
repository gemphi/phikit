// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/scene/background.cpp` (`Background::
//! device_update`, lines 50-120): the world's shader as the kernel finds it,
//! with the rays that see it. The light of the background (its importance
//! map and `use_mis`) belongs to the light manager and waits for gate 3.
use crate::kernel::flag;
use crate::scene::{DeviceScene, Shader};

/// `Background`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Background {
    /// Film > Transparent: a ray that hits nothing adds transparency, not
    /// the world's colour.
    pub transparent: bool,
    /// `visibility`: the `PATH_RAY_VISIBILITY_*` bits of the rays that see
    /// the world. All of them, as Blender's default world has.
    pub visibility: u32,
}

impl Default for Background {
    fn default() -> Self {
        let camera = flag("PATH_RAY_VISIBILITY_CAMERA");
        let seen = camera | flag("PATH_RAY_VISIBILITY_DIFFUSE") | flag("PATH_RAY_VISIBILITY_GLOSSY") | flag("PATH_RAY_VISIBILITY_TRANSMIT") | flag("PATH_RAY_VISIBILITY_VOLUME_SCATTER");
        Self { transparent: false, visibility: seen }
    }
}

impl Background {
    /// `device_update`: `shader` is the world's shader, which has a
    /// background node when `has_node`.
    pub fn device_update(&self, dscene: &mut DeviceScene, shader: &Shader, has_node: bool) {
        let data = &mut dscene.data;
        data.set_int("background", "transparent", i32::from(self.transparent));
        let mut surface_shader = shader.shader_id(false);
        // No background node: the world is invisible to every ray, so the
        // kernel skips its shader.
        let exclude = [
            ("PATH_RAY_VISIBILITY_DIFFUSE", "SHADER_EXCLUDE_DIFFUSE"),
            ("PATH_RAY_VISIBILITY_GLOSSY", "SHADER_EXCLUDE_GLOSSY"),
            ("PATH_RAY_VISIBILITY_TRANSMIT", "SHADER_EXCLUDE_TRANSMIT"),
            ("PATH_RAY_VISIBILITY_VOLUME_SCATTER", "SHADER_EXCLUDE_SCATTER"),
            ("PATH_RAY_VISIBILITY_CAMERA", "SHADER_EXCLUDE_CAMERA"),
        ];
        match has_node {
            false => surface_shader |= flag("SHADER_EXCLUDE_ANY"),
            true => exclude.iter().filter(|(ray, _)| self.visibility & flag(ray) == 0).for_each(|(_, shader_flag)| surface_shader |= flag(shader_flag)),
        }
        data.set_int("background", "surface_shader", surface_shader as i32);
        data.set_int("background", "volume_shader", -1);
        data.set_int("background", "object_index", -1);
        data.set_float("background", "transparent_roughness_squared_threshold", -1.0);
    }
}
