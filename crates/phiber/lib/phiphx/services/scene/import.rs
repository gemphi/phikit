//! Blend → scene conversion helpers. Split out of `scene/mod.rs` so both
//! the file stays under the 200-line cap and the pure geometry pipeline lives
//! in a focused module.
//!
//! Public entry point: [`scene_from_blend`]. Everything else is `pub(super)`
//! so consumers go through the aggregate `SceneModel` rather than reaching
//! into per-object builders.

use super::{
    SceneAction, SceneCamera, SceneLight, SceneMaterial, SceneMesh, SceneModel, SceneNormals,
    SceneObject, SceneRender, SceneTimeline, SceneWorld,
};
use crate::phiphx::importer::blend::{BlendScene, BlendSceneObject};
use crate::phx::exporter::scene::PhxScene;

/// Map an imported [`BlendScene`] into a renderable [`SceneModel`].
///
/// This is the single blend→scene traversal in the codebase (Requirement 7.2);
/// it lives in the shared, kit-agnostic layer and nowhere under
/// `crates/phiber/src/`. It mirrors the fan-triangulation
/// `phx_export::blend_scene_to_phx` already performs so both paths agree.
///
/// For every mesh object it:
///
/// - Fan-triangulates each polygon: face `[c0, c1, …, cN]` becomes triangles
///   `[c0, ci, ci+1]` for `i in 1..N`.
/// - Drops any corner whose vertex index is out of range for that object's
///   vertex buffer.
/// - Takes normals from `loop_normals` when present (one per triangle,
///   [`SceneNormals::PerFace`]), otherwise synthesizes a per-face normal.
///
/// Cameras and lights are partitioned into `cameras` / `lights` vectors so
/// the Rendering / Shading spaces can iterate them without a full scan.
///
/// Requirements: 2.5, 2.6, 7.2
pub fn scene_from_blend(blend: &BlendScene, source_path: &str) -> SceneModel {
    let mut objects: Vec<SceneObject> = Vec::new();
    let mut cameras: Vec<SceneCamera> = Vec::new();
    let mut lights: Vec<SceneLight> = Vec::new();
    for obj in &blend.objects {
        match obj.object_type.as_str() {
            "CAMERA" => cameras.push(camera_from_blend(obj)),
            "LAMP" | "LIGHT" => lights.push(light_from_blend(obj)),
            _ => objects.push(object_from_blend(obj)),
        }
    }
    SceneModel {
        objects,
        source_path: Some(source_path.to_string()),
        timeline: SceneTimeline::default(),
        world: SceneWorld::default(),
        render: SceneRender::default(),
        cameras,
        lights,
        actions: Vec::<SceneAction>::new(),
    }
}

/// Build a `SceneCamera` from a `BlendSceneObject` whose type is `CAMERA`.
/// The `Camera` DNA is not currently walked by phiphx, so focal length /
/// clip distances fall back to Blender defaults (50 mm, 0.1 → 100 m).
fn camera_from_blend(obj: &BlendSceneObject) -> SceneCamera {
    SceneCamera {
        name: obj.name.clone(),
        location: obj.location,
        focal_length_mm: 50.0,
        clip_start: 0.1,
        clip_end: 100.0,
    }
}

/// Build a `SceneLight` from a `BlendSceneObject` whose type is `LAMP` /
/// `LIGHT`. Falls back to Blender's factory point-light values (1000 W, white).
fn light_from_blend(obj: &BlendSceneObject) -> SceneLight {
    SceneLight {
        name: obj.name.clone(),
        location: obj.location,
        color: [1.0, 1.0, 1.0],
        energy: 1000.0,
        kind: String::from("POINT"),
    }
}

