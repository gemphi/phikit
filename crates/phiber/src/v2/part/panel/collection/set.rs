//! Collection context set — sections + RNA properties for `bpy.types.Collection`.
//! Blender ref: `bl_ui/properties_collection.py`.

use super::super::prop::*;
use super::super::set::{BSectionSpec, sec};

static COLLECTION: &[BPropSpec] = &[
    b("hide_viewport", "Hide in Viewport"),
    b("hide_select", "Disable Selection"),
    b("hide_render", "Disable in Renders"),
];

static LINE_ART: &[BPropSpec] = &[
    e(
        "lineart_usage",
        "Usage",
        &[
            en("INCLUDE", "Include"),
            en("OCCLUSION_ONLY", "Occlusion Only"),
            en("EXCLUDE", "Exclude"),
            en("INTERSECTION_ONLY", "Intersection Only"),
            en("NO_INTERSECTION", "No Intersection"),
        ],
    ),
    b("use_lineart_intersection_mask", "Intersection Priority"),
];

static INSTANCING: &[BPropSpec] = &[v3("instance_offset", "Instance Offset", BUnit::Length)];

pub static SECTIONS: &[BSectionSpec] = &[
    sec("Collection", COLLECTION),
    sec("Line Art", LINE_ART),
    sec("Instancing", INSTANCING),
];
