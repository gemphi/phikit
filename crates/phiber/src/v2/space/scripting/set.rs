//! Scripting workspace — its areas, and which hosts each one offers.
//!
//! Scripting workspace — Text editor, Console, Info, View3D, Outliner+Buts.
//! Blender ref: startup.blend default `Scripting` workspace.

use crate::v2::area::{BAreaSpec, BAreaType};
use crate::v2::host::BHostType;
use crate::v2::zone::BZoneType;

pub const LABEL: &str = "Scripting";
pub const ICON: &str = "code";

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
    // Blender's left column: a viewport (393 of 1054 px) …
    BAreaSpec {
        kind: BAreaType::Left,
        zone: BZoneType::Left,
        default_host: BHostType::View3D,
        hosts: BHostType::SWITCHABLE,
    },
    // … the Python console (463 px) …
    BAreaSpec {
        kind: BAreaType::Left,
        zone: BZoneType::Left,
        default_host: BHostType::Console,
        hosts: &[BHostType::Console, BHostType::Info, BHostType::Text],
    },
    // … and the Info log (169 px).
    BAreaSpec {
        kind: BAreaType::Left,
        zone: BZoneType::Left,
        default_host: BHostType::Info,
        hosts: &[BHostType::Info, BHostType::Console],
    },
    // The Text editor fills the centre.
    BAreaSpec {
        kind: BAreaType::Center,
        zone: BZoneType::Center,
        default_host: BHostType::Text,
        hosts: &[BHostType::Text, BHostType::Console, BHostType::Info],
    },
    // Blender stacks two Outliners here; a second one would share this slot's key.
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
    BAreaSpec {
        kind: BAreaType::Bottom,
        zone: BZoneType::Bottom,
        default_host: BHostType::Statusbar,
        hosts: &[BHostType::Statusbar],
    },
];
