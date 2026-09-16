//! Geometry Nodes space area assignments — matches Blender's Geometry Nodes workspace:
//! 3D Viewport (left), Geometry Nodes editor (center), Spreadsheet (right).

use crate::store::areas::{BAreaSpec, BAreaType};
use crate::store::hosts::BHostType;
use crate::store::zones::BZoneType;

pub static GEOMETRY_NODES_AREAS: &[BAreaSpec] = &[
    BAreaSpec {
        idx: BAreaType::Top,
        zone: BZoneType::Top,
        default_host: BHostType::Topbar,
        items: &[BHostType::Topbar],
    },
    BAreaSpec {
        idx: BAreaType::Left,
        zone: BZoneType::Left,
        default_host: BHostType::View3D,
        items: &[BHostType::View3D],
    },
    BAreaSpec {
        idx: BAreaType::Center,
        zone: BZoneType::Center,
        default_host: BHostType::GeoNode,
        items: &[BHostType::GeoNode, BHostType::Node, BHostType::Shader],
    },
    BAreaSpec {
        idx: BAreaType::Right,
        zone: BZoneType::Right,
        default_host: BHostType::Spreadsheet,
        items: &[BHostType::Spreadsheet, BHostType::Buts, BHostType::Outliner],
    },
];

pub static ZONES: &[BZoneType] = &[
    BZoneType::Top,
    BZoneType::Left,
    BZoneType::Center,
    BZoneType::Right,
];
pub static COLUMNS: &[f32] = &[0.18, 0.64, 0.18];
