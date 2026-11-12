// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Tests of this module. It also holds the small scene the others, and the
//! tests of the objects, update a device with.


pub(crate) use crate::bvh::{Bvh2, BvhLayout, BvhType, PackedBvh};
pub(crate) use crate::graph::Value;
pub(crate) use crate::scene::*;
pub(crate) use crate::scene::mesh::test::{mesh_of, moving_square, square, std};

/// The parts of a scene the objects and the geometry are updated from.
pub(crate) struct Rig {
    pub dscene: DeviceScene,
    pub mesh: Vec<Mesh>,
    pub object: Vec<Object>,
    pub shader: Vec<Shader>,
    pub default_surface: Shader,
    pub global: AttributeRequestSet,
    pub id: AttributeIdMap,
    pub geometry: GeometryManager,
    pub object_manager: ObjectManager,
    pub need_motion: MotionType,
    pub param: SceneParam,
    /// `Scene::bvh`.
    pub bvh: Option<Bvh2>,
    /// The layouts the device can walk.
    pub bvh_layout_mask: i32,
    pub lightgroup: Vec<String>,
}

impl Rig {
    /// These meshes, each placed once, and one shader numbered 7.
    pub fn new(mesh: Vec<Mesh>) -> Rig {
        Rig {
            dscene: DeviceScene::new(),
            object: (0..mesh.len()).map(Object::new).collect(),
            mesh,
            shader: vec![Shader { id: 7, ..Shader::default() }],
            default_surface: Shader { id: 99, ..Shader::default() },
            global: AttributeRequestSet::new(),
            id: AttributeIdMap::new(),
            geometry: GeometryManager::new(),
            object_manager: ObjectManager::new(),
            need_motion: MotionType::None,
            param: SceneParam::default(),
            bvh: None,
            bvh_layout_mask: BvhLayout::of("BVH2").0,
            lightgroup: Vec::new(),
        }
    }

    /// The objects alone: `ObjectManager::device_update`.
    pub fn update_object(&mut self) {
        let input = ObjectInput { need_motion: self.need_motion, particle_count: &[3, 5], lightgroup: &self.lightgroup };
        self.object_manager.device_update(&mut self.dscene, &mut self.object, &self.mesh, &input);
    }

    /// The steps of `Scene::device_update` for the objects and the
    /// geometry, in its order.
    pub fn update(&mut self) {
        self.geometry.device_update_preprocess(&mut self.mesh, &self.shader);
        self.update_object();
        let input = GeometryInput { shader: &self.shader, default_surface: &self.default_surface, global_attribute: &self.global, need_motion: self.need_motion, param: &self.param, bvh_layout_mask: self.bvh_layout_mask };
        self.geometry.device_update(&mut self.dscene, &mut self.mesh, &mut self.object, &mut self.id, &mut self.bvh, &input);
        self.object_manager.device_update_flag(&mut self.dscene, &mut self.object, &self.mesh, true);
    }
}

impl Rig {
    /// The tree the device was given, as a packed tree to walk.
    pub fn packed(&self) -> PackedBvh {
        let dscene = &self.dscene;
        PackedBvh {
            node: dscene.bvh_node.clone(),
            leaf_node: dscene.bvh_leaf_node.clone(),
            object_node: dscene.object_node.clone(),
            prim_type: dscene.prim_type.clone(),
            prim_visibility: dscene.prim_visibility.clone(),
            prim_index: dscene.prim_index.clone(),
            prim_object: dscene.prim_object.clone(),
            prim_time: dscene.prim_time.clone(),
            root_index: dscene.data.int("bvh", "root"),
        }
    }
}

pub(crate) fn geometry() -> Geometry {
    Geometry::new(GeometryKind::Mesh).expect("data/node.tsv has the mesh")
}

