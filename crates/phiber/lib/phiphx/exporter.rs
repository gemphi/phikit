//! Point cloud exporter.
//!
//! Implements export to various point cloud formats.

pub mod blend;
pub mod gltf;
pub mod phx;
pub mod scene;
pub mod usd;
pub use scene::*;

pub mod binary;
pub mod gltf_export;
pub mod json;
pub mod las;
pub mod pcd;
pub mod ply;
pub mod usd_export;
pub mod xyz;

use crate::phx::{PointCloud, PointCloudFormat, Result};

/// Main exporter service for point clouds.
pub struct PointCloudExporter;

impl PointCloudExporter {
    /// Export a point cloud to the specified format
    pub fn export_point_cloud(cloud: &PointCloud, format: &PointCloudFormat) -> Result<String> {
        match format {
            PointCloudFormat::Ply {
                binary,
                include_normals,
                include_colors,
            } => ply::PlyExporter::export_phx(cloud, *binary, *include_normals, *include_colors),
            PointCloudFormat::Pcd { binary, compressed } => {
                pcd::PcdExporter::export_pcd(cloud, *binary, *compressed)
            }
            PointCloudFormat::Las { version } => las::LasExporter::export_las(cloud, version),
            PointCloudFormat::Xyz {
                include_normals,
                delimiter,
            } => xyz::XyzExporter::export_xyz(cloud, *include_normals, delimiter),
            PointCloudFormat::Json { pretty } => json::JsonExporter::export_json(cloud, *pretty),
            PointCloudFormat::Binary {
                include_normals,
                include_colors,
            } => binary::BinaryExporter::export_binary(cloud, *include_normals, *include_colors),
        }
    }

    /// Import a point cloud from the specified format
    pub fn import_point_cloud(data: &str, format: &PointCloudFormat) -> Result<PointCloud> {
        match format {
            PointCloudFormat::Ply { .. } => ply::PlyExporter::import_phx(data),
            PointCloudFormat::Xyz { delimiter, .. } => {
                xyz::XyzExporter::import_xyz(data, delimiter)
            }
            PointCloudFormat::Json { .. } => json::JsonExporter::import_json(data),
            _ => Err(crate::phx::PointCloudError::InvalidFormat(
                "Import not implemented for this format".to_string(),
            )),
        }
    }

    /// Export point cloud to bytes (for binary formats)
    pub fn export_point_cloud_to_bytes(
        cloud: &PointCloud,
        format: &PointCloudFormat,
    ) -> Result<Vec<u8>> {
        match format {
            PointCloudFormat::Binary {
                include_normals,
                include_colors,
            } => binary::BinaryExporter::export_binary_to_bytes(
                cloud,
                *include_normals,
                *include_colors,
            ),
            PointCloudFormat::Ply {
                binary: true,
                include_normals,
                include_colors,
            } => ply::PlyExporter::export_phx_to_bytes(cloud, *include_normals, *include_colors),
            _ => {
                let text = Self::export_point_cloud(cloud, format)?;
                Ok(text.into_bytes())
            }
        }
    }
}
