//! Animation workspace — its areas, and which hosts each one offers.
//!
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
    // The activity rail: the same fixed strip down the far left of every space.
    BAreaSpec {
        kind: BAreaType::Left,
        zone: BZoneType::Rail,
        default_host: BHostType::Activity,
        hosts: &[BHostType::Activity],
    },
    // Blender opens this viewport in camera view (`startup.blend`: 559 of 1920 px,
    // `persp = CAMERA`); `view3d.md` row 13 tracks the camera view itself.
    BAreaSpec {
        kind: BAreaType::Left,
        zone: BZoneType::Left,
        default_host: BHostType::View3D,
        hosts: BHostType::SWITCHABLE,
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
    // The Dope Sheet takes 433 of 1054 px in Blender's Animation workspace.
    BAreaSpec {
        kind: BAreaType::Bottom,
        zone: BZoneType::Bottom,
        default_host: BHostType::Action,
        hosts: &[
            BHostType::Action,
            BHostType::Time,
            BHostType::Play,
            BHostType::Project,
            BHostType::Ipo,
            BHostType::Drivers,
            BHostType::Nla,
        ],
    },
    BAreaSpec {
        kind: BAreaType::Bottom,
        zone: BZoneType::Bottom,
        default_host: BHostType::Statusbar,
        hosts: &[BHostType::Statusbar],
    },
];
