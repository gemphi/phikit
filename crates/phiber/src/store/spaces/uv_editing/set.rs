//! UV Editing space area assignments — matches Blender's UV Editing workspace:
//! UV Editor (left), 3D Viewport (center).

use crate::store::areas::{BAreaSpec, BAreaType};
use crate::store::hosts::{BHostType, SWITCHABLE_HOSTS};
use crate::store::zones::BZoneType;

pub static UV_EDITING_AREAS: &[BAreaSpec] = &[
    BAreaSpec {
        idx: BAreaType::Top,
        zone: BZoneType::Top,
        default_host: BHostType::Topbar,
        items: &[BHostType::Topbar],
    },
    BAreaSpec {
        idx: BAreaType::Left,
        zone: BZoneType::Left,
        default_host: BHostType::Uv,
        items: SWITCHABLE_HOSTS,
    },
    BAreaSpec {
        idx: BAreaType::Center,
        zone: BZoneType::Center,
        default_host: BHostType::View3D,
        items: SWITCHABLE_HOSTS,
    },
];

pub static ZONES: &[BZoneType] = &[BZoneType::Top, BZoneType::Left, BZoneType::Center];
pub static COLUMNS: &[f32] = &[0.5, 0.5];
