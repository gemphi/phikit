//! Properties (Buts) host — region declarations only. Widget content lives in
//! `part::custom::buts`. Blender ref: `editors/space_buttons/space_buttons.cc`.

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
// The context tabs live in their own NavBar *region* (not a pane inside the
// Center region) so they pick up `strip_bg` — the darker vertical rail — and
// the region walker's Column axis for a proper icon stack. Blender draws the
// Properties `RGN_TYPE_NAV_BAR` as a distinct left rail exactly this way.
static NAVBAR_PANES: &[BPaneSpec] = &[
    BPaneSpec {
        kind: BPaneType::Center,
        parts: &[BPartType::NavBar],
    }, // 15 context tabs
];
static CENTER_PANES: &[BPaneSpec] = &[BPaneSpec {
    kind: BPaneType::Center,
    parts: &[BPartType::Panel],
}];

pub static REGIONS: &[BRegionSpec] = &[
    BRegionSpec {
        kind: BRegionType::Header,
        panes: HEADER_PANES,
    },
    BRegionSpec {
        kind: BRegionType::NavBar,
        panes: NAVBAR_PANES,
    },
    BRegionSpec {
        kind: BRegionType::Center,
        panes: CENTER_PANES,
    },
];
