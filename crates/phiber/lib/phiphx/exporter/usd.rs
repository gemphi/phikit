//! Native USD Exporter
//!
//! Direct export to USDA/USDC format without external dependencies.
//! Optimized for PHI PHX workflow.

pub mod mesh;
mod point_cloud;
mod writer;

pub use writer::UsdaWriter;

/// Exporter for USDA and USDC formats.
pub struct UsdExporter;

impl UsdExporter {
    pub fn export_mesh_usda(mesh: &crate::phx::Mesh) -> crate::phx::Result<String> {
        let name = &mesh.name;
        let positions = &mesh.positions;
        let normals = mesh.normals.as_deref();
        let uvs = mesh.uvs.as_deref();
        let colors = mesh.colors.as_deref();
        let face_vertex_counts = vec![3; mesh.indices.len()];
        let mut face_vertex_indices = Vec::with_capacity(mesh.indices.len() * 3);
        for tri in &mesh.indices {
            face_vertex_indices.push(tri[0] as i32);
            face_vertex_indices.push(tri[1] as i32);
            face_vertex_indices.push(tri[2] as i32);
        }
        let transform = mesh.transform.map(|m| m.to_cols_array_2d());
        mesh::export_mesh_usda(
            name,
            positions,
            normals,
            uvs,
            colors,
            &face_vertex_counts,
            &face_vertex_indices,
            transform,
        )
    }

    /// Export a point cloud to USDA format.
    pub fn export_point_cloud_usda(cloud: &crate::phx::PointCloud) -> crate::phx::Result<String> {
        point_cloud::export_point_cloud_usda(cloud)
    }

    /// Export a point cloud to USDA format using spheres representation.
    pub fn export_point_cloud_usda_with_spheres(
        cloud: &crate::phx::PointCloud,
        radius: f32,
    ) -> crate::phx::Result<String> {
        point_cloud::export_point_cloud_usda_with_spheres(cloud, radius as f64)
    }

    /// Export a point cloud to USDC format (binary USD).
    pub fn export_usdc(cloud: &crate::phx::PointCloud) -> crate::phx::Result<Vec<u8>> {
        point_cloud::export_usdc(cloud)
    }
}

#[cfg(test)]
mod tests;
