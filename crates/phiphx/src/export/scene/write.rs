//! PHX scene binary serialization and conversion helpers.

use super::types::{PHX_SCENE_MAGIC, PhxMaterial, PhxMesh, PhxScene};
use crate::{PointCloudError, Result};
use byteorder::{LittleEndian, WriteBytesExt};
use std::io::Write;

impl PhxScene {
/// Write the scene (magic, JSON length, JSON).
pub fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
    let scene = self;
    // Write magic
    writer.write_all(PHX_SCENE_MAGIC)?;

    // Serialize JSON metadata
    let json_bytes =
        serde_json::to_vec(scene).map_err(|e| PointCloudError::Serialization(e.to_string()))?;

    // Write JSON length
    writer.write_u32::<LittleEndian>(json_bytes.len() as u32)?;

    // Write JSON data
    writer.write_all(&json_bytes)?;

    Ok(())
}

/// The scene as PHX bytes.
pub fn to_bytes(&self) -> Result<Vec<u8>> {
    let mut output = Vec::new();
    self.write(&mut output)?;
    Ok(output)
}

/// The scene's points as PLY text (ASCII or binary-in-string).
pub fn to_ply(&self, binary: bool) -> Result<String> {
    let scene = self;
    use crate::PointCloudFormat;
    use crate::export::PointCloudExporter;

    let cloud = scene.to_point_cloud();
    let format = PointCloudFormat::Ply {
        binary,
        include_normals: true,
        include_colors: false,
    };
    PointCloudExporter::export_point_cloud(&cloud, &format)
}

/// A one-triangle sample scene (tests, demos).
pub fn sample() -> PhxScene {
    let mut scene = PhxScene::new("SampleScene");

    // Add a simple triangle mesh
    let mesh = PhxMesh {
        name: "Triangle".to_string(),
        vertices: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
        normals: vec![[0.0, 0.0, 1.0], [0.0, 0.0, 1.0], [0.0, 0.0, 1.0]],
        uvs: vec![[0.0, 0.0], [1.0, 0.0], [0.5, 1.0]],
        colors: Vec::new(),
        indices: vec![[0, 1, 2]],
        materials: vec!["DefaultMaterial".to_string()],
        matrix_world: [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ],
        visible: true,
    };
    scene.add_mesh(mesh);

    // Add a material
    let material = PhxMaterial {
        name: "DefaultMaterial".to_string(),
        base_color: [0.8, 0.2, 0.2, 1.0],
        metallic: 0.0,
        roughness: 0.5,
        emission: [0.0, 0.0, 0.0],
        emission_strength: 0.0,
        alpha: 1.0,
        use_nodes: true,
        base_color_texture: None,
        normal_texture: None,
        roughness_texture: None,
    };
    scene.add_material(material);

    scene
}
}
