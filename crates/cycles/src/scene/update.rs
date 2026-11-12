// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/scene/scene.cpp` (`Scene::device_update`,
//! lines 380-560): the managers run in Cycles' order, each writing its part
//! of the device scene. The shaders first (the surfaces, then the world's),
//! then the background, the camera, the geometry's preprocess, the objects
//! and the geometry with its tree, the integrator and the film. The light
//! manager, the particle systems, the images, baking and the scene
//! attributes are not here; the managers are made anew for each update and
//! not kept, so everything is written whole.
use super::background::Background;
use super::{AttributeIdMap, AttributeRequestSet, DeviceScene, GeometryInput, GeometryManager, IntegratorInput, LookupTableSet, MotionType, ObjectInput, ObjectManager, Scene, Shader, ShaderManager, Surface};
use crate::bvh::BvhLayout;

impl Scene {
    /// The surfaces of the shaders the kernel finds: the scene's, the
    /// world's after them, and `default_surface` (a triangle whose shader
    /// is missing) after that.
    pub fn shader_surface(&self) -> Vec<Surface> {
        let mut surface = self.surface.clone();
        surface.push(Surface::Background { color: self.world.color, strength: self.world.strength });
        surface.push(Surface::default());
        surface
    }

    /// `Scene::device_update`: the device scene of this scene.
    pub fn device_update(&mut self) -> DeviceScene {
        // A new device scene holds nothing, so every manager writes its part
        // whole, whether or not it was updated before.
        self.camera.need_device_update = true;
        self.film.node.tag_modified();
        self.integrator.node.tag_modified();
        self.lookup = LookupTableSet::new();
        let mut dscene = DeviceScene::new();
        // The shaders, then the world's among them.
        let surface = self.shader_surface();
        ShaderManager::device_update(&mut dscene, &surface);
        let world_shader = Shader { id: self.shader.len() as u32, ..Shader::default() };
        let background = Background { transparent: self.background.transparent, ..Background::default() };
        background.device_update(&mut dscene, &world_shader, true);
        // The camera, with the lookup tables the shutter would need.
        self.camera.device_update(&mut dscene, &mut self.lookup, MotionType::None);
        // The geometry's preprocess, the objects and the geometry.
        let mut geometry = GeometryManager::new();
        let mut objects = ObjectManager::new();
        let default_surface = Shader { id: world_shader.id + 1, ..Shader::default() };
        let global = AttributeRequestSet::new();
        let mut id = AttributeIdMap::new();
        let mut bvh = None;
        geometry.device_update_preprocess(&mut self.mesh, &self.shader);
        objects.device_update(&mut dscene, &mut self.object, &self.mesh, &ObjectInput { need_motion: MotionType::None, particle_count: &[], lightgroup: &[] });
        let input = GeometryInput { shader: &self.shader, default_surface: &default_surface, global_attribute: &global, need_motion: MotionType::None, param: &self.param, bvh_layout_mask: BvhLayout::of("BVH2").0 };
        geometry.device_update(&mut dscene, &mut self.mesh, &mut self.object, &mut id, &mut bvh, &input);
        objects.device_update_flag(&mut dscene, &mut self.object, &self.mesh, true);
        // The integrator and the film.
        self.integrator.device_update(&mut dscene, &IntegratorInput { exposure: self.film.node.float("exposure"), ..IntegratorInput::default() });
        crate::scene::fill_sample_pattern(&mut dscene);
        self.film.device_update(&mut dscene, &mut self.lookup, &[crate::scene::Film::default_pass()], false);
        dscene
    }
}

#[cfg(test)]
mod test;
