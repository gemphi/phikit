//! Project Browser regions, as Reality Composer Pro lays its browser out:
//! the header (the editor switcher, the filter), the grid's toolbar, the
//! navigator at the left, the cards in the middle, the inspector at the
//! right.

use crate::v2::pane::{BPaneSpec, BPaneType};
use crate::v2::part::BPartType;
use crate::v2::region::{BRegionSpec, BRegionType};

static HEADER_PANES: &[BPaneSpec] = &[
    BPaneSpec { kind: BPaneType::Left, parts: &[BPartType::Switcher] },
    BPaneSpec { kind: BPaneType::Center, parts: &[] },
];
static BAR_PANES: &[BPaneSpec] = &[BPaneSpec { kind: BPaneType::Center, parts: &[BPartType::ProjectBar] }];
static NAV_PANES: &[BPaneSpec] = &[BPaneSpec { kind: BPaneType::Center, parts: &[BPartType::ProjectNav] }];
static CENTER_PANES: &[BPaneSpec] = &[BPaneSpec { kind: BPaneType::Center, parts: &[BPartType::Custom] }];
static INFO_PANES: &[BPaneSpec] = &[BPaneSpec { kind: BPaneType::Center, parts: &[BPartType::ProjectInfo] }];

pub static REGIONS: &[BRegionSpec] = &[
    BRegionSpec { kind: BRegionType::Header, panes: HEADER_PANES },
    BRegionSpec { kind: BRegionType::ToolHeader, panes: BAR_PANES },
    BRegionSpec { kind: BRegionType::Tools, panes: NAV_PANES },
    BRegionSpec { kind: BRegionType::Center, panes: CENTER_PANES },
    BRegionSpec { kind: BRegionType::UI, panes: INFO_PANES },
];
