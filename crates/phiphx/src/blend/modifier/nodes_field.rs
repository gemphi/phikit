//! The field evaluator behind the Geometry Nodes modifier: the few node
//! types the "Smooth by Angle" group uses, evaluated per edge or per face
//! over an evaluated mesh, and the Set Shade Smooth nodes they feed.
//!
//! A field is asked for on a domain and answers with one value per element
//! of that domain. Attribute reads are adapted to the asked domain first,
//! as Blender's `AttributeFieldInput` is (`mesh_attributes.cc`,
//! `adapt_mesh_domain_*`: a bool face → edge is true when ANY face is, a
//! bool edge → face when ALL edges are, a float either way the mean), and
//! only then inverted — which is why the Is Shade Smooth node on the edge
//! domain is "no adjacent face is flat", not "some adjacent face is smooth".
//!
//! | node | Blender source |
//! |---|---|
//! | Edge Angle | `node_geo_input_mesh_edge_angle.cc` (Unsigned Angle) |
//! | Is Edge Smooth | `node_geo_input_edge_smooth.cc` |
//! | Is Shade Smooth | `node_geo_input_shade_smooth.cc` |
//! | Boolean Math | `node_fn_boolean_math.cc` |
//! | Compare | `node_fn_compare.cc` (Float, Element) |
//! | Set Shade Smooth | `node_geo_set_shade_smooth.cc` |
//!
//! The edges the fields are evaluated over are [`BTopo`] (`nodes_topo.rs`).

use super::BlendEvalMesh;
use super::nodes_corners::face_normal_calc;
use super::nodes_ops::{angle_normalized, boolean_math, compare, invert};
use super::nodes_topo::BTopo;
use crate::blend::shade::graph::{BlendNode, BlendNodeGraph};
use crate::blend::shade::socket::BSocketDefault;
use std::collections::HashMap;

/// `AttrDomain`, the two the tree writes to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum BDomain {
    Edge,
    Face,
}

/// A field evaluated on one domain.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum BField {
    Bool(Vec<bool>),
    Float(Vec<f32>),
}

/// A group input's value from the modifier's settings.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum BInput {
    Float(f32),
    Bool(bool),
    Int(i32),
}

/// `NodeFunctionCompare` as read from a Compare node's storage.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct BCompare {
    pub operation: i32,
    pub data_type: i32,
    pub mode: i32,
}

/// One evaluation over the mesh as it stands between two Set Shade Smooth
/// nodes: the sharp flags it reads are the current ones.
pub(crate) struct BEval<'a> {
    pub graph: &'a BlendNodeGraph,
    pub inputs: &'a HashMap<String, BInput>,
    pub compares: &'a HashMap<String, BCompare>,
    pub topo: &'a BTopo,
    pub mesh: &'a BlendEvalMesh,
    /// The flags as they stand, which the modifier updates between nodes.
    pub sharp_edge: &'a [bool],
    pub smooth_face: &'a [bool],
}

