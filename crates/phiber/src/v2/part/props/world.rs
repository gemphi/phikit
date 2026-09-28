//! World context — `bl_ui/properties_world.py`.
// World context set — sections + RNA properties for `bpy.types.World`.
// Blender ref: `bl_ui/properties_world.py`, `rna_world.cc`.

use super::prop::*;
use super::set::BSectionSpec;
use super::set::sec;

static SURFACE: &[BPropSpec] = &[
    b("use_nodes", "Use Nodes"),
    p("node_tree", "Node Tree", "ShaderNodeTree"),
    col("color", "Color", false),
];

static VOLUME: &[BPropSpec] = &[p("node_tree", "Volume Shader", "ShaderNodeTree")];

static MIST: &[BPropSpec] = &[
    b("mist_settings.use_mist", "Use Mist"),
    f("mist_settings.start", "Start", BUnit::Length),
    f("mist_settings.depth", "Depth", BUnit::Length),
    f("mist_settings.height", "Height", BUnit::Length),
    f("mist_settings.intensity", "Intensity", BUnit::None),
    e(
        "mist_settings.falloff",
        "Falloff",
        &[
            en("QUADRATIC", "Quadratic"),
            en("LINEAR", "Linear"),
            en("INVERSE_QUADRATIC", "Inverse Quadratic"),
        ],
    ),
];

static VIEWPORT_DISPLAY: &[BPropSpec] = &[col("color", "Color", false)];

pub static SECTIONS: &[BSectionSpec] = &[
    sec("Surface", SURFACE),
    sec("Volume", VOLUME).closed(),
    sec("Mist Pass", MIST).closed(),
    sec("Viewport Display", VIEWPORT_DISPLAY).closed(),
];
