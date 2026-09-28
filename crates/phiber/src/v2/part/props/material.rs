//! Material context — `bl_ui/properties_material.py`.
// Material context set — Blender ref: `bl_ui/properties_material.py`.

use super::prop::*;
use super::set::BSectionSpec;
use super::set::sec;

static BLEND_METHOD: &[BEnumItem] = &[
    en("OPAQUE", "Opaque"),
    en("CLIP", "Alpha Clip"),
    en("HASHED", "Alpha Hashed"),
    en("BLEND", "Alpha Blend"),
];
static SHADOW_METHOD: &[BEnumItem] = &[
    en("NONE", "None"),
    en("OPAQUE", "Opaque"),
    en("CLIP", "Alpha Clip"),
    en("HASHED", "Alpha Hashed"),
];
static DISPLACEMENT: &[BEnumItem] = &[
    en("BUMP", "Bump Only"),
    en("DISPLACEMENT", "Displacement Only"),
    en("BOTH", "Displacement and Bump"),
];

static PREVIEW: &[BPropSpec] = &[e(
    "preview_render_type",
    "Preview Type",
    &[
        en("FLAT", "Flat"),
        en("SPHERE", "Sphere"),
        en("CUBE", "Cube"),
        en("HAIR", "Hair"),
        en("SHADERBALL", "Shader Ball"),
        en("CLOTH", "Cloth"),
        en("FLUID", "Fluid"),
    ],
)];

static SURFACE: &[BPropSpec] = &[
    b("use_nodes", "Use Nodes"),
    p("node_tree", "Node Tree", "ShaderNodeTree"),
    col("diffuse_color", "Base Color", true),
    f("metallic", "Metallic", BUnit::None),
    f("roughness", "Roughness", BUnit::None),
    f("specular_intensity", "Specular", BUnit::None),
];

static VOLUME: &[BPropSpec] = &[p("node_tree", "Volume Shader", "ShaderNodeTree")];

static SETTINGS: &[BPropSpec] = &[
    b("use_backface_culling", "Backface Culling"),
    b("show_transparent_back", "Transparent Shadows"),
    e("blend_method", "Blend Mode", BLEND_METHOD),
    e("shadow_method", "Shadow Mode", SHADOW_METHOD),
    f("alpha_threshold", "Alpha Threshold", BUnit::None),
    e("displacement_method", "Displacement", DISPLACEMENT),
    b("use_screen_refraction", "Screen Space Refraction"),
    b("use_sss_translucency", "Subsurface Translucency"),
];

static VIEWPORT_DISPLAY: &[BPropSpec] = &[
    col("diffuse_color", "Color", true),
    f("metallic", "Metallic", BUnit::None),
    f("roughness", "Roughness", BUnit::None),
];

pub static SECTIONS: &[BSectionSpec] = &[
    sec("Preview", PREVIEW).closed(),
    sec("Surface", SURFACE),
    sec("Volume", VOLUME).closed(),
    sec("Settings", SETTINGS).closed(),
    sec("Viewport Display", VIEWPORT_DISPLAY).closed(),
];
