//! Per-host header menus - each host type gets its own set of header menu items.
//!
//! def.rs  - constants (labels, icons)
//! set.rs  - static `MenuItem` arrays per host
//! mod.rs  - host -> header menu dispatch

pub mod def;
pub mod set;

use super::MenuItem;
use crate::store::hosts::BHostType;

/// Get the header menu items for a given host type.
pub fn items_for(host: BHostType) -> &'static [MenuItem] {
    match host {
        BHostType::View3D => set::VIEW3D_ITEMS,
        BHostType::Clip => set::CLIP_ITEMS,
        BHostType::Seq => set::SEQ_ITEMS,
        BHostType::Outliner => set::OUTLINER_ITEMS,
        BHostType::Buts => set::BUTS_ITEMS,
        BHostType::Time
        | BHostType::Action
        | BHostType::Nla
        | BHostType::Ipo
        | BHostType::Drivers => set::ANIM_ITEMS,
        BHostType::Node | BHostType::Compositor | BHostType::Shader | BHostType::GeoNode => {
            set::NODE_ITEMS
        }
        BHostType::Image | BHostType::Uv => set::IMAGE_ITEMS,
        BHostType::Text | BHostType::Console | BHostType::Info => set::TEXT_ITEMS,
        BHostType::File | BHostType::Asset => set::FILE_ITEMS,
        _ => set::DEFAULT_ITEMS,
    }
}

/// Level-generic alias for items_for.
pub fn for_items(host: BHostType) -> &'static [MenuItem] {
    items_for(host)
}
