// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/scene/shader_graph.h` and `shader_graph.cpp`
//! (`ShaderNodeSpecialType`, `ShaderNode`): a node of a shader graph, with
//! its input and output sockets.
//!
//! Cycles has a class for each kind of node; here every node is this one
//! type, and what a class says of itself is `data/shader.tsv`, which
//! `tool/shader.py` writes from `scene/shader_nodes.h`. Two members of
//! those classes that the graph reads are fields here. Not ported: what a
//! class works out in code, but for `is_linear_operation` (`linear.rs`):
//! `compile`, `constant_fold`, `expand`, `simplify_settings`,
//! `attributes`, `get_feature`, `get_closure_type`, and the `equals` of
//! the two image nodes.

use std::sync::OnceLock;

use super::{ShaderBump, ShaderInput, ShaderOutput};
use crate::graph::{Node, SocketDirection};
use crate::kernel::flag;
use crate::util::row_list;

/// `ShaderNodeSpecialType`: the kinds of node the graph has to tell from
/// the rest.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ShaderNodeSpecialType {
    #[default]
    None,
    /// A convert node from a type to itself, which only passes a value on.
    Proxy,
    /// A convert node the graph put in to join two types.
    Autoconvert,
    Geometry,
    Osl,
    ImageSlot,
    Closure,
    /// A mix or an add of two closures.
    CombineClosure,
    Output,
    Bump,
    OutputAov,
    LightPath,
    SceneTime,
}

impl ShaderNodeSpecialType {
    /// The type Cycles names so: `SHADER_SPECIAL_TYPE_BUMP`.
    pub fn named(name: &str) -> ShaderNodeSpecialType {
        match name.strip_prefix("SHADER_SPECIAL_TYPE_").unwrap_or("") {
            "PROXY" => ShaderNodeSpecialType::Proxy,
            "AUTOCONVERT" => ShaderNodeSpecialType::Autoconvert,
            "GEOMETRY" => ShaderNodeSpecialType::Geometry,
            "OSL" => ShaderNodeSpecialType::Osl,
            "IMAGE_SLOT" => ShaderNodeSpecialType::ImageSlot,
            "CLOSURE" => ShaderNodeSpecialType::Closure,
            "COMBINE_CLOSURE" => ShaderNodeSpecialType::CombineClosure,
            "OUTPUT" => ShaderNodeSpecialType::Output,
            "BUMP" => ShaderNodeSpecialType::Bump,
            "OUTPUT_AOV" => ShaderNodeSpecialType::OutputAov,
            "LIGHT_PATH" => ShaderNodeSpecialType::LightPath,
            "SCENE_TIME" => ShaderNodeSpecialType::SceneTime,
            _ => ShaderNodeSpecialType::None,
        }
    }
}

/// The rows of `data/shader.tsv`.
fn table() -> &'static [Vec<&'static str>] {
    static TABLE: OnceLock<Vec<Vec<&'static str>>> = OnceLock::new();
    TABLE.get_or_init(|| row_list(include_str!("../../../../data/shader.tsv")).collect())
}

/// `ShaderNode`.
#[derive(Debug, Clone, PartialEq)]
pub struct ShaderNode {
    /// The settings and the values of the inputs that are not linked.
    pub node: Node,
    /// `inputs`: the sockets others may link to.
    pub input: Vec<ShaderInput>,
    /// `outputs`.
    pub output: Vec<ShaderOutput>,
    /// Its number in its graph, given when it is added.
    pub id: usize,
    pub bump: ShaderBump,
    pub bump_filter_width: f32,
    pub special_type: ShaderNodeSpecialType,
    /// `need_derivatives_`.
    pub need_derivative: bool,
    pub added_to_svm: bool,
    /// `EmissionNode::from_auto_conversion`: the graph made it to turn a
    /// colour or a number into a closure.
    pub from_auto_conversion: bool,
    /// `AttributeNode::stochastic_sample`: a volume may sample it at
    /// random, since nothing after it is nonlinear.
    pub stochastic_sample: bool,
}

