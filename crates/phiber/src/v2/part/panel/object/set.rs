//! Object context set — sections + RNA properties for `bpy.types.Object`.

use super::super::prop::*;
use super::super::set::{BSectionSpec, sec};

// ── Rotation mode enum (rna_enum_object_rotation_mode_items) ────────────────
static ROTATION_MODE: &[BEnumItem] = &[
    en("QUATERNION", "Quaternion (WXYZ)"),
    en("XYZ", "XYZ Euler"),
    en("XZY", "XZY Euler"),
    en("YXZ", "YXZ Euler"),
    en("YZX", "YZX Euler"),
    en("ZXY", "ZXY Euler"),
    en("ZYX", "ZYX Euler"),
    en("AXIS_ANGLE", "Axis Angle"),
];

// ── Parent Type enum (parent_type_items) ────────────────────────────────────
static PARENT_TYPE: &[BEnumItem] = &[
    en("OBJECT", "Object"),
    en("ARMATURE", "Armature"),
    en("LATTICE", "Lattice"),
    en("VERTEX", "Vertex"),
    en("VERTEX_3", "Vertex (Triangle)"),
    en("BONE", "Bone"),
];

// ── Instance Type enum ──────────────────────────────────────────────────────
static INSTANCE_TYPE: &[BEnumItem] = &[
    en("NONE", "None"),
    en("VERTS", "Vertices"),
    en("FACES", "Faces"),
    en("COLLECTION", "Collection"),
];

// ── Display type (rna_enum_object_display_type_items) ───────────────────────
static DISPLAY_TYPE: &[BEnumItem] = &[
    en("BOUNDS", "Bounds"),
    en("WIRE", "Wire"),
    en("SOLID", "Solid"),
    en("TEXTURED", "Textured"),
];

// ── Section tables ──────────────────────────────────────────────────────────

static TRANSFORM: &[BPropSpec] = &[
    v3("location", "Location", BUnit::Length),
    v3("rotation_euler", "Rotation", BUnit::Angle),
    e("rotation_mode", "Mode", ROTATION_MODE),
    v3("scale", "Scale", BUnit::None),
    v3("dimensions", "Dimensions", BUnit::Length),
];

static DELTA_TRANSFORM: &[BPropSpec] = &[
    v3("delta_location", "Location", BUnit::Length),
    v3("delta_rotation_euler", "Rotation", BUnit::Angle),
    v3("delta_scale", "Scale", BUnit::None),
];

static RELATIONS: &[BPropSpec] = &[
    p("parent", "Parent", "Object"),
    e("parent_type", "Parent Type", PARENT_TYPE),
    s("parent_bone", "Parent Bone"),
];

static COLLECTIONS: &[BPropSpec] = &[p("users_collection", "Collections", "Collection")];

static INSTANCING: &[BPropSpec] = &[
    e("instance_type", "Instance Type", INSTANCE_TYPE),
    p("instance_collection", "Instance Collection", "Collection"),
    b("show_instancer_for_render", "Show Instancer for Render"),
    b("show_instancer_for_viewport", "Show Instancer for Viewport"),
];

static MOTION_PATHS: &[BPropSpec] = &[b("show_in_front", "Show in Front")];

static VISIBILITY: &[BPropSpec] = &[
    b("hide_viewport", "Disable in Viewports"),
    b("hide_render", "Disable in Renders"),
    b("hide_select", "Disable Selection"),
    b("show_in_front", "In Front"),
];

static VIEWPORT_DISPLAY: &[BPropSpec] = &[
    e("display_type", "Display As", DISPLAY_TYPE),
    b("show_name", "Name"),
    b("show_axis", "Axes"),
    b("show_wire", "Wireframe"),
    b("show_bounds", "Bounds"),
    b("show_texture_space", "Texture Space"),
    b("show_transparent", "Transparent"),
];

static LINE_ART: &[BPropSpec] = &[
    b("use_lineart_intersection_priority", "Intersection Priority"),
    i("lineart_intersection_priority", "Priority", 0, 255),
];

pub static SECTIONS: &[BSectionSpec] = &[
    sec("Transform", TRANSFORM),
    sec("Delta Transform", DELTA_TRANSFORM),
    sec("Relations", RELATIONS),
    sec("Collections", COLLECTIONS),
    sec("Instancing", INSTANCING),
    sec("Motion Paths", MOTION_PATHS),
    sec("Visibility", VISIBILITY),
    sec("Viewport Display", VIEWPORT_DISPLAY),
    sec("Line Art", LINE_ART),
];
