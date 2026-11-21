// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/scene/geometry.h` and `geometry.cpp`
//! (`GeometryManager`: its update flags, `tag_update`, `need_update`,
//! `update_interactive_motion`, `geom_calc_offset`,
//! `device_update_preprocess`, `device_update`, `device_free`): the
//! geometry of a scene, flattened into the device's arrays.
//!
//! Of `device_update`'s steps these are here: normals, generated
//! coordinates, the offsets, static transforms, each geometry's tree and
//! bounds, the scene's tree, attributes and triangles (the trees are in
//! `bvh.rs`). Not ported: tessellation, tangents, displacement, hair
//! shadow transparency and volumes. Cycles tags each device array as
//! modified or to reallocate (`device_update_preprocess`) and copies only
//! those; here an update writes the arrays whole, and the scene's tree is
//! kept until it is built again.

use super::{AttributeIdMap, Geometry, Shader};
use crate::bvh::{Bvh2, BvhLayout, BvhType};
use crate::kernel::flag;
use crate::scene::{AttributeRequestSet, AttributeStandard, DeviceScene, Mesh, MotionType, Object, ObjectManager, SceneParam};

/// What the geometry reads of the rest of the scene.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GeometryInput<'a> {
    /// `Scene::shaders`, as far as geometry reads them.
    pub shader: &'a [Shader],
    /// `Scene::default_surface`.
    pub default_surface: &'a Shader,
    /// `Scene::need_global_attributes`: what every geometry must have,
    /// such as UVs for a UV pass.
    pub global_attribute: &'a AttributeRequestSet,
    /// `Scene::need_motion`.
    pub need_motion: MotionType,
    /// `Scene::params`. With a static tree, transforms may be put into
    /// the geometry.
    pub param: &'a SceneParam,
    /// `Device::get_bvh_layout_mask`: the layouts the device can walk,
    /// as bits.
    pub bvh_layout_mask: i32,
}

/// `GeometryManager`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeometryManager {
    update_flag: u32,
    /// The flags of each geometry must be worked out again.
    pub need_flag_update: bool,
}

impl Default for GeometryManager {
    fn default() -> Self {
        Self::new()
    }
}

impl GeometryManager {
    /// A manager with everything to update.
    pub fn new() -> GeometryManager {
        GeometryManager { update_flag: Self::flag("UPDATE_ALL"), need_flag_update: true }
    }

    /// One of the manager's update flags, such as `GEOMETRY_MODIFIED`.
    pub fn flag(name: &str) -> u32 {
        flag(&format!("GeometryManager::{name}"))
    }

    /// `need_update`.
    pub fn need_update(&self) -> bool {
        self.update_flag != Self::flag("UPDATE_NONE")
    }

    /// `tag_update`: something about the geometry changed. The answer is
    /// the flag the object manager is to be tagged with, zero when it was
    /// the one that tagged this.
    pub fn tag_update(&mut self, update: u32) -> u32 {
        self.update_flag |= update;
        match update & Self::flag("OBJECT_MANAGER") == 0 {
            true => ObjectManager::flag("GEOMETRY_MANAGER"),
            false => 0,
        }
    }

    /// `update_interactive_motion`: in a viewport a mesh's positions of
    /// the frame before are kept as its first motion step. A mesh whose
    /// step differs from where it is now has it brought up to date.
    /// `true` when one was, and the transforms are tagged.
    pub fn update_interactive_motion(&mut self, mesh: &mut [Mesh]) -> bool {
        let mut update = false;
        for mesh in mesh.iter_mut() {
            let moved = mesh.geometry.position_attribute().is_some_and(|position| position.has_motion() && position.center != *position.data(1));
            if moved {
                mesh.copy_center_to_motion_step(0);
                let position = mesh.geometry.attribute.find_standard_mut(AttributeStandard::of("POSITION"));
                position.into_iter().for_each(|position| position.modified = true);
                update = true;
            }
        }
        match update {
            true => self.update_flag |= Self::flag("TRANSFORM_MODIFIED"),
            false => {}
        }
        update
    }

    /// `geom_calc_offset`: each mesh's first triangle, face and corner
    /// among every mesh's.
    pub fn geom_calc_offset(mesh: &mut [Mesh]) {
        let (mut triangle, mut face, mut corner) = (0, 0, 0);
        for mesh in mesh.iter_mut() {
            mesh.geometry.need_update_bvh_for_offset |= mesh.geometry.prim_offset != triangle;
            (mesh.geometry.prim_offset, mesh.face_offset, mesh.corner_offset) = (triangle, face, corner);
            let size = mesh.size();
            triangle += size.triangle;
            face += size.subd_face;
            corner += size.subd_corner;
        }
    }

