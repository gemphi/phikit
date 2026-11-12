// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
use crate::scene::shader::graph::test::*;

fn node(kind: &str) -> ShaderNode {
    ShaderNode::new(kind).expect("in data/node.tsv")
}

#[test]
fn a_node_has_an_input_for_each_socket_others_may_link_to() {
    let bump = node("bump");
    assert_eq!(bump.input.iter().map(ShaderInput::name).collect::<Vec<_>>(), ["Height", "SampleCenter", "SampleX", "SampleY", "Normal", "Strength", "Distance", "Filter Width"]);
    assert_eq!(bump.output.iter().map(ShaderOutput::name).collect::<Vec<_>>(), ["Normal"]);
    // Its settings are sockets of the node, not inputs.
    assert!(bump.node.has("invert") && bump.input_named("Invert").is_none());
    assert_eq!((bump.input_named("Strength"), bump.output_named("Normal"), bump.type_name()), (Some(5), Some(0), "bump"));
    assert!(ShaderNode::new("no_such_node").is_none());
}

#[test]
fn a_node_s_class_gives_it_its_special_type() {
    let special = |kind: &str| node(kind).special_type;
    assert_eq!((special("output"), special("aov_output")), (ShaderNodeSpecialType::Output, ShaderNodeSpecialType::OutputAov));
    assert_eq!((special("diffuse_bsdf"), special("principled_bsdf")), (ShaderNodeSpecialType::Closure, ShaderNodeSpecialType::Closure));
    assert_eq!((special("mix_closure"), special("add_closure")), (ShaderNodeSpecialType::CombineClosure, ShaderNodeSpecialType::CombineClosure));
    assert_eq!((special("bump"), special("geometry"), special("image_texture")), (ShaderNodeSpecialType::Bump, ShaderNodeSpecialType::Geometry, ShaderNodeSpecialType::ImageSlot));
    assert_eq!((special("math"), special("emission")), (ShaderNodeSpecialType::None, ShaderNodeSpecialType::None));
    assert_eq!(ShaderNodeSpecialType::named("SHADER_SPECIAL_TYPE_LIGHT_PATH"), ShaderNodeSpecialType::LightPath);
    assert_eq!(ShaderNodeSpecialType::named("no such type"), ShaderNodeSpecialType::None);
}

#[test]
fn a_convert_node_from_a_type_to_itself_is_a_proxy() {
    assert_eq!(node("convert_float_to_float").special_type, ShaderNodeSpecialType::Proxy);
    let convert = node("convert_float_to_color");
    assert_eq!(convert.special_type, ShaderNodeSpecialType::None);
    assert_eq!((convert.input[0].kind(), convert.output[0].kind(), convert.input[0].name()), (SocketKind::Float, SocketKind::Color, "value_float"));
}

#[test]
fn a_node_answers_as_its_class_does() {
    assert!(node("emission").has_volume_support() && node("scatter_volume").has_volume_support() && !node("diffuse_bsdf").has_volume_support());
    // Said outright in the header, or worked out in code.
    assert_eq!((node("emission").says("has_surface_emission"), node("principled_bsdf").says("has_surface_bssrdf")), (Some("true"), Some("code")));
    assert_eq!(node("math").says("has_surface_emission"), None);
    assert!(node("math").has_override("constant_fold") && node("output").has_override("equals") && !node("geometry").has_override("constant_fold"));
}

#[test]
fn only_an_attribute_node_may_be_sampled_at_random() {
    assert!(node("attribute").stochastic_sample && !node("math").stochastic_sample && !node("emission").from_auto_conversion);
}

#[test]
fn a_copy_has_the_values_and_none_of_the_links() {
    let mut graph = graph_of(&["noise_texture", "math"]);
    set(&mut graph, 2, "value2", Value::Float(4.0));
    link(&mut graph, (1, "Fac"), (2, "Value1"));
    let original = graph.node_mut(2).expect("the math node");
    (original.bump, original.bump_filter_width, original.need_derivative) = (ShaderBump::Dx, 0.5, true);
    let copy = original.copy();
    assert_eq!((copy.node.float("value2"), copy.bump, copy.special_type), (4.0, ShaderBump::Dx, ShaderNodeSpecialType::None));
    assert!(copy.input.iter().all(|input| input.link.is_none()) && copy.output.iter().all(|output| output.link.is_empty()));
    assert_eq!((copy.bump_filter_width, copy.need_derivative, copy.node.is_modified()), (0.0, false, true));
}

#[test]
fn two_nodes_are_equal_when_their_settings_values_and_links_are() {
    let mut graph = graph_of(&["math", "math", "noise_texture"]);
    let equal = |graph: &ShaderGraph| graph.node(1).expect("one").equals(graph.node(2).expect("two"));
    assert!(equal(&graph));
    set(&mut graph, 1, "value1", Value::Float(0.25));
    assert!(!equal(&graph));
    set(&mut graph, 2, "value1", Value::Float(0.25));
    set(&mut graph, 2, "math_type", Value::Enum(constant("NODE_MATH_MULTIPLY").expect("in data/constant.tsv") as i32));
    assert!(!equal(&graph));
    set(&mut graph, 1, "math_type", Value::Enum(constant("NODE_MATH_MULTIPLY").expect("in data/constant.tsv") as i32));
    // One linked and one not are not equal, whatever value the first holds.
    link(&mut graph, (3, "Fac"), (1, "Value2"));
    assert!(!equal(&graph));
    link(&mut graph, (3, "Fac"), (2, "Value2"));
    assert!(equal(&graph));
    graph.node_mut(2).expect("two").bump = ShaderBump::Center;
    assert!(!equal(&graph));
}

#[test]
fn nodes_of_two_types_or_of_a_class_that_says_no_are_never_equal() {
    assert!(!node("math").equals(&node("vector_math")));
    // Cycles does not merge closures or outputs.
    assert!(!node("diffuse_bsdf").equals(&node("diffuse_bsdf")) && !node("output").equals(&node("output")));
    assert!(node("noise_texture").equals(&node("noise_texture")));
}
