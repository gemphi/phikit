//! Viewport model - camera state and scene data.
use std::sync::{Arc, Mutex};

/// Active manipulation tool in the viewport.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NSViewportTool {
    /// Click to select objects
    Select,
    /// Translate gizmo (move objects along axes)
    Move,
    /// Rotation gizmo
    Rotate,
    /// Scale gizmo
    Scale,
    /// Pan the camera (middle-mouse behavior)
    Pan,
    /// Zoom the camera
    Zoom,
}

impl Default for NSViewportTool {
    fn default() -> Self {
        Self::Select
    }
}

#[derive(Debug, Clone)]
pub struct NSViewportState {
    pub shading_mode: NSShadingMode,
    pub show_grid: bool,
    pub show_overlays: bool,
    pub selected_entity: Arc<Mutex<Option<u64>>>,
    pub active_tool: Arc<Mutex<NSViewportTool>>,
}

impl NSViewportState {
    pub fn new() -> Self {
        Self {
            shading_mode: NSShadingMode::Solid,
            show_grid: true,
            show_overlays: true,
            selected_entity: Arc::new(Mutex::new(None)),
            active_tool: Arc::new(Mutex::new(NSViewportTool::default())),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NSShadingMode {
    Wireframe,
    Solid,
    Material,
    Rendered,
}
