//! Layout workspace — its areas, and which hosts each one offers.
//!
//! Layout space content — area declarations. Blender 5.2 reference: the
//! default Layout workspace (Topbar + View3D + Outliner/Properties column +
//! Timeline strip + global Statusbar strip).

use crate::v2::area::{BAreaSpec, BAreaType};
use crate::v2::host::BHostType;
use crate::v2::zone::BZoneType;

pub const LABEL: &str = "Layout";
pub const ICON: &str = "entity";

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
        hosts: &[BHostType::Outliner, BHostType::File, BHostType::Asset],
    },
    BAreaSpec {
        kind: BAreaType::Right,
        zone: BZoneType::Right,
        default_host: BHostType::Buts,
        hosts: &[BHostType::Buts, BHostType::Spreadsheet, BHostType::Asset],
    },
    BAreaSpec {
        kind: BAreaType::Bottom,
        zone: BZoneType::Bottom,
        default_host: BHostType::Time,
        hosts: &[
            BHostType::Time,
            BHostType::Play,
            BHostType::Project,
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
