// SPDX-License-Identifier: Apache-2.0
//! The one file of this crate that names phiphx: its `SceneModel` into a
//! [`SceneSource`]. phiphx is being reorganised (`variant-layout` moves the
//! `blend` paths); only the public paths `phiphx::Phx`, `phiphx::scene::*`
//! are used here, so a layout change is a change to this file alone.
//!
//! Cycles has no counterpart: its `blender/` reads the open Blender through
//! RNA; this reads the file.
use std::path::Path;
use std::sync::Arc;

use phiphx::Phx;
use phiphx::scene::{SceneCamera, SceneMaterial, SceneMesh, SceneModel, SceneObject};

use super::source::{CameraSource, MaterialSource, MeshSource, ObjectSource, SceneSource, SensorFit};

/// A material of phiphx as the source has it: the base colour, and the
/// emission of the Principled BSDF.
fn material_of(material: &SceneMaterial) -> MaterialSource {
    let [r, g, b, _] = material.base_color;
    MaterialSource { name: material.name.clone(), base_color: [r, g, b], emission_color: material.bsdf.emission, emission_strength: material.bsdf.emission_strength }
}

/// `blender_camera_from_object`: the camera's lens and sensor.
fn camera_of(camera: &SceneCamera) -> CameraSource {
    let sensor_fit = match camera.sensor_fit.as_str() {
        "HORIZONTAL" => SensorFit::Horizontal,
        "VERTICAL" => SensorFit::Vertical,
        _ => SensorFit::Auto,
    };
    // `sensor_height` is `Camera.sensor_y`; Blender's default is 24 mm.
    let sensor_height = if camera.sensor_height > 0.0 { camera.sensor_height } else { 24.0 };
    CameraSource {
        name: camera.name.clone(),
        matrix: camera.matrix_world,
        orthographic: camera.kind == "ORTHO",
        lens: camera.focal_length_mm,
        sensor_width: camera.sensor_width,
        sensor_height,
        sensor_fit,
        shift: camera.shift,
        ortho_scale: camera.ortho_scale,
        clip_start: camera.clip_start,
        clip_end: camera.clip_end,
    }
}

/// The triangles of `mesh` with the object's slots, smooth flags and UVs.
fn mesh_of(object: &SceneObject, mesh: &SceneMesh) -> MeshSource {
    let triangle: Vec<[u32; 3]> = mesh.indices.chunks_exact(3).map(|corner| [corner[0], corner[1], corner[2]]).collect();
    // A triangle is smooth when any of its corners has a shading normal;
    // a flat corner is the zero normal (`SceneCorners`).
    let smooth = (0..triangle.len()).map(|t| (0..3).any(|c| mesh.corners.normal(t * 3 + c) != [0.0; 3])).collect();
    let uv = if object.uvs.len() == mesh.vertices.len() { object.uvs.as_ref().clone() } else { Vec::new() };
    MeshSource {
        vertex: mesh.vertices.clone(),
        slot: (0..triangle.len()).map(|t| object.material_indices.get(t).copied().unwrap_or(0)).collect(),
        triangle,
        smooth,
        uv,
        material: object.materials.iter().map(material_of).collect(),
    }
}

/// A `SceneModel` as the sync reads it. An object that shares its mesh,
/// its UVs and its material names with another shares the source mesh.
pub fn source_of_model(model: &SceneModel) -> SceneSource {
    let render = &model.render;
    let percent = if render.resolution_percent > 0 { render.resolution_percent } else { 100 };
    let camera = model.cameras.iter().find(|camera| camera.name == render.camera).or(model.cameras.first());
    let mut source = SceneSource {
        width: render.resolution_x * percent / 100,
        height: render.resolution_y * percent / 100,
        camera: camera.map(camera_of),
        world_color: model.world.background.color,
        world_strength: model.world.background.strength,
        ..SceneSource::default()
    };
    type Key = (*const SceneMesh, *const Vec<[f32; 2]>, Vec<String>);
    let mut place: Vec<Key> = Vec::new();
    for object in &model.objects {
        let key: Key = (Arc::as_ptr(&object.mesh), Arc::as_ptr(&object.uvs), object.materials.iter().map(|m| m.name.clone()).collect());
        let mesh = place.iter().position(|seen| *seen == key).unwrap_or_else(|| {
            place.push(key);
            source.mesh.push(mesh_of(object, &object.mesh));
            place.len() - 1
        });
        source.object.push(ObjectSource { name: object.name.clone(), mesh, matrix: object.matrix_world });
    }
    source
}

/// Read a `.blend` (or any file phiphx opens) into a source.
pub fn read_blend(path: &Path) -> Result<SceneSource, String> {
    let model = read_model(path)?;
    Ok(source_of_model(&model))
}

/// Open a file with phiphx: what the tests compare the sync against.
pub fn read_model(path: &Path) -> Result<SceneModel, String> {
    Phx::open(path).map_err(|error| error.message())
}

pub use phiphx::scene::SceneModel as Model;