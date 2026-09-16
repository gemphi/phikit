//! Animation space area assignments — matches Blender's Animation workspace:
//! 3D Viewport (center), Dope Sheet (right), Timeline (bottom).

use crate::store::areas::{BAreaSpec, BAreaType};
use crate::store::hosts::{BHostType, SWITCHABLE_HOSTS};
use crate::store::zones::BZoneType;

pub static ANIMATION_AREAS: &[BAreaSpec] = &[
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
        default_host: BHostType::Action,
        items: &[BHostType::Action, BHostType::Ipo, BHostType::Nla],
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