    /// `device_update_preprocess`, the part that changes the geometry:
    /// what each has of its shaders, and what a changed shader makes due.
    pub fn device_update_preprocess(&mut self, mesh: &mut [Mesh], shader: &[Shader]) {
        if !self.need_update() && !self.need_flag_update {
            return;
        }
        for mesh in mesh.iter_mut() {
            let any = |of: fn(&Shader) -> bool| mesh.geometry.shader_list(shader).any(of);
            let (volume, bssrdf) = (any(|shader| shader.has_volume), any(|shader| shader.has_surface_bssrdf));
            // Attributes a shader now asks for may have to be diced too.
            let attribute = any(|shader| shader.need_update_uv || shader.need_update_attribute);
            let displaced = any(|shader| shader.need_update_displacement);
            mesh.geometry.has_volume = volume;
            mesh.geometry.has_surface_bssrdf |= bssrdf;
            if attribute && mesh.need_tesselation() {
                mesh.geometry.node.tag_modified();
            }
            if displaced {
                mesh.geometry.tag_position_modified();
            }
        }
        self.need_flag_update = false;
    }

    /// `device_update`. `bvh` is the scene's tree, `Scene::bvh`.
    pub fn device_update(&mut self, dscene: &mut DeviceScene, mesh: &mut [Mesh], object: &mut [Object], id: &mut AttributeIdMap, bvh: &mut Option<Bvh2>, input: &GeometryInput) {
        if !self.need_update() {
            return;
        }
        let generated = AttributeStandard::of("GENERATED");
        for mesh in mesh.iter_mut().filter(|mesh| mesh.geometry.node.is_modified()) {
            mesh.add_vertex_normal();
            let needed = mesh.geometry.need_standard(input.global_attribute, input.shader, generated);
            mesh.update_generated(needed);
        }
        Self::device_free(dscene, false);
        let layout = BvhLayout::best(input.param.bvh_layout, input.bvh_layout_mask);
        Self::geom_calc_offset(mesh);
        if input.param.bvh_type == BvhType::Static {
            ObjectManager::apply_static_transform(dscene, object, mesh, input.shader, input.need_motion);
        }
        // The scene's tree is built again when there is none, when an
        // object moved or is seen by other rays, and when a geometry
        // changed.
        let placed = Self::flag("TRANSFORM_MODIFIED") | Self::flag("VISIBILITY_MODIFIED");
        let mut need_update_scene_bvh = bvh.is_none() || self.update_flag & placed != 0;
        for index in 0..mesh.len() {
            if mesh[index].geometry.node.is_modified() || mesh[index].geometry.need_update_bvh_for_offset {
                need_update_scene_bvh = true;
                Geometry::compute_bvh(mesh, index, dscene, input.param, layout);
            }
        }
        let motion_blur = input.need_motion == MotionType::Blur;
        for object in object.iter_mut() {
            match mesh.get(object.geometry) {
                Some(mesh) => object.compute_bound(&mesh.geometry, motion_blur),
                None => debug_assert!(false, "object {} has no geometry", object.index),
            }
        }
        if need_update_scene_bvh {
            Self::device_update_bvh(dscene, mesh, object, bvh, input);
        }
        // Set again: freeing the arrays said there is no tree to walk.
        dscene.data.set_int("bvh", "bvh_layout", layout.0);
        Self::device_update_attribute(dscene, mesh, object, id, input);
        Self::device_update_mesh(dscene, mesh, input);
        for mesh in mesh.iter_mut() {
            mesh.geometry.node.clear_modified();
            mesh.geometry.attribute.clear_modified();
            mesh.subd_attribute.clear_modified();
        }
        self.update_flag = Self::flag("UPDATE_NONE");
    }

    /// `device_free`: empty the arrays an update writes whole; with
    /// `force_free` the scene's tree too, which else stays until it is
    /// built again.
    pub fn device_free(dscene: &mut DeviceScene, force_free: bool) {
        if force_free {
            dscene.bvh_node.clear();
            dscene.bvh_leaf_node.clear();
            dscene.object_node.clear();
            dscene.prim_type.clear();
            dscene.prim_visibility.clear();
            dscene.prim_index.clear();
            dscene.prim_object.clear();
            dscene.prim_time.clear();
        }
        // A shader run meanwhile, as for displacement, must not trace rays.
        dscene.data.set_int("bvh", "bvh_layout", BvhLayout::NONE.0);
        dscene.tri_shader.clear();
        dscene.tri_vindex.clear();
        dscene.tri_vert.clear();
        dscene.attribute_map.alloc(0);
        dscene.attribute_float.clear();
        dscene.attribute_float2.clear();
        dscene.attribute_float3.clear();
        dscene.attribute_float4.clear();
        dscene.attribute_uchar4.clear();
        dscene.attribute_normal.clear();
    }
}
