//! Rendering workspace — View3D (left) + Image Editor (right).
//! Blender ref: startup.blend default `Rendering` workspace.

use crate::v2::area::{BAreaSpec, BAreaType};
use crate::v2::host::BHostType;
use crate::v2::zone::BZoneType;

pub const LABEL: &str = "Rendering";
pub const ICON: &str = "camera";

pub static AREAS: &[BAreaSpec] = &[
    BAreaSpec {
        kind: BAreaType::Top,
        zone: BZoneType::Top,
        default_host: BHostType::Topbar,
        hosts: &[BHostType::Topbar],
    },
    BAreaSpec {
        kind: BAreaType::Center,
        zone: BZoneType::Center,
        default_host: BHostType::View3D,
        hosts: BHostType::SWITCHABLE,
    },
    BAreaSpec {
        kind: BAreaType::Right,
        zone: BZoneType::Right,
        default_host: BHostType::Image,
        hosts: &[BHostType::Image, BHostType::Uv],
    },
    BAreaSpec {
        kind: BAreaType::Bottom,
        zone: BZoneType::Bottom,
        default_host: BHostType::Statusbar,
        hosts: &[BHostType::Statusbar],
    },
];
