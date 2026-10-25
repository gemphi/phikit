//! The boxes each header popover draws, and which host draws which.
//!
//! One table per FAMILY, not per host. Four node editors draw the same six
//! overlays and two image editors draw the same seven; a table per host is
//! how they come to disagree about which boxes exist. Where Blender shares
//! one RNA struct between hosts (`SpaceNodeEditor.overlay`,
//! `SpaceImageEditor.overlay`) the family shares one scope too, so switching
//! a box off in the Shader editor switches it off in Geometry Nodes.
//!
//! Defaults are Blender's, box by box. Most overlays ship on — an overlay
//! you have to go and switch on is not an overlay — but the ones Blender
//! ships off (`show_stretch`, `show_marker_search`, the metadata read-outs)
//! ship off here, because a fresh file that draws MORE than Blender's is as
//! wrong as one that draws less.

use crate::v2::host::BHostType;

/// One box: its label as Blender writes it, the RNA name (which is also its
/// key), what a fresh file answers, and the heading it opens.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BOverlaySpec {
    pub label: &'static str,
    /// The RNA name, which is also the key under `ctx.overlay.<family>`.
    pub key: &'static str,
    pub default_on: bool,
    /// The heading this box opens, or `""` to continue the one above.
    pub group: &'static str,
}

/// A box Blender ships on.
const fn o(key: &'static str, label: &'static str) -> BOverlaySpec {
    BOverlaySpec { label, key, default_on: true, group: "" }
}

/// A box Blender ships off.
const fn off(key: &'static str, label: &'static str) -> BOverlaySpec {
    BOverlaySpec { label, key, default_on: false, group: "" }
}

impl BOverlaySpec {
    /// Open a heading above this box.
    pub const fn under(mut self, group: &'static str) -> Self {
        self.group = group;
        self
    }
}

/// One header popover: what Blender calls it, the icon it wears, the scope
/// its boxes are stored under, and the boxes themselves.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BOverlayPanel {
    pub title: &'static str,
    pub icon: &'static str,
    /// The state scope, shared by every host that draws this panel.
    pub family: &'static str,
    pub boxes: &'static [BOverlaySpec],
}

/// `NODE_PT_overlay` — what a node editor draws over its tree.
///
/// `show_named_attributes` and `show_timing` are polled onto the geometry and
/// compositor trees in Blender; the table carries them for every node editor
/// because a box that is off costs nothing and a box that is missing is a
/// feature the user cannot find.
pub static NODE: [BOverlaySpec; 6] = [
    o("show_overlays", "Node Editor Overlays").under("Overlays"),
    o("show_wire_color", "Wire Colors"),
    o("show_context_path", "Context Path"),
    o("show_named_attributes", "Named Attributes"),
    o("show_previews", "Node Previews"),
    o("show_timing", "Timings"),
];

/// `SEQUENCER_PT_overlay` — what the Sequencer draws over its strips.
pub static SEQ: [BOverlaySpec; 8] = [
    o("show_strip_overlay", "Sequencer Overlays").under("Overlays"),
    o("show_strip_name", "Name"),
    o("show_strip_source", "Source"),
    o("show_strip_duration", "Duration"),
    o("show_grid", "Grid"),
    o("show_fcurves", "F-Curves"),
    o("show_thumbnails", "Thumbnails"),
    o("show_strip_offset", "Offsets"),
];

/// `IMAGE_PT_overlay` over `SpaceImageEditor.overlay` — the Image editor and
/// the UV editor are one space in Blender, so they are one family here.
///
/// `show_stretch` and `show_metadata` ship off: stretch recolours every face
/// and the metadata read-out covers the corner of the image.
pub static IMAGE: [BOverlaySpec; 7] = [
    o("show_overlays", "Image Editor Overlays").under("Overlays"),
    o("show_grid_background", "Grid").under("Guides"),
    o("show_grid_over_image", "Grid Over Image"),
    o("show_faces", "Faces").under("UV Editing"),
    off("show_modified_edges", "Modified Edges"),
    off("show_stretch", "Display Stretch"),
    off("show_metadata", "Metadata").under("Image"),
];

/// `CLIP_PT_display` over `SpaceClipEditor` — the Movie Clip editor's two
/// display groups, `CLIP_PT_marker_display` and `CLIP_PT_clip_display`.
///
/// Most of these ship off. A clip with every marker read-out on is unreadable,
/// which is why Blender turns the search area, the names and the 3D markers on
/// only when you are looking for them.
pub static CLIP: [BOverlaySpec; 12] = [
    o("show_marker_pattern", "Pattern").under("Marker Display"),
    off("show_marker_search", "Search"),
    off("show_tiny_markers", "Thin Markers"),
    o("show_track_path", "Path"),
    o("show_disabled", "Show Disabled"),
    off("show_names", "Info"),
    off("show_bundles", "3D Markers"),
    off("show_stable", "Stable").under("Clip Display"),
    off("show_grid", "Grid"),
    off("use_manual_calibration", "Manual Calibration"),
    o("show_annotation", "Annotations"),
    o("show_metadata", "Metadata"),
];

/// `SPREADSHEET_PT_filter` over `SpaceSpreadsheet` — the two standing
/// toggles. Blender's row filters are rows the user adds one at a time, which
/// is a different mechanism (a list, not a table) and is not here yet; these
/// two are what the panel offers before you add one.
pub static SHEET: [BOverlaySpec; 2] = [
    o("use_filter", "Enable Filters").under("Filters"),
    off("show_only_selected", "Selected Only"),
];

/// The panel a host hangs off its header's right end, if it has one.
///
/// Absent for a host whose right end is empty in Blender too: the Text editor
/// keeps its display toggles in `TEXT_PT_properties` (the sidebar), the
/// Console and Info have none, and the Outliner, the viewport, Properties and
/// the file browsers are answered elsewhere in `part::filter`.
pub fn panel_for(host: BHostType) -> Option<BOverlayPanel> {
    let panel = |title, icon, family, boxes| BOverlayPanel { title, icon, family, boxes };
    Some(match host {
        BHostType::Shader | BHostType::Node | BHostType::GeoNode | BHostType::Compositor => {
            panel("Overlays", "overlays", "node", &NODE)
        }
        BHostType::Seq => panel("Overlays", "overlays", "seq", &SEQ),
        BHostType::Image | BHostType::Uv => panel("Overlays", "overlays", "image", &IMAGE),
        BHostType::Clip => panel("Display", "eye", "clip", &CLIP),
        BHostType::Spreadsheet => panel("Filters", "filter", "spreadsheet", &SHEET),
        _ => return None,
    })
}
