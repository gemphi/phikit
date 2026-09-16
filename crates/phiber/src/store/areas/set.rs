//! Area set - static spec arrays and labels.

use super::def::*;
use super::{BAreaSpec, BAreaType};
use crate::store::hosts::BHostType;
use crate::store::zones::BZoneType;

pub static ALL_AREAS: &[BAreaType] = &[
    BAreaType::Top,
    BAreaType::Left,
    BAreaType::Right,
    BAreaType::Center,
    BAreaType::Bottom,
];

pub static AREA_LABELS: &[(&str, &str)] = &[
    (TOP_LABEL, TOP_LABEL),
    (LEFT_LABEL, LEFT_LABEL),
    (RIGHT_LABEL, RIGHT_LABEL),
    (CENTER_LABEL, CENTER_LABEL),
    (BOTTOM_LABEL, BOTTOM_LABEL),
];

pub static ALL_SPECS: &[BAreaSpec] = &[
    BAreaSpec {
        idx: BAreaType::Top,
        zone: BZoneType::Top,
        default_host: BHostType::Topbar,
        items: &[BHostType::Topbar],
    },
    BAreaSpec {
        idx: BAreaType::Left,
        zone: BZoneType::Left,
        default_host: BHostType::Outliner,
        items: &[BHostType::Outliner],
    },
    BAreaSpec {
        idx: BAreaType::Right,
        zone: BZoneType::Right,
        default_host: BHostType::Buts,
        items: &[BHostType::Buts],
    },
    BAreaSpec {
        idx: BAreaType::Center,
        zone: BZoneType::Center,
        default_host: BHostType::View3D,
        items: &[BHostType::View3D],
    },
    BAreaSpec {
        idx: BAreaType::Bottom,
        zone: BZoneType::Bottom,
        default_host: BHostType::Time,
        items: &[BHostType::Time],
    },
];
