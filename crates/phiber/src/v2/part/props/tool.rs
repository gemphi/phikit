//! Active Tool context — `bl_ui/properties_tool.py`.
// Active Tool context set — properties for the currently selected tool.
// Blender ref: `bl_ui/properties_tool.py`, `bl_ui/space_toolsystem_common.py`.

use super::prop::*;
use super::set::BSectionSpec;
use super::set::sec;

pub static ACTIVE_TOOL: &[BPropSpec] = &[s("workspace_tool.idname", "Tool")];

/// `VIEW3D_PT_gizmo_display` — also the Tool tab's Options panel. One
/// table: the header popover and the panel must not disagree.
///
/// The `.on()` rows are Blender's own defaults. Every one of these used to
/// default off, which is why a fresh file had no floor, no axes and no
/// gizmo — the panel said "Floor" with the box clear, and the viewport drew
/// what the box said.
pub static OPTIONS: &[BPropSpec] = &[
    b("show_gizmo", "Show Gizmo").on(),
    // `V3D_GIZMO_HIDE_NAVIGATE`: the orientation gizmo and the zoom and pan
    // buttons in the viewport's corner.
    b("show_gizmo_navigate", "Navigate").on(),
    // The three object gizmos ship OFF in Blender: the move/rotate/scale
    // handles appear when you pick the matching tool, not over every object
    // in the default file.
    b("show_gizmo_object_translate", "Move"),
    b("show_gizmo_object_rotate", "Rotate"),
    b("show_gizmo_object_scale", "Scale"),
    b("show_gizmo_context", "Context").on(),
    b("show_gizmo_tool", "Tool").on(),
];

/// `VIEW3D_PT_overlay` — also the Tool tab's Overlays panel.
pub static OVERLAYS: &[BPropSpec] = &[
    b("show_overlays", "Overlays").on(),
    b("show_floor", "Floor").on(),
    b("show_axis_x", "X Axis").on(),
    b("show_axis_y", "Y Axis").on(),
    // Z alone ships off: the default view looks down the floor plane, where
    // a vertical axis line would run through every object.
    b("show_axis_z", "Z Axis"),
    b("show_text", "Text Info").on(),
    // Statistics ships off, as in Blender: the status bar carries the counts.
    b("show_stats", "Statistics"),
    b("show_cursor", "3D Cursor").on(),
    b("show_extras", "Extras").on(),
    b("show_relationship_lines", "Relationships").on(),
    b("show_outline_selected", "Outline Selected").on(),
    b("show_bones", "Bones").on(),
    b("show_object_origins", "Origins").on(),
    b("show_object_origins_all", "Origins (All)"),
];

/// Where `OVERLAYS` turns from Guides to Objects, the two headings
/// `VIEW3D_PT_overlay_guides` and `VIEW3D_PT_overlay_object` put them under.
pub const OVERLAY_OBJECTS: usize = 8;

pub static SECTIONS: &[BSectionSpec] = &[
    sec("Active Tool", ACTIVE_TOOL),
    sec("Options", OPTIONS),
    sec("Overlays", OVERLAYS),
];