/// Convert one `BlendSceneObject` into a renderable [`SceneObject`].
///
/// Fan-triangulates the object's faces and builds a bounds-safe index buffer.
pub(super) fn object_from_blend(obj: &BlendSceneObject) -> SceneObject {
    let vertices = obj.vertices.clone();
    let vert_count = vertices.len();
    let mut indices: Vec<u32> = Vec::new();
    let mut tri_loop_starts: Vec<usize> = Vec::new();
    let mut loop_base = 0usize;
    for face in &obj.faces {
        let len = face.len();
        if len < 3 {
            loop_base += len;
            continue;
        }
        for i in 1..(len - 1) {
            let (a, b, c) = (face[0], face[i], face[i + 1]);
            if (a as usize) >= vert_count
                || (b as usize) >= vert_count
                || (c as usize) >= vert_count
            {
                continue;
            }
            indices.push(a);
            indices.push(b);
            indices.push(c);
            tri_loop_starts.push(loop_base);
        }
        loop_base += len;
    }
    let tri_count = indices.len() / 3;
    let normals = build_normals(
        &vertices,
        &indices,
        &obj.loop_normals,
        &tri_loop_starts,
        tri_count,
    );
    let uvs = per_vertex_uvs(&obj.faces, &obj.loop_uvs, vert_count);
    let materials = materials_from_blend(obj);
    SceneObject {
        name: obj.name.clone(),
        mesh: SceneMesh {
            vertices,
            indices,
            normals,
        },
        location: obj.location,
        scale: obj.scale,
        materials,
        // Per-face material assignment is not walked yet (phiphx does not
        // read `material_index` per polygon), so every triangle uses slot 0.
        material_indices: Vec::new(),
        uvs,
    }
}

/// Map the object's material slots into kit-agnostic [`SceneMaterial`]s.
///
/// Prefers the explicit `materials` list (multi-slot). Falls back to the
/// object's flat `base_color` / `roughness` / `metallic` when only the
/// single-material fields are populated, and returns an empty vec when the
/// object declared no material at all (renderer then uses its neutral colour).
fn materials_from_blend(obj: &BlendSceneObject) -> Vec<SceneMaterial> {
    if !obj.materials.is_empty() {
        return obj
            .materials
            .iter()
            .map(|m| SceneMaterial {
                name: m.name.clone(),
                base_color: m.base_color,
                roughness: m.roughness,
                metallic: m.metallic,
                base_color_texture: m.base_color_texture.clone(),
            })
            .collect();
    }
    // A fully-default material record (black, rough 0, metal 0) means the
    // importer found no material; don't fabricate a slot for it.
    let is_default = obj.base_color == [0.0, 0.0, 0.0, 0.0]
        && obj.roughness == 0.0
        && obj.metallic == 0.0;
    if is_default {
        return Vec::new();
    }
    vec![SceneMaterial {
        name: String::from("Material"),
        base_color: obj.base_color,
        roughness: obj.roughness,
        metallic: obj.metallic,
        base_color_texture: obj.texture_paths.first().cloned(),
    }]
}

/// Build a per-vertex UV buffer from the object's per-loop UVs.
///
/// Blender stores UVs per loop (corner), so a vertex shared by multiple faces
/// can carry several UVs. For flat-colour rendering we only need a stable
/// per-vertex sample, so each vertex takes the UV of the first loop that
/// references it. Returns an empty vec when the source carried no UV layer,
/// which the renderer reads as "no UVs".
fn per_vertex_uvs(faces: &[Vec<u32>], loop_uvs: &[[f32; 2]], vert_count: usize) -> Vec<[f32; 2]> {
    if loop_uvs.is_empty() || vert_count == 0 {
        return Vec::new();
    }
    let mut uvs = vec![[0.0f32, 0.0]; vert_count];
    let mut seen = vec![false; vert_count];
    let mut loop_index = 0usize;
    for face in faces {
        for &vert in face {
            let v = vert as usize;
            if v < vert_count && !seen[v] {
                if let Some(&uv) = loop_uvs.get(loop_index) {
                    uvs[v] = uv;
                    seen[v] = true;
                }
            }
            loop_index += 1;
        }
    }
    uvs
}

