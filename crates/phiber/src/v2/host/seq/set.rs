//! Sequencer regions, panes and parts — the content of the host.
//!
//! Video Sequencer host. Blender ref: `space_sequencer/space_sequencer.cc:1210-`.
//! Regions: WINDOW, PREVIEW, UI, TOOLS, CHANNELS, TOOL_HEADER, HEADER, FOOTER, SCRUBBING.

use crate::v2::host::header;
use crate::v2::pane::{BPaneSpec, BPaneType};
use crate::v2::part::BPartType;
use crate::v2::region::{BRegionSpec, BRegionType};

static CENTER_PANES: &[BPaneSpec] = &[
    BPaneSpec {
        kind: BPaneType::Left,
        parts: &[BPartType::Toolbar],
    },
    BPaneSpec {
        kind: BPaneType::Center,
        parts: &[BPartType::Custom],
    },
    BPaneSpec {
        kind: BPaneType::Right,
        parts: &[BPartType::Sidebar],
    },
];
static CHANNELS_PANES: &[BPaneSpec] = &[BPaneSpec {
    kind: BPaneType::Center,
    parts: &[BPartType::Channels],
}];
static SCRUBBING_PANES: &[BPaneSpec] = &[BPaneSpec {
    kind: BPaneType::Center,
    parts: &[BPartType::Scrubbing],
}];

pub static REGIONS: &[BRegionSpec] = &[
    BRegionSpec {
        kind: BRegionType::ToolHeader,
        panes: header::TOOL,
    },
    BRegionSpec {
        kind: BRegionType::Header,
        panes: header::EDITOR,
    },
    BRegionSpec {
        kind: BRegionType::Channels,
        panes: CHANNELS_PANES,
    },
    BRegionSpec {
        kind: BRegionType::Scrubbing,
        panes: SCRUBBING_PANES,
    },
    BRegionSpec {
        kind: BRegionType::Center,
        panes: CENTER_PANES,
    },
];
