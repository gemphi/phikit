//! Status Bar regions, panes and parts — the content of the host.
//!
//! Statusbar host — region declarations only. No center content.
//! Blender ref: DNA `SPACE_STATUSBAR` (single HEADER region, global screen area).

use crate::v2::pane::{BPaneSpec, BPaneType};
use crate::v2::part::BPartType;
use crate::v2::region::{BRegionSpec, BRegionType};

// The foot of the window: what is loaded on the left, the scene read-out
// (objects, vertices, faces, memory, version) on the right — Blender's
// `STATUSBAR_HT_header`.
static HEADER_PANES: &[BPaneSpec] = &[
    BPaneSpec {
        kind: BPaneType::Left,
        parts: &[BPartType::Menubar],
    },
    // The action log: the newest line between the menus and the read-out,
    // and every line when unfolded.
    BPaneSpec {
        kind: BPaneType::Center,
        parts: &[BPartType::StatusLog],
    },
    BPaneSpec {
        kind: BPaneType::Right,
        parts: &[BPartType::Statusbar],
    },
];

pub static REGIONS: &[BRegionSpec] = &[BRegionSpec {
    kind: BRegionType::Header,
    panes: HEADER_PANES,
}];
