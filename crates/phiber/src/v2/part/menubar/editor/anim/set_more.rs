//! `anim/set.rs`'s second half — the Select / Channel / Transform / Snap /
//! Mirror / Keyframe / Action menus, split so each file sits under the
//! 240-line read cap (`tests/level_structure.rs`).

use super::super::super::super::item::BMenuItem;
use super::*;
use crate::v2::state::OpKey;

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
