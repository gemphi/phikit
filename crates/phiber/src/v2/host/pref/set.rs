//! Preferences regions, panes and parts — the content of the host.
//!
//! Preferences host. Blender ref: `space_userpref/space_userpref.cc:419-`.
//! Regions: WINDOW, HEADER, UI (NAV_BAR wrapping). Blender's EXECUTE region
//! (`USERPREF_PT_save_preferences`) polls itself away while the header is
//! visible, and the header is; its Save Preferences sits in the header's
//! right group instead (`USERPREF_HT_header.draw_buttons`).

use crate::v2::pane::{BPaneSpec, BPaneType};
use crate::v2::part::BPartType;
use crate::v2::region::{BRegionSpec, BRegionType};

// Blender declares the Preferences category list as `RGN_TYPE_NAV_BAR`, the
// same region kind Properties uses. Modelling it as a Left *pane* of the
// Center region instead gave it the pane treatment — no rail fill, Row axis,
// content-width — so the one thing that looks identical in Blender looked
// different here. It is a NavBar region, like Properties'.
static NAVBAR_PANES: &[BPaneSpec] = &[BPaneSpec {
    kind: BPaneType::Center,
    parts: &[BPartType::NavBar],
}];
// The window region holds the `USERPREF_PT_*` panels of the chosen
// category. They are drawn by `part::custom::prefs` (the category's style
// sections, each a folding `BCard`), not by `part::props`, which is the
// Properties editor's context tabs and has no Preferences sections — pointing
// here at `Panel` left the dialog with an empty body.
static CENTER_PANES: &[BPaneSpec] = &[BPaneSpec {
    kind: BPaneType::Center,
    parts: &[BPartType::Custom],
}];
// `USERPREF_HT_header`: the switcher and the editor menus at the left, a
// spacer, Save Preferences at the right.
static HEADER_PANES: &[BPaneSpec] = &[
    BPaneSpec { kind: BPaneType::Left, parts: &[BPartType::Switcher, BPartType::Modebar, BPartType::Menubar] },
    BPaneSpec { kind: BPaneType::Center, parts: &[] },
    BPaneSpec { kind: BPaneType::Right, parts: &[BPartType::Execute] },
];

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
