//! USDA for a `BlenderScene` export description: turned into the same
//! [`SceneModel`] every reader makes, then written by the one scene path
//! (`SceneModel::to_usda`), so nothing here is Blender-only.

use super::types::BlenderScene;
use crate::Result;
use crate::scene::{SceneMaterial, SceneModel};

impl From<&BlenderScene> for SceneModel {
    /// Its visible meshes as objects at their world matrices (Blender's
    /// space, Z up), each with the materials it names.
    fn from(scene: &BlenderScene) -> SceneModel {
        let visible: Vec<_> = scene.meshes.iter().filter(|m| m.visible).collect();
        let mesh = visible
            .iter()
            .map(|m| crate::Mesh {
                name: m.name.clone(),
                positions: m.vertices.clone(),
                normals: Some(m.normals.clone()).filter(|n| n.len() == m.vertices.len()),
                uvs: Some(m.uvs.clone()).filter(|u| u.len() == m.vertices.len()),
                indices: m.indices.clone(),
                transform: Some(glam::Mat4::from_cols_array_2d(&m.matrix_world)),
                ..Default::default()
            })
            .collect();
        let mut model = SceneModel::from_meshes(mesh, "", false);
        model.source_path = None;
        model.report.scene = scene.name.clone();
        for (o, m) in model.objects.iter_mut().zip(&visible) {
            o.materials = m
                .material_names
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
        }
        model
    }
}

impl BlenderScene {
    /// The scene as USDA (`SceneModel::to_usda`): every visible mesh an
    /// Xform with its mesh and material.
    pub fn to_usda(&self) -> Result<String> {
        Ok(SceneModel::from(self).to_usda().text)
    }
}
