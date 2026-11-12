// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Tests of `blender/shader.rs` and `node.rs` (`blender/shader.cpp`): a
//! node tree handed over as data becomes a `ShaderGraph`; the same tree
//! from a file of Blender's tests arrives whole.
use crate::blender::*;
use crate::scene::{InputId, OutputId, ShaderGraph};
use crate::tests::truth::verdict::root;
use crate::truth::TestScene;

fn socket(identifier: &str, class: SocketClass, value: Option<SocketValue>) -> SocketSource {
    SocketSource { identifier: identifier.into(), class, available: true, value }
}
fn node(idname: &str, name: &str, input: Vec<SocketSource>, output: Vec<SocketSource>) -> NodeSource {
    NodeSource { idname: idname.into(), name: name.into(), input, output, ..NodeSource::default() }
}
fn link(from: &str, from_socket: &str, to: &str, to_socket: &str) -> LinkSource {
    LinkSource { from: from.into(), from_socket: from_socket.into(), to: to.into(), to_socket: to_socket.into() }
}
fn output() -> NodeSource {
    let mut output = node("ShaderNodeOutputMaterial", "Material Output", vec![socket("Surface", SocketClass::Closure, None)], vec![]);
    output.active_output = true;
    output
}
fn count(graph: &ShaderGraph, type_name: &str) -> usize {
    graph.node_list().filter(|node| node.type_name() == type_name).count()
}

#[test]
fn a_diffuse_with_a_checker_for_its_colour_keeps_every_node_value_and_link() {
    let color = SocketValue::Color([0.1, 0.2, 0.3]);
    let tree = GraphSource {
        node: vec![
            output(),
            node("ShaderNodeBsdfDiffuse", "Diffuse BSDF", vec![socket("Color", SocketClass::Color, Some(color)), socket("Roughness", SocketClass::Float, Some(SocketValue::Float(0.25)))], vec![socket("BSDF", SocketClass::Closure, None)]),
            node("ShaderNodeTexChecker", "Checker", vec![socket("Color1", SocketClass::Color, Some(SocketValue::Color([1.0, 0.0, 0.0]))), socket("Scale", SocketClass::Float, Some(SocketValue::Float(5.0)))], vec![socket("Color", SocketClass::Color, None)]),
        ],
        link: vec![link("Checker", "Color", "Diffuse BSDF", "Color"), link("Diffuse BSDF", "BSDF", "Material Output", "Surface")],
        output: Some("Material Output".into()),
    };
    let graph = graph_of(&tree);
    // The output, the diffuse and the checker.
    assert_eq!(graph.node_list().count(), 3);
    let diffuse = graph.node_list().find(|node| node.type_name() == "diffuse_bsdf").unwrap().id;
    assert_eq!(graph.node(diffuse).unwrap().node.float("roughness"), 0.25);
    let checker = graph.node_list().find(|node| node.type_name() == "checker_texture").unwrap().id;
    assert_eq!(graph.node(checker).unwrap().node.float("scale"), 5.0);
    assert_eq!(graph.node(checker).unwrap().node.float3("color1"), [1.0, 0.0, 0.0]);
    // The checker feeds the colour; the diffuse feeds the output's surface.
    let color = graph.input_of(diffuse, "Color").unwrap();
    assert_eq!(graph.link_of(color), graph.output_of(checker, "Color"));
    let surface = graph.input_of(graph.output(), "Surface").unwrap();
    assert_eq!(graph.link_of(surface), graph.output_of(diffuse, "BSDF"));
}

#[test]
fn a_node_s_settings_in_custom1_and_custom2_reach_the_cycles_node() {
    let mut math = node("ShaderNodeMath", "Math", vec![socket("Value", SocketClass::Float, Some(SocketValue::Float(2.0))), socket("Value_001", SocketClass::Float, Some(SocketValue::Float(3.0)))], vec![socket("Value", SocketClass::Float, None)]);
    // `NODE_MATH_MULTIPLY` and a clamp.
    math.custom1 = 2;
    math.custom2 = 1;
    let graph = graph_of(&GraphSource { node: vec![output(), math], ..GraphSource::default() });
    let math = graph.node_list().find(|node| node.type_name() == "math").unwrap();
    assert_eq!((math.node.int("math_type"), math.node.boolean("use_clamp")), (2, true));
    // `Value_001` is Cycles' `Value2`.
    assert_eq!((math.node.float("value1"), math.node.float("value2")), (2.0, 3.0));
}

