//! Shading space area assignments — matches Blender's Shading workspace:
//! 3D Viewport (center), Shader Editor (right).

use crate::store::areas::{BAreaSpec, BAreaType};
use crate::store::hosts::{BHostType, SWITCHABLE_HOSTS};
use crate::store::zones::BZoneType;

pub static SHADING_AREAS: &[BAreaSpec] = &[
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
        default_host: BHostType::Shader,
        items: &[BHostType::Shader, BHostType::Node, BHostType::Compositor],
    },
];

pub static ZONES: &[BZoneType] = &[BZoneType::Top, BZoneType::Center, BZoneType::Right];
pub static COLUMNS: &[f32] = &[0.72, 0.28];
