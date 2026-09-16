//! Modeling workspace — View3D fills the whole area.
//! Blender ref: startup.blend default `Modeling` workspace.

use crate::v2::area::{BAreaSpec, BAreaType};
use crate::v2::host::BHostType;
use crate::v2::zone::BZoneType;

pub const LABEL: &str = "Modeling";
pub const ICON: &str = "box";

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
        kind: BAreaType::Bottom,
        zone: BZoneType::Bottom,
        default_host: BHostType::Statusbar,
        hosts: &[BHostType::Statusbar],
    },
];
