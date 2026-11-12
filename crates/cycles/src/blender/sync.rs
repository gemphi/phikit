// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/blender/sync.cpp` (`BlenderSync::sync_data`),
//! `blender/object.cpp` (`sync_object`), `blender/shader.cpp`
//! (`sync_materials`, `sync_world`) and `blender/camera.cpp`
//! (`sync_camera`): a [`SceneSource`] into a Cycles [`Scene`]. A material
//! is a diffuse colour or an emission; the node graph waits for gate 2.
use super::camera::{sync_camera, transform_of};
use super::mesh::sync_mesh;
use super::integrator::{sync_film, sync_integrator};
use super::source::{MaterialSource, NodeSurface, SceneSource};
use crate::scene::{Object, Scene, Surface, World};

/// A material as the surface it stands for: the surface of its graph when
/// that is a lone Diffuse BSDF or Emission node (`sync_materials`), else an
/// emission when the Principled BSDF emits, else a diffuse of its base colour.
pub fn surface_of(material: &MaterialSource) -> Surface {
    match &material.node {
        NodeSurface::Diffuse { color, .. } => return Surface::Diffuse { color: *color },
        NodeSurface::Emission { color, strength } => return Surface::Emission { color: *color, strength: *strength },
        NodeSurface::Unknown => {}
    }
    // A material with no node tree is a diffuse BSDF of its diffuse colour.
    if material.node_kind.is_empty() {
        return Surface::Diffuse { color: material.diffuse_color };
    }
    let emits = material.emission_strength > 0.0 && material.emission_color.iter().any(|&channel| channel > 0.0);
    match emits {
        true => Surface::Emission { color: material.emission_color, strength: material.emission_strength },
        false => Surface::Diffuse { color: material.base_color },
    }
}
/// `sync_data`: the scene of `source`. A source with no camera keeps the
/// scene's default one at the source's size.
pub fn sync_scene(source: &SceneSource) -> Scene {
    let mut scene = Scene::new();
    let (width, height) = (source.width.max(1), source.height.max(1));
    if let Some(camera) = &source.camera {
        scene.camera = sync_camera(camera, width, height);
    }
    scene.world = World { color: source.world_color, strength: source.world_strength };
    sync_integrator(source, &mut scene);
    sync_film(source, &mut scene);
    // `sync_materials`: a shader for each distinct material.
    let mut known: Vec<MaterialSource> = Vec::new();
    let mut shader_of = |scene: &mut Scene, material: &MaterialSource| match known.iter().position(|seen| seen == material) {
        Some(place) => place,
        None => {
            known.push(material.clone());
            scene.add_shader(surface_of(material))
        }
    };
    // `sync_mesh`: each mesh once, on the shaders of its slots.
    for mesh in &source.mesh {
        let shader: Vec<usize> = mesh.material.iter().map(|material| shader_of(&mut scene, material)).collect();
        scene.mesh.push(sync_mesh(mesh, &shader));
    }
    // `sync_object`: each object places its mesh.
    for object in &source.object {
        let mut placed = Object::new(object.mesh);
        placed.set_transform(transform_of(&object.matrix));
        scene.object.push(placed);
    }
    scene
}