#[test]
fn a_geometry_is_made_for_the_kinds_that_have_a_node_type() {
    assert!(Geometry::new(GeometryKind::Hair).is_some() && Geometry::new(GeometryKind::PointCloud).is_some());
    assert!(Geometry::new(GeometryKind::SunLight).is_none());
    assert!(GeometryKind::SunLight.is_light() && !GeometryKind::Volume.is_light());
    assert!(GeometryKind::Volume.is_mesh_like() && !GeometryKind::Hair.is_mesh_like());
}

#[test]
fn a_new_geometry_has_no_box_and_its_transform_is_not_applied() {
    let geometry = geometry();
    assert!(!geometry.bound.valid() && !geometry.transform_applied && geometry.is_instanced());
    assert!(geometry.position().is_empty() && !geometry.position_is_modified() && !geometry.has_motion_blur());
    assert_eq!((geometry.motion_step_count(), geometry.use_motion_blur()), (0, false));
}

#[test]
fn a_geometry_with_its_transform_applied_is_instanced_only_for_subsurface() {
    let mut geometry = geometry();
    geometry.transform_applied = true;
    assert!(!geometry.is_instanced());
    geometry.has_surface_bssrdf = true;
    assert!(geometry.is_instanced());
}

#[test]
fn motion_steps_run_from_the_shutter_s_open_to_its_close() {
    let mut geometry = geometry();
    assert_eq!((geometry.motion_time(0), geometry.motion_step(0.0)), (0.0, None));
    geometry.node.set("motion_steps", Value::Uint(3));
    assert_eq!([0, 1, 2].map(|step| geometry.motion_time(step)), [-1.0, 0.0, 1.0]);
    // The centre is not a motion step of an attribute: before is 0, after is 1.
    assert_eq!((geometry.motion_step(-1.0), geometry.motion_step(1.0), geometry.motion_step(0.5)), (Some(0), Some(1), None));
}

#[test]
fn writing_positions_tags_the_geometry() {
    let mut mesh = square();
    mesh.geometry.node.clear_modified();
    mesh.geometry.attribute.clear_modified();
    assert!(!mesh.geometry.position_is_modified());
    mesh.geometry.position_mut()[0] = [5.0; 3];
    assert!(mesh.geometry.position_is_modified() && mesh.geometry.node.is_modified());
    assert_eq!(mesh.geometry.position()[0], [5.0; 3]);
}

#[test]
fn setting_the_same_shaders_is_no_change() {
    let mut geometry = geometry();
    geometry.node.clear_modified();
    geometry.set_used_shader(Vec::new());
    assert!(!geometry.node.is_modified());
    geometry.set_used_shader(vec![1]);
    assert!(geometry.node.socket_is_modified("used_shaders"));
}

#[test]
fn a_geometry_needs_what_the_scene_or_its_own_shaders_ask_for() {
    let mut asking = Shader::default();
    asking.attribute.add_standard(std("GENERATED"));
    asking.attribute.add_name("weight");
    let shader = [Shader::default(), asking];
    let mut global = AttributeRequestSet::new();
    global.add_standard(std("UV"));

    let mut geometry = geometry();
    geometry.set_used_shader(vec![0]);
    assert!(geometry.need_standard(&global, &shader, std("UV")) && !geometry.need_standard(&global, &shader, std("GENERATED")));
    assert!(!geometry.need_name(&global, &shader, "weight") && geometry.needed_attribute(&shader).is_empty());
    geometry.set_used_shader(vec![0, 1]);
    assert!(geometry.need_standard(&global, &shader, std("GENERATED")) && geometry.need_name(&global, &shader, "weight"));
    assert!(!geometry.need_standard(&global, &shader, AttributeStandard::NONE) && !geometry.need_name(&global, &shader, ""));
    assert_eq!(geometry.needed_attribute(&shader).len(), 2);
}

#[test]
fn true_displacement_is_a_shader_s_to_say() {
    let shader = [Shader::default(), Shader { has_true_displacement: true, ..Shader::default() }];
    let mut geometry = geometry();
    geometry.set_used_shader(vec![0]);
    assert!(!geometry.has_true_displacement(&shader));
    geometry.set_used_shader(vec![0, 1]);
    assert!(geometry.has_true_displacement(&shader));
}
