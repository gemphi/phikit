//! Pane set - static spec arrays and per-host pane layouts.
//!
//! All pane layouts are static data. No runtime computation.
//! Each host type maps to a static &[BPaneSpec] per region.

use super::BPaneSpec;
use super::def::*;
use crate::store::hosts::BHostType;
use crate::store::parts::{BPart, BPartType};
use crate::store::regions::BRegionType;

pub static ALL: &[BPaneType] = &[BPaneType::Left, BPaneType::Center, BPaneType::Right];

pub static ALL_SPECS: &[BPaneSpec] = &[
    BPaneSpec::new(BPaneType::Left),
    BPaneSpec::new(BPaneType::Center),
    BPaneSpec::new(BPaneType::Right),
];

pub static LABELS: &[&str] = &[LEFT_LABEL, CENTER_LABEL, RIGHT_LABEL];

// ── Reusable part arrays ──

static TOOLBAR_PART: &[BPart] = &[BPart::new(BPartType::Toolbar)];
static SIDEBAR_PART: &[BPart] = &[BPart::new(BPartType::Sidebar)];
static VIEWPORT_PART: &[BPart] = &[BPart::new(BPartType::Custom)];
static PANEL_PART: &[BPart] = &[BPart::new(BPartType::Panel)];
static CHANNELS_PART: &[BPart] = &[BPart::new(BPartType::Channels)];

// ── Header region panes ──

// View3D header: menus + mode left (Blender order), control cluster center, switcher right.
static VIEW3D_HEADER: &[BPaneSpec] = &[
    BPaneSpec::with_items(
        BPaneType::Left,
        &[
            BPart::new(BPartType::Menubar),
            BPart::new(BPartType::Modebar),
        ],
    ),
    BPaneSpec::with_items(BPaneType::Center, &[BPart::new(BPartType::Controls)]),
    BPaneSpec::with_items(BPaneType::Right, &[BPart::new(BPartType::Switcher)]),
];
static NODE_HEADER: &[BPaneSpec] = &[
    BPaneSpec::with_items(BPaneType::Left, &[BPart::new(BPartType::Menubar)]),
    BPaneSpec::with_items(BPaneType::Center, &[BPart::new(BPartType::Modebar)]),
    BPaneSpec::with_items(BPaneType::Right, &[BPart::new(BPartType::Switcher)]),
];
static BUTS_HEADER: &[BPaneSpec] = &[
    BPaneSpec::with_items(BPaneType::Left, &[BPart::new(BPartType::Menubar)]),
    BPaneSpec::new(BPaneType::Center),
    BPaneSpec::with_items(BPaneType::Right, &[BPart::new(BPartType::Switcher)]),
];
static SIMPLE_HEADER: &[BPaneSpec] = &[
    BPaneSpec::with_items(BPaneType::Left, &[BPart::new(BPartType::Menubar)]),
    BPaneSpec::new(BPaneType::Center),
    BPaneSpec::with_items(BPaneType::Right, &[BPart::new(BPartType::Switcher)]),
];
static TOPBAR_HEADER: &[BPaneSpec] = &[
    BPaneSpec::with_items(BPaneType::Left, &[BPart::new(BPartType::Menubar)]),
    BPaneSpec::with_items(BPaneType::Center, &[BPart::new(BPartType::Spacebar)]),
    BPaneSpec::with_items(BPaneType::Right, &[BPart::new(BPartType::Statusbar)]),
];
static STATUSBAR_HEADER: &[BPaneSpec] = &[BPaneSpec::with_items(
    BPaneType::Center,
    &[BPart::new(BPartType::Statusbar)],
)];
// Timeline header: transport cluster lives in the Scrubbing slot (Blender space_action header).
static TIME_HEADER: &[BPaneSpec] = &[
    BPaneSpec::with_items(BPaneType::Left, &[BPart::new(BPartType::Menubar)]),
    BPaneSpec::with_items(BPaneType::Center, &[BPart::new(BPartType::Scrubbing)]),
    BPaneSpec::with_items(BPaneType::Right, &[BPart::new(BPartType::Switcher)]),
];
// Outliner header: display-mode dropdown lives in the Modebar slot.
static OUTLINER_HEADER: &[BPaneSpec] = &[
    BPaneSpec::with_items(BPaneType::Left, &[BPart::new(BPartType::Menubar)]),
    BPaneSpec::with_items(BPaneType::Center, &[BPart::new(BPartType::Modebar)]),
    BPaneSpec::with_items(BPaneType::Right, &[BPart::new(BPartType::Switcher)]),
];
static EMPTY_PANES: &[BPaneSpec] = &[];

