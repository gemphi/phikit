//! Node-class tests — split out so `mod.rs` stays inside the 240-line read
//! budget (`level_structure`), the way `canvas/time/row/` is.

use super::*;

/// What a header looks like in one dark theme, for the lightness assertions.
const HEADER: BColor = (210.0, 8.0, 22.0, 1.0);

#[test]
fn an_inactive_output_node_is_plain_and_the_active_one_is_tinted() {
    // `node_get_colorid`: `is_output_node ? TH_NODE_OUTPUT : TH_NODE`. A
    // material with two Material Outputs shows which one is rendering, and
    // that is the whole reason the flag is consulted at all.
    assert_eq!(class_of("ShaderNodeOutputMaterial", true), BNodeClass::Output);
    assert_eq!(class_of("ShaderNodeOutputMaterial", false), BNodeClass::Plain);
    assert_eq!(class_of("CompositorNodeComposite", true), BNodeClass::Output);
    assert_eq!(class_of("GeometryNodeViewer", false), BNodeClass::Plain);
}

/// Every mapping here was read out of the node's own `ntype.nclass = …`, not
/// guessed from its name. The first draft of this test guessed, and got
/// `GeometryNodeInputNamedAttribute` wrong — Blender registers it as
/// `NODE_CLASS_INPUT` despite the word "Attribute" in it
/// (`node_geo_input_named_attribute.cc:129`).
#[test]
fn the_classes_are_the_ones_blender_registers() {
    for (idname, class, source) in [
        // Shader (`nodes/shader/nodes/`).
        ("ShaderNodeTexImage", BNodeClass::Texture, "node_shader_tex_image.cc"),
        ("ShaderNodeMath", BNodeClass::Converter, "node_shader_math.cc"),
        ("ShaderNodeSeparateXYZ", BNodeClass::Converter, "node_shader_sepcomb_xyz.cc"),
        ("ShaderNodeMixRGB", BNodeClass::Color, "node_shader_mix_rgb.cc"),
        ("ShaderNodeHueSaturation", BNodeClass::Color, "node_shader_hueSatVal.cc"),
        ("ShaderNodeNormalMap", BNodeClass::Vector, "node_shader_normal_map.cc"),
        ("ShaderNodeMapping", BNodeClass::Vector, "node_shader_mapping.cc"),
        ("ShaderNodeValue", BNodeClass::Input, "node_shader_value.cc"),
        ("ShaderNodeTangent", BNodeClass::Input, "node_shader_tangent.cc"),
        ("ShaderNodeNewGeometry", BNodeClass::Input, "node_shader_geometry.cc"),
        ("ShaderNodeBsdfPrincipled", BNodeClass::Shader, "node_shader_bsdf_principled.cc"),
        ("ShaderNodeMixShader", BNodeClass::Shader, "node_shader_mix_shader.cc"),
        ("ShaderNodeScript", BNodeClass::Script, "node_shader_script.cc"),
        // Geometry (`nodes/geometry/nodes/`) — classified by TAIL, since all
        // 270 types share the one prefix.
        ("GeometryNodeInputPosition", BNodeClass::Input, "node_geo_input_position.cc"),
        ("GeometryNodeInputNormal", BNodeClass::Input, "node_geo_input_normal.cc"),
        ("GeometryNodeInputNamedAttribute", BNodeClass::Input, "node_geo_input_named_attribute.cc"),
        ("GeometryNodeStoreNamedAttribute", BNodeClass::Attribute, "node_geo_store_named_attribute.cc"),
        ("GeometryNodeRemoveAttribute", BNodeClass::Attribute, "node_geo_remove_attribute.cc"),
        ("GeometryNodeSetPosition", BNodeClass::Geometry, "node_geo_set_position.cc"),
        ("GeometryNodeTransformGeometry", BNodeClass::Geometry, "node_geo_transform_geometry.cc"),
        ("GeometryNodeMeshCube", BNodeClass::Geometry, "node_geo_mesh_primitive_cube.cc"),
        // Compositor (`nodes/composite/nodes/`).
        ("CompositorNodeBlur", BNodeClass::Filter, "node_composite_blur.cc"),
        ("CompositorNodeTransform", BNodeClass::Distort, "node_composite_transform.cc"),
        ("CompositorNodeKeying", BNodeClass::Matte, "node_composite_keying.cc"),
        // Shared (`nodes/intern/node_common.cc`).
        ("NodeGroupInput", BNodeClass::Interface, "node_common.cc:1112"),
        ("NodeGroupOutput", BNodeClass::Interface, "node_common.cc:1198"),
    ] {
        assert_eq!(class_of(idname, true), class, "{idname} ({source})");
    }
}

