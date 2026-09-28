//! Texture context — `bl_ui/properties_texture.py`.
// Texture context set — Blender ref: `bl_ui/properties_texture.py`.

use super::prop::*;
use super::set::BSectionSpec;
use super::set::sec;

static TEXTURE_TYPE: &[BEnumItem] = &[
    en("NONE", "None"),
    en("IMAGE", "Image or Movie"),
    en("BLEND", "Blend"),
    en("CLOUDS", "Clouds"),
    en("DISTORTED_NOISE", "Distorted Noise"),
    en("MAGIC", "Magic"),
    en("MARBLE", "Marble"),
    en("MUSGRAVE", "Musgrave"),
    en("NOISE", "Noise"),
    en("STUCCI", "Stucci"),
    en("VORONOI", "Voronoi"),
    en("WOOD", "Wood"),
];
static INTERPOLATION: &[BEnumItem] = &[
    en("Linear", "Linear"),
    en("Closest", "Closest"),
    en("Cubic", "Cubic"),
    en("Smart", "Smart"),
];
static EXTENSION: &[BEnumItem] = &[
    en("REPEAT", "Repeat"),
    en("EXTEND", "Extend"),
    en("CLIP", "Clip"),
    en("CLIP_CUBE", "Clip Cube"),
    en("CHECKER", "Checker"),
];

static PREVIEW: &[BPropSpec] = &[e(
    "preview_type",
    "Preview Type",
    &[
        en("TEXTURE", "Texture"),
        en("MATERIAL", "Material"),
        en("BOTH", "Both"),
    ],
)];

static TYPE: &[BPropSpec] = &[e("type", "Type", TEXTURE_TYPE)];

static IMAGE: &[BPropSpec] = &[
    p("image", "Image", "Image"),
    e("extension", "Extension", EXTENSION),
    e("interpolation", "Interpolation", INTERPOLATION),
    b("use_alpha", "Use Alpha"),
    b("invert_alpha", "Invert Alpha"),
];

static COLORS: &[BPropSpec] = &[
    b("use_color_ramp", "Use Color Ramp"),
    f("factor_red", "Red", BUnit::None),
    f("factor_green", "Green", BUnit::None),
    f("factor_blue", "Blue", BUnit::None),
    f("intensity", "Intensity", BUnit::None),
    f("contrast", "Contrast", BUnit::None),
    f("saturation", "Saturation", BUnit::None),
];

static MAPPING: &[BPropSpec] = &[
    e(
        "mapping",
        "Projection",
        &[
            en("FLAT", "Flat"),
            en("CUBE", "Cube"),
            en("TUBE", "Tube"),
            en("SPHERE", "Sphere"),
        ],
    ),
    v3("offset", "Offset", BUnit::None),
    v3("scale", "Size", BUnit::None),
];

static INFLUENCE: &[BPropSpec] = &[
    b("use_map_color_diffuse", "Diffuse Color"),
    b("use_map_normal", "Normal"),
    b("use_map_alpha", "Alpha"),
    b("use_map_specular", "Specular"),
    f("diffuse_color_factor", "Diffuse Factor", BUnit::None),
    f("normal_factor", "Normal Factor", BUnit::None),
];

pub static SECTIONS: &[BSectionSpec] = &[
    sec("Preview", PREVIEW),
    sec("Type", TYPE),
    sec("Image", IMAGE),
    sec("Colors", COLORS).closed(),
    sec("Mapping", MAPPING),
    sec("Influence", INFLUENCE).closed(),
];
