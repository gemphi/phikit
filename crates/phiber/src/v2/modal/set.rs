//! Modal set — one `BModalSpec` per `BModalType`.

use super::{BModalSpec, BModalType, BModalShape};
use crate::v2::area::{BAreaSpec, BAreaType};
use crate::v2::host::BHostType;
use crate::v2::zone::BZoneType;

/// One editor filling the dialog.
const fn editor(default_host: BHostType, hosts: &'static [BHostType]) -> BAreaSpec {
    BAreaSpec { kind: BAreaType::Center, zone: BZoneType::Center, default_host, hosts }
}

static FILE_AREAS: &[BAreaSpec] = &[editor(BHostType::File, &[BHostType::File, BHostType::Asset])];
static PREFS_AREAS: &[BAreaSpec] = &[editor(BHostType::Prefs, &[BHostType::Prefs])];
static RENDER_AREAS: &[BAreaSpec] = &[editor(BHostType::Image, &[BHostType::Image])];


// Order MUST match `BModalType` variant order.
pub static SET: &[BModalSpec] = &[
    BModalSpec {
        kind: BModalType::Settings,
        label: "Preferences",
        icon: "settings",
        width: 960.0,
        height: 640.0,
        shape: BModalShape::Card,
        areas: PREFS_AREAS,
    },
    BModalSpec {
        kind: BModalType::Help,
        label: "Help",
        icon: "help",
        width: 900.0,
        height: 640.0,
        shape: BModalShape::Card,
        areas: &[],
    },
    BModalSpec {
        kind: BModalType::Profile,
        label: "Profile",
        icon: "profile",
        width: 480.0,
        height: 520.0,
        shape: BModalShape::Card,
        areas: &[],
    },
    BModalSpec {
        kind: BModalType::Command,
        label: "Search",
        icon: "search",
        width: 0.0,
        height: 0.0,
        shape: BModalShape::Palette,
        areas: &[],
    },
    BModalSpec {
        kind: BModalType::Splash,
        label: "Welcome to Phiber",
        icon: "phiber",
        width: 880.0,
        height: 560.0,
        shape: BModalShape::Fullscreen,
        areas: &[],
    },
    BModalSpec {
        kind: BModalType::About,
        label: "About Phiber",
        icon: "about",
        width: 640.0,
        height: 480.0,
        shape: BModalShape::Fullscreen,
        areas: &[],
    },
    BModalSpec {
        kind: BModalType::FileBrowser,
        label: "Phiber File View",
        icon: "folder",
        width: 1024.0,
        height: 640.0,
        shape: BModalShape::Card,
        areas: FILE_AREAS,
    },
    // Blender titles this window "Blender Render".
    BModalSpec {
        kind: BModalType::Render,
        label: "Phiber Render",
        icon: "render",
        width: 1100.0,
        height: 720.0,
        shape: BModalShape::Card,
        areas: RENDER_AREAS,
    },
    BModalSpec {
        kind: BModalType::Spaces,
        label: "Spaces",
        icon: "grid",
        width: 560.0,
        height: 420.0,
        shape: BModalShape::Card,
        areas: &[],
    },
    BModalSpec {
        kind: BModalType::Collab,
        label: "Collaboration",
        icon: "chat",
        width: 640.0,
        height: 520.0,
        shape: BModalShape::Card,
        areas: &[],
    },
    // A page to read: half the window wide, the whole height, so a manual
    // page or the release notes read at a column width instead of a
    // browser's.
    BModalSpec {
        kind: BModalType::Reader,
        label: "Reader",
        icon: "manual",
        width: 0.0,
        height: 0.0,
        shape: BModalShape::Sheet { share: 1.0 / 2.0 },
        areas: &[],
    },
    // `TOPBAR_PT_name`: a popup the width of a name, one field tall.
    BModalSpec {
        kind: BModalType::Rename,
        label: super::rename::TITLE,
        icon: "edit",
        width: 360.0,
        height: 150.0,
        shape: BModalShape::Card,
        areas: &[],
    },
    // `block_create__close_file_dialog`: `uiItemsAlertBox` 34 units wide.
    BModalSpec {
        kind: BModalType::FileClose,
        label: super::file_close::TITLE,
        icon: "alert_triangle",
        width: 440.0,
        height: 150.0,
        shape: BModalShape::Card,
        areas: &[],
    },
    // `WM_operator_confirm_ex`: the same alert card.
    BModalSpec {
        kind: BModalType::Revert,
        label: super::revert::TITLE,
        icon: "alert_triangle",
        width: 440.0,
        height: 150.0,
        shape: BModalShape::Card,
        areas: &[],
    },
    // `image_new_draw`: a property dialog, seven rows.
    BModalSpec {
        kind: BModalType::NewImage,
        label: super::new_image::TITLE,
        icon: "image",
        width: 380.0,
        height: 340.0,
        shape: BModalShape::Card,
        areas: &[],
    },
];
