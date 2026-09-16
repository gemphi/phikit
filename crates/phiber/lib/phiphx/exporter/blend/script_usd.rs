//! USDA scene writer (delegates to `usd_native`).

use super::types::BlenderScene;
use crate::phx::Result;

/// Export a BlenderScene to USDA format
///
/// Uses `usd_native` for actual format writing.
pub fn export_scene_to_usda(scene: &BlenderScene) -> Result<String> {
    if let Some(mesh) = scene.meshes.first() {
        let mut cloud = crate::PointCloud::new(&scene.name);
        for (i, pos) in mesh.vertices.iter().enumerate() {
            let mut vertex = crate::Vertex::new(pos[0], pos[1], pos[2]);
            if i < mesh.normals.len() {
                let n = &mesh.normals[i];
                vertex = vertex.with_normal(n[0], n[1], n[2]);
            }
            cloud.add_vertex(vertex);
        }
        crate::phx::exporter::usd::UsdExporter::export_point_cloud_usda(&cloud)
    } else {
        let cloud = crate::PointCloud::new(&scene.name);
        crate::phx::exporter::usd::UsdExporter::export_point_cloud_usda(&cloud)
    }
}
