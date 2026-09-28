//! The Dope Sheet's menu entries (`bl_ui/space_dopesheet.py`), and the
//! submenus every animation editor shares: the channel settings, the key
//! types and modes, and the Area menu (`INFO_MT_area`).

use super::super::super::item::set::*;
use super::super::super::item::BMenuItem;
use crate::v2::host::BHostType;
use crate::v2::part::sidebar::show::BSidebarShown;
use crate::v2::state::{BKey as _, BPath, CtxKey, val::V_FALSE};
use crate::v2::view::prim::BAction;

/// What the animation editors' menus act with.
pub struct BAnimMenu;

impl BAnimMenu {
    fn dope_sheet_sidebar() -> BPath {
        BSidebarShown(BHostType::Action).key()
    }

    /// `anim.previewrange_clear`: the scene plays its whole range again.
    pub fn clear_preview() -> BAction {
        BAction::Set(CtxKey::PreviewRange.path(), V_FALSE.to_string())
    }
}

/// `INFO_MT_area`, at the foot of every View menu.
pub static AREA: &[BMenuItem] = &[
    op("Horizontal Split", ""),
    op("Vertical Split", ""),
    SEP,
    op("Maximize Area", "Ctrl Space"),
    op("Focus Mode", "Ctrl Alt Space"),
    op("Duplicate Area into New Window", ""),
    SEP,
    op("Close Area", ""),
];
pub static LASSO: &[BMenuItem] = &[op("Set", ""), op("Extend", ""), op("Subtract", "")];
/// `anim.channels_setting_toggle / _enable / _disable`'s `type`.
pub static CHANNEL_SETTING: &[BMenuItem] = &[op("Protect", ""), op("Mute", "")];
pub static MOVE: &[BMenuItem] = &[op("To Top", "Shift PageUp"), op("Up", "PageUp"), op("Down", "PageDown"), op("To Bottom", "Shift PageDown")];
pub static EXTRAPOLATION: &[BMenuItem] = &[
    op("Constant Extrapolation", ""),
    op("Linear Extrapolation", ""),
    op("Make Cyclic (F-Modifier)", ""),
    op("Clear Cyclic (F-Modifier)", ""),
];
pub static HANDLE_TYPE: &[BMenuItem] = &[op("Free", ""), op("Aligned", ""), op("Vector", ""), op("Automatic", ""), op("Auto Clamped", "")];
pub static INTERPOLATION: &[BMenuItem] = &[
    op("Constant", ""),
    op("Linear", ""),
    op("Bézier", ""),
    SEP,
    op("Sinusoidal", ""),
    op("Quadratic", ""),
    op("Cubic", ""),
    op("Quartic", ""),
    op("Quintic", ""),
    op("Exponential", ""),
    op("Circular", ""),
    SEP,
    op("Back", ""),
    op("Bounce", ""),
    op("Elastic", ""),
];
pub static EASING: &[BMenuItem] = &[op("Automatic Easing", ""), op("Ease In", ""), op("Ease Out", ""), op("Ease In and Out", "")];