/// Build a per-face normal set for the triangulated mesh.
///
/// When `loop_normals` covers a triangle's leading corner it is reused
/// (per-face), otherwise the normal is synthesized from the triangle's
/// winding (`(b - a) × (c - a)`, normalized). The result always has exactly
/// `tri_count` entries so it satisfies the [`SceneNormals::PerFace`]
/// invariant.
fn build_normals(
    vertices: &[[f32; 3]],
    indices: &[u32],
    loop_normals: &[[f32; 3]],
    tri_loop_starts: &[usize],
    tri_count: usize,
) -> SceneNormals {
    let mut normals: Vec<[f32; 3]> = Vec::with_capacity(tri_count);
    let have_loop_normals = !loop_normals.is_empty();
    for t in 0..tri_count {
        if have_loop_normals {
            if let Some(&start) = tri_loop_starts.get(t) {
                if let Some(n) = loop_normals.get(start) {
                    normals.push(*n);
                    continue;
                }
            }
        }
        let base = t * 3;
        let a = vertices[indices[base] as usize];
        let b = vertices[indices[base + 1] as usize];
        let c = vertices[indices[base + 2] as usize];
        normals.push(face_normal(a, b, c));
    }
    SceneNormals::PerFace(normals)
}

/// Normalized normal of the triangle `(a, b, c)` via `(b - a) × (c - a)`.
///
/// Falls back to `[0, 0, 1]` for a degenerate (zero-area) triangle so the
/// result is always a finite unit-ish vector.
fn face_normal(a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> [f32; 3] {
    let u = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let v = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
    let n = [
        u[1] * v[2] - u[2] * v[1],
        u[2] * v[0] - u[0] * v[2],
        u[0] * v[1] - u[1] * v[0],
    ];
    let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
    if len > f32::EPSILON {
        [n[0] / len, n[1] / len, n[2] / len]
    } else {
        [0.0, 0.0, 1.0]
    }
}

// ── PHX sidecar fast-path ────────────────────────────────────────────────
//
// Every successful `.blend` import writes a `.phx` sidecar next to the
// source. The phx format stores already-triangulated indices as `[u32; 3]`
// triples, so re-loading a scene from its sidecar skips both the O(n²)
// SDNA block walk and the fan-triangulation step. `load_blend_scene` calls
// this when it finds a fresh sidecar.

/// Map a `.phx` scene into the runtime [`SceneModel`]. Triangulation is
/// already done in the file format, so we just copy vertex / normal /
/// index buffers straight through and compute a per-face normal fallback
/// if the sidecar doesn't carry per-face normals.
pub fn scene_from_phx(phx: &PhxScene, source_path: &str) -> SceneModel {
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
    }
}

/// Convert one [`PhxMesh`] into a renderable [`SceneObject`]. Indices land
/// as a flat triangle list (`Vec<u32>`) matching the [`SceneMesh`] shape;
/// per-vertex normals are used when the sidecar cardinality matches, else
/// per-face normals are synthesized from the winding.
fn object_from_phx(
    mesh: &crate::phx::exporter::scene::PhxMesh,
    scene_materials: &[crate::phx::exporter::scene::PhxMaterial],
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
        },
        location,
        scale: [1.0, 1.0, 1.0],
        materials,
        material_indices: Vec::new(),
        uvs,
    }
}

/// Resolve a mesh's material-name references against the scene material
/// table into kit-agnostic [`SceneMaterial`]s. Names with no matching entry
/// are skipped. Returns empty when the mesh references no materials.
fn materials_from_phx(
    names: &[String],
    table: &[crate::phx::exporter::scene::PhxMaterial],
) -> Vec<SceneMaterial> {
    names
        .iter()
        .filter_map(|name| table.iter().find(|m| &m.name == name))
        .map(|m| SceneMaterial {
            name: m.name.clone(),
            base_color: m.base_color,
            roughness: m.roughness,
            metallic: m.metallic,
            base_color_texture: m.base_color_texture.clone(),
        })
        .collect()
}
