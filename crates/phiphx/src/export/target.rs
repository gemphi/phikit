//! The write side's one `match`: a scene goes out in the format its
//! output's extension names, each arm one writer module — USD
//! (`export/usd`), PHX (`export/scene`), glTF (`export/gltf`), PLY
//! (`export/ply`). Nothing here knows what the scene was read from.

use crate::export::scene::{PhxMaterial, PhxMesh, PhxScene};
use crate::import::FileFormat;
use crate::scene::SceneModel;
use crate::{Mesh, PointCloud, PointCloudFormat, Result, Vertex};
use std::path::Path;

/// A format a whole scene can be written to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SceneTarget {
    /// USD as text (`.usda`, or `.usd` — either encoding is a `.usd`).
    Usda,
    /// PHX scene: magic + JSON (`.phx`).
    Phx,
    /// glTF 2.0, buffers embedded (`.gltf`): every shown object in one mesh.
    Gltf,
    /// PLY points (`.ply`): every shown object's points, world space.
    Ply,
}

impl SceneTarget {
    /// The target a format is written as, if a scene can be.
    pub fn of(format: FileFormat) -> Option<SceneTarget> {
        match format {
            FileFormat::Usd | FileFormat::Usda => Some(Self::Usda),
            FileFormat::Phx => Some(Self::Phx),
            FileFormat::Gltf => Some(Self::Gltf),
            FileFormat::Ply => Some(Self::Ply),
            FileFormat::Usdc | FileFormat::Usdz | FileFormat::Glb => None,
            FileFormat::Obj | FileFormat::Stl | FileFormat::Fbx | FileFormat::Blend => None,
        }
    }

    /// The target the extension of `path` names.
    pub fn of_path(path: &Path) -> Option<SceneTarget> {
        let ext = path.extension()?.to_str()?;
        Self::of(FileFormat::from_extension(ext)?)
    }

    /// `scene` written in this format.
    pub fn write(self, scene: &SceneModel) -> Result<Vec<u8>> {
        match self {
            Self::Usda => Ok(scene.to_usda().text.into_bytes()),
            Self::Phx => PhxScene::from(scene).to_bytes(),
            Self::Gltf => crate::export::gltf::GltfExporter::export_mesh_gltf(&scene.world_mesh(crate::export::usd::Z_TO_Y)).map(String::into_bytes),
            Self::Ply => {
                let mesh = scene.world_mesh(glam::Mat4::IDENTITY);
                let mut cloud = PointCloud::new(&scene.report.scene);
                let normal = mesh.normals.unwrap_or_default();
                for (i, p) in mesh.positions.iter().enumerate() {
                    let n = normal.get(i).copied().unwrap_or_default();
                    cloud.add_vertex(Vertex::new(p[0], p[1], p[2]).with_normal(n[0], n[1], n[2]));
                }
                let format = PointCloudFormat::Ply { binary: false, include_normals: true, include_colors: false };
                crate::export::PointCloudExporter::export_point_cloud(&cloud, &format).map(String::into_bytes)
            }
        }
    }
}

impl SceneModel {
    /// Every shown object's triangles in one mesh, in world space turned by
    /// `turn` (e.g. Z up to Y up), with a normal per point (the triangle's
    /// own; points are not shared between objects).
    pub fn world_mesh(&self, turn: glam::Mat4) -> Mesh {
        let mut out = Mesh { name: self.report.scene.clone(), ..Default::default() };
        let mut normal = Vec::new();
        for (_, o) in self.objects.iter().enumerate().filter(|(i, _)| self.shows(*i)) {
            let world = turn * glam::Mat4::from_cols_array_2d(&o.matrix_world);
            for t in o.mesh.indices.chunks_exact(3) {
                let p = [t[0], t[1], t[2]].map(|i| world.transform_point3(o.mesh.vertices[i as usize].into()));
                let n: [f32; 3] = (p[1] - p[0]).cross(p[2] - p[0]).normalize_or_zero().into();
                let base = out.positions.len() as u32;
                out.positions.extend(p.map(<[f32; 3]>::from));
                normal.extend([n; 3]);
                out.indices.push([base, base + 1, base + 2]);
            }
        }
        out.normals = Some(normal);
        out
    }
}

impl From<&SceneModel> for PhxScene {
    /// The shown objects and their materials as a PHX scene (Blender's space).
    fn from(scene: &SceneModel) -> PhxScene {
        let mut out = PhxScene::new(&scene.report.scene);
        let mut seen = std::collections::HashSet::new();
        for (_, o) in scene.objects.iter().enumerate().filter(|(i, _)| scene.shows(*i)) {
            let normals = match &o.mesh.normals {
                crate::scene::SceneNormals::PerVertex(n) => n.clone(),
                crate::scene::SceneNormals::PerFace(_) => Vec::new(),
            };
            out.add_mesh(PhxMesh {
                name: o.name.clone(),
                vertices: o.mesh.vertices.clone(),
                normals,
                uvs: o.uvs.to_vec(),
                colors: Vec::new(),
                indices: o.mesh.indices.chunks_exact(3).map(|t| [t[0], t[1], t[2]]).collect(),
                materials: o.materials.iter().map(|m| m.name.clone()).collect(),
                matrix_world: o.matrix_world,
                visible: true,
            });
            for m in o.materials.iter().filter(|m| seen.insert(m.name.clone())) {
                out.add_material(PhxMaterial {
                    name: m.name.clone(),
                    base_color: m.base_color,
                    metallic: m.metallic,
                    roughness: m.roughness,
                    emission: [0.0; 3],
                    emission_strength: 0.0,
                    alpha: m.base_color[3],
                    use_nodes: true,
                    base_color_texture: m.base_color_texture.clone(),
                    normal_texture: None,
                    roughness_texture: None,
                });
            }
        }
        out
    }
}
