//! The Dope Sheet's menu entries (`bl_ui/space_dopesheet.py`), and the
//! submenus every animation editor shares: the channel settings, the key
//! types and modes, and the Area menu (`INFO_MT_area`). The Action menu,
//! and the Channel menu in the Action Editor, show while the active object
//! has an action (`DOPESHEET_MT_editor_menus`).

use super::super::super::item::set::*;
use super::super::super::item::BMenuItem;
use crate::v2::host::BHostType;
use crate::v2::part::action::template::ACTION_EDITOR;
use crate::v2::part::modebar::active_mode;
use crate::v2::part::sidebar::show::BSidebarShown;
use crate::v2::scene::action::BSceneActions;
use crate::v2::state::{BKey as _, BPath, BState, CtxKey, OpKey, val::V_FALSE};
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

    fn action_editor(s: &BState) -> bool {
        active_mode(s, BHostType::Action) == ACTION_EDITOR
    }

    /// The Action menu: in the Action Editor, over an action.
    pub fn action_menu(s: &BState) -> bool {
        Self::action_editor(s) && BSceneActions::active(s).is_some()
    }

    /// The Channel menu: in the Action Editor only over an action.
    pub fn channel_menu(s: &BState) -> bool {
        !Self::action_editor(s) || BSceneActions::active(s).is_some()
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
/// `anim.channels_setting_toggle` — flip Protect / Mute on the shown channels.
pub static CHANNEL_SETTING_TOGGLE: &[BMenuItem] = &[
    fire("Protect", "", OpKey::Channels, "protect:toggle"),
    fire("Mute", "", OpKey::Channels, "mute:toggle"),
];
/// `anim.channels_setting_enable`.
pub static CHANNEL_SETTING_ENABLE: &[BMenuItem] = &[
    fire("Protect", "", OpKey::Channels, "protect:on"),
    fire("Mute", "", OpKey::Channels, "mute:on"),
];
/// `anim.channels_setting_disable`.
pub static CHANNEL_SETTING_DISABLE: &[BMenuItem] = &[
    fire("Protect", "", OpKey::Channels, "protect:off"),
    fire("Mute", "", OpKey::Channels, "mute:off"),
];
pub static MOVE: &[BMenuItem] = &[op("To Top", "Shift PageUp"), op("Up", "PageUp"), op("Down", "PageDown"), op("To Bottom", "Shift PageDown")];
pub static EXTRAPOLATION: &[BMenuItem] = &[
    fire("Constant Extrapolation", "", OpKey::Channels, "extend:const"),
    fire("Linear Extrapolation", "", OpKey::Channels, "extend:lin"),
    op("Make Cyclic (F-Modifier)", ""),
    op("Clear Cyclic (F-Modifier)", ""),
];
pub static HANDLE_TYPE: &[BMenuItem] = &[
    fire("Free", "", OpKey::Keys, "handle:free"),
    fire("Aligned", "", OpKey::Keys, "handle:aligned"),
    fire("Vector", "", OpKey::Keys, "handle:vector"),
    fire("Automatic", "", OpKey::Keys, "handle:auto"),
    fire("Auto Clamped", "", OpKey::Keys, "handle:auto_clamped"),
];
pub static INTERPOLATION: &[BMenuItem] = &[
    fire("Constant", "", OpKey::Keys, "ipo:const"),
    fire("Linear", "", OpKey::Keys, "ipo:lin"),
    fire("Bézier", "", OpKey::Keys, "ipo:bez"),
    SEP,
    fire("Sinusoidal", "", OpKey::Keys, "ipo:sine"),
    fire("Quadratic", "", OpKey::Keys, "ipo:quad"),
    fire("Cubic", "", OpKey::Keys, "ipo:cubic"),
    fire("Quartic", "", OpKey::Keys, "ipo:quart"),
    fire("Quintic", "", OpKey::Keys, "ipo:quint"),
    fire("Exponential", "", OpKey::Keys, "ipo:expo"),
    fire("Circular", "", OpKey::Keys, "ipo:circ"),
    SEP,
    fire("Back", "", OpKey::Keys, "ipo:back"),
    fire("Bounce", "", OpKey::Keys, "ipo:bounce"),
    fire("Elastic", "", OpKey::Keys, "ipo:elastic"),
];
pub static EASING: &[BMenuItem] = &[
    fire("Automatic Easing", "", OpKey::Keys, "easing:auto"),
    fire("Ease In", "", OpKey::Keys, "easing:in"),
    fire("Ease Out", "", OpKey::Keys, "easing:out"),
    fire("Ease In and Out", "", OpKey::Keys, "easing:inout"),
];

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
    fire("Frame All", "Home", OpKey::TimeView, "action:all"),
    fire("Frame Scene Range", "", OpKey::TimeView, "action:scene"),
    fire_host("Go to Current Frame", "Numpad 0", OpKey::TimeView, ":center"),
    SEP,
    op("Multi-Word Match Search", ""),
    SEP,
    op("Realtime Updates", ""),
    op("Show Sliders", ""),
    op("Show Handles and Interpolation", ""),
    op("Show Curve Extremes", ""),
    op("Auto-Merge Keyframes", ""),
    SEP,
    tog_host_on("Show Markers", "", crate::v2::view::draw::canvas::time::show::BShowMarkers::key_for),
    op("Use Timecode", "Ctrl T"),
    op("Sync Visible Range", ""),
    SEP,
    op("Set Preview Range", "P"),
    act("Clear Preview Range", "Alt P", BAnimMenu::clear_preview),
    fire("Set Preview Range to Selected", "Ctrl Alt P", OpKey::PreviewRange, "selected"),
    SEP,
    fire_host("Toggle Graph Editor", "Ctrl Tab", OpKey::EditorType, ":ipo"),
    SEP,
    sub("Cache", CACHE),
    SEP,
    sub("Area", AREA),
];
/// `DOPESHEET_MT_select`.
pub static SELECT: &[BMenuItem] = &[
    fire("All", "A", OpKey::Keys, "all"),
    fire("None", "Alt A", OpKey::Keys, "none"),
    fire("Invert", "Ctrl I", OpKey::Keys, "invert"),
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
    fire("Clean Channels", "", OpKey::Keys, "clean:channels"),
    SEP,
    op("Group Channels", "Ctrl G"),
    op("Ungroup Channels", "Ctrl Alt G"),
    SEP,
    sub("Toggle Channel Setting", CHANNEL_SETTING_TOGGLE),
    sub("Enable Channel Setting", CHANNEL_SETTING_ENABLE),
    sub("Disable Channel Setting", CHANNEL_SETTING_DISABLE),
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
pub static TRANSFORM: &[BMenuItem] = &[fire("Move", "G", OpKey::Keys, "grab:start"), op("Extend", "E"), op("Slide", "Shift T"), op("Scale", "S")];
pub static SNAP: &[BMenuItem] = &[
    fire("Selection to Current Frame", "", OpKey::Keys, "snap:current"),
    fire("Selection to Nearest Frame", "", OpKey::Keys, "snap:nearest_frame"),
    fire("Selection to Nearest Second", "", OpKey::Keys, "snap:nearest_second"),
    fire("Selection to Nearest Marker", "", OpKey::Keys, "snap:nearest_marker"),
];
pub static MIRROR: &[BMenuItem] = &[
    fire("By Times Over Current Frame", "", OpKey::Keys, "mirror:current"),
    op("By Values Over Zero Value", ""),
    fire("By Times Over First Selected Marker", "", OpKey::Keys, "mirror:marker"),
];
pub static KEYFRAME_TYPE: &[BMenuItem] = &[op("Keyframe", ""), op("Breakdown", ""), op("Moving Hold", ""), op("Extreme", ""), op("Jitter", ""), op("Generated", "")];
/// `DOPESHEET_MT_key`.
pub static KEY: &[BMenuItem] = &[
    sub("Transform", TRANSFORM),
    sub("Snap", SNAP),
    sub("Mirror", MIRROR),
    SEP,
    // `action.keyframe_insert` asks which channels; I keys them all.
    fire("Insert Keyframes", "I", OpKey::Keyframe, "channels"),
    SEP,
    fire("Jump to Keyframes", "Ctrl G", OpKey::Keys, "jump"),
    SEP,
    op("Copy Keyframes", "Ctrl C"),
    op("Paste Keyframes", "Ctrl V"),
    op("Paste Flipped", "Shift Ctrl V"),
    fire("Duplicate", "Shift D", OpKey::Keys, "duplicate"),
    fire("Delete Keyframes", "X", OpKey::Keys, "delete"),
    SEP,
    sub("Keyframe Type", KEYFRAME_TYPE),
    sub("Handle Type", HANDLE_TYPE),
    sub("Interpolation Mode", INTERPOLATION),
    sub("Easing Mode", EASING),
    SEP,
    fire("Clean Keyframes", "", OpKey::Keys, "clean:keys"),
    op("Bake Keyframes", "Shift Alt O"),
    SEP,
    op("Discontinuity (Euler) Filter", ""),
];
/// `DOPESHEET_MT_action`.
pub static ACTION: &[BMenuItem] = &[
    op("Merge Animation", ""),
    op("Separate Slots", ""),
    SEP,
    op("Move Slots to new Action", ""),
    SEP,
    fire("Push Down Action", "", OpKey::Action, "push_down"),
    fire("Stash Action", "", OpKey::Action, "stash"),
];
