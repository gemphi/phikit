//! Animation workspace — View3D center, Dope Sheet bottom, Outliner+Buts right.
//! Blender ref: startup.blend default `Animation` workspace.

use crate::v2::area::{BAreaSpec, BAreaType};
use crate::v2::host::BHostType;
use crate::v2::zone::BZoneType;

pub const LABEL: &str = "Animation";
pub const ICON: &str = "film";

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
        default_host: BHostType::Outliner,
        hosts: &[BHostType::Outliner],
    },
    BAreaSpec {
        kind: BAreaType::Right,
        zone: BZoneType::Right,
        default_host: BHostType::Buts,
        hosts: &[BHostType::Buts],
    },
    BAreaSpec {
        kind: BAreaType::Bottom,
        zone: BZoneType::Bottom,
        default_host: BHostType::Action,
        hosts: &[
            BHostType::Action,
            BHostType::Time,
            BHostType::Nla,
            BHostType::Ipo,
        ],
    },
    BAreaSpec {
        kind: BAreaType::Bottom,
        zone: BZoneType::Bottom,
        default_host: BHostType::Statusbar,
        hosts: &[BHostType::Statusbar],
    },
];
