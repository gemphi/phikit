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

/// `INFO_MT_area`, at the foot of every View menu. Maximize grows the
/// editor whose menu it is (`area::maximize`), as Ctrl Space over it does.
pub static AREA: &[BMenuItem] = &[
    op("Horizontal Split", ""),
    op("Vertical Split", ""),
    SEP,
    fire("Maximize Area", "Ctrl Space", OpKey::AreaMaximize, ""),
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

#[path = "set_more.rs"]
mod more;
pub use more::*;
