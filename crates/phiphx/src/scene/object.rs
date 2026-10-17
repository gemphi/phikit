//! One `BlendSceneObject` → renderable [`SceneObject`]: Blender-style
//! triangulation, bounds-safe indices, normals, UVs and material slots.

use super::geometry::{build_normals, per_vertex_uvs};
use super::SceneCorners;
use super::material::materials_from_blend;
use super::polyfill::triangulate;
use super::{SceneMesh, SceneNormals, SceneObject};
use crate::blend::BlendSceneObject;

/// The triangles `src` is drawn with — its modifier-evaluated mesh's when
/// given, as [`evaluated_object`] makes them.
pub(super) fn evaluated_triangles(src: &BlendSceneObject, eval: Option<&crate::blend::BlendEvalMesh>) -> STriangles {
    match eval {
        Some(e) => STriangles::of(&BlendSceneObject {
            vertices: e.vertices.clone(),
            faces: e.faces.clone(),
            face_materials: e.face_materials.clone(),
            face_smooth: e.face_smooth.clone(),
            loop_normals: Vec::new(),
            loose_edges: e.loose_edges.clone(),
            ..src.clone()
        }),
        None => STriangles::of(src),
    }
}

/// Convert one `BlendSceneObject`, drawing the modifier-evaluated mesh when
/// given; saved split normals describe the rest mesh, so an evaluated mesh
/// recomputes its normals.
pub(super) fn evaluated_object(src: &BlendSceneObject, eval: Option<&crate::blend::BlendEvalMesh>) -> SceneObject {
    let owned;
    let obj = match eval {
        Some(e) => {
            owned = BlendSceneObject {
                vertices: e.vertices.clone(),
                faces: e.faces.clone(),
                loop_uvs: e.loop_uvs.clone(),
                loop_colors: e.loop_colors.clone(),
                face_materials: e.face_materials.clone(),
                face_smooth: e.face_smooth.clone(),
                loop_normals: Vec::new(),
                loose_edges: e.loose_edges.clone(),
                ..src.clone()
            };
            &owned
        }
        None => src,
    };
    let vertices = obj.vertices.clone();
    let vert_count = vertices.len();
    let mut tris = STriangles::of(obj);
    let mesh = tris.mesh(vertices, &obj.loop_normals, &obj.loop_colors);
    let material_indices = std::mem::take(&mut tris.material_indices);
    let uvs = per_vertex_uvs(&obj.faces, &obj.loop_uvs, vert_count);
    let materials = materials_from_blend(obj);
    SceneObject {
        name: obj.name.clone(),
        mesh: mesh.into(),
        location: obj.location,
        scale: obj.scale,
        matrix_world: obj.matrix_world,
        materials,
        material_indices: material_indices.into(),
        uvs: uvs.into(),
        color: obj.color,
        shadow: obj.shadow,
    }
}


/// How a mesh's faces became triangles, kept so the same mesh in another
/// pose (`rig.rs`) is drawn with the same triangles: indices, which face
/// corners each came from, and which were smooth.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct STriangles {
    pub indices: Vec<u32>,
    tri_loop_starts: Vec<usize>,
    tri_loops: Vec<[usize; 3]>,
    tri_smooth: Vec<bool>,
    /// Per-triangle material slot (empty for a single-material mesh).
    pub material_indices: Vec<u16>,
}

impl STriangles {
    /// Blender-style triangulation of `obj`'s faces, bounds-safe.
    pub fn of(obj: &BlendSceneObject) -> Self {
        let vertices = &obj.vertices;
        let vert_count = vertices.len();
        let mut t = Self::default();
        let mut loop_base = 0usize;
        for (face_i, face) in obj.faces.iter().enumerate() {
            let len = face.len();
            if len < 3 || face.iter().any(|&v| (v as usize) >= vert_count) {
                loop_base += len;
                continue;
            }
            let slot = obj.face_materials.get(face_i).copied().unwrap_or(0);
            let smooth = obj.face_smooth.get(face_i).copied().unwrap_or(false);
            let pts: Vec<[f32; 3]> = face.iter().map(|&v| vertices[v as usize]).collect();
            for [i, j, k] in triangulate(&pts) {
                t.indices.extend([face[i], face[j], face[k]]);
                t.tri_loop_starts.push(loop_base);
                t.tri_loops.push([loop_base + i, loop_base + j, loop_base + k]);
                t.material_indices.push(slot);
                t.tri_smooth.push(smooth);
            }
            loop_base += len;
        }
        // Only keep per-triangle slots when the mesh really is multi-material.
        if t.material_indices.iter().all(|&m| m == 0) {
            t.material_indices.clear();
        }
        t
    }


