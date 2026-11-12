// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/scene/geometry.h`, `geometry.cpp` and
//! `geometry_attributes.cpp` (`Geometry`): what a mesh, hair, a point
//! cloud, a volume and a light have in common. Its positions are the
//! attribute `ATTR_STD_POSITION`.
//!
//! The sockets are the `geometry_base` type of `data/node.tsv` and those
//! of the kind's own type. The shaders it uses are places in the scene's
//! list of shaders. Its own tree is in `bvh.rs`.

mod attribute;
mod bvh;
mod manager;
mod map;
mod mesh;
mod shader;

pub use manager::{GeometryInput, GeometryManager};
pub use map::find_attribute;
pub use shader::{AttributeIdMap, Shader};

use super::{Attribute, AttributePrimitive, AttributeRequestSet, AttributeSet, AttributeStandard};
use crate::bvh::Bvh2;
use crate::graph::Node;
use crate::util::{BoundBox, Transform};

/// `Geometry::Type`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GeometryKind {
    Mesh,
    Hair,
    Volume,
    PointCloud,
    AreaLight,
    BackgroundLight,
    PointLight,
    SpotLight,
    SunLight,
}

impl GeometryKind {
    /// The kind as `data/attribute.tsv` and `data/node.tsv` write it. A
    /// light has no attributes of its own.
    pub fn key(self) -> Option<&'static str> {
        match self {
            GeometryKind::Mesh => Some("mesh"),
            GeometryKind::Hair => Some("hair"),
            GeometryKind::Volume => Some("volume"),
            GeometryKind::PointCloud => Some("pointcloud"),
            _ => None,
        }
    }

    /// `is_light`.
    pub fn is_light(self) -> bool {
        self.key().is_none()
    }

    /// A mesh, or a volume, which is drawn as the mesh around it.
    pub fn is_mesh_like(self) -> bool {
        matches!(self, GeometryKind::Mesh | GeometryKind::Volume)
    }
}

/// `Geometry`.
#[derive(Debug, Clone, PartialEq)]
pub struct Geometry {
    /// The settings, by socket name.
    pub node: Node,
    /// `geometry_type`.
    pub kind: GeometryKind,
    /// `attributes`.
    pub attribute: AttributeSet,
    /// `used_shaders`: places in the scene's list of shaders.
    pub used_shader: Vec<usize>,
    /// Its own tree, when it is instanced.
    pub bvh: Option<Bvh2>,
    /// `bounds`: the box it fits in, in its own space until its
    /// transform is applied.
    pub bound: BoundBox,
    /// The one object's transform is in the positions already.
    pub transform_applied: bool,
    pub transform_negative_scaled: bool,
    pub transform_normal: Transform,
    /// Its first entry in the attribute map.
    pub attr_map_offset: usize,
    /// Its first primitive in the device arrays.
    pub prim_offset: usize,
    /// A shader of it has a volume; set when the device is updated.
    pub has_volume: bool,
    pub has_surface_bssrdf: bool,
    pub need_update_rebuild: bool,
    pub need_update_bvh_for_offset: bool,
    /// Its place in the scene's list, while the device is updated.
    pub index: usize,
}

impl Geometry {
    /// A geometry of `kind`, every socket at its default. `None` for a
    /// kind `data/node.tsv` has no type for under that name.
    pub fn new(kind: GeometryKind) -> Option<Geometry> {
        Some(Geometry {
            node: Node::new(kind.key()?)?,
            kind,
            attribute: AttributeSet::new(kind, AttributePrimitive::Geometry),
            used_shader: Vec::new(),
            bvh: None,
            bound: BoundBox::EMPTY,
            transform_applied: false,
            transform_negative_scaled: false,
            transform_normal: Transform::IDENTITY,
            attr_map_offset: 0,
            prim_offset: 0,
            has_volume: false,
            has_surface_bssrdf: false,
            need_update_rebuild: false,
            need_update_bvh_for_offset: false,
            index: 0,
        })
    }

    /// `clear`.
    pub fn clear(&mut self, preserve_shader: bool) {
        match preserve_shader {
            true => {}
            false => self.used_shader.clear(),
        }
        self.transform_applied = false;
        self.transform_negative_scaled = false;
        self.transform_normal = Transform::IDENTITY;
        self.node.tag_modified();
    }

    /// `set_used_shaders`.
    pub fn set_used_shader(&mut self, shader: Vec<usize>) {
        match self.used_shader == shader {
            true => {}
            false => {
                self.used_shader = shader;
                self.node.tag_socket_modified("used_shaders");
            }
        }
    }