impl ShaderNode {
    /// A node of the type named so, as its class makes one: an input for
    /// each linkable socket and an output for each output. `None` when
    /// `data/node.tsv` has no such type.
    pub fn new(type_name: &str) -> Option<ShaderNode> {
        let node = Node::new(type_name)?;
        let linkable = flag("SocketType::LINKABLE");
        let socket = || node.socket_list().map(|(socket, _)| socket);
        let input: Vec<ShaderInput> = socket().filter(|socket| socket.direction != SocketDirection::Output && socket.flag & linkable != 0).map(ShaderInput::new).collect();
        let output: Vec<ShaderOutput> = socket().filter(|socket| socket.direction == SocketDirection::Output).map(ShaderOutput::new).collect();
        let special = table().iter().find(|row| row.first() == Some(&"special") && row.get(1) == Some(&type_name)).and_then(|row| row.get(2));
        // A convert node from a type to itself is a proxy.
        let proxy = node.node_type().class == "ConvertNode" && input.first().map(ShaderInput::kind) == output.first().map(ShaderOutput::kind);
        let special_type = match (proxy, special) {
            (true, _) => ShaderNodeSpecialType::Proxy,
            (false, Some(name)) => ShaderNodeSpecialType::named(name),
            (false, None) => ShaderNodeSpecialType::None,
        };
        let stochastic_sample = node.node_type().class == "AttributeNode";
        Some(ShaderNode { node, input, output, id: 0, bump: ShaderBump::None, bump_filter_width: 0.0, special_type, need_derivative: false, added_to_svm: false, from_auto_conversion: false, stochastic_sample })
    }

    /// The node's type, by name.
    pub fn type_name(&self) -> &'static str {
        &self.node.node_type().name
    }

    /// `clone`, the copy constructor: the same settings, values, bump
    /// sample and special type, with no links.
    pub fn copy(&self) -> ShaderNode {
        let unlinked = |input: &ShaderInput| ShaderInput::new(input.socket_type);
        let mut node = self.node.clone();
        node.tag_modified();
        ShaderNode {
            node,
            input: self.input.iter().map(unlinked).collect(),
            output: self.output.iter().map(|output| ShaderOutput::new(output.socket_type)).collect(),
            bump_filter_width: 0.0,
            need_derivative: false,
            added_to_svm: false,
            ..self.clone()
        }
    }

    /// `input(name)`: the place of the input named so, such as `Fac`.
    pub fn input_named(&self, name: &str) -> Option<usize> {
        self.input.iter().position(|input| input.name() == name)
    }

    /// `output(name)`.
    pub fn output_named(&self, name: &str) -> Option<usize> {
        self.output.iter().position(|output| output.name() == name)
    }

    /// What the node's class answers to a yes-or-no question of
    /// `ShaderNode`, such as `has_volume_support`: `true`, or `code` when
    /// the class works the answer out. `None` is no.
    pub fn says(&self, question: &str) -> Option<&'static str> {
        let row = table().iter().find(|row| row.first() == Some(&"flag") && row.get(1) == Some(&self.type_name()) && row.get(2) == Some(&question));
        row.and_then(|row| row.get(3).copied())
    }

    /// The node's class has its own version of this function of
    /// `ShaderNode`, such as `constant_fold`.
    pub fn has_override(&self, function: &str) -> bool {
        table().iter().any(|row| row.first() == Some(&"override") && row.get(1) == Some(&self.type_name()) && row.get(2) == Some(&function))
    }

    /// `has_volume_support`: a volume can be made of it.
    pub fn has_volume_support(&self) -> bool {
        self.says("has_volume_support") == Some("true")
    }

    /// `equals`: the two could be one node. They are of one type and one
    /// bump sample, their settings are the same, and each input either
    /// has the same value in both or is linked to the same output. The
    /// classes that have their own `equals` all say no, but for the two
    /// image nodes, which are taken to say no here.
    pub fn equals(&self, other: &ShaderNode) -> bool {
        if self.type_name() != other.type_name() || self.bump != other.bump || self.has_override("equals") {
            return false;
        }
        let linkable = flag("SocketType::LINKABLE");
        let same_value = |name: &str| self.node.get(name) == other.node.get(name);
        let mut setting = self.node.socket_list().filter(|(socket, _)| socket.direction != SocketDirection::Output && socket.flag & linkable == 0);
        let mut input = self.input.iter().zip(&other.input);
        setting.all(|(socket, _)| same_value(&socket.name))
            && input.all(|(own, their)| match (own.link, their.link) {
                (None, None) => same_value(&own.socket_type.name),
                (Some(own), Some(their)) => own == their,
                _ => false,
            })
    }
}
