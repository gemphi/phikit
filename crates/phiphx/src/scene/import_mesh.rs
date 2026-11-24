//! Scenes from readers other than the `.blend` one: the meshes USD, glTF,
//! OBJ and PLY readers make, and a `.phx` scene, each into the same
//! [`SceneModel`] a `.blend` opens into. The model is Blender's space
//! (Z up); a Y-up source is turned on the way in.

use super::{SceneCorners, SceneMaterial, SceneMesh, SceneModel, SceneNormals, SceneObject};
use crate::export::scene::PhxScene;
use std::sync::Arc;

/// Y up to Z up: the inverse of `export::usd::Z_TO_Y`.
const Y_TO_Z: glam::Mat4 = glam::Mat4::from_cols(glam::Vec4::X, glam::Vec4::Z, glam::Vec4::NEG_Y, glam::Vec4::W);

impl SceneModel {
    /// A scene of `mesh`, one object each, read from `source`; `y_up` when
    /// the source stands Y up (USD, glTF, OBJ), so it is turned to Z up.
    pub fn from_meshes(mesh: Vec<crate::Mesh>, source: &str, y_up: bool) -> SceneModel {
        let mut model = SceneModel { source_path: Some(source.to_string()), ..Default::default() };
        let turn = match y_up {
            true => Y_TO_Z,
            false => glam::Mat4::IDENTITY,
        };
        for m in mesh {
            let world = turn * m.transform.unwrap_or(glam::Mat4::IDENTITY);
            let index: Vec<u32> = m.indices.iter().flatten().copied().collect();
            let uv = m.uvs.filter(|u| u.len() == m.positions.len()).unwrap_or_default();
            let colour = m.colors.as_ref().and_then(|c| c.first().copied()).unwrap_or([1.0; 4]);
            let mut object = object(&m.name, m.positions, index, m.normals, world);
            object.uvs = Arc::new(uv);
            object.color = colour;
            model.add_object(object, None);
        }
        model.fill_report(source);
        model
    }

    /// A scene from a `.phx` scene file read from `source` (already Blender's space).
    pub fn from_phx(scene: &PhxScene, source: &str) -> SceneModel {
        let mut model = SceneModel { source_path: Some(source.to_string()), ..Default::default() };
        for m in scene.meshes.iter().filter(|m| m.visible) {
            let world = glam::Mat4::from_cols_array_2d(&m.matrix_world);
            let index: Vec<u32> = m.indices.iter().flatten().copied().collect();
            let normal = Some(m.normals.clone()).filter(|n| !n.is_empty());
            let mut object = object(&m.name, m.vertices.clone(), index, normal, world);
            object.materials = m
                .materials
                .iter()
                .filter_map(|name| scene.materials.iter().find(|x| &x.name == name))
                .map(|x| SceneMaterial {
                    name: x.name.clone(),
                    base_color: x.base_color,
                    viewport_color: x.base_color,
                    roughness: x.roughness,
                    metallic: x.metallic,
                    ..SceneMaterial::neutral()
                })
                .collect();
            if m.uvs.len() == m.vertices.len() {
                object.uvs = Arc::new(m.uvs.clone());
            }
            model.add_object(object, None);
        }
        model.report.scene = scene.name.clone();
        model.fill_report(source);
        model
    }

    /// The read report's counts, from the objects as they stand.
    fn fill_report(&mut self, source: &str) {
        let stem = std::path::Path::new(source).file_stem().and_then(|s| s.to_str()).unwrap_or_default();
        if self.report.scene.is_empty() {
            self.report.scene = stem.to_string();
        }
        self.report.objects = self.objects.len();
        self.report.verts = self.objects.iter().map(|o| o.mesh.vertices.len()).sum();
        self.report.tris = self.objects.iter().map(|o| o.mesh.indices.len() / 3).sum();
        self.report.faces = self.report.tris;
    }
}

/// One object at `world`: its triangles, its normals per vertex when the
/// reader gave one for each, else per face.
fn object(name: &str, vertex: Vec<[f32; 3]>, index: Vec<u32>, normal: Option<Vec<[f32; 3]>>, world: glam::Mat4) -> SceneObject {
    let normals = match normal.filter(|n| n.len() == vertex.len()) {
        Some(n) => SceneNormals::PerVertex(n),
        None => SceneNormals::PerFace(
            index
                .chunks_exact(3)
                .map(|t| {
                    let p = |k: usize| glam::Vec3::from(vertex.get(t[k] as usize).copied().unwrap_or_default());
                    (p(1) - p(0)).cross(p(2) - p(0)).normalize_or_zero().into()
                })
                .collect(),
        ),
    };
    let mesh = SceneMesh { vertices: vertex, indices: index, normals, corners: SceneCorners::default() };
    let mut object = SceneObject::identity(name, mesh);
    let (scale, _, location) = world.to_scale_rotation_translation();
    object.matrix_world = world.to_cols_array_2d();
    object.location = location.into();
    object.scale = scale.into();
    object
}
