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
static IMAGE_UV: &[BSidebarTabSpec] = &[
    t("image", "Image"),
    t("tool", "Tool"),
    t("view_layer", "View"),
];
static SHADER: &[BSidebarTabSpec] = &[
    t("node", "Node"),
    t("group", "Group"),
    t("options", "Options"),
];
static NODE: &[BSidebarTabSpec] = &[
    t("node", "Node"),
    t("group", "Group"),
    t("options", "Options"),
];
static GEONODE: &[BSidebarTabSpec] = &[
    t("group", "Group"),
    t("data", "Attributes"),
    t("node", "Node"),
];
static COMPOSITOR: &[BSidebarTabSpec] = &[t("node", "Node"), t("options", "Options")];
static SEQ: &[BSidebarTabSpec] = &[
    t("strip", "Strip"),
    t("modifier", "Modifiers"),
    t("preview", "Preview"),
];
static CLIP: &[BSidebarTabSpec] = &[
    t("track", "Track"),
    t("solve", "Solve"),
    t("view_layer", "View"),
];
static TEXT: &[BSidebarTabSpec] = &[t("text", "Text"), t("options", "Properties")];
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
static SPREADSHEET: &[BSidebarTabSpec] = &[t("filter", "Filter")];

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
        BHostType::Nla => NLA,
        BHostType::Ipo | BHostType::Drivers => IPO,
        BHostType::Spreadsheet => SPREADSHEET,
        _ => &[],
    }
}
