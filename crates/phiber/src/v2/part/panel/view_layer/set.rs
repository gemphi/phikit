//! View Layer context set — sections + RNA properties for `bpy.types.ViewLayer`.
//! Blender ref: `bl_ui/properties_view_layer.py`.

use super::super::prop::*;
use super::super::set::{BSectionSpec, sec};

static LAYER: &[BPropSpec] = &[
    s("name", "Name"),
    b("use", "Use for Rendering"),
    b("use_sky", "Sky"),
    b("use_solid", "Solid"),
    b("use_strand", "Strand"),
    b("use_edge_enhance", "Edge Enhance"),
    b("use_freestyle", "Freestyle"),
];

static PASSES: &[BPropSpec] = &[
    b("use_pass_combined", "Combined"),
    b("use_pass_z", "Z"),
    b("use_pass_mist", "Mist"),
    b("use_pass_normal", "Normal"),
    b("use_pass_position", "Position"),
    b("use_pass_vector", "Vector"),
    b("use_pass_uv", "UV"),
    b("use_pass_object_index", "Object Index"),
    b("use_pass_material_index", "Material Index"),
    b("use_pass_diffuse_direct", "Diffuse Direct"),
    b("use_pass_diffuse_indirect", "Diffuse Indirect"),
    b("use_pass_diffuse_color", "Diffuse Color"),
    b("use_pass_glossy_direct", "Glossy Direct"),
    b("use_pass_glossy_indirect", "Glossy Indirect"),
    b("use_pass_glossy_color", "Glossy Color"),
    b("use_pass_emit", "Emission"),
    b("use_pass_environment", "Environment"),
    b("use_pass_shadow", "Shadow"),
    b("use_pass_ambient_occlusion", "Ambient Occlusion"),
];

static FILTER: &[BPropSpec] = &[
    b("use_ao", "Ambient Occlusion"),
    b("use_solid", "Solid"),
    b("use_strand", "Strand"),
    b("use_volumes", "Volumes"),
    b("use_motion_blur", "Motion Blur"),
];

static OVERRIDE: &[BPropSpec] = &[
    p("material_override", "Material", "Material"),
    p("world_override", "World", "World"),
    f("samples", "Samples", BUnit::None),
];

pub static SECTIONS: &[BSectionSpec] = &[
    sec("View Layer", LAYER),
    sec("Passes", PASSES),
    sec("Filter", FILTER),
    sec("Override", OVERRIDE),
];
