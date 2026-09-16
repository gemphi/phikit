//! Error and result types for point cloud operations.

// why not use phi::std?
use std::io;
use thiserror::Error;

/// Errors that can occur during point cloud operations
#[derive(Error, Debug)]
pub enum PointCloudError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    #[error("Invalid format: {0}")]
    InvalidFormat(String),

    #[error("Import error: {0}")]
    ImportError(String),

    #[error("Export error: {0}")]
    ExportError(String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("No vertices in point cloud")]
    EmptyPointCloud,
}

/// Result type for point cloud operations
pub type Result<T> = std::result::Result<T, PointCloudError>;
