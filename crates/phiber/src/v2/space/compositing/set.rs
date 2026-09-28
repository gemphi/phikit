//! ECS Wand workspace — its areas, and which hosts each one offers.
//!
//! Blender calls this workspace Compositing and fills it with a node editor.
//! Phiber keeps the layout and renames the workspace to what the nodes are
//! for here: wiring entities, components and systems — the wand you compose
//! a world with. Blender ref: startup.blend default `Compositing` workspace.

use crate::v2::area::{BAreaSpec, BAreaType};
use crate::v2::host::BHostType;
use crate::v2::zone::BZoneType;

pub const LABEL: &str = "ECS Wand";
pub const ICON: &str = "layers";

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
        kind: BAreaType::Center,
        zone: BZoneType::Center,
        default_host: BHostType::Compositor,
        hosts: BHostType::SWITCHABLE,
    },
    BAreaSpec {
        kind: BAreaType::Right,
        zone: BZoneType::Right,
        default_host: BHostType::Outliner,
        hosts: &[BHostType::Outliner],
    },
    BAreaSpec {
        kind: BAreaType::Right,
        zone: BZoneType::Right,
        default_host: BHostType::Buts,
        hosts: &[BHostType::Buts],
    },
    // Blender's Compositing workspace keeps a Dope Sheet under the node editor (189 of 1054 px).
    BAreaSpec {
        kind: BAreaType::Bottom,
        zone: BZoneType::Bottom,
        default_host: BHostType::Action,
        hosts: &[
            BHostType::Action,
            BHostType::Time,
            BHostType::Ipo,
            BHostType::Drivers,
            BHostType::Nla,
        ],
    },
    BAreaSpec {
        kind: BAreaType::Bottom,
        zone: BZoneType::Bottom,
        default_host: BHostType::Statusbar,
        hosts: &[BHostType::Statusbar],
    },
];
