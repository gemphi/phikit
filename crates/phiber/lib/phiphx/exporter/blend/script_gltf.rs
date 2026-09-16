//! glTF 2.0 scene writer (delegates to `gltf_native`).

use super::types::BlenderScene;
use crate::phx::Result;

/// Export a BlenderScene to glTF 2.0 format
///
/// Uses `gltf_native` for actual format writing.
pub fn export_scene_to_gltf(scene: &BlenderScene, binary: bool) -> Result<String> {
    // Convert first mesh to point cloud for glTF export
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
        crate::phx::exporter::gltf::GltfExporter::export_point_cloud_gltf(&cloud, binary)
    } else {
        let cloud = crate::PointCloud::new(&scene.name);
        crate::phx::exporter::gltf::GltfExporter::export_point_cloud_gltf(&cloud, binary)
    }
}
