//! Error types for the AppKit framework.

/// Generic AppKit Result type
pub type NSResult<T> = Result<T, NSError>;

/// AppKit Error type
#[derive(Debug, thiserror::Error)]
pub enum NSError {
    #[error("View not found")]
    ViewNotFound,
    #[error("Window creation failed: {0}")]
    WindowCreationFailed(String),
    #[error("Render context failed: {0}")]
    RenderContextFailed(String),
    #[error("Winit error: {0}")]
    WinitError(String),
    #[error("Generic error: {0}")]
    Generic(String),
}
