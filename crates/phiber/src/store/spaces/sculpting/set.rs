//! Sculpting space area assignments.

use crate::store::areas::{BAreaSpec, BAreaType};
use crate::store::hosts::{BHostType, SWITCHABLE_HOSTS};
use crate::store::zones::BZoneType;

pub static SCULPTING_AREAS: &[BAreaSpec] = &[
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
];

pub static ZONES: &[BZoneType] = &[BZoneType::Top, BZoneType::Center];
pub static COLUMNS: &[f32] = &[1.0];
