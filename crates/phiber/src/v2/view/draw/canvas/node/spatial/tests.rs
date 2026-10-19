//! The spatial view's one load-bearing claim: seen from straight above, it is
//! the flat editor. Every socket the flat `plan` paints is where a tube end
//! projects at pitch 90, and every card's corner is where its solid's is.

use super::camera::{BOrbit, BSpatialCamera};
use super::layout::BSpatialLayout;
use super::super::plan::{BCardSize, plan};
use super::super::select::BNodeSelection;
use super::super::view::BNodeView;
use crate::phiphx::blend::shade::graph::{BlendNode, BlendNodeGraph, BlendNodeLink};
use crate::phiphx::blend::shade::socket::BlendSocket;

fn sock(identifier: &str) -> BlendSocket {
    BlendSocket { identifier: identifier.into(), name: identifier.into(), kind: 0, ..Default::default() }
}

fn node(name: &str, idname: &str, x: f32, y: f32, inputs: &[&str], outputs: &[&str]) -> BlendNode {
    BlendNode {
        name: name.into(),
        idname: idname.into(),
        x,
        y,
        width: 140.0,
        inputs: inputs.iter().map(|s| sock(s)).collect(),
        outputs: outputs.iter().map(|s| sock(s)).collect(),
        ..Default::default()
    }
}

/// Three nodes of three classes and two links — a texture into a shader into
/// the output, which is the smallest graph that has every kind of solid.
fn graph() -> BlendNodeGraph {
    let mut output = node("Material Output", "ShaderNodeOutputMaterial", 400.0, 100.0, &["Surface"], &[]);
    output.flag |= crate::phiphx::blend::shade::graph::NODE_DO_OUTPUT;
    BlendNodeGraph {
        nodes: vec![
            node("Image Texture", "ShaderNodeTexImage", -200.0, 140.0, &["Vector"], &["Color", "Alpha"]),
            node("Principled BSDF", "ShaderNodeBsdfPrincipled", 100.0, 120.0, &["Base Color", "Roughness"], &["BSDF"]),
            output,
        ],
        links: vec![
            BlendNodeLink { from: "Image Texture".into(), from_socket: "Color".into(), to: "Principled BSDF".into(), to_socket: "Base Color".into() },
            BlendNodeLink { from: "Principled BSDF".into(), from_socket: "BSDF".into(), to: "Material Output".into(), to_socket: "Surface".into() },
        ],
    }
}

fn size() -> BCardSize {
    BCardSize { header: 30.0, unit: 36.0, width: 140.0 }
}

const REGION: (f32, f32) = (900.0, 600.0);

/// The flat editor places its cards by FRACTION of the canvas and sizes them by
/// Blender's `(width + 1) / cur_width` zoom (`view/mod.rs`), which differ by
/// one part in nine hundred here; the camera follows the zoom, as
/// `BNodeView::to_region` does. So the two agree to within a pixel, not to
/// the sub-pixel.
const PIXEL: f32 = 1.0;

fn view() -> BNodeView {
    // Any framing will do, as long as both modes read the same one; this is a
    // window round the graph at the region's aspect.
    BNodeView { xmin: -300.0, ymin: -250.0, xmax: 600.0, ymax: 350.0 }
}

#[test]
fn seen_from_above_every_tube_end_is_the_sockets_flat_pixel() {
    let graph = graph();
    let size = size();
    let view = view();
    let placed: Vec<(String, (f32, f32))> = graph.nodes.iter().map(|n| (n.name.clone(), view.to_fraction((n.x, n.y)))).collect();
    let (_, flat_links) = plan(&graph, &graph.links, &placed, REGION.0, REGION.1, size, &view);
    let offsets: Vec<(String, (f32, f32))> = graph.nodes.iter().map(|n| (n.name.clone(), (0.0, 0.0))).collect();
    let layout = BSpatialLayout::of(&graph, &graph.links, &offsets, size, &BNodeSelection::default());
    let camera = BSpatialCamera::of(&view, BOrbit { yaw: 0.0, pitch: 90.0 }, REGION);
    assert_eq!(layout.tubes.len(), flat_links.len(), "every link is a tube");
    for (tube, (from, to)) in layout.tubes.iter().zip(flat_links.iter()) {
        // On the floor: the end's height is the view's own axis and must not
        // move it on the plane.
        let (fx, fy, _) = camera.project([tube.from[0], tube.from[1], 0.0]).unwrap();
        let (tx, ty, _) = camera.project([tube.to[0], tube.to[1], 0.0]).unwrap();
        assert!((fx - from.0).abs() < PIXEL && (fy - from.1).abs() < PIXEL, "from {:?} vs flat {from:?}", (fx, fy));
        assert!((tx - to.0).abs() < PIXEL && (ty - to.1).abs() < PIXEL, "to {:?} vs flat {to:?}", (tx, ty));
        assert!(tube.from[2] > 0.0 && tube.to[2] > 0.0, "an end sits on its rim, not on the floor");
    }
}

#[test]
fn seen_from_above_every_solid_is_its_card() {
    let graph = graph();
    let size = size();
    let view = view();
    let placed: Vec<(String, (f32, f32))> = graph.nodes.iter().map(|n| (n.name.clone(), view.to_fraction((n.x, n.y)))).collect();
    let (cards, _) = plan(&graph, &graph.links, &placed, REGION.0, REGION.1, size, &view);
    let offsets: Vec<(String, (f32, f32))> = graph.nodes.iter().map(|n| (n.name.clone(), (0.0, 0.0))).collect();
    let layout = BSpatialLayout::of(&graph, &graph.links, &offsets, size, &BNodeSelection::default());
    let camera = BSpatialCamera::of(&view, BOrbit { yaw: 0.0, pitch: 90.0 }, REGION);
    assert_eq!(layout.solids.len(), cards.len());
    for (solid, card) in layout.solids.iter().zip(cards.iter()) {
        assert_eq!(solid.name, card.name);
        let (x, y, _) = camera.project([solid.rect.0, solid.rect.3, 0.0]).unwrap();
        assert!((x - card.x).abs() < PIXEL && (y - card.y).abs() < PIXEL, "{}: top-left {x},{y} vs card {},{}", solid.name, card.x, card.y);
        let (x1, y1, _) = camera.project([solid.rect.2, solid.rect.1, 0.0]).unwrap();
        assert!((x1 - (card.x + card.width)).abs() < PIXEL && (y1 - (card.y + card.height)).abs() < PIXEL, "{}: bottom-right", solid.name);
    }
}

#[test]
fn the_graph_rises_toward_its_output() {
    let offsets: Vec<(String, (f32, f32))> = Vec::new();
    let layout = BSpatialLayout::of(&graph(), &graph().links, &offsets, size(), &BNodeSelection::default());
    let height = |name: &str| layout.solids.iter().find(|s| s.name == name).map(|s| s.height).unwrap();
    assert!(height("Image Texture") < height("Principled BSDF"), "texture under shader");
    assert!(height("Principled BSDF") < height("Material Output"), "shader under output");
    assert!(layout.plates.is_empty(), "no frames, no plates");
}
