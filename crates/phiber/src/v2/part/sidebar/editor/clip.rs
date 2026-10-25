//! The Movie Clip editor's Track and Solve tabs (`editor`).

use crate::v2::part::props::prop::{BEnumItem, BPropSpec, BUnit, b, e, en, f, i, s};
use crate::v2::part::props::set::{BSectionSpec, sec};

static TRACK: &[BPropSpec] = &[
    s("track.name", "Name"),
    b("track.use_red_channel", "Red"),
    b("track.use_green_channel", "Green"),
    b("track.use_blue_channel", "Blue"),
    b("track.use_grayscale_preview", "Grayscale"),
    b("track.use_alpha_preview", "Alpha"),
    f("track.weight", "Weight", BUnit::None),
    f("track.weight_stab", "Stab Weight", BUnit::None),
];

static MOTION: &[BEnumItem] = &[en("Loc", "Loc"), en("LocRot", "LocRot"), en("LocScale", "LocScale"), en("LocRotScale", "LocRotScale"), en("Affine", "Affine"), en("Perspective", "Perspective")];
static MATCH: &[BEnumItem] = &[en("KEYFRAME", "Keyframe"), en("PREV_FRAME", "Previous Frame")];

static TRACK_SETTINGS: &[BPropSpec] = &[
    e("track.motion_model", "Motion Model", MOTION),
    e("track.pattern_match", "Match", MATCH),
    b("track.use_brute", "Prepass"),
    b("track.use_normalization", "Normalize"),
    f("track.correlation_min", "Correlation Min", BUnit::None),
    i("track.margin", "Margin", 0, 300),
];

static TRACK_EXTRAS: &[BPropSpec] = &[i("track.frames_limit", "Frames Limit", 0, 32767), f("track.minimum_correlation", "Minimum Correlation", BUnit::None)];

pub static CLIP_TRACK: &[BSectionSpec] = &[
    sec("Track", TRACK),
    sec("Tracking Settings", TRACK_SETTINGS),
    sec("Tracking Settings Extra", TRACK_EXTRAS).closed(),
];

static REFINE: &[BEnumItem] = &[en("NONE", "Nothing"), en("FOCAL_LENGTH", "Focal Length"), en("FOCAL_LENGTH_RADIAL_K1", "Focal Length, K1"), en("PRINCIPAL_POINT", "Optical Center")];

static SOLVE: &[BPropSpec] = &[
    b("tracking.settings.use_tripod_solver", "Tripod"),
    b("tracking.settings.use_keyframe_selection", "Keyframe"),
    i("tracking.reconstruction.keyframe_a", "Keyframe A", 0, 1_048_574),
    i("tracking.reconstruction.keyframe_b", "B", 0, 1_048_574),
    e("tracking.settings.refine_intrinsics", "Refine", REFINE),
];

static CLEANUP: &[BPropSpec] = &[
    i("tracking.settings.clean_frames", "Tracked Frames", 0, 1000),
    f("tracking.settings.clean_error", "Reprojection Error", BUnit::None),
];

pub static CLIP_SOLVE: &[BSectionSpec] = &[sec("Solve", SOLVE), sec("Clean Up", CLEANUP)];
