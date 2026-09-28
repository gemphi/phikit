//! Timeline regions, panes and parts — the content of the host.
//!
//! Timeline host — region declarations only. Widget content lives in
//! `part::time`. Blender ref: `editors/space_action/space_action.cc`
//! in Timeline mode.

use crate::v2::pane::{BPaneSpec, BPaneType};
use crate::v2::part::BPartType;
use crate::v2::region::{BRegionSpec, BRegionType};

/// Blender's Timeline header (`DOPESHEET_HT_header` in Timeline mode, then
/// `playback_controls`): the editor switcher and menus on the left, the
/// transport group in the middle, the current frame and range on the right.
static HEADER: &[BPaneSpec] = &[
    BPaneSpec {
        kind: BPaneType::Left,
        parts: &[BPartType::Switcher, BPartType::Menubar],
    },
    BPaneSpec {
        kind: BPaneType::Center,
        parts: &[BPartType::Playback],
    },
    BPaneSpec {
        kind: BPaneType::Right,
        parts: &[BPartType::FrameRange],
    },
];

/// The channel list is hidden by default in Timeline mode
/// (`action_create`: `RGN_FLAG_HIDDEN`), so the window is the whole centre.
static CENTER_PANES: &[BPaneSpec] = &[BPaneSpec {
    kind: BPaneType::Center,
    parts: &[BPartType::Custom],
}];

pub static REGIONS: &[BRegionSpec] = &[
    BRegionSpec {
        kind: BRegionType::Header,
        panes: HEADER,
    },
    BRegionSpec {
        kind: BRegionType::Center,
        panes: CENTER_PANES,
    },
];