// ── Center region panes per host type (Matching Blender 4.x exactly) ──

static VIEW3D_CENTER: &[BPaneSpec] = &[
    BPaneSpec::with_items(BPaneType::Left, TOOLBAR_PART),
    BPaneSpec::with_items(BPaneType::Center, VIEWPORT_PART),
    BPaneSpec::with_items(BPaneType::Right, SIDEBAR_PART),
];
static IMAGE_CENTER: &[BPaneSpec] = &[
    BPaneSpec::with_items(BPaneType::Left, TOOLBAR_PART),
    BPaneSpec::with_items(BPaneType::Center, VIEWPORT_PART),
    BPaneSpec::with_items(BPaneType::Right, SIDEBAR_PART),
];
static UV_CENTER: &[BPaneSpec] = &[
    BPaneSpec::with_items(BPaneType::Left, TOOLBAR_PART),
    BPaneSpec::with_items(BPaneType::Center, VIEWPORT_PART),
    BPaneSpec::with_items(BPaneType::Right, SIDEBAR_PART),
];
static CLIP_CENTER: &[BPaneSpec] = &[
    BPaneSpec::with_items(BPaneType::Left, TOOLBAR_PART),
    BPaneSpec::with_items(BPaneType::Center, VIEWPORT_PART),
    BPaneSpec::with_items(BPaneType::Right, SIDEBAR_PART),
];
static NODE_CENTER: &[BPaneSpec] = &[
    BPaneSpec::with_items(BPaneType::Left, TOOLBAR_PART),
    BPaneSpec::with_items(BPaneType::Center, VIEWPORT_PART),
    BPaneSpec::with_items(BPaneType::Right, SIDEBAR_PART),
];
static COMPOSITOR_CENTER: &[BPaneSpec] = &[
    BPaneSpec::with_items(BPaneType::Left, TOOLBAR_PART),
    BPaneSpec::with_items(BPaneType::Center, VIEWPORT_PART),
    BPaneSpec::with_items(BPaneType::Right, SIDEBAR_PART),
];
static SHADER_CENTER: &[BPaneSpec] = &[
    BPaneSpec::with_items(BPaneType::Left, TOOLBAR_PART),
    BPaneSpec::with_items(BPaneType::Center, VIEWPORT_PART),
    BPaneSpec::with_items(BPaneType::Right, SIDEBAR_PART),
];
static GEONODE_CENTER: &[BPaneSpec] = &[
    BPaneSpec::with_items(BPaneType::Left, TOOLBAR_PART),
    BPaneSpec::with_items(BPaneType::Center, VIEWPORT_PART),
    BPaneSpec::with_items(BPaneType::Right, SIDEBAR_PART),
];
static OUTLINER_CENTER: &[BPaneSpec] = &[BPaneSpec::with_items(BPaneType::Center, VIEWPORT_PART)];
// Properties center: NavBar context tabs on the left (Blender RGN_TYPE_NAV_BAR).
static BUTS_CENTER: &[BPaneSpec] = &[
    BPaneSpec::with_items(BPaneType::Left, &[BPart::new(BPartType::NavBar)]),
    BPaneSpec::with_items(BPaneType::Center, PANEL_PART),
];
// File browser header: menus left, directory path breadcrumb center, switcher right.
static FILE_HEADER: &[BPaneSpec] = &[
    BPaneSpec::with_items(BPaneType::Left, &[BPart::new(BPartType::Menubar)]),
    BPaneSpec::with_items(BPaneType::Center, &[BPart::new(BPartType::NavBar)]),
    BPaneSpec::with_items(BPaneType::Right, &[BPart::new(BPartType::Switcher)]),
];
// File browser center: bookmarks/sources column left, file list right.
static FILE_CENTER: &[BPaneSpec] = &[
    BPaneSpec::with_items(BPaneType::Left, CHANNELS_PART),
    BPaneSpec::with_items(BPaneType::Center, VIEWPORT_PART),
];
// File browser footer: filename field + Open / Cancel (Blender RGN_TYPE_EXECUTE).
static FILE_FOOTER: &[BPaneSpec] = &[BPaneSpec::with_items(
    BPaneType::Center,
    &[BPart::new(BPartType::Execute)],
)];
static INFO_CENTER: &[BPaneSpec] = &[BPaneSpec::with_items(BPaneType::Center, VIEWPORT_PART)];
static SEQ_CENTER: &[BPaneSpec] = &[BPaneSpec::with_items(BPaneType::Center, VIEWPORT_PART)];
static TEXT_CENTER: &[BPaneSpec] = &[BPaneSpec::with_items(BPaneType::Center, VIEWPORT_PART)];
static ACTION_CENTER: &[BPaneSpec] = &[
    BPaneSpec::with_items(BPaneType::Left, CHANNELS_PART),
    BPaneSpec::with_items(BPaneType::Center, VIEWPORT_PART),
];
static NLA_CENTER: &[BPaneSpec] = &[
    BPaneSpec::with_items(BPaneType::Left, CHANNELS_PART),
    BPaneSpec::with_items(BPaneType::Center, VIEWPORT_PART),
];
// Timeline center: ruler/scrub only — no channel list (that's the Dope Sheet).
static TIME_CENTER: &[BPaneSpec] = &[BPaneSpec::with_items(BPaneType::Center, VIEWPORT_PART)];
static IPO_CENTER: &[BPaneSpec] = &[
    BPaneSpec::with_items(BPaneType::Left, CHANNELS_PART),
    BPaneSpec::with_items(BPaneType::Center, VIEWPORT_PART),
];
static CONSOLE_CENTER: &[BPaneSpec] = &[BPaneSpec::with_items(BPaneType::Center, VIEWPORT_PART)];
static LOGIC_CENTER: &[BPaneSpec] = &[BPaneSpec::with_items(BPaneType::Center, VIEWPORT_PART)];
static DRIVERS_CENTER: &[BPaneSpec] = &[
    BPaneSpec::with_items(BPaneType::Left, CHANNELS_PART),
    BPaneSpec::with_items(BPaneType::Center, VIEWPORT_PART),
];
static ASSET_CENTER: &[BPaneSpec] = &[BPaneSpec::with_items(BPaneType::Center, VIEWPORT_PART)];
static SPREADSHEET_CENTER: &[BPaneSpec] =
    &[BPaneSpec::with_items(BPaneType::Center, VIEWPORT_PART)];
