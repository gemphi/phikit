//! File Browser regions, panes and parts — the content of the host.
//!
//! File Browser host. Blender ref: `space_file/space_file.cc:1043-`.
//!
//! Blender's regions, declared as ours, in Blender's order:
//! ```text
//! ┌─ header      View / Select menus ──────────────────────────────────┐
//! ├─ ui          [← → ↑ ⟳] /directory/ [search] [≡][⊞][⊠] [👁][⛛▾]   ┤
//! ├─ tools ────────────┬─ window ──────────────────────────────────────┤
//! │ ▾ Volumes          │  Name        Modified      Size               │
//! │ ▾ System           │  … file rows …                                │
//! │ ▾ Bookmarks        │                                               │
//! ├─ execute     [file name]               [Cancel] [Open Phiber] ────┤
//! └────────────────────────────────────────────────────────────────────┘
//! ```
//! The same declaration serves the editor in a workspace and the Open /
//! Save As dialog (`modal::file_browser`), which raises this host in a card.

use crate::v2::host::header;
use crate::v2::pane::{BPaneSpec, BPaneType};
use crate::v2::part::BPartType;
use crate::v2::region::{BRegionSpec, BRegionType};

/// Blender's `ui` region at the top: the directory strip.
// The path strip is the columns under it, read across: the folder moves
// and the refresh span the source list's width (`pane`: a Left pane over a
// tools column takes that column's width, its two parts at either end), so
// the directory field starts where the file list does.
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
/// Blender's `tools` region: the source list.
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
static EXECUTE_PANES: &[BPaneSpec] = &[BPaneSpec {
    kind: BPaneType::Center,
    parts: &[BPartType::Execute],
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
    BRegionSpec {
        kind: BRegionType::Execute,
        panes: EXECUTE_PANES,
    },
];
