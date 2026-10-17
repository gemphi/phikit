//! The NLA editor's menu entries (`bl_ui/space_nla.py`). Selecting,
//! deleting and muting strips, adding and deleting tracks and Add ▸ Action
//! act (`ctx.op.nla`, `scene::action::nla_editor`).

use super::super::super::item::set::*;
use super::super::super::item::BMenuItem;
use super::set::{AREA, BAnimMenu, MOVE};
use crate::v2::scene::action::BSceneActions;
use crate::v2::state::{BKey as _, BState, OpKey};
use crate::v2::view::BView;
use crate::v2::view::prim::{BAction, BControl};

impl BAnimMenu {
    /// Add ▸ Action (`nla.actionclip_add`'s search): every action, each
    /// adding a strip of it at the current frame.
    pub fn nla_actions(s: &BState) -> Vec<BView> {
        let names = crate::v2::scene::source::with_loaded(|m| m.map(|m| BSceneActions::now(s).names(m)).unwrap_or_default());
        if names.is_empty() {
            return vec![BView::MenuItem(BControl::new().with_label("No Actions"))];
        }
        let entry = |name: String| BControl::new().with_icon("action").with_action(BAction::Set(OpKey::Nla.path(), format!("add_action:{name}"))).with_label(name);
        names.into_iter().map(|name| BView::MenuItem(entry(name))).collect()
    }
}

/// `NLA_MT_view`.
pub static VIEW: &[BMenuItem] = &[
    tog_host("Sidebar", "N", crate::v2::part::sidebar::show::BSidebarShown::key_for),
    op("Adjust Last Operation", ""),
    op("Channels", ""),
    op("Playback Controls", ""),
    SEP,
    op("Frame Selected", "Numpad ."),
    fire("Frame All", "Home", OpKey::TimeView, "nla:all"),
    fire("Frame Scene Range", "", OpKey::TimeView, "nla:scene"),
    fire_host("Go to Current Frame", "Numpad 0", OpKey::TimeView, ":center"),
    SEP,
    op("Realtime Updates", ""),
    op("Show Control F-Curves", ""),
    SEP,
    tog_host_on("Show Markers", "", crate::v2::view::draw::canvas::time::show::BShowMarkers::key_for),
    op("Show Local Markers", ""),
    op("Use Timecode", "Ctrl T"),
    op("Sync Visible Range", ""),
    SEP,
    op("Set Preview Range", "P"),
    act("Clear Preview Range", "Alt P", BAnimMenu::clear_preview),
    fire("Set Preview Range to Selected", "Ctrl Alt P", OpKey::PreviewRange, "selected"),
    SEP,
    sub("Area", AREA),
];
/// `NLA_MT_select`.
pub static SELECT: &[BMenuItem] = &[
    fire("All", "A", OpKey::Nla, "select:all"),
    fire("None", "Alt A", OpKey::Nla, "select:none"),
    fire("Invert", "Ctrl I", OpKey::Nla, "select:invert"),
    SEP,
    op("Box Select", "B"),
    op("Box Select (Axis Range)", "Alt B"),
    SEP,
    op("Before Current Frame", "["),
    op("After Current Frame", "]"),
];
/// `NLA_MT_add`.
pub static ADD: &[BMenuItem] = &[
    live("Action", BAnimMenu::nla_actions),
    op("Transition", "Shift T"),
    op("Sound", "Shift K"),
    SEP,
    op("Selected Objects", ""),
];
/// `NLA_MT_tracks`.
pub static TRACKS: &[BMenuItem] = &[
    fire("Add", "", OpKey::Nla, "tracks_add"),
    fire("Add Above Selected", "", OpKey::Nla, "tracks_add_above"),
    fire("Delete", "", OpKey::Nla, "tracks_delete"),
    SEP,
    sub("Move", MOVE),
    SEP,
    op("Remove Empty Animation Data", ""),
];
pub static SNAP: &[BMenuItem] = &[
    op("Selection to Current Frame", ""),
    op("Selection to Nearest Frame", ""),
    op("Selection to Nearest Second", ""),
    op("Selection to Nearest Marker", ""),
];
/// `NLA_MT_strips_transform`.
pub static TRANSFORM: &[BMenuItem] = &[op("Move", "G"), op("Extend", "E"), op("Scale", "S"), SEP, op("Swap", "Alt F"), SEP, op("Move Up", "PageUp"), op("Move Down", "PageDown")];
/// `NLA_MT_strips`, outside tweak mode.
pub static STRIPS: &[BMenuItem] = &[
    sub("Transform", TRANSFORM),
    sub("Snap", SNAP),
    SEP,
    op("Split", "Y"),
    SEP,
    op("Duplicate", "Alt D"),
    op("Linked Duplicate", "Shift D"),
    fire("Delete", "X", OpKey::Nla, "delete"),
    SEP,
    op("Make Meta", "Ctrl G"),
    op("Remove Meta", "Ctrl Alt G"),
    SEP,
    fire("Toggle Muting", "H", OpKey::Nla, "mute"),
    SEP,
    op("Bake Action", ""),
    SEP,
    op("Apply Scale", "Ctrl A"),
    op("Clear Scale", "Alt S"),
    op("Sync Action Length", ""),
    SEP,
    op("Make Single User", "U"),
    SEP,
    op("Start Editing Stashed Action", "Shift Tab"),
    op("Start Tweaking Strip Actions (Full Stack)", "Tab"),
    op("Start Tweaking Strip Actions (Lower Stack)", ""),
];
