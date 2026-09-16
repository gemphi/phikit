//! Statusbar host — region declarations only. No center content.
//! Blender ref: DNA `SPACE_STATUSBAR` (single HEADER region, global screen area).

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
        parts: &[BPartType::Modebar],
    },
    BPaneSpec {
        kind: BPaneType::Right,
        parts: &[BPartType::Switcher],
    },
];

pub static REGIONS: &[BRegionSpec] = &[BRegionSpec {
    kind: BRegionType::Header,
    panes: HEADER_PANES,
}];
