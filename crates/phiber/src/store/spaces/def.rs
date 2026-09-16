//! BSpace constants - labels, icons, default object modes, and workspace definitions.
//!
//! Matches `DNA_workspace_types.h` from Blender 5.2.

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum BSpaceType {
    Layout = 0,
    Modeling = 1,
    Sculpting = 2,
    UvEditing = 3,
    TexturePaint = 4,
    Shading = 5,
    Animation = 6,
    Rendering = 7,
    Compositing = 8,
    GeometryNodes = 9,
    Scripting = 10,
}

pub const LAYOUT_LABEL: &str = "Layout";
pub const MODELING_LABEL: &str = "Modeling";
pub const SCULPTING_LABEL: &str = "Sculpting";
pub const UV_EDITING_LABEL: &str = "UV Editing";
pub const TEXTURE_PAINT_LABEL: &str = "Texture Paint";
pub const SHADING_LABEL: &str = "Shading";
pub const ANIMATION_LABEL: &str = "Animation";
pub const RENDERING_LABEL: &str = "Rendering";
pub const COMPOSITING_LABEL: &str = "Compositing";
pub const GEOMETRY_NODES_LABEL: &str = "Geometry Nodes";
pub const SCRIPTING_LABEL: &str = "Scripting";

pub const LAYOUT_ICON: &str = "layout-grid";
pub const MODELING_ICON: &str = "box";
pub const SCULPTING_ICON: &str = "brush";
pub const UV_EDITING_ICON: &str = "grid-2x2";
pub const TEXTURE_PAINT_ICON: &str = "paintbrush";
pub const SHADING_ICON: &str = "circle";
pub const ANIMATION_ICON: &str = "film";
pub const RENDERING_ICON: &str = "camera";
pub const COMPOSITING_ICON: &str = "layers";
pub const GEOMETRY_NODES_ICON: &str = "cpu";
pub const SCRIPTING_ICON: &str = "code";

/// Object interaction mode enum matching Blender 5.2 `eContextObjectMode`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum BObjectMode {
    Object = 0,
    Edit = 1,
    Sculpt = 2,
    VertexPaint = 3,
    WeightPaint = 4,
    TexturePaint = 5,
    Pose = 6,
    GreasePencil = 7,
}

impl BSpaceType {
    /// Default object interaction mode for this workspace.
    pub fn default_object_mode(self) -> BObjectMode {
        match self {
            BSpaceType::Layout
            | BSpaceType::Animation
            | BSpaceType::Rendering
            | BSpaceType::Compositing
            | BSpaceType::GeometryNodes
            | BSpaceType::Scripting => BObjectMode::Object,
            BSpaceType::Modeling | BSpaceType::UvEditing => BObjectMode::Edit,
            BSpaceType::Sculpting => BObjectMode::Sculpt,
            BSpaceType::TexturePaint => BObjectMode::TexturePaint,
            BSpaceType::Shading => BObjectMode::Object,
        }
    }
}
