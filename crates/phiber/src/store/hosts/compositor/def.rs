//! Compositor host definitions matching Blender 5.2 `SpaceNode` (Compositor mode) in `DNA_space_types.h`.

#[derive(Clone, Debug, PartialEq)]
pub struct CompositorConfig {
    pub scene_name: Option<String>,
    pub use_backdrop: bool,
    pub zoom: f32,
    pub pan_x: f32,
    pub pan_y: f32,
}

impl Default for CompositorConfig {
    fn default() -> Self {
        Self {
            scene_name: None,
            use_backdrop: true,
            zoom: 1.0,
            pan_x: 0.0,
            pan_y: 0.0,
        }
    }
}
