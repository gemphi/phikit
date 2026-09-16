//! Topbar host — menu, workspace tabs, zone visibility, and host switcher.
//! Blender ref: DNA `SPACE_TOPBAR` (single HEADER region, global screen area).

use crate::v2::pane::{BPaneSpec, BPaneType};
use crate::v2::part::BPartType;
use crate::v2::region::{BRegionSpec, BRegionType};

static HEADER_PANES: &[BPaneSpec] = &[
    BPaneSpec {
        kind: BPaneType::Left,
        parts: &[BPartType::Menubar],
    },
    BPaneSpec {
        kind: BPaneType::Center,
        parts: &[BPartType::Spacebar],
    },
    // Right pane: Statusbar summary → global Zone-visibility toggles → Host
    // switcher. Switcher stays rightmost so it lines up with every editor's
    // header switcher directly beneath it.
    BPaneSpec {
        kind: BPaneType::Right,
        parts: &[
            BPartType::Statusbar,
            BPartType::Zonebar,
            BPartType::Switcher,
        ],
    },
];

pub static REGIONS: &[BRegionSpec] = &[BRegionSpec {
    kind: BRegionType::Header,
    panes: HEADER_PANES,
}];
