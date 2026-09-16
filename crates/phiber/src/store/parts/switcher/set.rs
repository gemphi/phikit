//! Switcher set — host-to-switcher association, modes query, and groups.
//! Follows the standard phiber store pattern: `with_idx(host)`.

use super::def;
use super::{ModeEntry, SwitcherGroup};
use crate::store::hosts::BHostType;

pub static DEFAULT_SWITCHABLE: &[BHostType] = def::SWITCHABLE_HOSTS;

/// Mode capability entries for a given host (standard with_idx pattern).
pub fn with_idx(host: BHostType) -> &'static [ModeEntry] {
    match host {
        BHostType::View3D => def::VIEW3D_MODES,
        BHostType::Buts => def::BUTS_MODES,
        BHostType::Outliner => def::OUTLINER_MODES,
        BHostType::Image => def::IMAGE_MODES,
        BHostType::Uv => def::UV_MODES,
        BHostType::Compositor => def::COMPOSITOR_MODES,
        BHostType::GeoNode => def::GEONODE_MODES,
        BHostType::Shader => def::SHADER_MODES,
        BHostType::Seq => def::SEQ_MODES,
        BHostType::Clip => def::CLIP_MODES,
        BHostType::Action => def::ACTION_MODES,
        BHostType::Drivers => def::DRIVERS_MODES,
        BHostType::Nla => def::NLA_MODES,
        BHostType::Text => def::TEXT_MODES,
        BHostType::Console => def::CONSOLE_MODES,
        BHostType::Info => def::INFO_MODES,
        BHostType::Prefs => def::PREFS_MODES,
        BHostType::File => def::FILE_MODES,
        BHostType::Asset => def::ASSET_MODES,
        BHostType::Spreadsheet => def::SPREADSHEET_MODES,
        _ => def::DEFAULT_MODES,
    }
}

/// Category groupings for all switchable hosts.
pub fn host_groups() -> &'static [SwitcherGroup] {
    def::HOST_GROUPS
}

/// Default list of all switchable hosts.
pub fn default_switchable() -> &'static [BHostType] {
    DEFAULT_SWITCHABLE
}
