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

// Sculpt-mode brush props (representative).
static SCULPT: &[BToolPropSpec] = &[p("Radius"), p("Strength"), p("Falloff")];
static SELECT: &[BToolPropSpec] = &[p("Select Through")];
static MOVE_RS: &[BToolPropSpec] = &[p("Orientation"), p("Pivot"), p("Snap")];

pub fn props_for(host: BHostType, tool: &str) -> &'static [BToolPropSpec] {
    if !matches!(host, BHostType::View3D | BHostType::Image | BHostType::Uv) {
        return &[];
    }
    match tool {
        "Select" | "Select Box" => SELECT,
        "Move" | "Rotate" | "Scale" | "Transform" => MOVE_RS,
        _ => SCULPT,
    }
}
