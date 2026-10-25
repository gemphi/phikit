//! File Browser regions, panes and parts — the content of the host.
//!
//! File Browser host. Blender ref: `space_file/space_file.cc:1043-`.
//!
//! Three columns, read down — the source list, the files, the details —
//! and the two strips over them are those columns read across, as
//! Explorer's address bar and tool bar are:
//! ```text
//! ┌─ header      File Browser View Select │ This PC › C: › Users …   │ [search]  ┐
//! ├─ ui          ← → ↑  ⟳ +               │                          │ ≡ ⊞ ⛛ ▾ ⚙ ┤
//! ├─ tools ──────────────────────────────┬─ window ──────────────────┬─ details ─┤
//! │ ▾ Bookmarks                           │  Name     Modified   Size │ preview   │
//! │ ▾ System                              │  … file rows …            │ name …    │
//! ├─ execute     Selected: …            Open Blender File      [Cancel] [Open] ──┤
//! └──────────────────────────────────────────────────────────────────────────────┘
//! ```
//! The same declaration serves the editor in a workspace and the Open /
//! Save As dialog (`modal::file_browser`), which raises this host in a card.

use crate::v2::pane::{BPaneSpec, BPaneType};
use crate::v2::part::BPartType;
use crate::v2::region::{BRegionSpec, BRegionType};

/// The header, over the three columns: the switcher and menus over the
/// source list, the path over the files, the search (`complete_header`)
/// over the details. Each pane is its column's width (`pane`, `columns`).
pub static HEADER_PANES: &[BPaneSpec] = &[
    BPaneSpec {
        kind: BPaneType::Left,
        parts: &[BPartType::Switcher, BPartType::Modebar, BPartType::Menubar],
    },
    BPaneSpec {
        kind: BPaneType::Center,
        parts: &[BPartType::PathBar],
    },
    BPaneSpec {
        kind: BPaneType::Right,
        parts: &[],
    },
];
/// Blender's `ui` region: the folder moves and refresh over the source list,
/// the display modes and filters spread over the details column.
pub static PATH_PANES: &[BPaneSpec] = &[
    BPaneSpec {
        kind: BPaneType::Left,
        parts: &[BPartType::FileNav, BPartType::FileRefresh],
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
/// The details column at the right: the picked file (`file::info`).
static INFO_PANES: &[BPaneSpec] = &[BPaneSpec {
    kind: BPaneType::Center,
    parts: &[BPartType::FileInfo],
}];
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
        kind: BRegionType::UI,
        panes: INFO_PANES,
    },
    BRegionSpec {
        kind: BRegionType::Execute,
        panes: EXECUTE_PANES,
    },
];
