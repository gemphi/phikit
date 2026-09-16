//! Node Editor host definitions matching Blender 5.2 `SpaceNode` in `DNA_space_types.h`.

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum NodeTreeType {
    Shader,
    Compositor,
    Geometry,
    Texture,
}

#[derive(Clone, Debug, PartialEq)]
pub struct NodeEditorConfig {
    pub tree_type: NodeTreeType,
    pub zoom: f32,
    pub pan_x: f32,
    pub pan_y: f32,
}

impl Default for NodeEditorConfig {
    fn default() -> Self {
        Self {
            tree_type: NodeTreeType::Shader,
            zoom: 1.0,
            pan_x: 0.0,
            pan_y: 0.0,
        }
    }
}
