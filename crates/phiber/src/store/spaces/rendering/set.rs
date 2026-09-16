//! Rendering space area assignments — matches Blender's Rendering workspace:
//! Image Editor for viewing render results, with optional side panel for render settings.

use crate::store::areas::{BAreaSpec, BAreaType};
use crate::store::hosts::{BHostType, SWITCHABLE_HOSTS};
use crate::store::zones::BZoneType;

pub static RENDERING_AREAS: &[BAreaSpec] = &[
    BAreaSpec {
        idx: BAreaType::Top,
        zone: BZoneType::Top,
        default_host: BHostType::Topbar,
        items: &[BHostType::Topbar],
    },
    BAreaSpec {
        idx: BAreaType::Center,
        zone: BZoneType::Center,
        default_host: BHostType::Image,
        items: SWITCHABLE_HOSTS,
    },
    BAreaSpec {
        idx: BAreaType::Right,
        zone: BZoneType::Right,
        default_host: BHostType::Buts,
        items: &[BHostType::Buts, BHostType::Outliner],
    },
];

pub static ZONES: &[BZoneType] = &[BZoneType::Top, BZoneType::Center, BZoneType::Right];
pub static COLUMNS: &[f32] = &[0.72, 0.28];
