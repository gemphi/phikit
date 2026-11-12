// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
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

use super::graph::{GraphSource, LinkSource, NodeSource, NodeStorage, SocketClass, SocketSource, SocketValue};
use super::source::{CameraSource, MaterialSource, MeshSource, NodeSurface, ObjectSource, SceneSource, SensorFit};

/// The default of a socket the graph leaves unlinked, as three floats.
fn color_of(socket: &phiphx::blend::shade::socket::BlendSocket) -> Option<[f32; 3]> {
    match socket.default.as_ref()? {
        phiphx::blend::shade::socket::BSocketDefault::Color([r, g, b, _]) => Some([*r, *g, *b]),
        _ => None,
    }
}

fn float_of(socket: &phiphx::blend::shade::socket::BlendSocket) -> Option<f32> {
    match socket.default.as_ref()? {
        phiphx::blend::shade::socket::BSocketDefault::Float(value) => Some(*value),
        _ => None,
    }
}

/// `ntreeShaderOutputNode(ntree, SHD_OUTPUT_CYCLES)` (`node.cc`): of the
/// Material Output nodes, the one a Cycles render uses. An output for all
/// engines is the first found, and one that is active beats one that is
/// not; an output whose target is Cycles beats one for all, and among
/// those the active one wins.
fn output_node(material: &SceneMaterial) -> Option<&phiphx::blend::shade::graph::BlendNode> {
    // `SHD_OUTPUT_ALL`, `SHD_OUTPUT_CYCLES` (`DNA_node_types.h`).
    const ALL: i32 = 0;
    const CYCLES: i32 = 2;
    let mut chosen: Option<&phiphx::blend::shade::graph::BlendNode> = None;
    for node in material.graph.nodes.iter().filter(|node| node.idname == "ShaderNodeOutputMaterial") {
        let better = |current: &phiphx::blend::shade::graph::BlendNode| !current.active_output() && node.active_output();
        chosen = match (chosen, node.custom1) {
            (None, ALL | CYCLES) => Some(node),
            (Some(current), ALL) if current.custom1 == ALL && better(current) => Some(node),
            (Some(current), CYCLES) if current.custom1 == ALL || better(current) => Some(node),
            (kept, _) => kept,
        };
    }
    chosen
}
/// `blender/shader.cpp` (`BlenderSync::sync_materials`): what the graph's
/// Material Output takes its surface from. A lone Diffuse BSDF or Emission
/// node, with its inputs left as they are set, is a surface of gate 1; any
/// other graph is [`NodeSurface::Unknown`].
fn node_surface(material: &SceneMaterial) -> NodeSurface {
    let graph = &material.graph;
    let Some(output) = output_node(material) else { return NodeSurface::Unknown };
    let Some(link) = graph.links.iter().find(|link| link.to == output.name && link.to_socket == "Surface") else { return NodeSurface::Unknown };
    let Some(node) = graph.nodes.iter().find(|node| node.name == link.from) else { return NodeSurface::Unknown };
    let linked = |name: &str| graph.links.iter().any(|link| link.to == node.name && link.to_socket == name);
    let input = |name: &str| node.inputs.iter().find(|socket| socket.name == name);
    match node.idname.as_str() {
        "ShaderNodeBsdfDiffuse" if !linked("Color") && !linked("Roughness") && !linked("Normal") => {
            let color = input("Color").and_then(color_of);
            let roughness = input("Roughness").and_then(float_of).unwrap_or(0.0);
            color.map_or(NodeSurface::Unknown, |color| NodeSurface::Diffuse { color, roughness })
        }
        "ShaderNodeEmission" if !linked("Color") && !linked("Strength") => {
            let color = input("Color").and_then(color_of);
            let strength = input("Strength").and_then(float_of).unwrap_or(1.0);
            color.map_or(NodeSurface::Unknown, |color| NodeSurface::Emission { color, strength })
        }
        _ => NodeSurface::Unknown,
    }
}

