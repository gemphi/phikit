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

/// Which words name which class, in the order they are tried.
///
/// A TABLE, not a chain of `else if`. The order is the whole rule here — a
/// keyword search has no other way to break a tie — and a table shows that
/// order as data you can read down, where a chain hides it in control flow.
/// "Mapping" is a vector node and "MapRange" a converter; the only thing that
/// decides is which row comes first, and that should be visible.
const BY_KEYWORD: &[(BNodeClass, &[&str])] = &[
    (
        BNodeClass::Shader,
        &["Bsdf", "MixShader", "AddShader", "Subsurface", "Emission", "Volume", "Background", "Holdout"],
    ),
    (BNodeClass::Script, &["Script"]),
    (BNodeClass::Attribute, &["Attribute"]),
    (BNodeClass::Matte, &["Matte", "Keying"]),
    (
        BNodeClass::Distort,
        &["Distort", "Transform", "Translate", "Lensdist", "Stabilize", "CornerPin"],
    ),
    (BNodeClass::Filter, &["Blur", "Filter", "Glare", "Denoise", "Defocus", "Dilate"]),
    (BNodeClass::Vector, &["Normal", "Mapping", "Bump", "Displacement", "Vector"]),
    (BNodeClass::Color, &["RGB", "Color", "Hue", "Gamma", "Bright", "Invert"]),
    (
        BNodeClass::Input,
        &["Value", "Camera", "Light", "Fresnel", "Geometry", "Object", "Particle", "UVMap", "Tangent", "Wireframe"],
    ),
    (
        BNodeClass::Converter,
        &["Math", "Mix", "Clamp", "Map", "Separate", "Combine", "Blackbody", "Wavelength", "Float"],
    ),
];

/// The classes an idname's SHAPE decides, before any keyword is looked at.
///
/// These are the ones a keyword table cannot express, because each is about
/// where in the name the word sits rather than whether it appears at all —
/// `NodeGroupOutput` contains "Output" and is not an output.
fn by_shape(idname: &str, tail: &str, active_output: bool) -> Option<BNodeClass> {
    match idname {
        // A group's own in and out. Blender registers both as
        // `NODE_CLASS_INTERFACE` (`node_common.cc:1112` and `:1198`), and both
        // contain a word the arms below would claim.
        "NodeGroupInput" | "NodeGroupOutput" => Some(BNodeClass::Interface),
        _ if idname.contains("NodeFrame") => Some(BNodeClass::Frame),
        _ if idname.ends_with("NodeGroup") || idname.contains("CustomGroup") => {
            Some(BNodeClass::Group)
        }
        // The tree's own output, and only when it is the one in use.
        _ if tail.starts_with("Output") || tail.starts_with("Viewer") || tail == "Composite" => {
            Some(if active_output { BNodeClass::Output } else { BNodeClass::Plain })
        }
        // A texture is named by its prefix, not by a word anywhere in it.
        _ if tail.starts_with("Tex") => Some(BNodeClass::Texture),
        _ => None,
    }
}

/// Geometry Nodes name by suffix: all 270 types share the one prefix, so the
/// prefix says nothing and the tail decides.
///
/// `GeometryNodeInput*` is `NODE_CLASS_INPUT` — `node_geo_input_position.cc`,
/// `node_geo_input_normal.cc` and `node_geo_input_named_attribute.cc` all say
/// so, the last one despite the word "Attribute" in it. Anything else naming an
/// attribute is `NODE_CLASS_ATTRIBUTE` (`node_geo_store_named_attribute.cc`,
/// `node_geo_remove_attribute.cc`); the rest are `NODE_CLASS_GEOMETRY`.
fn in_geometry_tree(tail: &str) -> BNodeClass {
    match tail {
        _ if tail.starts_with("Input") => BNodeClass::Input,
        _ if tail.contains("Attribute") => BNodeClass::Attribute,
        _ => BNodeClass::Geometry,
    }
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
/// the tree and the tail is the type. That also settles a word that means two
/// things: "Transform" is `NODE_CLASS_DISTORT` in the compositor
/// (`node_composite_transform.cc`) and `NODE_CLASS_GEOMETRY` in a geometry tree
/// (`node_geo_transform_geometry.cc`), so the family is decided first.
pub fn class_of(idname: &str, active_output: bool) -> BNodeClass {
    let tail = tail_of(idname);
    match by_shape(idname, tail, active_output) {
        Some(class) => class,
        None if idname.starts_with("GeometryNode") => in_geometry_tree(tail),
        None => BY_KEYWORD
            .iter()
            .find(|(_, words)| words.iter().any(|word| tail.contains(word)))
            .map_or(BNodeClass::Plain, |(class, _)| *class),
    }
}
