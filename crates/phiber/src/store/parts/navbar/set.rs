//! NavBar set - per-host breadcrumb trails (static data).

use crate::store::hosts::BHostType;

pub static BUTS_CRUMBS: &[&str] = &["Scene Collection", "Collection", "Cube"];

// Blender file browser path bar (directory breadcrumb).
pub static FILE_CRUMBS: &[&str] = &["C:", "Users", "phiac", "Workspace", "pic-Pods", "Female-02"];

pub static DEFAULT_CRUMBS: &[&str] = &[];

/// Breadcrumb trail for a given host.
pub fn with_idx(host: BHostType) -> &'static [&'static str] {
    match host {
        BHostType::Buts => BUTS_CRUMBS,
        BHostType::File => FILE_CRUMBS,
        _ => DEFAULT_CRUMBS,
    }
}
