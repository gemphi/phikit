// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/scene/mesh.cpp` (`Mesh::pack_shaders`,
//! `Mesh::pack_triangles`, `Mesh::has_motion_blur`,
//! `Mesh::primitive_type`): a mesh's triangles as the device takes them.

use super::Mesh;
use crate::kernel::constant;
use crate::scene::{Attribute, AttributeStandard, Shader};

impl Mesh {
    /// The mesh is subdivided: its `subdivision_type` is not `none`.
    pub fn is_subdivided(&self) -> bool {
        let node = &self.geometry.node;
        let none = node.node_type().enum_named("subdivision_type_enum").and_then(|table| table.number("none"));
        Some(node.int("subdivision_type")) != none
    }

    /// `pack_shaders`: for each triangle the shader it is drawn with and
    /// whether its normals are smooth, into `tri_shader`. `shader` is the
    /// scene's; a triangle whose shader the mesh does not have is drawn
    /// with `default_surface`.
    pub fn pack_shader(&self, shader: &[Shader], default_surface: &Shader, tri_shader: &mut [u32]) {
        let slot = self.geometry.node.int_array("shader");
        let smooth = self.geometry.node.boolean_array("smooth");
        // Corner normals say how flat a triangle is themselves, and are
        // always blended.
        let corner_normal = self.geometry.attribute.find_standard(AttributeStandard::of("CORNER_NORMAL")).is_some();
        for (index, out) in tri_shader.iter_mut().enumerate().take(self.triangle_count()) {
            let used = slot.get(index).and_then(|slot| usize::try_from(*slot).ok()).and_then(|slot| self.geometry.used_shader.get(slot));
            let used = used.and_then(|place| shader.get(*place)).unwrap_or(default_surface);
            *out = used.shader_id(corner_normal || smooth.get(index).copied().unwrap_or(false));
        }
    }

    /// `pack_triangles`: the three vertices of each triangle, into
    /// `tri_vindex`.
    pub fn pack_triangle(&self, tri_vindex: &mut [[u32; 3]]) {
        let corner = self.geometry.node.int_array("triangles");
        tri_vindex.iter_mut().zip(corner.chunks_exact(3)).for_each(|(out, corner)| *out = [corner[0] as u32, corner[1] as u32, corner[2] as u32]);
    }

    /// `Mesh::has_motion_blur`: it is blurred, and its positions move, or
    /// those of the faces it is subdivided from.
    pub fn has_motion_blur(&self) -> bool {
        let moving = self.geometry.position_attribute().is_some_and(Attribute::has_motion);
        let subd = self.subd_attribute.find_standard(AttributeStandard::of("POSITION")).is_some_and(Attribute::has_motion);
        self.geometry.use_motion_blur() && (moving || (self.is_subdivided() && subd))
    }

    /// `primitive_type`: `PRIMITIVE_MOTION_TRIANGLE` or
    /// `PRIMITIVE_TRIANGLE`.
    pub fn primitive_type(&self) -> i32 {
        let name = match self.has_motion_blur() {
            true => "PRIMITIVE_MOTION_TRIANGLE",
            false => "PRIMITIVE_TRIANGLE",
        };
        constant(name).unwrap_or(0) as i32
    }
}
