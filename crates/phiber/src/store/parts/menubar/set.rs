//! Menubar set - host-to-menubar association and menu label arrays.

use super::def;
use crate::store::hosts::BHostType;

pub static VIEW3D_MENUS: &[&str] = &[
    def::MENU_VIEW,
    def::MENU_SELECT,
    def::MENU_ADD,
    def::MENU_OBJECT,
    def::MENU_MESH,
    def::MENU_GREASE_PENCIL,
];
pub static OUTLINER_MENUS: &[&str] = &[
    def::MENU_VIEW,
    def::MENU_SELECT,
    def::MENU_OBJECT,
    def::MENU_COLLECTION,
];
pub static BUTS_MENUS: &[&str] = &[def::MENU_VIEW, def::MENU_OBJECT];
pub static TIME_MENUS: &[&str] = &[
    def::MENU_VIEW,
    def::MENU_SELECT,
    def::MENU_MARKER,
    def::MENU_CHANNEL,
    def::MENU_KEY,
];
pub static SEQ_MENUS: &[&str] = &[
    def::MENU_VIEW,
    def::MENU_SELECT,
    def::MENU_MARKER,
    def::MENU_CLIP,
    def::MENU_EFFECT,
];
pub static NODE_MENUS: &[&str] = &[
    def::MENU_VIEW,
    def::MENU_SELECT,
    def::MENU_ADD,
    def::MENU_NODE,
];
pub static IMAGE_MENUS: &[&str] = &[
    def::MENU_VIEW,
    def::MENU_SELECT,
    def::MENU_IMAGE,
    def::MENU_UV,
];
pub static TEXT_MENUS: &[&str] = &[
    def::MENU_FILE,
    def::MENU_EDIT,
    def::MENU_FORMAT,
    def::MENU_TEXT,
    def::MENU_TOOLS,
];
pub static COMPOSITOR_MENUS: &[&str] = &[
    def::MENU_VIEW,
    def::MENU_SELECT,
    def::MENU_ADD,
    def::MENU_NODE,
];
pub static CLIP_MENUS: &[&str] = &[
    def::MENU_VIEW,
    def::MENU_SELECT,
    def::MENU_CLIP,
    def::MENU_TRACK,
    def::MENU_SOLVE,
];
pub static TOPBAR_MENUS: &[&str] = &[
    def::MENU_FILE,
    def::MENU_EDIT,
    def::MENU_RENDER,
    def::MENU_WINDOW,
    def::MENU_HELP,
];
pub static INFO_MENUS: &[&str] = &[def::MENU_INFO];
pub static DEFAULT_MENUS: &[&str] = &[def::MENU_VIEW];

/// Menu labels for a given host.
pub fn with_idx(host: BHostType) -> &'static [&'static str] {
    match host {
        BHostType::View3D => VIEW3D_MENUS,
        BHostType::Outliner => OUTLINER_MENUS,
        BHostType::Buts => BUTS_MENUS,
        BHostType::Time => TIME_MENUS,
        BHostType::Seq => SEQ_MENUS,
        BHostType::Node | BHostType::Compositor | BHostType::Shader | BHostType::GeoNode => {
            NODE_MENUS
        }
        BHostType::Image | BHostType::Uv => IMAGE_MENUS,
        BHostType::Text => TEXT_MENUS,
        BHostType::Clip => CLIP_MENUS,
        BHostType::Topbar => TOPBAR_MENUS,
        BHostType::Info => INFO_MENUS,
        _ => DEFAULT_MENUS,
    }
}
