//! Data context set — Mesh properties. Blender ref: `bl_ui/properties_data_mesh.py`.

use super::super::prop::*;
use super::super::set::{BSectionSpec, sec};

static NORMALS: &[BPropSpec] = &[
    b("use_auto_smooth", "Auto Smooth"),
    f("auto_smooth_angle", "Angle", BUnit::Angle),
];

static UV_MAPS: &[BPropSpec] = &[p("uv_layers", "UV Maps", "UVLoopLayers")];

static SHAPE_KEYS: &[BPropSpec] = &[
    p("shape_keys", "Shape Keys", "Key"),
    b("show_only_shape_key", "Show Only Shape"),
];

static VERTEX_GROUPS: &[BPropSpec] = &[p("vertex_groups", "Vertex Groups", "VertexGroups")];

static ATTRIBUTES: &[BPropSpec] = &[
    p("attributes", "Attributes", "AttributeGroup"),
    p("color_attributes", "Color Attributes", "AttributeGroup"),
];

static REMESH: &[BPropSpec] = &[
    e(
        "remesh_mode",
        "Mode",
        &[en("VOXEL", "Voxel"), en("QUAD", "Quad")],
    ),
    f("remesh_voxel_size", "Voxel Size", BUnit::Length),
    f("remesh_voxel_adaptivity", "Adaptivity", BUnit::None),
];

static GEOMETRY_NODES: &[BPropSpec] = &[p("modifiers", "Geometry Nodes", "ModifierStack")];

pub static SECTIONS: &[BSectionSpec] = &[
    sec("Normals", NORMALS),
    sec("UV Maps", UV_MAPS),
    sec("Vertex Groups", VERTEX_GROUPS),
    sec("Shape Keys", SHAPE_KEYS),
    sec("Attributes", ATTRIBUTES),
    sec("Remesh", REMESH),
    sec("Geometry Nodes", GEOMETRY_NODES),
];
