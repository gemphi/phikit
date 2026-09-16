//! Host set — resolves `BHostType` → `&'static BHostSpec`. Regions come from
//! each per-host module's `REGIONS` static.

use super::{BHostSpec, BHostType};
use super::{
    action, activity, asset, buts, clip, compositor, console, drivers, file, geonode, image, info,
    ipo, logic, nla, node, outliner, prefs, seq, shader, spreadsheet, statusbar, text, time,
    topbar, uv, view3d,
};
use crate::v2::region::BRegionSpec;

static EMPTY_REGIONS: &[BRegionSpec] = &[];

pub fn spec(host: BHostType) -> &'static BHostSpec {
    &SET[host as usize]
}

// Order MUST match `BHostType` variant order.
pub static SET: &[BHostSpec] = &[
    BHostSpec {
        kind: BHostType::Topbar,
        label: "Top Bar",
        icon: "topbar",
        regions: topbar::REGIONS,
    },
    BHostSpec {
        kind: BHostType::View3D,
        label: "3D Viewport",
        icon: "view3d",
        regions: view3d::REGIONS,
    },
    BHostSpec {
        kind: BHostType::Outliner,
        label: "Outliner",
        icon: "outliner",
        regions: outliner::REGIONS,
    },
    BHostSpec {
        kind: BHostType::Buts,
        label: "Properties",
        icon: "properties",
        regions: buts::REGIONS,
    },
    BHostSpec {
        kind: BHostType::Time,
        label: "Timeline",
        icon: "time",
        regions: time::REGIONS,
    },
    BHostSpec {
        kind: BHostType::Statusbar,
        label: "Status Bar",
        icon: "statusbar",
        regions: statusbar::REGIONS,
    },
    BHostSpec {
        kind: BHostType::File,
        label: "File Browser",
        icon: "file",
        regions: file::REGIONS,
    },
    BHostSpec {
        kind: BHostType::Image,
        label: "Image Editor",
        icon: "image",
        regions: image::REGIONS,
    },
    BHostSpec {
        kind: BHostType::Info,
        label: "Info",
        icon: "info",
        regions: info::REGIONS,
    },
    BHostSpec {
        kind: BHostType::Seq,
        label: "Sequencer",
        icon: "seq",
        regions: seq::REGIONS,
    },
    BHostSpec {
        kind: BHostType::Text,
        label: "Text Editor",
        icon: "text",
        regions: text::REGIONS,
    },
    BHostSpec {
        kind: BHostType::Action,
        label: "Dope Sheet",
        icon: "action",
        regions: action::REGIONS,
    },
    BHostSpec {
        kind: BHostType::Nla,
        label: "NLA",
        icon: "nla",
        regions: nla::REGIONS,
    },
    BHostSpec {
        kind: BHostType::Node,
        label: "Node Editor",
        icon: "node",
        regions: node::REGIONS,
    },
    BHostSpec {
        kind: BHostType::Logic,
        label: "Logic Editor",
        icon: "logic",
        regions: logic::REGIONS,
    },
    BHostSpec {
        kind: BHostType::Console,
        label: "Python Console",
        icon: "console",
        regions: console::REGIONS,
    },
    BHostSpec {
        kind: BHostType::Uv,
        label: "UV Editor",
        icon: "uv",
        regions: uv::REGIONS,
    },
    BHostSpec {
        kind: BHostType::Compositor,
        label: "Compositor",
        icon: "compositor",
        regions: compositor::REGIONS,
    },
    BHostSpec {
        kind: BHostType::Shader,
        label: "Shader Editor",
        icon: "shader",
        regions: shader::REGIONS,
    },
    BHostSpec {
        kind: BHostType::GeoNode,
        label: "Geometry Nodes",
        icon: "geonode",
        regions: geonode::REGIONS,
    },
    BHostSpec {
        kind: BHostType::Clip,
        label: "Clip Editor",
        icon: "clip",
        regions: clip::REGIONS,
    },
    BHostSpec {
        kind: BHostType::Drivers,
        label: "Drivers",
        icon: "drivers",
        regions: drivers::REGIONS,
    },
    BHostSpec {
        kind: BHostType::Asset,
        label: "Asset Browser",
        icon: "asset",
        regions: asset::REGIONS,
    },
    BHostSpec {
        kind: BHostType::Spreadsheet,
        label: "Spreadsheet",
        icon: "spreadsheet",
        regions: spreadsheet::REGIONS,
    },
    BHostSpec {
        kind: BHostType::Ipo,
        label: "Graph Editor",
        icon: "ipo",
        regions: ipo::REGIONS,
    },
    BHostSpec {
        kind: BHostType::Prefs,
        label: "Preferences",
        icon: "prefs",
        regions: prefs::REGIONS,
    },
    BHostSpec {
        kind: BHostType::Activity,
        label: "Activity",
        icon: "activity",
        regions: activity::REGIONS,
    },
    BHostSpec {
        kind: BHostType::Empty,
        label: "Empty",
        icon: "",
        regions: EMPTY_REGIONS,
    },
];

/// Slug used inside `BPath` (state key). Data-like enum → string mapping —
/// canonical source, referenced by `type.rs::host_slug` and `state::path`.
pub fn slug(h: BHostType) -> &'static str {
    match h {
        BHostType::Topbar => "topbar",
        BHostType::View3D => "view3d",
        BHostType::Outliner => "outliner",
        BHostType::Buts => "buts",
        BHostType::Time => "time",
        BHostType::Statusbar => "statusbar",
        BHostType::File => "file",
        BHostType::Image => "image",
        BHostType::Info => "info",
        BHostType::Seq => "seq",
        BHostType::Text => "text",
        BHostType::Action => "action",
        BHostType::Nla => "nla",
        BHostType::Node => "node",
        BHostType::Logic => "logic",
        BHostType::Console => "console",
        BHostType::Uv => "uv",
        BHostType::Compositor => "compositor",
        BHostType::Shader => "shader",
        BHostType::GeoNode => "geonode",
        BHostType::Clip => "clip",
        BHostType::Drivers => "drivers",
        BHostType::Asset => "asset",
        BHostType::Spreadsheet => "spreadsheet",
        BHostType::Ipo => "ipo",
        BHostType::Prefs => "prefs",
        BHostType::Activity => "activity",
        BHostType::Empty => "empty",
    }
}
