//! Shader graph model - node-based material authoring canvas.
//!
//! Provides a node editor UI with draggable nodes, connectable input/output
//! ports, and connection wires between ports.
use crate::prelude::*;

// ═══════════════════════════════════════════════════════════════════════
// Port & Node types
// ═══════════════════════════════════════════════════════════════════════

/// Direction of a shader port (input receives data, output emits data).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PortDirection {
    Input,
    Output,
}

/// A connectable port on a shader node.
#[derive(Debug, Clone)]
pub struct NSShaderPort {
    pub name: String,
    pub direction: PortDirection,
    pub port_type: String, // e.g. "float", "vec3", "color", "texture2d"
}

impl NSShaderPort {
    pub fn input(name: &str, port_type: &str) -> Self {
        Self {
            name: name.to_string(),
            direction: PortDirection::Input,
            port_type: port_type.to_string(),
        }
    }
    pub fn output(name: &str, port_type: &str) -> Self {
        Self {
            name: name.to_string(),
            direction: PortDirection::Output,
            port_type: port_type.to_string(),
        }
    }
}

/// A node in the shader graph with input and output ports.
#[derive(Debug, Clone)]
pub struct NSShaderNode {
    pub id: u32,
    pub title: String,
    pub rect: NSRect,
    pub input_ports: Vec<NSShaderPort>,
    pub output_ports: Vec<NSShaderPort>,
}

/// A connection between two ports on two nodes.
#[derive(Debug, Clone)]
pub struct NSShaderConnection {
    pub from_node: u32,
    pub from_port: usize,
    pub to_node: u32,
    pub to_port: usize,
}

/// Mutable state of the shader graph (nodes, connections, selection).
#[derive(Debug, Clone, Default)]
pub struct NSShaderGraphState {
    pub nodes: Vec<NSShaderNode>,
    pub connections: Vec<NSShaderConnection>,
    pub selected_node: Option<u32>,
}

// ═══════════════════════════════════════════════════════════════════════
// NSShaderGraphView
// ═══════════════════════════════════════════════════════════════════════

/// Node-based material editor canvas with draggable nodes and connection wires.
pub struct NSShaderGraphView {
    pub view: NSView,
    pub state: NSShaderGraphState,
}

impl NSShaderGraphView {
    pub fn new(rect: NSRect) -> Self {
        let mut view = NSView::new(rect).with_name("ShaderGraphView");
        view.set_autoresizing_mask(
            NSAutoresizingMask::WIDTH_SIZABLE | NSAutoresizingMask::HEIGHT_SIZABLE,
        );
        Self {
            view,
            state: NSShaderGraphState::default(),
        }
    }

    pub fn view(&self) -> &NSView {
        &self.view
    }

    /// Add a node to the graph canvas.
    pub fn add_node(&mut self, node: NSShaderNode) {
        self.state.nodes.push(node);
    }

    /// Connect an output port on one node to an input port on another.
    pub fn add_connection(&mut self, connection: NSShaderConnection) {
        self.state.connections.push(connection);
    }

    /// Get all nodes in the graph.
    pub fn nodes(&self) -> &[NSShaderNode] {
        &self.state.nodes
    }

    /// Get all connections in the graph.
    pub fn connections(&self) -> &[NSShaderConnection] {
        &self.state.connections
    }

    /// Select a node by id.
    pub fn select_node(&mut self, id: Option<u32>) {
        self.state.selected_node = id;
    }
}
