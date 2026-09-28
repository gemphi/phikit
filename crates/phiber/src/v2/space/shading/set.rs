//! Shading workspace — its areas, and which hosts each one offers.
//!
//! Shading workspace — Blender ref: startup.blend default `Shading` workspace.
//!
//! The three middle-band zones are real stacked columns:
//! Left   : File Browser over Image/UV editor
//! Center : View3D over Shader Editor
//! Right  : Outliner over Properties
//!
//! This is composition data only. The generic Space → Zone → Area → Host
//! renderer owns layout; no Shading-only renderer branch is required.

use crate::v2::area::{BAreaSpec, BAreaType};
use crate::v2::host::BHostType;
use crate::v2::zone::BZoneType;

pub const LABEL: &str = "Shading";
pub const ICON: &str = "shader";

pub static AREAS: &[BAreaSpec] = &[
    BAreaSpec {
        kind: BAreaType::Top,
        zone: BZoneType::Top,
        default_host: BHostType::Topbar,
        hosts: &[BHostType::Topbar],
    },
    // The activity rail: the same fixed strip down the far left of every space.
    BAreaSpec {
        kind: BAreaType::Left,
        zone: BZoneType::Rail,
        default_host: BHostType::Activity,
        hosts: &[BHostType::Activity],
    },
    BAreaSpec {
        kind: BAreaType::Left,
        zone: BZoneType::Left,
        default_host: BHostType::File,
        hosts: &[BHostType::File, BHostType::Asset],
    },
    BAreaSpec {
        kind: BAreaType::Left,
        zone: BZoneType::Left,
        default_host: BHostType::Image,
        hosts: &[BHostType::Image, BHostType::Uv],
    },
    BAreaSpec {
        kind: BAreaType::Center,
        zone: BZoneType::Center,
        default_host: BHostType::View3D,
        hosts: BHostType::SWITCHABLE,
    },
    BAreaSpec {
        kind: BAreaType::Center,
        zone: BZoneType::Center,
        default_host: BHostType::Shader,
        hosts: &[
            BHostType::Shader,
            BHostType::Node,
            BHostType::Compositor,
            BHostType::GeoNode,
        ],
    },
    BAreaSpec {
        kind: BAreaType::Right,
        zone: BZoneType::Right,
        default_host: BHostType::Outliner,
        hosts: &[BHostType::Outliner, BHostType::File, BHostType::Asset],
    },
    BAreaSpec {
        kind: BAreaType::Right,
        zone: BZoneType::Right,
        default_host: BHostType::Buts,
        hosts: &[BHostType::Buts, BHostType::Spreadsheet],
    },
    // The window's status bar, as in every workspace.
    BAreaSpec {
        kind: BAreaType::Bottom,
        zone: BZoneType::Bottom,
        default_host: BHostType::Statusbar,
        hosts: &[BHostType::Statusbar],
    },
];
