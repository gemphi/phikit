//! What the UV editor's header offers, in Blender 5.0's words and order
//! (`IMAGE_HT_header` over `ToolSettings`, checked with bpy): each setting
//! with its options and the default a new scene has.

use crate::v2::state::UvKey;

/// One header setting: its key, the icon it shows, its tooltip, its options
/// and its default.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BUvSetting {
    pub key: UvKey,
    pub icon: &'static str,
    pub tooltip: &'static str,
    pub options: &'static [&'static str],
    pub default: &'static str,
}

const fn setting(key: UvKey, icon: &'static str, tooltip: &'static str, options: &'static [&'static str], default: &'static str) -> BUvSetting {
    BUvSetting { key, icon, tooltip, options, default }
}

/// On / off, as a toggle stores it.
pub const ON: &str = "true";
pub const OFF: &str = "false";
const SWITCH: &[&str] = &[ON, OFF];

/// `use_uv_select_sync`: on in a new 5.0 scene.
pub const SYNC: BUvSetting = setting(UvKey::Sync, "arrows_horizontal", "UV Sync Selection", SWITCH, ON);
/// `uv_select_mode` / `mesh_select_mode` (while syncing): Vertex.
pub const SELECT_MODES: &[(&str, &str)] = &[("Vertex", "point"), ("Edge", "edge_slide"), ("Face", "square")];
pub const SELECT_DEFAULT: &str = "Vertex";
/// `use_uv_select_island`.
pub const ISLAND: BUvSetting = setting(UvKey::Island, "island", "UV Island Selection", SWITCH, OFF);
/// `uv_sticky_select_mode`.
pub const STICKY: BUvSetting = setting(UvKey::Sticky, "link_2", "Sticky Selection Mode", &["Disabled", "Shared Location", "Shared Vertex"], "Shared Location");
/// `SpaceImage.pivot_point` in the UV editor (`rna_SpaceImageEditor_pivot_itemf`).
pub const PIVOT: BUvSetting = setting(UvKey::Pivot, "target", "Pivot Point", &["Bounding Box Center", "Median Point", "2D Cursor", "Individual Origins"], "Bounding Box Center");
/// `use_snap_uv`, and `snap_uv_element`.
pub const SNAP: BUvSetting = setting(UvKey::Snap, "magnet", "Snap", SWITCH, OFF);
pub const SNAP_ELEMENT: BUvSetting = setting(UvKey::SnapElement, "grid", "Snap To", &["Increment", "Grid", "Vertex"], "Increment");
/// `use_proportional_edit`, and `proportional_edit_falloff`.
pub const PROPORTIONAL: BUvSetting = setting(UvKey::Proportional, "circle_dashed", "Proportional Editing", SWITCH, OFF);
pub const FALLOFF: BUvSetting = setting(UvKey::Falloff, "trending_up", "Proportional Editing Falloff", &["Smooth", "Sphere", "Root", "Inverse Square", "Sharp", "Linear", "Constant", "Random"], "Smooth");
