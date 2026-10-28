//! Geometry Nodes modifier (`NodesModifierData`, `eModifierType_Nodes`
//! 57, Blender `MOD_nodes.cc`), for the one group the files use: the
//! bundled "Smooth by Angle" (4.1's "Auto Smooth"), which changes shading
//! only — an edge goes sharp where its faces meet at more than the angle,
//! every face goes smooth — so the mesh's faces and vertices pass through
//! untouched and only `face_smooth` and `sharp_edges` change.
//!
//! The group is read with the shader editor's tree reader
//! (`shade/graph.rs`) and run by `nodes_field.rs`: the geometry flows from
//! Group Input through Set Shade Smooth nodes to Group Output
//! (`geometry_chain`), and each Set Shade Smooth evaluates its Selection
//! and Shade Smooth fields on its domain over the mesh as the previous one
//! left it. A tree with any other node on that path, or one feeding those
//! fields through a node type `nodes_field.rs` does not have, is not read,
//! and the modifier is reported as skipped as it was before (`report.rs`).

use super::nodes_field::{BCompare, BDomain, BEval, BInput, shade_domain, shade_fields};
use super::nodes_topo::BTopo;
use super::{BModifier, BlendEvalMesh, EvalCtx};
use crate::blend::file::types::*;
use crate::blend::shade::graph::{BlendNode, BlendNodeGraph};
use std::collections::HashMap;

/// `eModifierType_Nodes`.
pub(crate) const DNA_TYPE: u32 = 57;

/// The node types `nodes_field.rs` evaluates.
const FIELD_NODES: &[&str] = &[
    "NodeReroute",
    "NodeGroupInput",
    "GeometryNodeInputMeshEdgeAngle",
    "GeometryNodeInputEdgeSmooth",
    "GeometryNodeInputShadeSmooth",
    "FunctionNodeBooleanMath",
    "FunctionNodeCompare",
];

#[derive(Clone)]
pub(crate) struct BNodes {
    graph: BlendNodeGraph,
    inputs: HashMap<String, BInput>,
    compares: HashMap<String, BCompare>,
    /// The Set Shade Smooth nodes on the geometry path, input to output.
    chain: Vec<String>,
}

/// `NodesModifierData.node_group` read as a graph and its inputs; `None`
/// when the group is missing or not one this evaluates.
pub(crate) fn read(f: &BlendFile, md: &BlendBlock, _ob: &BlendBlock, dna: &DnaType) -> Option<Box<dyn BModifier>> {
    let tree = f.ptr(md, f.off("NodesModifierData", "*node_group", dna));
    if tree == 0 {
        return None;
    }
    let graph = f.node_graph_at(tree, dna)?;
    let inputs = f.nodes_inputs(md, tree, dna)?;
    let compares = graph
        .nodes
        .iter()
        .filter(|n| n.idname == "FunctionNodeCompare")
        .map(|n| (n.name.clone(), compare_storage(f, n.storage, dna)))
        .collect();
    from_parts(graph, inputs, compares).map(|m| Box::new(m) as Box<dyn BModifier>)
}

/// A tree, its inputs and its Compare settings as an evaluator — the
/// file's own group, the one Blender's legacy conversion invents
/// (`nodes_legacy.rs`), or a test's. `None` when this evaluator does not
/// cover the tree.
pub(crate) fn from_parts(graph: BlendNodeGraph, inputs: HashMap<String, BInput>, compares: HashMap<String, BCompare>) -> Option<BNodes> {
    let chain = geometry_chain(&graph)?;
    let built = BNodes { graph, inputs, compares, chain };
    built.supported().then_some(built)
}

/// `NodeFunctionCompare` from the node's `storage` (zeros without one).
fn compare_storage(f: &BlendFile, storage: u64, dna: &DnaType) -> BCompare {
    let Some(block) = f.block_at(storage) else { return BCompare::default() };
    let field = |name: &str| f.int_field(block, "NodeFunctionCompare", name, dna) as i32;
    BCompare { operation: field("operation"), data_type: field("data_type"), mode: field("mode") }
}