pub static CACHE: &[BMenuItem] = &[
    op("Show Cache", ""),
    SEP,
    op("Softbody", ""),
    op("Particles", ""),
    op("Cloth", ""),
    op("Simulation Nodes", ""),
    op("Smoke", ""),
    op("Dynamic Paint", ""),
    op("Rigid Body", ""),
];
/// `DOPESHEET_MT_view`.
pub static VIEW: &[BMenuItem] = &[
    tog_keyed("Sidebar", "N", BAnimMenu::dope_sheet_sidebar),
    op("Adjust Last Operation", ""),
    op("Channels", ""),
    op("Playback Controls", ""),
    SEP,
    op("Frame Selected", "Numpad ."),
    op("Frame All", "Home"),
    op("Frame Scene Range", ""),
    op("Go to Current Frame", "Numpad 0"),
    SEP,
    op("Multi-Word Match Search", ""),
    SEP,
    op("Realtime Updates", ""),
    op("Show Sliders", ""),
    op("Show Handles and Interpolation", ""),
    op("Show Curve Extremes", ""),
    op("Auto-Merge Keyframes", ""),
    SEP,
    op("Show Markers", ""),
    op("Use Timecode", "Ctrl T"),
    op("Sync Visible Range", ""),
    SEP,
    op("Set Preview Range", "P"),
    act("Clear Preview Range", "Alt P", BAnimMenu::clear_preview),
    op("Set Preview Range to Selected", "Ctrl Alt P"),
    SEP,
    op("Toggle Graph Editor", "Ctrl Tab"),
    SEP,
    sub("Cache", CACHE),
    SEP,
    sub("Area", AREA),
];
/// `DOPESHEET_MT_select`.
pub static SELECT: &[BMenuItem] = &[
    op("All", "A"),
    op("None", "Alt A"),
    op("Invert", "Ctrl I"),
    SEP,
    op("Box Select", "B"),
    op("Box Select (Axis Range)", "Alt B"),
    op("Circle Select", "C"),
    sub("Lasso Select", LASSO),
    SEP,
    op("More", "Ctrl Numpad +"),
    op("Less", "Ctrl Numpad -"),
    SEP,
    op("Select Linked", "L"),
    SEP,
    op("Columns on Selected Keys", "K"),
    op("Column on Current Frame", "Ctrl K"),
    op("Columns on Selected Markers", "Shift K"),
    op("Between Selected Markers", "Alt K"),
    SEP,
    op("Before Current Frame", "["),
    op("After Current Frame", "]"),
];
/// `DOPESHEET_MT_channel`.
pub static CHANNEL: &[BMenuItem] = &[
    op("Delete Channels", "X"),
    op("Clean Channels", ""),
    SEP,
    op("Group Channels", "Ctrl G"),
    op("Ungroup Channels", "Ctrl Alt G"),
    SEP,
    sub("Toggle Channel Setting", CHANNEL_SETTING),
    sub("Enable Channel Setting", CHANNEL_SETTING),
    sub("Disable Channel Setting", CHANNEL_SETTING),
    SEP,
    op("Toggle Channel Editability", "Tab"),
    sub("Extrapolation Mode", EXTRAPOLATION),
    SEP,
    op("Expand Channels", "Numpad +"),
    op("Collapse Channels", "Numpad -"),
    SEP,
    sub("Move...", MOVE),
    SEP,
    op("Revive Disabled F-Curves", ""),
    SEP,
    op("Bake Channels", ""),
    SEP,
    op("Frame Selected Channels", ""),
];
pub static TRANSFORM: &[BMenuItem] = &[op("Move", "G"), op("Extend", "E"), op("Slide", "Shift T"), op("Scale", "S")];
pub static SNAP: &[BMenuItem] = &[
    op("Selection to Current Frame", ""),
    op("Selection to Nearest Frame", ""),
    op("Selection to Nearest Second", ""),
    op("Selection to Nearest Marker", ""),
];
pub static MIRROR: &[BMenuItem] = &[op("By Times Over Current Frame", ""), op("By Values Over Zero Value", ""), op("By Times Over First Selected Marker", "")];
pub static KEYFRAME_TYPE: &[BMenuItem] = &[op("Keyframe", ""), op("Breakdown", ""), op("Moving Hold", ""), op("Extreme", ""), op("Jitter", ""), op("Generated", "")];
/// `DOPESHEET_MT_key`.
pub static KEY: &[BMenuItem] = &[
    sub("Transform", TRANSFORM),
    sub("Snap", SNAP),
    sub("Mirror", MIRROR),
    SEP,
    op("Insert Keyframes", "I"),
    SEP,
    op("Jump to Keyframes", "Ctrl G"),
    SEP,
    op("Copy Keyframes", "Ctrl C"),
    op("Paste Keyframes", "Ctrl V"),
    op("Paste Flipped", "Shift Ctrl V"),
    op("Duplicate", "Shift D"),
    op("Delete Keyframes", "X"),
    SEP,
    sub("Keyframe Type", KEYFRAME_TYPE),
    sub("Handle Type", HANDLE_TYPE),
    sub("Interpolation Mode", INTERPOLATION),
    sub("Easing Mode", EASING),
    SEP,
    op("Clean Keyframes", ""),
    op("Bake Keyframes", "Shift Alt O"),
    SEP,
    op("Discontinuity (Euler) Filter", ""),
];
