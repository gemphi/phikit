//! The Sequencer's Strip, Modifiers and Preview tabs (`editor`).

use crate::v2::part::props::prop::{BEnumItem, BPropSpec, BUnit, b, e, en, f, i, s, v3};
use crate::v2::part::props::set::{BSectionSpec, sec};

static BLEND: &[BEnumItem] = &[
    en("REPLACE", "Replace"),
    en("CROSS", "Cross"),
    en("DARKEN", "Darken"),
    en("MULTIPLY", "Multiply"),
    en("LIGHTEN", "Lighten"),
    en("SCREEN", "Screen"),
    en("ADD", "Add"),
    en("OVERLAY", "Overlay"),
    en("SUBTRACT", "Subtract"),
    en("ALPHA_OVER", "Alpha Over"),
];

static STRIP: &[BPropSpec] = &[
    s("strip.name", "Name"),
    b("strip.mute", "Mute"),
    b("strip.lock", "Lock"),
    e("strip.blend_type", "Blend", BLEND),
    f("strip.blend_alpha", "Opacity", BUnit::None),
];

static TIME: &[BPropSpec] = &[
    i("strip.channel", "Channel", 1, 128),
    i("strip.frame_start", "Start", -1_048_574, 1_048_574),
    i("strip.frame_final_duration", "Duration", 1, 1_048_574),
    i("strip.frame_offset_start", "Strip Offset Start", -1_048_574, 1_048_574),
    i("strip.frame_offset_end", "End", -1_048_574, 1_048_574),
    b("strip.lock_time", "Lock"),
];

static FIT: &[BEnumItem] = &[en("FIT", "Scale to Fit"), en("FILL", "Scale to Fill"), en("STRETCH", "Stretch to Fill"), en("ORIGINAL", "Use Original Size")];

static TRANSFORM: &[BPropSpec] = &[
    f("strip.transform.offset_x", "Position X", BUnit::None),
    f("strip.transform.offset_y", "Y", BUnit::None),
    f("strip.transform.scale_x", "Scale X", BUnit::None),
    f("strip.transform.scale_y", "Y", BUnit::None),
    f("strip.transform.rotation", "Rotation", BUnit::Angle),
    e("strip.fit_method", "Fit Method", FIT),
    b("strip.use_flip_x", "Mirror X"),
    b("strip.use_flip_y", "Mirror Y"),
];

static COLOR: &[BPropSpec] = &[
    f("strip.color_saturation", "Saturation", BUnit::None),
    f("strip.color_multiply", "Multiply", BUnit::None),
    b("strip.use_float", "Convert to Float"),
];

static SOUND: &[BPropSpec] = &[
    f("strip.volume", "Volume", BUnit::None),
    f("strip.pan", "Pan", BUnit::None),
    b("strip.sound.use_mono", "Mono"),
    b("strip.show_waveform", "Display Waveform"),
];

pub static SEQ_STRIP: &[BSectionSpec] = &[
    sec("Strip", STRIP).headless(),
    sec("Time", TIME),
    sec("Transform", TRANSFORM),
    sec("Color", COLOR),
    sec("Sound", SOUND).closed(),
];

static MODIFIERS: &[BPropSpec] = &[b("strip.use_linear_modifiers", "Linear Modifiers"), b("strip.modifiers.show_expanded", "Expanded")];

pub static SEQ_MODIFIERS: &[BSectionSpec] = &[sec("Modifiers", MODIFIERS)];

static PREVIEW_CHANNELS: &[BEnumItem] = &[en("COLOR", "Color"), en("COLOR_ALPHA", "Color & Alpha")];

static VIEW_SETTINGS: &[BPropSpec] = &[
    e("space_data.preview_channels", "Channels", PREVIEW_CHANNELS),
    i("space_data.display_channel", "Display Channel", 0, 128),
    b("space_data.show_transform_preview", "Transform Preview"),
];

static OVERLAY_FRAME: &[BPropSpec] = &[b("scene.sequence_editor.show_overlay_frame", "Frame Overlay"), i("scene.sequence_editor.overlay_frame", "Frame Offset", -1_048_574, 1_048_574)];

static SAFE_AREAS: &[BPropSpec] = &[
    b("space_data.show_safe_areas", "Safe Areas"),
    v3("scene.safe_areas.title", "Title Safe", BUnit::None),
    v3("scene.safe_areas.action", "Action Safe", BUnit::None),
];

pub static SEQ_PREVIEW: &[BSectionSpec] = &[
    sec("View Settings", VIEW_SETTINGS),
    sec("Frame Overlay", OVERLAY_FRAME).closed(),
    sec("Safe Areas", SAFE_AREAS).closed(),
];