/// "Transform" is two different classes depending on the tree it is in, which
/// is why the family is decided before the keyword.
#[test]
fn one_word_can_mean_two_classes_in_two_trees() {
    assert_eq!(class_of("CompositorNodeTransform", true), BNodeClass::Distort);
    assert_eq!(class_of("GeometryNodeTransformGeometry", true), BNodeClass::Geometry);
}

/// A node whose type this has never heard of is plain, not a panic and not a
/// wrong tint — an add-on's node, or one added after 5.2.0.
#[test]
fn an_unknown_node_is_plain() {
    assert_eq!(class_of("ShaderNodeSomethingNewEntirely", true), BNodeClass::Plain);
    assert_eq!(class_of("", true), BNodeClass::Plain);
    assert_eq!(class_of("SomePluginThing", false), BNodeClass::Plain);
}

#[test]
fn a_texture_node_is_not_read_as_an_output() {
    // Order matters: `ShaderNodeOutputMaterial` must not be a material-class
    // node and `ShaderNodeTexImage` must not be anything but a texture, so
    // the specific tests come before the loose ones.
    assert_eq!(class_of("ShaderNodeTexEnvironment", true), BNodeClass::Texture);
    // A group's own output contains "Output" but Blender registers it as
    // `NODE_CLASS_INTERFACE` (`node_common.cc:1198`), so it must be answered
    // BEFORE the output arm — and the tree's own output still is not.
    assert_eq!(class_of("NodeGroupOutput", true), BNodeClass::Interface);
    assert_eq!(class_of("NodeGroupOutput", false), BNodeClass::Interface);
    assert_eq!(class_of("ShaderNodeOutputMaterial", true), BNodeClass::Output);
}

#[test]
fn a_class_keeps_blenders_hue_and_takes_the_themes_lightness() {
    // The divergence, stated: the hue is the information, the lightness is
    // the theme's. A light palette must not get near-black headers.
    let shader = class_color(BNodeClass::Shader, HEADER);
    assert_eq!(shader.2, HEADER.2, "the lightness is the header's");
    assert!((90.0..150.0).contains(&shader.0), "shader hue {} is not green", shader.0);

    let texture = class_color(BNodeClass::Texture, HEADER);
    assert!((15.0..45.0).contains(&texture.0), "texture hue {} is not orange", texture.0);

    let vector = class_color(BNodeClass::Vector, HEADER);
    assert!((220.0..260.0).contains(&vector.0), "vector hue {} is not blue", vector.0);

    let output = class_color(BNodeClass::Output, HEADER);
    assert!(
        output.0 < 20.0 || output.0 > 320.0,
        "output hue {} is not the maroon end",
        output.0
    );
}

#[test]
fn blenders_neutral_classes_tint_nothing() {
    // `TH_NODE` is `0x303030`, `TH_NODE_INTERFACE` is `0x1d1d1d` and a frame
    // is `0x0f0f0f` — all grey. Giving them a hue out of a rounding error
    // would be inventing one, so they come back as the header itself.
    for class in [BNodeClass::Plain, BNodeClass::Interface, BNodeClass::Frame] {
        assert_eq!(class_color(class, HEADER), HEADER, "{class:?} invented a tint");
    }
}

#[test]
fn every_class_resolves_against_a_light_header_as_well_as_a_dark_one() {
    // The point of taking the lightness from the theme: the same seventeen
    // classes have to work on a pale palette.
    let light: BColor = (210.0, 8.0, 92.0, 1.0);
    for class in BNodeClass::ALL {
        let on_light = class_color(*class, light);
        let on_dark = class_color(*class, HEADER);
        assert_eq!(on_light.2, 92.0, "{class:?}");
        assert_eq!(on_dark.2, 22.0, "{class:?}");
        // Same hue either way — the class is the hue.
        assert!((on_light.0 - on_dark.0).abs() < 1e-3, "{class:?} changed hue with the mode");
        assert!(on_light.1.is_finite() && (0.0..=100.0).contains(&on_light.1), "{class:?}");
    }
}

#[test]
fn the_output_flag_is_blenders_own_bit() {
    assert_eq!(NODE_DO_OUTPUT, 64, "NODE_DO_OUTPUT is 1 << 6");
}
