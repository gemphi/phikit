//! 3D Viewport host definitions matching Blender 5.2 `SpaceView3D` in `DNA_view3d_types.h`.

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum ShadingType {
    Wireframe = 0,
    Solid = 1,
    MaterialPreview = 2,
    Rendered = 3,
}

#[derive(Clone, Debug, PartialEq)]
pub struct View3DConfig {
    pub lens: f32,
    pub clip_start: f32,
    pub clip_end: f32,
    pub shading: ShadingType,
    pub show_floor: bool,
    pub show_axis_x: bool,
    pub show_axis_y: bool,
    pub show_axis_z: bool,
    pub show_overlays: bool,
    pub show_gizmos: bool,
}

impl Default for View3DConfig {
    fn default() -> Self {
        Self {
            lens: 50.0,
            clip_start: 0.1,
            clip_end: 1000.0,
            shading: ShadingType::Solid,
            show_floor: true,
            show_axis_x: true,
            show_axis_y: true,
            show_axis_z: false,
            show_overlays: true,
            show_gizmos: true,
        }
    }
}