    /// Triangulation of a modifier-evaluated mesh whose topology may have
    /// changed (a driven Subdivision level). No saved normals — every
    /// frame recomputes.
    pub(super) fn from_eval(eval: &crate::blend::BlendEvalMesh) -> Self {
        let vertices = &eval.vertices;
        let vert_count = vertices.len();
        let mut t = Self::default();
        let mut loop_base = 0usize;
        for (face_i, face) in eval.faces.iter().enumerate() {
            let len = face.len();
            if len < 3 || face.iter().any(|&v| (v as usize) >= vert_count) {
                loop_base += len;
                continue;
            }
            let slot = eval.face_materials.get(face_i).copied().unwrap_or(0);
            let smooth = eval.face_smooth.get(face_i).copied().unwrap_or(false);
            let pts: Vec<[f32; 3]> = face.iter().map(|&v| vertices[v as usize]).collect();
            for [i, j, k] in triangulate(&pts) {
                t.indices.extend([face[i], face[j], face[k]]);
                t.tri_loop_starts.push(loop_base);
                t.tri_loops.push([loop_base + i, loop_base + j, loop_base + k]);
                t.material_indices.push(slot);
                t.tri_smooth.push(smooth);
            }
            loop_base += len;
        }
        if t.material_indices.iter().all(|&m| m == 0) {
            t.material_indices.clear();
        }
        t
    }

    /// The mesh these triangles make over `vertices`, normals computed
    /// (`loop_normals`: saved split normals, empty for an evaluated mesh).
    pub fn mesh(&self, vertices: Vec<[f32; 3]>, loop_normals: &[[f32; 3]], loop_colors: &[[f32; 3]]) -> SceneMesh {
        if loop_normals.is_empty() && self.tri_smooth.iter().any(|s| *s) {
            let mut mesh = self.smooth_mesh(vertices);
            mesh.corners.colors = self.corner_colors(loop_colors);
            return mesh;
        }
        let normals = build_normals(&vertices, &self.indices, loop_normals, &self.tri_loop_starts, self.indices.len() / 3, &self.tri_smooth);
        let mut corners = SceneCorners::build(&vertices, &self.indices, loop_normals, &self.tri_loops, &self.tri_smooth);
        corners.colors = self.corner_colors(loop_colors);
        SceneMesh { vertices, indices: self.indices.clone(), normals, corners }
    }

    /// The loops' colours at the triangles' corners; none when the mesh has none.
    fn corner_colors(&self, loop_colors: &[[f32; 3]]) -> Vec<[f32; 3]> {
        if loop_colors.is_empty() {
            return Vec::new();
        }
        self.tri_loops.iter().flatten().map(|&l| loop_colors.get(l).copied().unwrap_or([1.0; 3])).collect()
    }

    /// `mesh` for smooth faces without saved normals — every frame of a
    /// rigged mesh: the face normals and the vertex normals worked out once
    /// for both the triangles' normals and the corners' (what
    /// `build_normals` and `SceneCorners::build` each work out alone).
    fn smooth_mesh(&self, vertices: Vec<[f32; 3]>) -> SceneMesh {
        use super::geometry::{normalize, smooth_vertex_normals, tri_face_normal};
        use rayon::prelude::*;
        let indices = &self.indices;
        let face: Vec<[f32; 3]> = (0..indices.len() / 3).into_par_iter().map(|t| tri_face_normal(&vertices, indices, t)).collect();
        let accum = smooth_vertex_normals(&vertices, indices, &face, &self.tri_smooth);
        let normals = (0..face.len())
            .into_par_iter()
            .map(|t| {
                if !self.tri_smooth.get(t).copied().unwrap_or(false) {
                    return face[t];
                }
                let mut n = [0.0f32; 3];
                for k in 0..3 {
                    let v = accum[indices[t * 3 + k] as usize];
                    (0..3).for_each(|a| n[a] += v[a]);
                }
                normalize(n).unwrap_or(face[t])
            })
            .collect();
        let mut table = vec![[0.0; 3]];
        table.par_extend(accum.par_iter().map(|n| normalize(*n).unwrap_or([0.0; 3])));
        let ids = (0..indices.len()).map(|c| if self.tri_smooth.get(c / 3).copied().unwrap_or(false) { indices[c] + 1 } else { 0 }).collect();
        SceneMesh { vertices, indices: indices.clone(), normals: SceneNormals::PerFace(normals), corners: SceneCorners { ids, normals: table, colors: Vec::new() } }
    }
}