/// The Set Shade Smooth nodes between Group Input and Group Output,
/// following each node's `Geometry` socket back from the output; `None`
/// when anything else sits on that path.
fn geometry_chain(graph: &BlendNodeGraph) -> Option<Vec<String>> {
    let output = graph.nodes.iter().filter(|n| n.idname == "NodeGroupOutput").max_by_key(|n| n.active_output())?;
    let mut to = (output.name.as_str(), output.inputs.first()?.identifier.as_str());
    let mut chain = Vec::new();
    for _ in 0..graph.nodes.len() + 1 {
        let link = graph.links.iter().find(|l| l.to == to.0 && l.to_socket == to.1)?;
        let node = graph.nodes.iter().find(|n| n.name == link.from)?;
        if node.muted() {
            return None;
        }
        match node.idname.as_str() {
            "NodeGroupInput" => {
                chain.reverse();
                return Some(chain);
            }
            "GeometryNodeSetShadeSmooth" => {
                chain.push(node.name.clone());
                to = (node.name.as_str(), "Geometry");
            }
            "NodeReroute" => to = (node.name.as_str(), "Input"),
            _ => return None,
        }
    }
    None
}

impl BNodes {
    fn node(&self, name: &str) -> Option<&BlendNode> {
        self.graph.nodes.iter().find(|n| n.name == name)
    }

    /// Whether every node feeding the chain's fields is one the evaluator
    /// runs, with settings it runs.
    fn supported(&self) -> bool {
        let mut pending: Vec<(&str, &str)> =
            self.chain.iter().flat_map(|n| [(n.as_str(), "Selection"), (n.as_str(), "Shade Smooth")]).collect();
        let mut seen = std::collections::HashSet::new();
        while let Some((node, socket)) = pending.pop() {
            let Some(link) = self.graph.links.iter().find(|l| l.to == node && l.to_socket == socket) else { continue };
            let Some(from) = self.node(&link.from) else { return false };
            if from.muted() || !FIELD_NODES.contains(&from.idname.as_str()) {
                return false;
            }
            let ok = match from.idname.as_str() {
                "NodeGroupInput" => self.inputs.contains_key(&link.from_socket),
                "GeometryNodeInputMeshEdgeAngle" => link.from_socket == "Unsigned Angle",
                "FunctionNodeBooleanMath" => (0..=8).contains(&from.custom1),
                "FunctionNodeCompare" => {
                    let c = self.compares.get(&from.name).copied().unwrap_or_default();
                    c.data_type == 0 && c.mode == 0 && (0..=5).contains(&c.operation)
                }
                _ => true,
            };
            if !ok {
                return false;
            }
            if seen.insert(from.name.as_str()) {
                pending.extend(from.inputs.iter().map(|s| (from.name.as_str(), s.identifier.as_str())));
            }
        }
        true
    }
}

impl BModifier for BNodes {
    /// `MOD_nodes.cc` `modify_mesh` for this group: each Set Shade Smooth
    /// in turn over the flags the previous one left, then the flags back
    /// on the mesh. A field that cannot be evaluated leaves the mesh as it
    /// was, which `supported` makes unreachable from a file.
    fn apply(&self, mesh: &mut BlendEvalMesh, _ctx: &EvalCtx) {
        let topo = BTopo::of(mesh);
        let marked: std::collections::HashSet<(u32, u32)> = mesh.sharp_edges.iter().map(|&(a, b)| (a.min(b), a.max(b))).collect();
        let mut sharp: Vec<bool> = topo.edges.iter().map(|e| marked.contains(e)).collect();
        // No `sharp_face` attribute means every face is smooth.
        let mut smooth: Vec<bool> = (0..mesh.faces.len()).map(|f| mesh.face_smooth.get(f).copied().unwrap_or(true)).collect();
        for name in &self.chain {
            let Some(node) = self.node(name) else { return };
            let domain = shade_domain(node);
            let eval = BEval { graph: &self.graph, inputs: &self.inputs, compares: &self.compares, topo: &topo, mesh, sharp_edge: &sharp, smooth_face: &smooth };
            let Some((selection, want)) = shade_fields(&eval, node, domain) else { return };
            match domain {
                BDomain::Face => (0..smooth.len()).filter(|&i| selection[i]).for_each(|i| smooth[i] = want[i]),
                BDomain::Edge => (0..sharp.len()).filter(|&i| selection[i]).for_each(|i| sharp[i] = !want[i]),
            }
        }
        mesh.face_smooth = smooth;
        mesh.sharp_edges = topo.edges.iter().zip(&sharp).filter(|(_, s)| **s).map(|(&e, _)| e).collect();
    }

    fn clone_box(&self) -> Box<dyn BModifier> {
        Box::new(self.clone())
    }
}