/// `bNodeSocket::type` (`eNodeSocketDatatype`) as Cycles sees it.
fn class_of(kind: i32) -> SocketClass {
    match kind {
        0 => SocketClass::Float,
        1 => SocketClass::Vector,
        2 => SocketClass::Color,
        3 => SocketClass::Closure,
        4 => SocketClass::Boolean,
        6 => SocketClass::Int,
        7 => SocketClass::String,
        _ => SocketClass::Undefined,
    }
}
fn socket_of(socket: &phiphx::blend::shade::socket::BlendSocket) -> SocketSource {
    use phiphx::blend::shade::socket::BSocketDefault;
    let value = socket.default.as_ref().map(|value| match value {
        BSocketDefault::Float(value) => SocketValue::Float(*value),
        BSocketDefault::Color([r, g, b, _]) => SocketValue::Color([*r, *g, *b]),
        BSocketDefault::Vector(value) => SocketValue::Vector(*value),
        BSocketDefault::Bool(value) => SocketValue::Boolean(*value),
        BSocketDefault::Int(value) => SocketValue::Int(*value),
    });
    SocketSource { identifier: socket.identifier.clone(), class: class_of(socket.kind), available: socket.available(), value }
}
/// A node of the material's tree. phiphx gives `custom1`, `custom2` and the
/// address of `storage` but not its contents, and not the tree a group
/// node instances.
fn node_of(node: &phiphx::blend::shade::graph::BlendNode) -> NodeSource {
    NodeSource {
        idname: node.idname.clone(),
        name: node.name.clone(),
        custom1: node.custom1,
        custom2: node.custom2,
        muted: node.muted(),
        reroute: node.reroute(),
        active_output: node.active_output(),
        group: None,
        storage: NodeStorage::None,
        input: node.inputs.iter().map(socket_of).collect(),
        output: node.outputs.iter().map(socket_of).collect(),
    }
}
/// The material's node tree, the frames left out, with the output a
/// Cycles render uses.
fn graph_source(material: &SceneMaterial) -> GraphSource {
    let graph = &material.graph;
    GraphSource {
        node: graph.nodes.iter().filter(|node| !node.frame()).map(node_of).collect(),
        link: graph.links.iter().map(|link| LinkSource { from: link.from.clone(), from_socket: link.from_socket.clone(), to: link.to.clone(), to_socket: link.to_socket.clone() }).collect(),
        output: output_node(material).map(|node| node.name.clone()),
    }
}
/// A material of phiphx as the source has it: the base colour, and the
/// emission of the Principled BSDF, and what its graph makes.
fn material_of(material: &SceneMaterial) -> MaterialSource {
    let [r, g, b, _] = material.base_color;
    MaterialSource {
        name: material.name.clone(),
        base_color: [r, g, b],
        emission_color: material.bsdf.emission,
        emission_strength: material.bsdf.emission_strength,
        diffuse_color: [material.viewport_color[0], material.viewport_color[1], material.viewport_color[2]],
        node: node_surface(material),
        node_kind: material.graph.nodes.iter().map(|node| node.idname.clone()).collect(),
        graph: graph_source(material),
    }
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
    let cycles = render.cycles.names().iter().filter_map(|name| Some((name.clone(), render.cycles.get(name)?.as_float()?))).collect();
    let mut source = SceneSource {
        width: render.resolution_x * percent / 100,
        height: render.resolution_y * percent / 100,
        camera: camera.map(camera_of),
        world_color: model.world.background.color,
        world_strength: model.world.background.strength,
        world_texture: model.world.background.texture.clone(),
        light_count: model.lights.len(),
        cycles,
        view_transform: render.view_transform.clone(),
        look: render.look.clone(),
        exposure: render.exposure,
        gamma: render.gamma,
        film_transparent: render.film_transparent,
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

#[cfg(test)]
mod test;