impl BEval<'_> {
    fn size(&self, d: BDomain) -> usize {
        match d {
            BDomain::Edge => self.topo.edges.len(),
            BDomain::Face => self.mesh.faces.len(),
        }
    }

    fn node(&self, name: &str) -> Option<&BlendNode> {
        self.graph.nodes.iter().find(|n| n.name == name)
    }

    /// The value an input socket carries on `d`: its link's, else its own
    /// default.
    pub fn input(&self, node: &BlendNode, socket: &str, d: BDomain) -> Option<BField> {
        if let Some(l) = self.graph.links.iter().find(|l| l.to == node.name && l.to_socket == socket) {
            let from = self.node(&l.from)?;
            return self.output(from, &l.from_socket, d);
        }
        let n = self.size(d);
        match node.inputs.iter().find(|s| s.identifier == socket)?.default.as_ref()? {
            BSocketDefault::Float(v) => Some(BField::Float(vec![*v; n])),
            BSocketDefault::Bool(v) => Some(BField::Bool(vec![*v; n])),
            BSocketDefault::Int(v) => Some(BField::Float(vec![*v as f32; n])),
            _ => None,
        }
    }

    /// An output socket's field on `d`; `None` for a node this cannot run.
    fn output(&self, node: &BlendNode, socket: &str, d: BDomain) -> Option<BField> {
        if node.muted() {
            return None;
        }
        let n = self.size(d);
        match node.idname.as_str() {
            "NodeReroute" => self.input(node, "Input", d),
            "NodeGroupInput" => Some(match *self.inputs.get(socket)? {
                BInput::Float(v) => BField::Float(vec![v; n]),
                BInput::Bool(v) => BField::Bool(vec![v; n]),
                BInput::Int(v) => BField::Float(vec![v as f32; n]),
            }),
            "GeometryNodeInputMeshEdgeAngle" if socket == "Unsigned Angle" => Some(self.adapt(BField::Float(self.edge_angles()), BDomain::Edge, d)),
            "GeometryNodeInputEdgeSmooth" => Some(invert(self.adapt(BField::Bool(self.sharp_edge.to_vec()), BDomain::Edge, d))),
            "GeometryNodeInputShadeSmooth" => {
                let sharp = self.smooth_face.iter().map(|&s| !s).collect();
                Some(invert(self.adapt(BField::Bool(sharp), BDomain::Face, d)))
            }
            "FunctionNodeBooleanMath" => {
                let a = self.input(node, "Boolean", d)?.bools()?;
                let b = match node.custom1 {
                    2 => vec![false; n],
                    _ => self.input(node, "Boolean_001", d)?.bools()?,
                };
                let op = boolean_math(node.custom1)?;
                Some(BField::Bool(a.iter().zip(&b).map(|(&a, &b)| op(a, b)).collect()))
            }
            "FunctionNodeCompare" => {
                let c = self.compares.get(&node.name).copied().unwrap_or_default();
                // Float, element-wise; the other types and modes are not the tree's.
                if c.data_type != 0 || c.mode != 0 {
                    return None;
                }
                let a = self.input(node, "A", d)?.floats()?;
                let b = self.input(node, "B", d)?.floats()?;
                let eps = self.input(node, "Epsilon", d).and_then(|f| f.floats()).unwrap_or_else(|| vec![0.001; n]);
                let op = compare(c.operation)?;
                Some(BField::Bool((0..n).map(|i| op(a[i], b[i], eps[i])).collect()))
            }
            _ => None,
        }
    }

    /// `adapt_mesh_domain_*` between the edge and face domains.
    fn adapt(&self, f: BField, from: BDomain, to: BDomain) -> BField {
        if from == to {
            return f;
        }
        let faces = &self.mesh.faces;
        match (f, to) {
            // Face → edge: any face true; the mean of the faces' values.
            (BField::Bool(v), BDomain::Edge) => {
                let mut out = vec![false; self.topo.edges.len()];
                for (fi, face) in faces.iter().enumerate() {
                    if v[fi] {
                        self.topo.face_edges(face).for_each(|e| out[e] = true);
                    }
                }
                BField::Bool(out)
            }
            (BField::Float(v), BDomain::Edge) => {
                let (mut sum, mut count) = (vec![0.0f32; self.topo.edges.len()], vec![0u32; self.topo.edges.len()]);
                for (fi, face) in faces.iter().enumerate() {
                    for e in self.topo.face_edges(face) {
                        sum[e] += v[fi];
                        count[e] += 1;
                    }
                }
                BField::Float(sum.iter().zip(&count).map(|(&s, &c)| if c > 0 { s / c as f32 } else { 0.0 }).collect())
            }
            // Edge → face: every edge true; the mean of the edges' values.
            (BField::Bool(v), BDomain::Face) => BField::Bool(faces.iter().map(|face| self.topo.face_edges(face).all(|e| v[e])).collect()),
            (BField::Float(v), BDomain::Face) => BField::Float(
                faces
                    .iter()
                    .map(|face| {
                        let (s, c) = self.topo.face_edges(face).fold((0.0, 0u32), |(s, c), e| (s + v[e], c + 1));
                        if c > 0 { s / c as f32 } else { 0.0 }
                    })
                    .collect(),
            ),
        }
    }

    /// `AngleFieldInput`: the angle between the two faces' normals
    /// (`angle_normalized_v3v3`), 0 for a flat, boundary or non-manifold edge.
    fn edge_angles(&self) -> Vec<f32> {
        let normal = |f: i32| {
            let face = self.mesh.faces.get(f as usize);
            if face.len() < 3 || face.iter().any(|&v| v as usize >= self.mesh.vertices.len()) {
                return [0.0, 0.0, 1.0];
            }
            face_normal_calc(&self.mesh.vertices, face)
        };
        self.topo
            .edge_faces
            .iter()
            .map(|&[a, b]| if a < 0 || b < 0 { 0.0 } else { angle_normalized(normal(a), normal(b)) })
            .collect()
    }
}

/// A Set Shade Smooth node's Selection and Shade Smooth fields on `d`
/// (`set_sharp`): where the selection holds, the element's sharp flag
/// becomes the inverse of Shade Smooth, and `try_capture_field_on_geometry`
/// leaves the rest as they were. `None` when a field could not be evaluated.
pub(crate) fn shade_fields(eval: &BEval, node: &BlendNode, d: BDomain) -> Option<(Vec<bool>, Vec<bool>)> {
    let selection = eval.input(node, "Selection", d)?.bools()?;
    let smooth = eval.input(node, "Shade Smooth", d)?.bools()?;
    Some((selection, smooth))
}

/// `node.custom1` of a Set Shade Smooth node as a domain: Edge (1) or
/// Face, which is also what a node saved before the domain existed means.
pub(crate) fn shade_domain(node: &BlendNode) -> BDomain {
    match node.custom1 {
        1 => BDomain::Edge,
        _ => BDomain::Face,
    }
}