#[test]
fn a_mix_shader_s_sockets_are_found_under_cycles_names() {
    let closure = |name: &str| socket(name, SocketClass::Closure, None);
    let tree = GraphSource {
        node: vec![
            output(),
            node("ShaderNodeMixShader", "Mix", vec![socket("Fac", SocketClass::Float, Some(SocketValue::Float(0.5))), closure("Shader"), closure("Shader_001")], vec![closure("Shader")]),
            node("ShaderNodeBsdfDiffuse", "A", vec![], vec![closure("BSDF")]),
            node("ShaderNodeEmission", "B", vec![], vec![closure("Emission")]),
        ],
        link: vec![link("A", "BSDF", "Mix", "Shader"), link("B", "Emission", "Mix", "Shader_001"), link("Mix", "Shader", "Material Output", "Surface")],
        output: Some("Material Output".into()),
    };
    let graph = graph_of(&tree);
    let mix = graph.node_list().find(|node| node.type_name() == "mix_closure").unwrap().id;
    let from = |name: &str| graph.link_of(graph.input_of(mix, name).unwrap()).map(|from| graph.node(from.node).unwrap().type_name());
    assert_eq!((from("Closure1"), from("Closure2")), (Some("diffuse_bsdf"), Some("emission")));
}

#[test]
fn a_muted_node_and_a_reroute_become_proxies_and_a_group_is_added_between_its_sockets() {
    let color = |name: &str| socket(name, SocketClass::Color, Some(SocketValue::Color([0.5; 3])));
    let mut reroute = node("NodeReroute", "Reroute", vec![color("Input")], vec![color("Output")]);
    reroute.reroute = true;
    // A group: its input goes through to its output, through a Group Input and a Group Output node.
    let mut group_output = node("NodeGroupOutput", "Group Output", vec![color("Color")], vec![]);
    group_output.active_output = true;
    let inner = GraphSource {
        node: vec![node("NodeGroupInput", "Group Input", vec![], vec![color("Color")]), group_output],
        link: vec![link("Group Input", "Color", "Group Output", "Color")],
        output: None,
    };
    let mut group = node("ShaderNodeGroup", "Group", vec![color("Color")], vec![color("Color")]);
    group.group = Some(Box::new(inner));
    let tree = GraphSource {
        node: vec![output(), reroute, group, node("ShaderNodeBsdfDiffuse", "Diffuse", vec![color("Color")], vec![socket("BSDF", SocketClass::Closure, None)])],
        link: vec![link("Reroute", "Output", "Group", "Color"), link("Group", "Color", "Diffuse", "Color"), link("Diffuse", "BSDF", "Material Output", "Surface")],
        output: Some("Material Output".into()),
    };
    let graph = graph_of(&tree);
    // The reroute, the group's input and output sockets, and the group's two interface nodes
    // (each a proxy), are proxies; `Shader::set_graph` removes them.
    assert_eq!(count(&graph, "convert_color_to_color"), 3);
    let mut simple = graph.clone();
    simple.remove_proxy_node();
    assert_eq!(count(&simple, "convert_color_to_color"), 0);
    let diffuse = simple.node_list().find(|node| node.type_name() == "diffuse_bsdf").unwrap().id;
    // Nothing is left between the reroute and the colour, which nothing feeds now.
    assert!(simple.link_of(simple.input_of(diffuse, "Color").unwrap()).is_none());
}

#[test]
fn a_node_with_settings_the_source_lacks_is_left_out() {
    // A Mix node keeps its blend type in `storage`, which phiphx does not hand over.
    let mix = node("ShaderNodeMix", "Mix", vec![], vec![socket("Result_Color", SocketClass::Color, None)]);
    let graph = graph_of(&GraphSource { node: vec![output(), mix.clone()], ..GraphSource::default() });
    assert_eq!(graph.node_list().count(), 1);
    let mut with = mix;
    with.storage = NodeStorage::Mix { data_type: SocketClass::Color, blend_type: 2, factor_mode: 0, clamp_factor: true, clamp_result: false };
    let graph = graph_of(&GraphSource { node: vec![output(), with], ..GraphSource::default() });
    let mix = graph.node_list().find(|node| node.type_name() == "mix_color").unwrap();
    assert_eq!(mix.node.int("blend_type"), 2);
    let _ = (InputId { node: 0, socket: 0 }, OutputId { node: 0, socket: 0 });
}

#[test]
fn a_materials_whole_graph_is_handed_over_from_a_file() {
    let Some(root) = root() else { return };
    let test = TestScene::new(&root, "shader", "emission");
    let source = read_blend(&test.blend).unwrap();
    for material in source.mesh.iter().flat_map(|mesh| &mesh.material).filter(|material| !material.node_kind.is_empty()) {
        // Every node of phiphx's graph but the frames arrives, with its sockets.
        assert_eq!(material.graph.node.len(), material.node_kind.len(), "{}", material.name);
        let graph = graph_of(&material.graph);
        assert!(graph.link_of(graph.input_of(graph.output(), "Surface").unwrap()).is_some(), "{} has a surface", material.name);
        assert!(count(&graph, "checker_texture") >= material.node_kind.iter().filter(|kind| *kind == "ShaderNodeTexChecker").count());
    }
}
