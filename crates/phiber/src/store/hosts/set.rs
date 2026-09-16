//! Host set - static arrays and host spec builder.

use std::sync::OnceLock;

use super::BHostSpec;
use super::def::*;
use crate::store::panes::set::panes_for;
use crate::store::regions::BRegionSpec;

/// Base region set per host type — pure data match.
pub fn base_regions(host: BHostType) -> &'static [BRegionSpec] {
    match host {
        BHostType::Empty => super::empty::set::REGIONS,
        BHostType::View3D => super::view3d::set::REGIONS,
        BHostType::Outliner => super::outliner::set::REGIONS,
        BHostType::Buts => super::properties::set::REGIONS,
        BHostType::File => super::file_browser::set::REGIONS,
        BHostType::Image => super::image_editor::set::REGIONS,
        BHostType::Info => super::info::set::REGIONS,
        BHostType::Seq => super::sequencer::set::REGIONS,
        BHostType::Text => super::text_editor::set::REGIONS,
        BHostType::Action => super::dope_sheet::set::REGIONS,
        BHostType::Nla => super::nla::set::REGIONS,
        BHostType::Time => super::timeline::set::REGIONS,
        BHostType::Node => super::node_editor::set::REGIONS,
        BHostType::Logic => super::logic_editor::set::REGIONS,
        BHostType::Console => super::console::set::REGIONS,
        BHostType::Uv => super::uv_editor::set::REGIONS,
        BHostType::Compositor => super::compositor::set::REGIONS,
        BHostType::Shader => super::shader_editor::set::REGIONS,
        BHostType::GeoNode => super::geo_nodes::set::REGIONS,
        BHostType::Clip => super::clip_editor::set::REGIONS,
        BHostType::Drivers => super::drivers::set::REGIONS,
        BHostType::Asset => super::asset_browser::set::REGIONS,
        BHostType::Spreadsheet => super::spreadsheet::set::REGIONS,
        BHostType::Ipo => super::graph_editor::set::REGIONS,
        BHostType::Prefs => super::prefs::set::REGIONS,
        BHostType::Topbar => super::topbar::set::REGIONS,
        BHostType::Statusbar => super::statusbar::set::REGIONS,
    }
}

/// Build the full spec for a host (regions populated with panes/parts).
fn build_spec(host: BHostType) -> BHostSpec {
    let regions: Vec<BRegionSpec> = base_regions(host)
        .iter()
        .map(|r| BRegionSpec::with_items(r.idx, panes_for(host, r.idx)))
        .collect();
    let regions: &'static [BRegionSpec] = Box::leak(regions.into_boxed_slice());
    BHostSpec::new(host, regions)
}

/// Build all host specs once, store as a leaked slice.
fn all_specs() -> &'static [BHostSpec] {
    static SPECS: OnceLock<&'static [BHostSpec]> = OnceLock::new();
    *SPECS.get_or_init(|| {
        let v: Vec<BHostSpec> = ALL_HOSTS.iter().map(|&h| build_spec(h)).collect();
        Box::leak(v.into_boxed_slice())
    })
}

/// Look up a host spec by type — simple array index.
pub fn host_spec(host: BHostType) -> &'static BHostSpec {
    &all_specs()[host as usize]
}

pub fn host_label(host: BHostType) -> &'static str {
    LABELS[host as usize]
}

pub fn host_icon(host: BHostType) -> &'static str {
    ICONS[host as usize]
}

pub static ALL_HOSTS: &[BHostType] = &[
    BHostType::Empty,
    BHostType::View3D,
    BHostType::Outliner,
    BHostType::Buts,
    BHostType::File,
    BHostType::Image,
    BHostType::Info,
    BHostType::Seq,
    BHostType::Text,
    BHostType::Action,
    BHostType::Nla,
    BHostType::Time,
    BHostType::Node,
    BHostType::Logic,
    BHostType::Console,
    BHostType::Uv,
    BHostType::Compositor,
    BHostType::Shader,
    BHostType::GeoNode,
    BHostType::Clip,
    BHostType::Drivers,
    BHostType::Asset,
    BHostType::Spreadsheet,
    BHostType::Ipo,
    BHostType::Prefs,
    BHostType::Topbar,
    BHostType::Statusbar,
];
pub static SWITCHABLE_HOSTS: &[BHostType] = &[
    BHostType::View3D,
    BHostType::Outliner,
    BHostType::Buts,
    BHostType::File,
    BHostType::Image,
    BHostType::Info,
    BHostType::Seq,
    BHostType::Text,
    BHostType::Action,
    BHostType::Nla,
    BHostType::Time,
    BHostType::Node,
    BHostType::Logic,
    BHostType::Console,
    BHostType::Uv,
    BHostType::Compositor,
    BHostType::Shader,
    BHostType::GeoNode,
    BHostType::Clip,
    BHostType::Drivers,
    BHostType::Asset,
    BHostType::Spreadsheet,
    BHostType::Ipo,
    BHostType::Prefs,
];
pub static LABELS: &[&str] = &[
    EMPTY_LABEL,
    VIEW3D_LABEL,
    OUTLINER_LABEL,
    BUTS_LABEL,
    FILE_LABEL,
    IMAGE_LABEL,
    INFO_LABEL,
    SEQ_LABEL,
    TEXT_LABEL,
    ACTION_LABEL,
    NLA_LABEL,
    TIME_LABEL,
    NODE_LABEL,
    LOGIC_LABEL,
    CONSOLE_LABEL,
    UV_LABEL,
    COMPOSITOR_LABEL,
    SHADER_LABEL,
    GEONODE_LABEL,
    CLIP_LABEL,
    DRIVERS_LABEL,
    ASSET_LABEL,
    SPREADSHEET_LABEL,
    IPO_LABEL,
    PREFS_LABEL,
    TOPBAR_LABEL,
    STATUSBAR_LABEL,
];
pub static ICONS: &[&str] = &[
    EMPTY_ICON,
    VIEW3D_ICON,
    OUTLINER_ICON,
    BUTS_ICON,
    FILE_ICON,
    IMAGE_ICON,
    INFO_ICON,
    SEQ_ICON,
    TEXT_ICON,
    ACTION_ICON,
    NLA_ICON,
    TIME_ICON,
    NODE_ICON,
    LOGIC_ICON,
    CONSOLE_ICON,
    UV_ICON,
    COMPOSITOR_ICON,
    SHADER_ICON,
    GEONODE_ICON,
    CLIP_ICON,
    DRIVERS_ICON,
    ASSET_ICON,
    SPREADSHEET_ICON,
    IPO_ICON,
    PREFS_ICON,
    TOPBAR_ICON,
    STATUSBAR_ICON,
];