static PREFS_CENTER: &[BPaneSpec] = &[BPaneSpec::with_items(BPaneType::Center, PANEL_PART)];
static EMPTY_CENTER: &[BPaneSpec] = &[BPaneSpec::with_items(BPaneType::Center, VIEWPORT_PART)];

/// Static pane lookup per host/region — pure data match.
pub fn panes_for(host: BHostType, region: BRegionType) -> &'static [BPaneSpec] {
    match region {
        BRegionType::Header => match host {
            BHostType::Topbar => TOPBAR_HEADER,
            BHostType::Statusbar => STATUSBAR_HEADER,
            BHostType::View3D | BHostType::Image | BHostType::Uv | BHostType::Clip => VIEW3D_HEADER,
            BHostType::Node | BHostType::Compositor | BHostType::Shader | BHostType::GeoNode => {
                NODE_HEADER
            }
            BHostType::Buts => BUTS_HEADER,
            BHostType::Time => TIME_HEADER,
            BHostType::Outliner => OUTLINER_HEADER,
            BHostType::File => FILE_HEADER,
            _ => SIMPLE_HEADER,
        },
        BRegionType::Footer => match host {
            BHostType::Statusbar => STATUSBAR_HEADER,
            BHostType::File => FILE_FOOTER,
            _ => EMPTY_PANES,
        },
        BRegionType::Center | BRegionType::Window => match host {
            BHostType::View3D => VIEW3D_CENTER,
            BHostType::Image => IMAGE_CENTER,
            BHostType::Uv => UV_CENTER,
            BHostType::Clip => CLIP_CENTER,
            BHostType::Node => NODE_CENTER,
            BHostType::Compositor => COMPOSITOR_CENTER,
            BHostType::Shader => SHADER_CENTER,
            BHostType::GeoNode => GEONODE_CENTER,
            BHostType::Outliner => OUTLINER_CENTER,
            BHostType::Buts => BUTS_CENTER,
            BHostType::File => FILE_CENTER,
            BHostType::Info => INFO_CENTER,
            BHostType::Seq => SEQ_CENTER,
            BHostType::Text => TEXT_CENTER,
            BHostType::Action => ACTION_CENTER,
            BHostType::Nla => NLA_CENTER,
            BHostType::Time => TIME_CENTER,
            BHostType::Ipo => IPO_CENTER,
            BHostType::Console => CONSOLE_CENTER,
            BHostType::Logic => LOGIC_CENTER,
            BHostType::Drivers => DRIVERS_CENTER,
            BHostType::Asset => ASSET_CENTER,
            BHostType::Spreadsheet => SPREADSHEET_CENTER,
            BHostType::Prefs => PREFS_CENTER,
            _ => EMPTY_CENTER,
        },
        _ => EMPTY_PANES,
    }
}
