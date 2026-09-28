//! Geometry Nodes workspace — its areas, and which hosts each one offers.
//!
//! Geometry Nodes workspace — View3D (top) + GeoNode/Spreadsheet (bottom).
//! Blender ref: startup.blend default `Geometry Nodes` workspace.

use crate::v2::area::{BAreaSpec, BAreaType};
use crate::v2::host::BHostType;
use crate::v2::zone::BZoneType;

pub const LABEL: &str = "Geometry Nodes";
pub const ICON: &str = "cpu";

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
    // Blender: the Spreadsheet beside the viewport (654 of 1920 px).
    BAreaSpec {
        kind: BAreaType::Left,
        zone: BZoneType::Left,
        default_host: BHostType::Spreadsheet,
        hosts: &[BHostType::Spreadsheet, BHostType::Outliner],
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
    // The node editor spans the width under both, 477 of 1054 px …
    BAreaSpec {
        kind: BAreaType::Bottom,
        zone: BZoneType::Bottom,
        default_host: BHostType::GeoNode,
        hosts: BHostType::SWITCHABLE,
    },
    // … and the Timeline strip runs under it.
    BAreaSpec {
        kind: BAreaType::Bottom,
        zone: BZoneType::Bottom,
        default_host: BHostType::Time,
        hosts: &[
            BHostType::Time,
            BHostType::Action,
            BHostType::Ipo,
            BHostType::Drivers,
            BHostType::Nla,
            BHostType::Seq,
        ],
    },
    BAreaSpec {
        kind: BAreaType::Bottom,
        zone: BZoneType::Bottom,
        default_host: BHostType::Statusbar,
        hosts: &[BHostType::Statusbar],
    },
];
