//! `.phx` scene → [`SceneModel`] (asset pipeline / collaboration sidecars).

use super::{
    SceneAction, SceneCamera, SceneLight, SceneMaterial, SceneMesh, SceneModel, SceneNormals,
    SceneObject, SceneRender, SceneTimeline, SceneWorld,
};
use super::geometry::face_normal;
use crate::export::scene::PhxScene;

// ── PHX sidecar fast-path ────────────────────────────────────────────────
//
// Every successful `.blend` import writes a `.phx` sidecar next to the
// source. The phx format stores already-triangulated indices as `[u32; 3]`
// triples, so re-loading a scene from its sidecar skips both the O(n²)
// SDNA block walk and the fan-triangulation step. `Phx::open` calls
// this when it finds a fresh sidecar.

/// Map a `.phx` scene into the runtime [`SceneModel`]. Triangulation is
/// already done in the file format, so we just copy vertex / normal /
/// index buffers straight through and compute a per-face normal fallback
/// if the sidecar doesn't carry per-face normals.
impl SceneModel {
/// Build the runtime scene from a `.phx` scene.
pub fn from_phx(phx: &PhxScene, source_path: &str) -> SceneModel {
    let objects: Vec<SceneObject> = phx
        .meshes
        .iter()
        .filter(|m| m.visible)
        .map(|m| object_from_phx(m, &phx.materials))
        .collect();
    SceneModel {
        objects,
        source_path: Some(source_path.to_string()),
        timeline: SceneTimeline::default(),
        world: SceneWorld::default(),
        render: SceneRender::default(),
        cameras: Vec::<SceneCamera>::new(),
        lights: Vec::<SceneLight>::new(),
        actions: Vec::<SceneAction>::new(),
        nla: Vec::new(),
        drivers: Vec::new(),
        collections: Vec::new(),
        selection: Default::default(),
        object_types: Vec::new(),
        empties: Vec::new(),
        ui: Default::default(),
        motion: Default::default(),
        markers: Vec::new(),
        playback: Default::default(),
        parents: Vec::new(),
        locals: Vec::new(),
        extra_locals: Vec::new(),
        uv_maps: Vec::new(),
        cursor: Default::default(),
        report: Default::default(),
    }
}
}

/// Convert one [`PhxMesh`] into a renderable [`SceneObject`]. Indices land
/// as a flat triangle list (`Vec<u32>`) matching the [`SceneMesh`] shape;
/// per-vertex normals are used when the sidecar cardinality matches, else
/// per-face normals are synthesized from the winding.
fn object_from_phx(
    mesh: &crate::export::scene::PhxMesh,
    scene_materials: &[crate::export::scene::PhxMaterial],
) -> SceneObject {
    let vertices = mesh.vertices.clone();
    let vert_count = vertices.len();
    // Filter out-of-range triangles so no emitted index can exceed the
    // vertex-buffer bounds. Some phx sidecars have been observed with
    // indices == vert_count on the boundary case, which would panic on
    // both the per-vertex normal remap and the per-face normal synth.
    let indices: Vec<u32> = mesh
        .indices
        .iter()
        .filter(|tri| {
            (tri[0] as usize) < vert_count
                && (tri[1] as usize) < vert_count
                && (tri[2] as usize) < vert_count
        })
        .flat_map(|tri| tri.iter().copied())
        .collect();
    let normals = if mesh.normals.len() == vertices.len() {
        SceneNormals::PerVertex(mesh.normals.clone())
    } else {
        let tri_count = indices.len() / 3;
        let mut per_face: Vec<[f32; 3]> = Vec::with_capacity(tri_count);
        for t in 0..tri_count {
            let base = t * 3;
            let a = vertices[indices[base] as usize];
            let b = vertices[indices[base + 1] as usize];
            let c = vertices[indices[base + 2] as usize];
            per_face.push(face_normal(a, b, c));
        }
        SceneNormals::PerFace(per_face)
    };
    // The phx matrix_world stores translation in row 3 (row-major); pull it
    // out as the object's location so loaded scenes preserve placement.
    let location = [
        mesh.matrix_world[3][0],
        mesh.matrix_world[3][1],
        mesh.matrix_world[3][2],
    ];
    // Resolve the mesh's material name references against the scene material
    // table. UVs copy straight through when their cardinality matches the
    // vertex buffer (phx stores per-vertex UVs).
    let materials = materials_from_phx(&mesh.materials, scene_materials);
    let uvs = if mesh.uvs.len() == vertices.len() {
        mesh.uvs.clone()
    } else {
        Vec::new()
    };
    SceneObject {
        name: mesh.name.clone(),
        mesh: SceneMesh {
            vertices,
            indices,
            normals,
            corners: Default::default(),
        }.into(),
        location,
        scale: [1.0, 1.0, 1.0],
        matrix_world: mesh.matrix_world,
        materials,
        material_indices: Default::default(),
        uvs: uvs.into(),
        color: [1.0; 4],
        shadow: Default::default(),
    }
}

/// Resolve a mesh's material-name references against the scene material
/// table into kit-agnostic [`SceneMaterial`]s. Names with no matching entry
/// are skipped. Returns empty when the mesh references no materials.
fn materials_from_phx(
    names: &[String],
    table: &[crate::export::scene::PhxMaterial],
) -> Vec<SceneMaterial> {
    names
        .iter()
        .filter_map(|name| table.iter().find(|m| &m.name == name))
        .map(|m| SceneMaterial {
            name: m.name.clone(),
            base_color: m.base_color,
            viewport_color: m.base_color,
            viewport_pbr: [m.roughness, m.metallic],
            roughness: m.roughness,
            metallic: m.metallic,
            // `.phx` carries no node tree.
            graph: Default::default(),
            base_color_texture: m.base_color_texture.clone(),
            texture: None,
            bsdf: Default::default(),
            mix: None,
            procedural: Default::default(),
            normal_texture: None,
            normal_map: None,
            maps: Default::default(),
        })
        .collect()
}
