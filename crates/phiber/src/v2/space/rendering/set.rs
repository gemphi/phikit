//! Rendering workspace — its areas, and which hosts each one offers.
//!
//! Rendering workspace — one full-width Image Editor showing the render.
//!
//! Blender ref: `startup.blend`'s `Rendering` workspace, which is a single
//! `SPACE_IMAGE` area on Render Result. Splitting it into a viewport beside a
//! preview was a different workspace's shape: the point of Rendering is to
//! look at the frame, at the size it was rendered.

use crate::v2::area::{BAreaSpec, BAreaType};
use crate::v2::host::BHostType;
use crate::v2::zone::BZoneType;

pub const LABEL: &str = "Rendering";
pub const ICON: &str = "camera";

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
    // Blender's Rendering workspace: the Image Editor showing the Render Result,
    // with its sidebar open (`startup.blend`: 1574 × 954 of 1920 × 1080).
    BAreaSpec {
        kind: BAreaType::Center,
        zone: BZoneType::Center,
        default_host: BHostType::Image,
        hosts: BHostType::SWITCHABLE,
    },
    // The Properties editor runs down the whole right side — no Outliner here.
    BAreaSpec {
        kind: BAreaType::Right,
        zone: BZoneType::Right,
        default_host: BHostType::Buts,
        hosts: &[BHostType::Buts],
    },
    // A Timeline strip across the bottom, as in Layout (74 px in Blender).
    BAreaSpec {
        kind: BAreaType::Bottom,
        zone: BZoneType::Bottom,
        default_host: BHostType::Time,
        hosts: &[
            BHostType::Time,
            BHostType::Action,
            BHostType::Ipo,
            BHostType::Drivers,
            BHostType::Nla,
            BHostType::Seq,
        ],
    },
    BAreaSpec {
        kind: BAreaType::Bottom,
        zone: BZoneType::Bottom,
        default_host: BHostType::Statusbar,
        hosts: &[BHostType::Statusbar],
    },
];
