//! The 3D Viewport's Tool and View tabs (`editor`).

use crate::v2::part::props::prop::{BEnumItem, BPropSpec, BUnit, b, e, en, f, v3};
use crate::v2::part::props::set::{BSectionSpec, sec};

/// `ToolSettings.use_transform_*`: what a transform affects.
static AFFECT: &[BPropSpec] = &[
    b("tool_settings.use_transform_data_origin", "Origins"),
    b("tool_settings.use_transform_pivot_point_align", "Locations"),
    b("tool_settings.use_transform_skip_children", "Parents"),
];

static OBJECT_MODE: &[BEnumItem] = &[
    en("OBJECT", "Object Mode"),
    en("EDIT", "Edit Mode"),
    en("SCULPT", "Sculpt Mode"),
    en("VERTEX_PAINT", "Vertex Paint"),
    en("WEIGHT_PAINT", "Weight Paint"),
    en("TEXTURE_PAINT", "Texture Paint"),
    en("POSE", "Pose Mode"),
];

/// `WorkSpace.object_mode` and the add-on filter.
static WORKSPACE: &[BPropSpec] = &[e("workspace.object_mode", "Mode", OBJECT_MODE), b("workspace.use_filter_by_owner", "Filter Add-ons")];

/// The Tool tab: the active tool, then what a transform affects, then the
/// workspace.
pub static VIEW3D_TOOL: &[BSectionSpec] = &[
    sec("Active Tool", crate::v2::part::props::tool::ACTIVE_TOOL),
    sec("Options", AFFECT),
    sec("Workspace", WORKSPACE).closed(),
];

static ROTATION_MODE: &[BEnumItem] = &[
    en("XYZ", "XYZ Euler"),
    en("XZY", "XZY Euler"),
    en("YXZ", "YXZ Euler"),
    en("YZX", "YZX Euler"),
    en("ZXY", "ZXY Euler"),
    en("ZYX", "ZYX Euler"),
    en("AXIS_ANGLE", "Axis Angle"),
    en("QUATERNION", "Quaternion"),
];

/// `View3DCursor`: where new things land.
static CURSOR: &[BPropSpec] = &[
    v3("scene.cursor.location", "Location", BUnit::Length),
    v3("scene.cursor.rotation_euler", "Rotation", BUnit::Angle),
    e("scene.cursor.rotation_mode", "Mode", ROTATION_MODE),
];

/// `View3D`'s clipping and local camera, beside the focal length the View
/// panel already edits.
static CLIP: &[BPropSpec] = &[
    f("space_data.clip_start", "Clip Start", BUnit::Length),
    f("space_data.clip_end", "End", BUnit::Length),
    b("space_data.use_local_camera", "Local Camera"),
];

static COLLECTIONS: &[BPropSpec] = &[b("space_data.use_local_collections", "Local Collections")];

static ANNOTATION_PLACEMENT: &[BEnumItem] = &[en("VIEW", "View"), en("CURSOR", "3D Cursor"), en("SURFACE", "Surface")];

static ANNOTATIONS: &[BPropSpec] = &[
    e("tool_settings.annotation_stroke_placement_view3d", "Placement", ANNOTATION_PLACEMENT),
    f("annotation.layer.thickness", "Thickness", BUnit::None),
    b("annotation.layer.annotation_hide", "Hide"),
];

/// The View tab's panels after the View panel itself (`panel::view_card`).
pub static VIEW3D_VIEW: &[BSectionSpec] = &[
    sec("Clipping", CLIP),
    sec("3D Cursor", CURSOR),
    sec("Collections", COLLECTIONS),
    sec("Annotations", ANNOTATIONS).closed(),
];
