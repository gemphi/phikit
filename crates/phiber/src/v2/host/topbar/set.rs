//! Top Bar regions, panes and parts — the content of the host.
//!
//! Topbar host — menu, workspace tabs, zone visibility, and host switcher.
//! Blender ref: DNA `SPACE_TOPBAR` (single HEADER region, global screen area).

use crate::v2::pane::{BPaneSpec, BPaneType};
use crate::v2::part::BPartType;
use crate::v2::region::{BRegionSpec, BRegionType};

static HEADER_PANES: &[BPaneSpec] = &[
    // Left: what the window is — the space switcher, the brand, the menus.
    // Centre: the search that finds anything in it (`find`). Right: the
    // spaces, what can be hidden, then the window controls.
    BPaneSpec {
        kind: BPaneType::Left,
        parts: &[BPartType::CaptionLead, BPartType::Menubar],
    },
    BPaneSpec {
        kind: BPaneType::Center,
        parts: &[BPartType::Search],
    },
    BPaneSpec {
        kind: BPaneType::Right,
        parts: &[BPartType::Spacebar, BPartType::Zonebar, BPartType::CaptionTrail],
    },
];

pub static REGIONS: &[BRegionSpec] = &[BRegionSpec {
    kind: BRegionType::Header,
    panes: HEADER_PANES,
}];
