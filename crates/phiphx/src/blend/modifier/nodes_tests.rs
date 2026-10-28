//! The Geometry Nodes evaluator on a hand-built "Smooth by Angle" tree:
//! the asset's wiring (`release/datafiles/assets`, dumped by
//! `tests/fixtures/make_smooth_by_angle.py`'s Blender) over a cube, and
//! the corner normals Blender draws the result with.

use super::nodes::from_parts;
use super::nodes_corners::{corner_normals, safe_acos_approx};
use super::nodes_field::{BCompare, BInput};
use super::{BModifier, BlendEvalMesh, EvalCtx};
use crate::blend::shade::graph::{BlendNode, BlendNodeGraph, BlendNodeLink};
use crate::blend::shade::socket::{BSocketDefault, BlendSocket};
use std::collections::HashMap;

fn socket(id: &str, default: Option<BSocketDefault>) -> BlendSocket {
    BlendSocket { identifier: id.into(), name: id.into(), default, ..Default::default() }
}

fn node(name: &str, idname: &str, custom1: i32, inputs: &[(&str, Option<BSocketDefault>)], outputs: &[&str]) -> BlendNode {
    BlendNode {
        name: name.into(),
        idname: idname.into(),
        custom1,
        inputs: inputs.iter().map(|(id, d)| socket(id, d.clone())).collect(),
        outputs: outputs.iter().map(|id| socket(id, None)).collect(),
        ..Default::default()
    }
}

fn link(from: &str, from_socket: &str, to: &str, to_socket: &str) -> BlendNodeLink {
    BlendNodeLink { from: from.into(), from_socket: from_socket.into(), to: to.into(), to_socket: to_socket.into() }
}

/// The asset: Set Shade Smooth (edge) with Selection = edge smooth OR
/// ignore, Shade Smooth = angle <= Angle AND (face smooth OR ignore), then
/// Set Shade Smooth (face) with constant true inputs.
fn smooth_by_angle() -> BlendNodeGraph {
    let t = Some(BSocketDefault::Bool(true));
    let f = Some(BSocketDefault::Float(0.0));
    let nodes = vec![
        node("In", "NodeGroupInput", 0, &[], &["Input_0", "Input_1", "Socket_1"]),
        node("Out", "NodeGroupOutput", 0, &[("Output_2", None)], &[]),
        node("Edge", "GeometryNodeSetShadeSmooth", 1, &[("Geometry", None), ("Selection", t.clone()), ("Shade Smooth", t.clone())], &["Geometry"]),
        node("Face", "GeometryNodeSetShadeSmooth", 2, &[("Geometry", None), ("Selection", t.clone()), ("Shade Smooth", t.clone())], &["Geometry"]),
        node("Angle", "GeometryNodeInputMeshEdgeAngle", 0, &[], &["Unsigned Angle", "Signed Angle"]),
        node("EdgeSmooth", "GeometryNodeInputEdgeSmooth", 0, &[], &["Smooth"]),
        node("ShadeSmooth", "GeometryNodeInputShadeSmooth", 0, &[], &["Smooth"]),
        node("Compare", "FunctionNodeCompare", 0, &[("A", f.clone()), ("B", f)], &["Result"]),
        node("And", "FunctionNodeBooleanMath", 0, &[("Boolean", None), ("Boolean_001", None)], &["Boolean"]),
        node("OrSel", "FunctionNodeBooleanMath", 1, &[("Boolean", None), ("Boolean_001", None)], &["Boolean"]),
        node("OrFace", "FunctionNodeBooleanMath", 1, &[("Boolean", None), ("Boolean_001", None)], &["Boolean"]),
    ];
    let links = vec![
        link("In", "Input_0", "Edge", "Geometry"),
        link("Edge", "Geometry", "Face", "Geometry"),
        link("Face", "Geometry", "Out", "Output_2"),
        link("Angle", "Unsigned Angle", "Compare", "A"),
        link("In", "Input_1", "Compare", "B"),
        link("Compare", "Result", "And", "Boolean"),
        link("ShadeSmooth", "Smooth", "OrFace", "Boolean"),
        link("In", "Socket_1", "OrFace", "Boolean_001"),
        link("OrFace", "Boolean", "And", "Boolean_001"),
        link("And", "Boolean", "Edge", "Shade Smooth"),
        link("EdgeSmooth", "Smooth", "OrSel", "Boolean"),
        link("In", "Socket_1", "OrSel", "Boolean_001"),
        link("OrSel", "Boolean", "Edge", "Selection"),
    ];
    BlendNodeGraph { nodes, links }
}

fn modifier(angle_deg: f32, ignore: bool) -> Box<dyn BModifier> {
    let inputs = HashMap::from([("Input_1".to_string(), BInput::Float(angle_deg.to_radians())), ("Socket_1".to_string(), BInput::Bool(ignore))]);
    let compares = HashMap::from([("Compare".to_string(), BCompare { operation: 1, data_type: 0, mode: 0 })]);
    Box::new(from_parts(smooth_by_angle(), inputs, compares).expect("the asset's tree is supported"))
}

fn cube() -> BlendEvalMesh {
    let v = |x: f32, y: f32, z: f32| [x, y, z];
    BlendEvalMesh {
        vertices: vec![v(-1., -1., -1.), v(1., -1., -1.), v(1., 1., -1.), v(-1., 1., -1.), v(-1., -1., 1.), v(1., -1., 1.), v(1., 1., 1.), v(-1., 1., 1.)],
        faces: [vec![0u32, 3, 2, 1], vec![4, 5, 6, 7], vec![0, 1, 5, 4], vec![2, 3, 7, 6], vec![0, 4, 7, 3], vec![1, 2, 6, 5]].iter().cloned().collect(),
        face_smooth: vec![true; 6],
        ..Default::default()
    }
}

