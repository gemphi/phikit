//! Which host gets which menus.
//!
//! The index, not the content: the menus themselves are in `topbar/set.rs`
//! and `editor/set.rs`, and what a menu *is* is in `item/mod.rs`.

use super::editor::set::*;
use super::item::BMenuSpec;
use super::topbar::set::TOPBAR;
use crate::v2::host::BHostType;

impl BMenuSpec {
    /// The top-level menus this host shows in its header. A host with none
    /// of its own gets none, rather than inheriting the top bar's.
    pub fn list_for(host: BHostType) -> &'static [BMenuSpec] {
        match host {
            BHostType::Topbar => TOPBAR,
            BHostType::View3D => VIEW3D,
            BHostType::Outliner => OUTLINER,
            BHostType::Buts => BUTS,
            BHostType::Time => TIME,
            BHostType::File => FILE_HOST,
            BHostType::Image => IMAGE,
            BHostType::Info => INFO,
            BHostType::Seq => SEQ,
            BHostType::Text => TEXT,
            BHostType::Action => ACTION,
            BHostType::Nla => NLA,
            BHostType::Node => NODE,
            BHostType::Console => CONSOLE,
            BHostType::Uv => UV,
            BHostType::Compositor => COMPOSITOR,
            BHostType::Shader => SHADER,
            BHostType::GeoNode => GEONODE,
            BHostType::Clip => CLIP,
            BHostType::Drivers => DRIVERS,
            BHostType::Asset => ASSET,
            BHostType::Spreadsheet => SPREADSHEET,
            BHostType::Ipo => IPO,
            BHostType::Prefs => PREFS,
            _ => &[],
        }
    }
}
