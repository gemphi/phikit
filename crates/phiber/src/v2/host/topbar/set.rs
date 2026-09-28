//! Top Bar regions, panes and parts — the content of the host.
//!
//! Topbar host — menu, workspace tabs, zone visibility, and host switcher.
//! Blender ref: DNA `SPACE_TOPBAR` (single HEADER region, global screen area).

use crate::v2::pane::{BPaneSpec, BPaneType};
use crate::v2::part::BPartType;
use crate::v2::region::{BRegionSpec, BRegionType};

static HEADER_PANES: &[BPaneSpec] = &[
    BPaneSpec {
        kind: BPaneType::Left,
        parts: &[BPartType::CaptionLead, BPartType::Menubar],
    },
    BPaneSpec {
        kind: BPaneType::Center,
        parts: &[BPartType::Spacebar],
    },
    // Right pane: what can be hidden (the Zone-visibility toggles, Blender's
    // per-area show/hide), then the window controls. The scene read-out lives
    // in the status bar at the bottom, where Blender keeps it.
    BPaneSpec {
        kind: BPaneType::Right,
        parts: &[BPartType::Zonebar, BPartType::CaptionTrail],
    },
];

pub static REGIONS: &[BRegionSpec] = &[BRegionSpec {
    kind: BRegionType::Header,
    panes: HEADER_PANES,
}];
