//! Scripting space area assignments — matches Blender's Scripting workspace:
//! Python Console (left), Text Editor (center), Outliner (right).

use crate::store::areas::{BAreaSpec, BAreaType};
use crate::store::hosts::BHostType;
use crate::store::zones::BZoneType;

pub static SCRIPTING_AREAS: &[BAreaSpec] = &[
    BAreaSpec {
        idx: BAreaType::Top,
        zone: BZoneType::Top,
        default_host: BHostType::Topbar,
        items: &[BHostType::Topbar],
    },
    BAreaSpec {
        idx: BAreaType::Left,
        zone: BZoneType::Left,
        default_host: BHostType::Console,
        items: &[BHostType::Console, BHostType::Info],
    },
    BAreaSpec {
        idx: BAreaType::Center,
        zone: BZoneType::Center,
        default_host: BHostType::Text,
        items: &[BHostType::Text, BHostType::Console],
    },
    BAreaSpec {
        idx: BAreaType::Right,
        zone: BZoneType::Right,
        default_host: BHostType::Outliner,
        items: &[BHostType::Outliner, BHostType::Buts],
    },
];

pub static ZONES: &[BZoneType] = &[
    BZoneType::Top,
    BZoneType::Left,
    BZoneType::Center,
    BZoneType::Right,
];
pub static COLUMNS: &[f32] = &[0.18, 0.64, 0.18];
