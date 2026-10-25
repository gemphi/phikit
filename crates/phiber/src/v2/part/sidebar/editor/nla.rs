//! The NLA's Edited Action and Item tabs (`editor`).

use crate::v2::part::props::prop::{BEnumItem, BPropSpec, BUnit, b, e, en, f, s};
use crate::v2::part::props::set::{BSectionSpec, sec};

static EXTEND: &[BEnumItem] = &[en("NOTHING", "Nothing"), en("HOLD", "Hold"), en("HOLD_FORWARD", "Hold Forward")];
static NLA_BLEND: &[BEnumItem] = &[en("REPLACE", "Replace"), en("COMBINE", "Combine"), en("ADD", "Add"), en("SUBTRACT", "Subtract"), en("MULTIPLY", "Multiply")];

static ANIMDATA: &[BPropSpec] = &[
    s("animation_data.action", "Action"),
    e("animation_data.action_extrapolation", "Extrapolation", EXTEND),
    e("animation_data.action_blend_type", "Blending", NLA_BLEND),
    f("animation_data.action_influence", "Influence", BUnit::None),
];

pub static NLA_EDITED: &[BSectionSpec] = &[sec("Animation Data", ANIMDATA)];

static STRIP_NAME: &[BPropSpec] = &[s("nla_strip.name", "Name"), b("nla_strip.mute", "Mute")];

static NLA_STRIP: &[BPropSpec] = &[
    f("nla_strip.frame_start_ui", "Frame Start", BUnit::None),
    f("nla_strip.frame_end_ui", "End", BUnit::None),
    e("nla_strip.extrapolation", "Extrapolation", EXTEND),
    e("nla_strip.blend_type", "Blending", NLA_BLEND),
    f("nla_strip.blend_in", "Blend In", BUnit::None),
    f("nla_strip.blend_out", "Out", BUnit::None),
    b("nla_strip.use_auto_blend", "Auto Blend In/Out"),
    b("nla_strip.use_reverse", "Reversed"),
];

static ACTION_CLIP: &[BPropSpec] = &[
    f("nla_strip.action_frame_start", "Frame Start", BUnit::None),
    f("nla_strip.action_frame_end", "End", BUnit::None),
    f("nla_strip.scale", "Playback Scale", BUnit::None),
    f("nla_strip.repeat", "Repeat", BUnit::None),
    b("nla_strip.use_sync_length", "Sync Length"),
];

pub static NLA_ITEM: &[BSectionSpec] = &[sec("Active Strip", STRIP_NAME).headless(), sec("Strip", NLA_STRIP), sec("Action Clip", ACTION_CLIP)];
