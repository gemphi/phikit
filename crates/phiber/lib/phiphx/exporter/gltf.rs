//! Native glTF 2.0 Exporter
//!
//! Direct export to glTF format without external dependencies.
//! Optimized for PHI PHX workflow.

mod base64;
mod buffers;
mod mesh;
mod point_cloud;

pub use buffers::{
    ARRAY_BUFFER, COMPONENT_TYPE_FLOAT, COMPONENT_TYPE_UNSIGNED_SHORT, ELEMENT_ARRAY_BUFFER,
    GltfBufferWriter, PRIMITIVE_MODE_POINTS, PRIMITIVE_MODE_TRIANGLES,
};

/// Exporter for glTF and GLB formats.
pub struct GltfExporter;

impl GltfExporter {
    /// Export a point cloud to GLB format.
    pub fn export_point_cloud_glb(cloud: &crate::phx::PointCloud) -> crate::phx::Result<Vec<u8>> {
        point_cloud::export_point_cloud_glb(cloud)
    }

    /// Export a point cloud to glTF format (JSON or embedded binary).
    pub fn export_point_cloud_gltf(
        cloud: &crate::phx::PointCloud,
        binary: bool,
    ) -> crate::phx::Result<String> {
        point_cloud::export_point_cloud_gltf(cloud, binary)
    }

    /// Export a mesh to glTF format.
    pub fn export_mesh_gltf(mesh: &crate::phx::Mesh) -> crate::phx::Result<String> {
        let uvs = mesh.uvs.as_ref().map(|v| v.as_slice());
        let normals = mesh.normals.as_ref().map(|v| v.as_slice());
        mesh::export_mesh_gltf(&mesh.positions, normals, uvs, &mesh.indices)
    }
}

#[cfg(test)]
mod tests;
