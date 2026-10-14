//! Sidebar set — per-host sidebar tab tables.
//! Blender ref: `VIEW3D_PT_*` / `<HOST>_PT_*` panels grouped by `bl_category`.
//!
//! Each tab carries a real icon slug (resolved to `assets/icons/<slug>.svg`)
//! plus its Blender category label. The Sidebar renders as a vertical strip
//! of tabs, same visual as the Buts navbar — icon-only with the label used
//! as a hover tooltip.

use crate::v2::host::BHostType;
use crate::v2::part::BPartType;
use crate::v2::part::props::set::{BSectionSpec, sec};
use crate::v2::spec::{BSpec, BSpecKind};

/// What a sidebar tab shows in the region beside the strip.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BSidebarBody {
    /// Nothing yet: the tab is on the strip only.
    None,
    /// Panels over the active object — shown while one is active, as
    /// Blender's `poll` on `context.object` has it.
    Object(&'static [BSectionSpec]),
    /// Panels that are always there.
    Props(&'static [BSectionSpec]),
    /// `VIEW3D_PT_view`: the area's own view — its focal length.
    View,
    /// `DOPESHEET_PT_action` / `_action_slot`: the active object's action.
    Action,
    /// `TIME_PT_playback` and `TIME_PT_playhead_snapping` — the transport's
    /// own panels. The same rows the header's popovers open
    /// (`playback::panel`), as folding cards: one set of rows, two places to
    /// reach them, so a change to the transport's settings cannot end up
    /// true in the popover and false in the sidebar.
    Playback,
    /// The host's own box panel, the one its header opens too
    /// (`filter::overlay`): `CLIP_PT_display` in the Movie Clip editor's View
    /// tab, `SPREADSHEET_PT_filter` in the Spreadsheet's Filter tab. One
    /// table, one card per sub-panel, so a box cannot end up ticked in the
    /// popover and clear in the sidebar.
    Overlay,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BSidebarTabSpec {
    pub icon: &'static str,
    pub label: &'static str,
    pub body: BSidebarBody,
}

impl BSidebarTabSpec {
    /// This tab with `body` in the region beside the strip.
    const fn with(mut self, body: BSidebarBody) -> Self {
        self.body = body;
        self
    }
}

impl BSpec for BSidebarTabSpec {
    fn kind(&self) -> BSpecKind {
        BSpecKind::Part(BPartType::Sidebar)
    }
    fn label(&self) -> &'static str {
        self.label
    }
    fn icon(&self) -> &'static str {
        self.icon
    }
}

const fn t(icon: &'static str, label: &'static str) -> BSidebarTabSpec {
    BSidebarTabSpec { icon, label, body: BSidebarBody::None }
}

/// `VIEW3D_PT_transform` — the Object tab's Transform, over the same keys,
/// so typing into either moves the object (`scene::transform`).
static VIEW3D_ITEM: &[BSectionSpec] = &[sec("Transform", crate::v2::part::props::object::TRANSFORM)];
/// `VIEW3D_PT_tools_active`'s header: the active tool.
static VIEW3D_TOOL: &[BSectionSpec] = &[sec("Active Tool", crate::v2::part::props::tool::ACTIVE_TOOL)];

