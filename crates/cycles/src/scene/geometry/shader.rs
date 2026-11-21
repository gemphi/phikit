// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! What a geometry and an object read of a shader (`scene/shader.h`,
//! `Shader`), handed in until the shader itself is ported, and of the
//! shader manager what the geometry needs now:
//! `ShaderManager::get_shader_id` and `get_attribute_id`
//! (`scene/shader.cpp:483-518`). The shader's graph is in
//! `scene/shader/graph`.

use crate::kernel::flag;
use crate::scene::{AttributeRequestSet, AttributeStandard};

/// `ShaderManager::unique_attribute_id`: a number for each attribute a
/// shader asks for by name, by which the attribute map finds it.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct AttributeIdMap {
    name: Vec<String>,
}

impl AttributeIdMap {
    pub fn new() -> AttributeIdMap {
        AttributeIdMap::default()
    }

    /// `get_attribute_id(name)`: the name's number, a new one the first
    /// time it is asked for. The numbers start after the standards'.
    pub fn id(&mut self, name: &str) -> u64 {
        let known = self.name.iter().position(|known| known == name);
        let place = known.unwrap_or_else(|| {
            self.name.push(name.to_string());
            self.name.len() - 1
        });
        AttributeStandard::count() as u64 + place as u64
    }

    /// `get_attribute_id(std)`: a standard attribute's own number.
    pub fn standard_id(std: AttributeStandard) -> u64 {
        std.0 as u64
    }
}

/// `Shader`, so far the members the geometry and the objects read.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Shader {
    /// `id`: its place in the device's shaders.
    pub id: u32,
    pub has_volume: bool,
    pub has_surface_bssrdf: bool,
    /// `has_displacement` with a displacement method other than bump: it
    /// moves vertices.
    pub has_true_displacement: bool,
    /// `emission_sampling` is not `EMISSION_SAMPLING_NONE`: a surface
    /// with it is sampled as a light.
    pub emission: bool,
    /// `attributes`: what it asks a geometry to have.
    pub attribute: AttributeRequestSet,
    /// `need_update_uvs`.
    pub need_update_uv: bool,
    pub need_update_attribute: bool,
    pub need_update_displacement: bool,
}

impl Shader {
    /// `ShaderManager::get_shader_id`: the shader as a primitive names it,
    /// with whether its normals are smooth. Every shader casts shadows.
    pub fn shader_id(&self, smooth: bool) -> u32 {
        let smooth = match smooth {
            true => flag("SHADER_SMOOTH_NORMAL"),
            false => 0,
        };
        self.id | smooth | flag("SHADER_CAST_SHADOW")
    }
}
