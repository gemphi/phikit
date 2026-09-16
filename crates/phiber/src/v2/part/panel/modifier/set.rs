//! Modifier context set — Blender ref: `bl_ui/properties_data_modifier.py`.
//! Modifiers are a stack; this set exposes the Add operator + common modifier
//! specs. Runtime state carries which modifiers are actually applied.

use super::super::prop::*;
use super::super::set::{BSectionSpec, sec};

static ADD: &[BPropSpec] = &[e(
    "add_modifier",
    "Add Modifier",
    &[
        // Modify
        en("DATA_TRANSFER", "Data Transfer"),
        en("MESH_CACHE", "Mesh Cache"),
        en("NORMAL_EDIT", "Normal Edit"),
        en("WEIGHTED_NORMAL", "Weighted Normal"),
        en("UV_PROJECT", "UV Project"),
        en("UV_WARP", "UV Warp"),
        en("VERTEX_WEIGHT_EDIT", "Vertex Weight Edit"),
        en("VERTEX_WEIGHT_MIX", "Vertex Weight Mix"),
        en("VERTEX_WEIGHT_PROXIMITY", "Vertex Weight Proximity"),
        // Generate
        en("ARRAY", "Array"),
        en("BEVEL", "Bevel"),
        en("BOOLEAN", "Boolean"),
        en("BUILD", "Build"),
        en("DECIMATE", "Decimate"),
        en("EDGE_SPLIT", "Edge Split"),
        en("MASK", "Mask"),
        en("MIRROR", "Mirror"),
        en("MULTIRES", "Multiresolution"),
        en("REMESH", "Remesh"),
        en("SCREW", "Screw"),
        en("SKIN", "Skin"),
        en("SOLIDIFY", "Solidify"),
        en("SUBSURF", "Subdivision Surface"),
        en("TRIANGULATE", "Triangulate"),
        en("WELD", "Weld"),
        en("WIREFRAME", "Wireframe"),
        // Deform
        en("ARMATURE", "Armature"),
        en("CAST", "Cast"),
        en("CURVE", "Curve"),
        en("DISPLACE", "Displace"),
        en("HOOK", "Hook"),
        en("LAPLACIAN_DEFORM", "Laplacian Deform"),
        en("LATTICE", "Lattice"),
        en("MESH_DEFORM", "Mesh Deform"),
        en("SHRINKWRAP", "Shrinkwrap"),
        en("SIMPLE_DEFORM", "Simple Deform"),
        en("SMOOTH", "Smooth"),
        en("CORRECTIVE_SMOOTH", "Corrective Smooth"),
        en("LAPLACIAN_SMOOTH", "Laplacian Smooth"),
        en("SURFACE_DEFORM", "Surface Deform"),
        en("WARP", "Warp"),
        en("WAVE", "Wave"),
        // Physics
        en("CLOTH", "Cloth"),
        en("COLLISION", "Collision"),
        en("DYNAMIC_PAINT", "Dynamic Paint"),
        en("EXPLODE", "Explode"),
        en("FLUID", "Fluid"),
        en("OCEAN", "Ocean"),
        en("PARTICLE_INSTANCE", "Particle Instance"),
        en("PARTICLE_SYSTEM", "Particle System"),
        en("SOFT_BODY", "Soft Body"),
        en("SURFACE", "Surface"),
    ],
)];

static STACK: &[BPropSpec] = &[p("modifiers", "Modifier Stack", "ModifierStack")];

pub static SECTIONS: &[BSectionSpec] = &[sec("Add Modifier", ADD), sec("Modifier Stack", STACK)];
