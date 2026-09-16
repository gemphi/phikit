//! PHX Format Support
//!
//! Data structures for point clouds, meshes, and materials.

pub mod material;

pub use material::{
    BindingStrength, GpuMaterial, MaterialBinding, MaterialLibrary, PreviewSurfaceMaterial,
};

/// Point cloud output format
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum PointCloudFormat {
    /// PHX format (Stanford Polygon Library)
    Ply {
        binary: bool,
        include_normals: bool,
        include_colors: bool,
    },
    /// PCD format (Point Cloud Data)
    Pcd { binary: bool, compressed: bool },
    /// LAS format (LiDAR data)
    Las { version: String },
    /// XYZ format (simple text)
    Xyz {
        include_normals: bool,
        delimiter: String,
    },
    /// JSON format
    Json { pretty: bool },
    /// Custom binary format
    Binary {
        include_normals: bool,
        include_colors: bool,
    },
}

impl PointCloudFormat {
    /// Default PHX ASCII format
    pub fn ply_ascii() -> Self {
        Self::Ply {
            binary: false,
            include_normals: true,
            include_colors: true,
        }
    }

    /// Default PHX binary format
    pub fn ply_binary() -> Self {
        Self::Ply {
            binary: true,
            include_normals: true,
            include_colors: true,
        }
    }

    /// Default XYZ format
    pub fn xyz_default() -> Self {
        Self::Xyz {
            include_normals: false,
            delimiter: " ".to_string(),
        }
    }

    /// Default JSON format
    pub fn json_pretty() -> Self {
        Self::Json { pretty: true }
    }

    /// Compact JSON format
    pub fn json_compact() -> Self {
        Self::Json { pretty: false }
    }

    /// Default PCD format
    pub fn pcd_default() -> Self {
        Self::Pcd {
            binary: true,
            compressed: false,
        }
    }
}
