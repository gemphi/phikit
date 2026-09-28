//! The NLA editor's menu entries (`bl_ui/space_nla.py`).

use super::super::super::item::set::*;
use super::super::super::item::BMenuItem;
use super::set::{AREA, BAnimMenu, MOVE};

/// `NLA_MT_view`.
pub static VIEW: &[BMenuItem] = &[
    op("Sidebar", "N"),
    op("Adjust Last Operation", ""),
    op("Channels", ""),
    op("Playback Controls", ""),
    SEP,
    op("Frame Selected", "Numpad ."),
    op("Frame All", "Home"),
    op("Frame Scene Range", ""),
    op("Go to Current Frame", "Numpad 0"),
    SEP,
    op("Realtime Updates", ""),
    op("Show Control F-Curves", ""),
    SEP,
    op("Show Markers", ""),
    op("Show Local Markers", ""),
    op("Use Timecode", "Ctrl T"),
    op("Sync Visible Range", ""),
    SEP,
    op("Set Preview Range", "P"),
    act("Clear Preview Range", "Alt P", BAnimMenu::clear_preview),
    op("Set Preview Range to Selected", "Ctrl Alt P"),
    SEP,
    sub("Area", AREA),
];
/// `NLA_MT_select`.
pub static SELECT: &[BMenuItem] = &[
    op("All", "A"),
    op("None", "Alt A"),
    op("Invert", "Ctrl I"),
    SEP,
    op("Box Select", "B"),
    op("Box Select (Axis Range)", "Alt B"),
    SEP,
    op("Before Current Frame", "["),
    op("After Current Frame", "]"),
];
/// `NLA_MT_add`.
pub static ADD: &[BMenuItem] = &[
    op("Action", "Shift A"),
    op("Transition", "Shift T"),
    op("Sound", "Shift K"),
    SEP,
    op("Selected Objects", ""),
];
/// `NLA_MT_tracks`.
pub static TRACKS: &[BMenuItem] = &[
    op("Add", ""),
    op("Add Above Selected", ""),
    op("Delete", ""),
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
    op("Delete", "X"),
    SEP,
    op("Make Meta", "Ctrl G"),
    op("Remove Meta", "Ctrl Alt G"),
    SEP,
    op("Toggle Muting", "H"),
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
