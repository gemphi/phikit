//! Native USD Exporter
//!
//! Direct export to USDA/USDC format without external dependencies:
//! meshes, point clouds, the shapes of `data/shape.tsv` (`Shape::to_usda`)
//! and whole scenes (`SceneModel::to_usda`).

pub mod mesh;
mod name;
mod point_cloud;
mod scene;
mod shape;
mod stage;
mod thing;
mod writer;

pub use name::{UsdNameSet, usd_word};
pub use scene::{UsdaScene, Z_TO_Y};
pub use stage::{UsdMesh, extent};
pub use writer::UsdaWriter;

/// Exporter for USDA and USDC formats.
pub struct UsdExporter;

impl UsdExporter {
    pub fn export_mesh_usda(mesh: &crate::Mesh) -> crate::Result<String> {
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
    pub fn export_point_cloud_usda(cloud: &crate::PointCloud) -> crate::Result<String> {
        point_cloud::export_point_cloud_usda(cloud)
    }

    /// Export a point cloud to USDA format using spheres representation.
    pub fn export_point_cloud_usda_with_spheres(
        cloud: &crate::PointCloud,
        radius: f32,
    ) -> crate::Result<String> {
        point_cloud::export_point_cloud_usda_with_spheres(cloud, radius as f64)
    }

    /// Export a point cloud to USDC format (binary USD).
    pub fn export_usdc(cloud: &crate::PointCloud) -> crate::Result<Vec<u8>> {
        point_cloud::export_usdc(cloud)
    }
}

#[cfg(test)]
mod tests;
