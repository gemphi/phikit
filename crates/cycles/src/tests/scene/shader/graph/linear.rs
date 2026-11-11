// SPDX-License-Identifier: Apache-2.0
use super::*;

fn number(name: &str) -> Value {
    Value::Enum(constant(name).expect("in data/constant.tsv") as i32)
}

/// A node of `kind` after two value nodes, with `setting` made, and the
/// inputs named in `linked` fed by the values.
fn linear(kind: &str, setting: &[(&str, Value)], linked: &[&str]) -> bool {
    let mut graph = graph_of(&["value", "value", kind]);
    setting.iter().for_each(|(socket, value)| set(&mut graph, 3, socket, value.clone()));
    for (at, name) in linked.iter().enumerate() {
        link(&mut graph, (1 + at % 2, "Value"), (3, name));
    }
    graph.node(3).expect("the node").is_linear_operation()
}

#[test]
fn some_classes_say_so_outright() {
    assert!(linear("output", &[], &[]) && linear("diffuse_bsdf", &[], &[]) && linear("emission", &[], &[]));
    assert!(!linear("noise_texture", &[], &[]) && !linear("bump", &[], &[]));
}

#[test]
fn a_sum_or_a_product_is_linear_in_one_variable() {
    assert!(linear("math", &[], &["Value1"]));
    assert!(!linear("math", &[], &["Value1", "Value2"]));
    assert!(linear("math", &[("math_type", number("NODE_MATH_MULTIPLY"))], &["Value2"]));
    assert!(!linear("math", &[("math_type", number("NODE_MATH_POWER"))], &["Value1"]));
    assert!(linear("vector_math", &[("math_type", number("NODE_VECTOR_MATH_SUBTRACT"))], &[]));
    assert!(!linear("vector_math", &[("math_type", number("NODE_VECTOR_MATH_CROSS_PRODUCT"))], &[]));
}

#[test]
fn a_quotient_is_linear_while_its_divisor_is_a_constant() {
    let divide = [("math_type", number("NODE_MATH_DIVIDE"))];
    assert!(linear("math", &divide, &["Value1"]) && !linear("math", &divide, &["Value2"]));
}

#[test]
fn a_mix_is_linear_with_a_constant_factor_and_no_clamp() {
    // A mix clamps its factor unless told not to.
    let free = ("use_clamp", Value::Boolean(false));
    assert!(linear("mix_float", std::slice::from_ref(&free), &["A", "B"]));
    assert!(!linear("mix_float", std::slice::from_ref(&free), &["Factor"]));
    assert!(!linear("mix_float", &[], &["A", "B"]));
    assert!(linear("mix_color", &[free.clone(), ("blend_type", number("NODE_MIX_ADD"))], &[]));
    assert!(!linear("mix_color", &[free.clone(), ("blend_type", number("NODE_MIX_SCREEN"))], &[]));
    assert!(!linear("mix_color", &[free, ("use_clamp_result", Value::Boolean(true))], &[]));
}

#[test]
fn a_range_is_linear_when_it_maps_straight_onto_constants() {
    assert!(linear("map_range", &[], &["Value"]));
    assert!(!linear("map_range", &[], &["To Max"]));
    assert!(!linear("map_range", &[("range_type", number("NODE_MAP_RANGE_STEPPED"))], &[]));
}
