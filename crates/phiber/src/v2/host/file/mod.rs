//! File Browser host. Blender ref: `space_file/space_file.cc:1043-`.
//! Regions: WINDOW, HEADER, UI, EXECUTE, TOOLS (directory list).

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
static TOOLS_PANES: &[BPaneSpec] = &[BPaneSpec {
    kind: BPaneType::Center,
    parts: &[BPartType::Toolbar],
}];
static CENTER_PANES: &[BPaneSpec] = &[
    BPaneSpec {
        kind: BPaneType::Center,
        parts: &[BPartType::Custom],
    },
    BPaneSpec {
        kind: BPaneType::Right,
        parts: &[BPartType::Sidebar],
    },
];
static EXECUTE_PANES: &[BPaneSpec] = &[BPaneSpec {
    kind: BPaneType::Center,
    parts: &[BPartType::Execute],
}];

pub static REGIONS: &[BRegionSpec] = &[
    BRegionSpec {
        kind: BRegionType::Header,
        panes: HEADER_PANES,
    },
    BRegionSpec {
        kind: BRegionType::Tools,
        panes: TOOLS_PANES,
    },
    BRegionSpec {
        kind: BRegionType::Center,
        panes: CENTER_PANES,
    },
    BRegionSpec {
        kind: BRegionType::Execute,
        panes: EXECUTE_PANES,
    },
];
