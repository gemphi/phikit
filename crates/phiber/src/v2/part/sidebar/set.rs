//! Sidebar set — per-host sidebar tab tables.
//! Blender ref: `VIEW3D_PT_*` / `<HOST>_PT_*` panels grouped by `bl_category`.
//!
//! Each tab carries a real icon slug (resolved to `assets/icons/<slug>.svg`)
//! plus its Blender category label. The Sidebar renders as a vertical strip
//! of tabs, same visual as the Buts navbar — icon-only with the label used
//! as a hover tooltip.

use crate::v2::host::BHostType;
use crate::v2::part::BPartType;
use crate::v2::spec::{BSpec, BSpecKind};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BSidebarTabSpec {
    pub icon: &'static str,
    pub label: &'static str,
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
    BSidebarTabSpec { icon, label }
}

static VIEW3D: &[BSidebarTabSpec] = &[
    t("object", "Item"),
    t("tool", "Tool"),
    t("view_layer", "View"),
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
static ACTION: &[BSidebarTabSpec] = &[t("view_layer", "View"), t("options", "Options")];
static NLA: &[BSidebarTabSpec] = &[t("action", "Edited Action"), t("object", "Item")];
static IPO: &[BSidebarTabSpec] = &[
    t("view_layer", "View"),
    t("curve", "F-Curve"),
    t("modifier", "Modifiers"),
    t("driver", "Drivers"),
];
static SPREADSHEET: &[BSidebarTabSpec] = &[t("filter", "Filter")];
static FILE: &[BSidebarTabSpec] = &[t("bookmark", "Bookmarks")];

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
        BHostType::File | BHostType::Asset => FILE,
        _ => &[],
    }
}
