//! Play regions: the header (the editor switcher) over the picker, which
//! fills the window region (`part::custom` routes it to `part::play`).

use crate::v2::pane::{BPaneSpec, BPaneType};
use crate::v2::part::BPartType;
use crate::v2::region::{BRegionSpec, BRegionType};

static HEADER_PANES: &[BPaneSpec] = &[
    BPaneSpec { kind: BPaneType::Left, parts: &[BPartType::Switcher] },
    BPaneSpec { kind: BPaneType::Center, parts: &[] },
];

static CENTER_PANES: &[BPaneSpec] = &[BPaneSpec { kind: BPaneType::Center, parts: &[BPartType::Custom] }];

pub static REGIONS: &[BRegionSpec] = &[
    BRegionSpec { kind: BRegionType::Header, panes: HEADER_PANES },
    BRegionSpec { kind: BRegionType::Center, panes: CENTER_PANES },
];
