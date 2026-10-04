//! Which class a node belongs to, inferred from its `idname`.
//!
//! `node_get_colorid` reads `node.typeinfo->nclass`, and `typeinfo` is a
//! registry pointer — nothing of it is in the .blend. The one thing in the file
//! that says what a node is, is its `idname`, which IS the type's registered
//! string. So this reads the idname.
//!
//! Everything in `mod.rs` next door is Blender's to the digit. Everything here
//! is a stand-in for the registry and will mis-sort an unusual node. It is also
//! the only thing a file reader can do, and the two live in separate files so
//! nobody mistakes one for the other.
//!
//! Every mapping asserted in `tests.rs` was read out of the node's own
//! registration (`ntype.nclass = …`), not guessed from its name — twice over,
//! because the first draft of this guessed `GeometryNodeInputNamedAttribute`
//! was an attribute node and Blender registers it as an INPUT.

use super::BNodeClass;

/// The family prefixes a registered idname starts with, longest first so
/// `GeometryNode` is never read as a bare `Node`.
const FAMILIES: &[&str] =
    &["GeometryNode", "CompositorNode", "FunctionNode", "TextureNode", "ShaderNode", "Node"];

/// The idname without its tree family — `ShaderNodeTexImage` → `TexImage`.
fn tail_of(idname: &str) -> &str {
    FAMILIES.iter().find_map(|p| idname.strip_prefix(p)).unwrap_or(idname)
}

/// Which class a node belongs to, from its `idname` and its `flag`.
///
/// `active_output` is `bNode.flag & NODE_DO_OUTPUT`, and it matters: Blender
/// gives `NODE_CLASS_OUTPUT` the output colour **only** for the node that is
/// actually the tree's output (`is_output_node ? TH_NODE_OUTPUT : TH_NODE`). A
/// material with two Material Output nodes shows one tinted and the other
/// plain, which is how you see at a glance which one is rendering.
///
/// Classification reads the TAIL, not the whole idname, because the prefix is
/// the tree and the tail is the type. That matters most in Geometry Nodes,
/// where all 270 types are spelt `GeometryNode…` and the prefix says nothing;
/// there the convention IS the tail — `GeometryNodeInput*` is
/// `NODE_CLASS_INPUT` (`node_geo_input_position.cc`, `node_geo_input_normal.cc`
/// and `node_geo_input_named_attribute.cc` all say so, the last one despite
/// being an attribute node), anything else naming an attribute is
/// `NODE_CLASS_ATTRIBUTE` (`node_geo_store_named_attribute.cc`,
/// `node_geo_remove_attribute.cc`), and the rest are `NODE_CLASS_GEOMETRY`.
///
/// The tail also settles a word that means two things: "Transform" is
/// `NODE_CLASS_DISTORT` in the compositor (`node_composite_transform.cc`) and
/// `NODE_CLASS_GEOMETRY` in a geometry tree
/// (`node_geo_transform_geometry.cc`), so the family has to be decided first.
pub fn class_of(idname: &str, active_output: bool) -> BNodeClass {
    // A group's own in and out come first: Blender registers both as
    // `NODE_CLASS_INTERFACE` (`nodes/intern/node_common.cc:1112` and `:1198`),
    // and `NodeGroupOutput` contains "Output", so an output test placed before
    // this one would claim it.
    if idname == "NodeGroupInput" || idname == "NodeGroupOutput" {
        return BNodeClass::Interface;
    }
    if idname.contains("NodeFrame") {
        return BNodeClass::Frame;
    }
    if idname.ends_with("NodeGroup") || idname.contains("CustomGroup") {
        return BNodeClass::Group;
    }
    let tail = tail_of(idname);
    let has = |s: &str| tail.contains(s);
    if tail.starts_with("Output") || tail.starts_with("Viewer") || tail == "Composite" {
        return if active_output { BNodeClass::Output } else { BNodeClass::Plain };
    }
    if idname.starts_with("GeometryNode") {
        return if tail.starts_with("Input") {
            BNodeClass::Input
        } else if has("Attribute") {
            BNodeClass::Attribute
        } else {
            BNodeClass::Geometry
        };
    }
    if tail.starts_with("Tex") {
        BNodeClass::Texture
    } else if has("Bsdf")
        || has("MixShader")
        || has("AddShader")
        || has("Subsurface")
        || has("Emission")
        || has("Volume")
        || has("Background")
        || has("Holdout")
    {
        BNodeClass::Shader
    } else if has("Script") {
        BNodeClass::Script
    } else if has("Attribute") {
        BNodeClass::Attribute
    } else if has("Matte") || has("Keying") {
        BNodeClass::Matte
    } else if has("Distort")
        || has("Transform")
        || has("Translate")
        || has("Lensdist")
        || has("Stabilize")
        || has("CornerPin")
    {
        BNodeClass::Distort
    } else if has("Blur")
        || has("Filter")
        || has("Glare")
        || has("Denoise")
        || has("Defocus")
        || has("Dilate")
    {
        BNodeClass::Filter
    } else if has("Normal") || has("Mapping") || has("Bump") || has("Displacement") || has("Vector")
    {
        BNodeClass::Vector
    } else if has("RGB") || has("Color") || has("Hue") || has("Gamma") || has("Bright") || has("Invert")
    {
        BNodeClass::Color
    } else if has("Value")
        || has("Camera")
        || has("Light")
        || has("Fresnel")
        || has("Geometry")
        || has("Object")
        || has("Particle")
        || has("UVMap")
        || has("Tangent")
        || has("Wireframe")
    {
        BNodeClass::Input
    } else if has("Math")
        || has("Mix")
        || has("Clamp")
        || has("Map")
        || has("Separate")
        || has("Combine")
        || has("Blackbody")
        || has("Wavelength")
        || has("Float")
    {
        BNodeClass::Converter
    } else {
        BNodeClass::Plain
    }
}

