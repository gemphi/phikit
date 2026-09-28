//! Asset Browser regions, panes and parts — the content of the host.
//!
//! Asset Browser host — Blender's `space_file` in asset mode, so it declares
//! the file browser's regions: the path strip (`ui`), the library and
//! catalog tree (`tools`), the asset grid (`window`) and the asset details
//! sidebar. No execute region: assets are dragged, not opened.

use crate::v2::host::header;
use crate::v2::pane::{BPaneSpec, BPaneType};
use crate::v2::part::BPartType;
use crate::v2::region::{BRegionSpec, BRegionType};

// The same strip as the file browser's (`host/file`): the folder moves and
// the refresh over the source list, the fields over the grid.
static PATH_PANES: &[BPaneSpec] = &[
    BPaneSpec {
        kind: BPaneType::Left,
        parts: &[BPartType::FileNav, BPartType::FileRefresh],
    },
    BPaneSpec {
        kind: BPaneType::Center,
        parts: &[BPartType::PathBar],
    },
    BPaneSpec {
        kind: BPaneType::Right,
        parts: &[BPartType::TypeFilter],
    },
];
static TOOLS_PANES: &[BPaneSpec] = &[BPaneSpec {
    kind: BPaneType::Center,
    parts: &[BPartType::SourceList],
}];
// The file list alone: its bookmarks and options are the source list's
// cards, so no N-sidebar tab strip stands beside it (one sat there empty,
// a strip wide, and doubled the list's right-hand gap).
static CENTER_PANES: &[BPaneSpec] = &[BPaneSpec {
    kind: BPaneType::Center,
    parts: &[BPartType::Custom],
}];

pub static REGIONS: &[BRegionSpec] = &[
    BRegionSpec {
        kind: BRegionType::Header,
        panes: header::EDITOR,
    },
    BRegionSpec {
        kind: BRegionType::ToolHeader,
        panes: PATH_PANES,
    },
    BRegionSpec {
        kind: BRegionType::Tools,
        panes: TOOLS_PANES,
    },
    BRegionSpec {
        kind: BRegionType::Center,
        panes: CENTER_PANES,
    },
];