static VIEW3D: &[BSidebarTabSpec] = &[
    t("object", "Item").with(BSidebarBody::Object(VIEW3D_ITEM)),
    t("tool", "Tool").with(BSidebarBody::Props(VIEW3D_TOOL)),
    t("view_layer", "View").with(BSidebarBody::View),
];
/// The Image and UV editors are one space in two modes, so they share one
/// tab set and one table of panels (`sidebar::image`, `space_image.py`).
static IMAGE_UV: &[BSidebarTabSpec] = &[
    t("image", "Image").with(BSidebarBody::Props(super::image::IMAGE)),
    t("tool", "Tool").with(BSidebarBody::Props(VIEW3D_TOOL)),
    t("view_layer", "View").with(BSidebarBody::Props(super::image::VIEW)),
    // Phiber's own: the render's settings beside the render (`image::RENDER`).
    t("render", "Render").with(BSidebarBody::Props(super::image::RENDER)),
];
/// The four node editors are one space with a different tree in it, and
/// Blender registers most of the N panel once for all of them
/// (`sidebar::node`, `space_node.py`). Their Node and Group tabs are the
/// same tables; the Options tab is the Compositor's own, and the Shader and
/// Node editors' Options and Geometry Nodes' Attributes are still empty —
/// each is a per-node-type `draw_buttons_ext`, not a table.
static SHADER: &[BSidebarTabSpec] = &[
    t("node", "Node").with(BSidebarBody::Props(super::node::NODE)),
    t("group", "Group").with(BSidebarBody::Props(super::node::GROUP)),
    t("options", "Options"),
];
static NODE: &[BSidebarTabSpec] = &[
    t("node", "Node").with(BSidebarBody::Props(super::node::NODE)),
    t("group", "Group").with(BSidebarBody::Props(super::node::GROUP)),
    t("options", "Options"),
];
static GEONODE: &[BSidebarTabSpec] = &[
    t("group", "Group").with(BSidebarBody::Props(super::node::GROUP)),
    t("data", "Attributes"),
    t("node", "Node").with(BSidebarBody::Props(super::node::NODE)),
];
static COMPOSITOR: &[BSidebarTabSpec] = &[
    t("node", "Node").with(BSidebarBody::Props(super::node::NODE)),
    t("options", "Options").with(BSidebarBody::Props(super::node::OPTIONS)),
];
static SEQ: &[BSidebarTabSpec] = &[
    t("strip", "Strip"),
    t("modifier", "Modifiers"),
    t("preview", "Preview"),
];
/// The Movie Clip editor. Its View tab is `CLIP_PT_display`, the same panel
/// its header opens — Marker Display and Clip Display, one card each.
static CLIP: &[BSidebarTabSpec] = &[
    t("track", "Track"),
    t("solve", "Solve"),
    t("view_layer", "View").with(BSidebarBody::Overlay),
];
/// `space_text.py` registers both of its panels under one `bl_category`,
/// "Text" — `TEXT_PT_find` then `TEXT_PT_properties`. This had a tab for
/// each, which is one more than Blender has and left nothing obvious to put
/// in either (`sidebar::text`).
static TEXT: &[BSidebarTabSpec] = &[t("text", "Text").with(BSidebarBody::Props(super::text::TEXT))];
// `DOPESHEET_PT_action` / `DOPESHEET_PT_action_slot`: one tab, Action; the
// View tab is Grease Pencil mode's alone.
static ACTION: &[BSidebarTabSpec] = &[t("action", "Action").with(BSidebarBody::Action)];
static NLA: &[BSidebarTabSpec] = &[t("action", "Edited Action"), t("object", "Item")];
static IPO: &[BSidebarTabSpec] = &[
    t("view_layer", "View"),
    t("curve", "F-Curve"),
    t("modifier", "Modifiers"),
    t("driver", "Drivers"),
];
/// The Spreadsheet's one tab is `SPREADSHEET_PT_filter`, which its header
/// carries too.
static SPREADSHEET: &[BSidebarTabSpec] = &[t("filter", "Filter").with(BSidebarBody::Overlay)];
/// The Timeline's sidebar. Its Center region had a Right pane of nothing in
/// it while every other timing editor beside it carried a strip, which is
/// the empty column down the right of the Timeline.
static TIME: &[BSidebarTabSpec] = &[t("play", "Playback").with(BSidebarBody::Playback)];

pub fn tabs_for(host: BHostType) -> &'static [BSidebarTabSpec] {
    match host {
        BHostType::View3D => VIEW3D,
        BHostType::Uv | BHostType::Image => IMAGE_UV,
        BHostType::Shader => SHADER,
        BHostType::Node => NODE,
        BHostType::GeoNode => GEONODE,
        BHostType::Compositor => COMPOSITOR,
        BHostType::Seq => SEQ,
        BHostType::Clip => CLIP,
        BHostType::Text => TEXT,
        BHostType::Action => ACTION,
        BHostType::Time => TIME,
        BHostType::Nla => NLA,
        BHostType::Ipo | BHostType::Drivers => IPO,
        BHostType::Spreadsheet => SPREADSHEET,
        _ => &[],
    }
}
