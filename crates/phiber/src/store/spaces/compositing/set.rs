//! Compositing space area assignments — matches Blender's Compositing workspace:
//! Compositor node editor (center), Image/Viewer (right).

use crate::store::areas::{BAreaSpec, BAreaType};
use crate::store::hosts::BHostType;
use crate::store::zones::BZoneType;

pub static COMPOSITING_AREAS: &[BAreaSpec] = &[
    BAreaSpec {
        idx: BAreaType::Top,
        zone: BZoneType::Top,
        default_host: BHostType::Topbar,
        items: &[BHostType::Topbar],
    },
    BAreaSpec {
        idx: BAreaType::Center,
        zone: BZoneType::Center,
        default_host: BHostType::Compositor,
        items: &[BHostType::Compositor, BHostType::Node, BHostType::Image],
    },
    BAreaSpec {
        idx: BAreaType::Right,
        zone: BZoneType::Right,
        default_host: BHostType::Image,
        items: &[BHostType::Image, BHostType::Node, BHostType::Compositor],
    },
];

pub static ZONES: &[BZoneType] = &[BZoneType::Top, BZoneType::Center, BZoneType::Right];
pub static COLUMNS: &[f32] = &[0.72, 0.28];
