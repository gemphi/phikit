//! Modeling space area assignments — matches Blender's Modeling workspace:
//! 3D Viewport (center), Properties (right), Timeline (bottom).

use crate::store::areas::{BAreaSpec, BAreaType};
use crate::store::hosts::{BHostType, SWITCHABLE_HOSTS};
use crate::store::zones::BZoneType;

pub static MODELING_AREAS: &[BAreaSpec] = &[
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
        default_host: BHostType::Buts,
        items: &[BHostType::Buts, BHostType::Outliner, BHostType::Asset],
    },
    BAreaSpec {
        idx: BAreaType::Bottom,
        zone: BZoneType::Bottom,
        default_host: BHostType::Time,
        items: &[BHostType::Time, BHostType::Action, BHostType::Nla],
    },
];

pub static ZONES: &[BZoneType] = &[
    BZoneType::Top,
    BZoneType::Center,
    BZoneType::Right,
    BZoneType::Bottom,
];
pub static COLUMNS: &[f32] = &[0.72, 0.28];
