//! Shader Editor host definitions matching Blender 5.2 `SpaceNode` (Shader mode) in `DNA_space_types.h`.

#[derive(Clone, Debug, PartialEq)]
pub struct ShaderEditorConfig {
    pub material_name: Option<String>,
    pub world_shader: bool,
    pub zoom: f32,
    pub pan_x: f32,
    pub pan_y: f32,
}

impl Default for ShaderEditorConfig {
    fn default() -> Self {
        Self {
            material_name: None,
            world_shader: false,
            zoom: 1.0,
            pan_x: 0.0,
            pan_y: 0.0,
        }
    }
}
