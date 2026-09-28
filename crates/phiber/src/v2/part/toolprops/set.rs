//! Tool Properties set — settings shown in the ToolHeader when a tool is active.
//! Blender ref: `RGN_TYPE_TOOL_HEADER` content (View3D_HT_tool_header).

use crate::v2::host::BHostType;
use crate::v2::part::BPartType;
use crate::v2::spec::{BSpec, BSpecKind};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BToolPropSpec {
    pub label: &'static str,
}

impl BSpec for BToolPropSpec {
    fn kind(&self) -> BSpecKind {
        BSpecKind::Part(BPartType::ToolProps)
    }
    fn label(&self) -> &'static str {
        self.label
    }
}

const fn p(label: &'static str) -> BToolPropSpec {
    BToolPropSpec { label }
}

/// `VIEW3D_HT_tool_header.draw_tool_settings` in Object mode: the select
/// tools' mode, the transform tools' orientation, pivot and snapping. A tool
/// without settings shows none — and an editor whose tool header Blender
/// keeps hidden in its workspace (the Image editor in View mode: Rendering,
/// Shading) shows no tool header at all, since an empty region is not drawn.
static SELECT: &[BToolPropSpec] = &[p("Mode")];
static MOVE_RS: &[BToolPropSpec] = &[p("Orientation"), p("Pivot"), p("Snap")];
/// `IMAGE_HT_tool_header` in UV mode: the UV select tools' mode.
static UV_SELECT: &[BToolPropSpec] = &[p("Mode"), p("Sticky Selection Mode")];

pub fn props_for(host: BHostType, tool: &str) -> &'static [BToolPropSpec] {
    match (host, tool) {
        (BHostType::View3D, "Select" | "Select Box" | "Select Circle" | "Select Lasso") => SELECT,
        (BHostType::View3D, "Move" | "Rotate" | "Scale" | "Transform") => MOVE_RS,
        (BHostType::Uv, "Select" | "Select Box" | "Select Circle" | "Select Lasso") => UV_SELECT,
        (BHostType::Uv, "Move" | "Rotate" | "Scale" | "Transform") => MOVE_RS,
        _ => &[],
    }
}
