//! Active Tool context — `bl_ui/properties_tool.py`.
// Active Tool context set — properties for the currently selected tool.
// Blender ref: `bl_ui/properties_tool.py`, `bl_ui/space_toolsystem_common.py`.

use super::prop::*;
use super::set::BSectionSpec;
use super::set::sec;

pub static ACTIVE_TOOL: &[BPropSpec] = &[s("workspace_tool.idname", "Tool")];

static OPTIONS: &[BPropSpec] = &[
    b("show_gizmo", "Show Gizmo"),
    b("show_gizmo_object_translate", "Move"),
    b("show_gizmo_object_rotate", "Rotate"),
    b("show_gizmo_object_scale", "Scale"),
    b("show_gizmo_context", "Context"),
    b("show_gizmo_tool", "Tool"),
];

static OVERLAYS: &[BPropSpec] = &[
    b("show_overlays", "Overlays"),
    b("show_floor", "Floor"),
    b("show_axis_x", "X Axis"),
    b("show_axis_y", "Y Axis"),
    b("show_axis_z", "Z Axis"),
    b("show_extras", "Extras"),
    b("show_bones", "Bones"),
    b("show_relationship_lines", "Relationships"),
];

pub static SECTIONS: &[BSectionSpec] = &[
    sec("Active Tool", ACTIVE_TOOL),
    sec("Options", OPTIONS),
    sec("Overlays", OVERLAYS),
];
