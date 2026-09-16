//! Layout space definitions - area assignments and grid fractions.
//! Matches Blender's default Layout workspace:
//! 3D Viewport (center), Outliner + Properties (right, stacked), Timeline (bottom).

use crate::store::areas::{BAreaSpec, BAreaType};
use crate::store::hosts::{BHostType, SWITCHABLE_HOSTS};
use crate::store::zones::BZoneType;

pub static LAYOUT_AREAS: &[BAreaSpec] = &[
    BAreaSpec {
        idx: BAreaType::Top,
        zone: BZoneType::Top,
        default_host: BHostType::Topbar,
        items: &[BHostType::Topbar],
    },
    BAreaSpec {
        idx: BAreaType::Center,
        zone: BZoneType::Center,
        default_host: BHostType::View3D,
        items: SWITCHABLE_HOSTS,
    },
    BAreaSpec {
        idx: BAreaType::Right,
        zone: BZoneType::Right,
        default_host: BHostType::Outliner,
        items: &[BHostType::Outliner, BHostType::File, BHostType::Asset],
    },
    BAreaSpec {
        idx: BAreaType::Right,
        zone: BZoneType::Right,
        default_host: BHostType::Buts,
        items: &[BHostType::Buts, BHostType::Spreadsheet, BHostType::Asset],
    },
    BAreaSpec {
        idx: BAreaType::Bottom,
        zone: BZoneType::Bottom,
        default_host: BHostType::Time,
        items: &[
            BHostType::Time,
            BHostType::Action,
            BHostType::Nla,
            BHostType::Seq,
        ],
    },
    // Global statusbar strip (Blender: screen-level area below all regions).
    BAreaSpec {
        idx: BAreaType::Bottom,
        zone: BZoneType::Bottom,
        default_host: BHostType::Statusbar,
        items: &[BHostType::Statusbar],
    },
];

/// Zones for the Layout space.
pub static ZONES: &[BZoneType] = &[
    BZoneType::Top,
    BZoneType::Center,
    BZoneType::Right,
    BZoneType::Bottom,
];

/// Column fractions for the Layout space.
pub static COLUMNS: &[f32] = &[0.72, 0.28];