    /// The position attribute.
    pub fn position_attribute(&self) -> Option<&Attribute> {
        self.attribute.find_standard(AttributeStandard::of("POSITION"))
    }

    /// `get_position`: the positions at the centre of the motion.
    pub fn position(&self) -> &[[f32; 3]] {
        self.position_attribute().map(|attribute| attribute.center.float3()).unwrap_or(&[])
    }

    /// `tag_position_modified`.
    pub fn tag_position_modified(&mut self) {
        let position = self.attribute.find_standard_mut(AttributeStandard::of("POSITION"));
        position.into_iter().for_each(|attribute| attribute.modified = true);
        self.node.tag_modified();
    }

    /// `get_position_for_write`: the positions, tagged as changed.
    pub fn position_mut(&mut self) -> &mut [[f32; 3]] {
        self.tag_position_modified();
        let position = self.attribute.find_standard_mut(AttributeStandard::of("POSITION"));
        position.map(|attribute| attribute.center.float3_mut()).unwrap_or(&mut [])
    }

    /// `position_is_modified`.
    pub fn position_is_modified(&self) -> bool {
        self.position_attribute().is_some_and(|attribute| attribute.modified)
    }

    /// `get_motion_steps`.
    pub fn motion_step_count(&self) -> u32 {
        self.node.uint("motion_steps")
    }

    /// `get_use_motion_blur`.
    pub fn use_motion_blur(&self) -> bool {
        self.node.boolean("use_motion_blur")
    }

    /// `motion_time`: a step's time, from -1 to 1.
    pub fn motion_time(&self, step: u32) -> f32 {
        let step_count = self.motion_step_count();
        match step_count > 1 {
            true => 2.0 * step as f32 / (step_count - 1) as f32 - 1.0,
            false => 0.0,
        }
    }

    /// `motion_step`: the motion step of an attribute at a time, when
    /// there is one exactly then. The centre is not among them: it is the
    /// attribute's own values.
    pub fn motion_step(&self, time: f32) -> Option<usize> {
        let step_count = self.motion_step_count();
        let before = |step: u32| (0..step).filter(|earlier| *earlier != step_count / 2).count();
        (0..step_count).filter(|_| step_count > 1).find(|step| self.motion_time(*step) == time).map(before)
    }

    /// `is_instanced`: it keeps a BVH of its own, which objects share.
    pub fn is_instanced(&self) -> bool {
        !self.transform_applied || self.has_surface_bssrdf
    }

    /// `has_true_displacement`: a shader of it moves its vertices.
    pub fn has_true_displacement(&self, shader: &[Shader]) -> bool {
        self.shader_list(shader).any(|shader| shader.has_true_displacement)
    }

    /// `Geometry::has_motion_blur`: it is blurred and its positions move.
    pub fn has_motion_blur(&self) -> bool {
        self.use_motion_blur() && self.position_attribute().is_some_and(Attribute::has_motion)
    }

    /// `has_voxel_attributes`.
    pub fn has_voxel_attribute(&self) -> bool {
        self.attribute.attribute.iter().any(|attribute| attribute.element.has("VOXEL"))
    }

    /// The shaders it uses, out of the scene's.
    pub fn shader_list<'a>(&'a self, shader: &'a [Shader]) -> impl Iterator<Item = &'a Shader> {
        self.used_shader.iter().filter_map(|index| shader.get(*index))
    }

    /// `need_attribute(scene, std)`: the scene or a shader of it asks for
    /// the standard attribute.
    pub fn need_standard(&self, global: &AttributeRequestSet, shader: &[Shader], std: AttributeStandard) -> bool {
        std != AttributeStandard::NONE && (global.find_standard(std) || self.shader_list(shader).any(|shader| shader.attribute.find_standard(std)))
    }

    /// `need_attribute(scene, name)`.
    pub fn need_name(&self, global: &AttributeRequestSet, shader: &[Shader], name: &str) -> bool {
        !name.is_empty() && (global.find_name(name) || self.shader_list(shader).any(|shader| shader.attribute.find_name(name)))
    }

    /// `needed_attributes`: everything its shaders ask for.
    pub fn needed_attribute(&self, shader: &[Shader]) -> AttributeRequestSet {
        let mut result = AttributeRequestSet::new();
        self.shader_list(shader).for_each(|shader| result.add_set(&shader.attribute));
        result
    }
}

#[cfg(test)]
pub(crate) mod test;