fn run(m: &mut BlendEvalMesh, modifier: &dyn BModifier) {
    let worlds = HashMap::new();
    modifier.apply(m, &EvalCtx { world: glam::Mat4::IDENTITY, worlds: &worlds });
}

#[test]
fn a_cube_goes_sharp_under_its_angle_and_smooth_over_it() {
    let mut m = cube();
    run(&mut m, &*modifier(30.0, false));
    assert_eq!(m.sharp_edges.len(), 12, "every 90° edge is sharp at 30°");
    assert_eq!(m.face_smooth, vec![true; 6]);
    let mut m = cube();
    run(&mut m, &*modifier(100.0, false));
    assert!(m.sharp_edges.is_empty(), "none at 100°: {:?}", m.sharp_edges);
}

#[test]
fn marked_edges_stay_sharp_unless_sharpness_is_ignored() {
    // Selection is "edge smooth OR ignore": a marked edge is unselected,
    // so `try_capture_field_on_geometry` leaves its flag alone.
    let mut m = cube();
    m.sharp_edges = vec![(4, 5), (6, 7)];
    run(&mut m, &*modifier(100.0, false));
    assert_eq!(m.sharp_edges, vec![(4, 5), (6, 7)]);
    let mut m = cube();
    m.sharp_edges = vec![(4, 5), (6, 7)];
    run(&mut m, &*modifier(100.0, true));
    assert!(m.sharp_edges.is_empty());
}

#[test]
fn flat_faces_sharpen_every_edge_they_touch_and_end_up_smooth() {
    // Is Shade Smooth on the edge domain is `!(any face sharp)`: the
    // attribute is adapted face → edge (any) BEFORE the node inverts it.
    let mut m = cube();
    m.face_smooth = vec![false, true, false, true, true, true];
    run(&mut m, &*modifier(100.0, false));
    let mut want = vec![(0, 1), (0, 3), (0, 4), (1, 2), (1, 5), (2, 3), (4, 5)];
    want.sort();
    let mut got = m.sharp_edges.clone();
    got.sort();
    assert_eq!(got, want);
    assert_eq!(m.face_smooth, vec![true; 6], "the face node smooths everything");
}

#[test]
fn loose_edges_are_carried_and_stay_smooth() {
    let mut m = cube();
    m.vertices.extend([[3.0, 0.0, 0.0], [4.0, 0.0, 0.0]]);
    m.loose_edges = vec![(8, 9)];
    run(&mut m, &*modifier(30.0, false));
    // The loose edge has no face to be flat, so it is smooth (Blender's
    // Grid10 case); the twelve cube edges are sharp.
    assert_eq!(m.sharp_edges.len(), 12);
    assert!(!m.sharp_edges.contains(&(8, 9)));
}

#[test]
fn sharp_edges_split_corner_normals_into_face_normals() {
    let mut m = cube();
    run(&mut m, &*modifier(30.0, false));
    let n = corner_normals(&m);
    assert_eq!(n.len(), 24);
    // Face 0 (bottom) faces -Z at every corner; face 2 faces -Y.
    for c in 0..4 {
        assert_eq!(n[c], [0.0, 0.0, -1.0]);
        assert_eq!(n[8 + c], [0.0, -1.0, 0.0]);
    }
}

#[test]
fn smooth_corners_take_the_vertex_normal() {
    let mut m = cube();
    run(&mut m, &*modifier(100.0, false));
    let n = corner_normals(&m);
    let s = 1.0 / 3f32.sqrt();
    // Corner 0 is vertex 0 (-1, -1, -1).
    for k in 0..3 {
        assert!((n[0][k] + s).abs() < 1e-6, "{:?}", n[0]);
    }
}

#[test]
fn one_sharp_edge_opens_a_fan_and_two_split_it() {
    // Vertex 0 sits in faces 0 (bottom), 2 (front) and 4 (left), which meet
    // in a ring: edge 0-1 between 0 and 2, edge 0-4 between 2 and 4, edge
    // 0-3 between 4 and 0. Corner 0 is face 0's at this vertex, corner 8
    // face 2's, corner 16 face 4's.
    let smooth = 1.0 / 3f32.sqrt();
    // ONE sharp edge only OPENS the ring — the fan walks round the other
    // way (`traverse_fan_local_corners` travels both directions), so all
    // three corners keep the smooth normal.
    let mut m = cube();
    m.sharp_edges = vec![(0, 1)];
    let n = corner_normals(&m);
    for c in [0, 8, 16] {
        assert!((0..3).all(|k| (n[c][k] + smooth).abs() < 1e-6), "corner {c}: {:?}", n[c]);
    }
    // TWO sharp edges round face 0 cut it out: its corner takes its own face
    // normal (a fan of one corner is the face's normal exactly), and faces 2
    // and 4 stay in one fan.
    let mut m = cube();
    m.sharp_edges = vec![(0, 1), (0, 3)];
    let n = corner_normals(&m);
    assert_eq!(n[0], [0.0, 0.0, -1.0]);
    let front = n[8];
    assert!(front[2].abs() < 1e-6 && front[0] < 0.0 && front[1] < 0.0, "{front:?}");
    assert_eq!(n[8], n[16], "faces 2 and 4 share the fan at vertex 0");
}

#[test]
fn the_acos_approximation_is_blenders() {
    for i in 0..=200 {
        let x = -1.0 + i as f32 / 100.0;
        assert!((safe_acos_approx(x) - x.acos()).abs() < 5e-5, "{x}");
    }
}
