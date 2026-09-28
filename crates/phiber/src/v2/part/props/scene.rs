//! Scene context — `bl_ui/properties_scene.py`.
// Scene context set — sections + RNA properties for `bpy.types.Scene`.
// Blender ref: `bl_ui/properties_scene.py`, `rna_scene.cc`.

use super::prop::*;
use super::set::BSectionSpec;
use super::set::sec;

static UNIT_SYSTEM: &[BEnumItem] = &[
    en("NONE", "None"),
    en("METRIC", "Metric"),
    en("IMPERIAL", "Imperial"),
];
static ROTATION_UNIT: &[BEnumItem] = &[en("DEGREES", "Degrees"), en("RADIANS", "Radians")];
static AUDIO_DIST: &[BEnumItem] = &[
    en("NONE", "None"),
    en("INVERSE", "Inverse"),
    en("INVERSE_CLAMPED", "Inverse Clamped"),
    en("LINEAR", "Linear"),
    en("LINEAR_CLAMPED", "Linear Clamped"),
    en("EXPONENT", "Exponent"),
    en("EXPONENT_CLAMPED", "Exponent Clamped"),
];

static SCENE: &[BPropSpec] = &[
    p("camera", "Camera", "Object"),
    p("background_set", "Background", "Scene"),
    p("active_clip", "Active Clip", "MovieClip"),
];

static UNITS: &[BPropSpec] = &[
    e("unit_settings.system", "Unit System", UNIT_SYSTEM),
    f("unit_settings.scale_length", "Unit Scale", BUnit::None),
    e("unit_settings.system_rotation", "Rotation", ROTATION_UNIT),
    e("unit_settings.length_unit", "Length", &[]), // itemf at runtime
    e("unit_settings.mass_unit", "Mass", &[]),
    e("unit_settings.time_unit", "Time", &[]),
    e("unit_settings.temperature_unit", "Temperature", &[]),
];

static KEYING_SETS: &[BPropSpec] = &[
    p("active_keying_set", "Active Keying Set", "KeyingSet"),
    b("use_active_keying_set_only", "Only Active Keying Set"),
];

static AUDIO: &[BPropSpec] = &[
    f("audio_volume", "Volume", BUnit::None),
    e("audio_distance_model", "Distance Model", AUDIO_DIST),
    f("audio_doppler_speed", "Speed", BUnit::Velocity),
    f("audio_doppler_factor", "Doppler", BUnit::None),
];

static RIGID_BODY_WORLD: &[BPropSpec] = &[
    p("rigidbody_world", "Rigid Body World", "RigidBodyWorld"),
    i("rigidbody_world.substeps_per_frame", "Sub Steps", 1, 1000),
    i(
        "rigidbody_world.solver_iterations",
        "Solver Iterations",
        1,
        1000,
    ),
    f("rigidbody_world.time_scale", "Speed", BUnit::None),
    b("rigidbody_world.use_split_impulse", "Split Impulse"),
];

pub static SECTIONS: &[BSectionSpec] = &[
    sec("Scene", SCENE),
    sec("Units", UNITS).closed(),
    sec("Keying Sets", KEYING_SETS).closed(),
    sec("Audio", AUDIO).closed(),
    sec("Rigid Body World", RIGID_BODY_WORLD).closed(),
];
