//! The scene's compositor over a finished render
//! (`scene/composite.rs`, `phiray`'s `Tracer::show`).
//!
//! `phiphx`'s own tests measure the tree — read from a `.blend` and run per
//! pixel — against Blender's compositor, and `phiray`'s measure the display
//! step. What is left, and what this covers, is the glue: the render's
//! scene-linear pixels out, through the tree, and back in so that the bytes,
//! the display floats and the radiance all come back composited.
//!
//! The tree is built by hand rather than read, because the point here is
//! the plumbing and a hand-built tree says exactly what it is.

use phiphx::blend::shade::graph::{BlendNode, BlendNodeGraph, BlendNodeLink};
use phiphx::blend::shade::socket::{BSocketDefault, BlendSocket};
use phiphx::comp::BCompTree;
use phiber::v2::scene::composite::BComposite;

fn socket(identifier: &str, name: &str, default: Option<BSocketDefault>) -> BlendSocket {
    BlendSocket { identifier: identifier.into(), name: name.into(), default, ..Default::default() }
}

fn node(idname: &str, name: &str, inputs: Vec<BlendSocket>, outputs: Vec<BlendSocket>) -> BlendNode {
    BlendNode { idname: idname.into(), name: name.into(), inputs, outputs, ..Default::default() }
}

fn link(from: &str, from_socket: &str, to: &str, to_socket: &str) -> BlendNodeLink {
    BlendNodeLink { from: from.into(), from_socket: from_socket.into(), to: to.into(), to_socket: to_socket.into() }
}

/// Render Layers → Exposure(`stops`) → Composite.
fn one_stop(stops: f32) -> BCompTree {
    let graph = BlendNodeGraph {
        nodes: vec![
            node("CompositorNodeRLayers", "Render Layers", Vec::new(), vec![socket("Image", "Image", None)]),
            node(
                "CompositorNodeExposure",
                "Exposure",
                vec![socket("Image", "Image", None), socket("Exposure", "Exposure", Some(BSocketDefault::Float(stops)))],
                vec![socket("Image", "Image", None)],
            ),
            node("CompositorNodeComposite", "Composite", vec![socket("Image", "Image", None)], Vec::new()),
        ],
        links: vec![
            link("Render Layers", "Image", "Exposure", "Image"),
            link("Exposure", "Image", "Composite", "Image"),
        ],
    };
    BCompTree { on: true, output: Some("Composite".into()), graph, setting: Vec::new() }
}

/// A wall facing the camera, lit by the world.
fn wall() -> Vec<phiray::Tri> {
    let q = [[-2.0f32, 0.0, -2.0], [2.0, 0.0, -2.0], [2.0, 0.0, 2.0], [-2.0, 0.0, 2.0]];
    let n = [0.0f32, -1.0, 0.0];
    vec![
        phiray::Tri::new([q[0], q[1], q[2]], [n; 3], [0.6, 0.5, 0.4, 1.0]),
        phiray::Tri::new([q[0], q[2], q[3]], [n; 3], [0.6, 0.5, 0.4, 1.0]),
    ]
}

fn camera() -> phiray::Camera {
    phiray::Camera {
        right: [1.0, 0.0, 0.0],
        up: [0.0, 0.0, 1.0],
        back: [0.0, -1.0, 0.0],
        units: 1.0,
        scale: 0.1,
        aspect: 1.0,
        dist: 10.0,
        sky: [0.2, 0.22, 0.25, 0.0],
        env: [0.5, 0.5, 0.5, 1.0],
        ..Default::default()
    }
}

#[test]
fn a_compositor_tree_grades_the_render_it_is_handed() {
    let Some(gpu) = phiray::Ray::gpu() else { return eprintln!("no GPU adapter; skipped") };
    let mut tracer = phiray::Tracer::new(&gpu, &wall(), &phiray::Atlas::default()).expect("scene");
    tracer.set_mode(phiray::Mode::Rendered);
    tracer.set_samples(4);
    let (w, h) = (24u32, 24u32);
    while !tracer.sample(&gpu, camera(), w, h).1 {}
    let before = tracer.radiance(&gpu);
    let shown = tracer.pixels(&gpu);
    assert!(before.iter().any(|c| c[0] > 0.05), "the wall is lit");

    // A tree that is switched off does nothing at all.
    let off = BCompTree { on: false, ..one_stop(-1.0) };
    assert!(!tracer.composite(&gpu, &off).any(), "a tree that is off is not run");
    assert_eq!(tracer.pixels(&gpu), shown, "and leaves the image alone");

    // One stop down: every lit pixel's radiance halves.
    let run = tracer.composite(&gpu, &one_stop(-1.0));
    assert_eq!(run.ran, 3, "three nodes ran: {run:?}");
    assert!(run.declined.is_empty(), "none declined: {run:?}");
    let after = tracer.radiance(&gpu);
    for (i, (a, b)) in before.iter().zip(&after).enumerate() {
        for k in 0..3 {
            assert!((b[k] - a[k] * 0.5).abs() <= 1e-3 * a[k].abs().max(1.0), "pixel {i}: {b:?} is half of {a:?}");
        }
    }
    // And the displayed bytes followed, which is the whole point of going
    // back through the tracer's own display step rather than grading the
    // bytes: the view transform runs AFTER the grade.
    assert_ne!(tracer.pixels(&gpu), shown, "the shown image changed too");
}

/// A node this pass does not run is named, not skipped silently, and the
/// image still goes through the rest of the tree.
#[test]
fn a_declined_node_is_named_and_passes_its_image_through() {
    let Some(gpu) = phiray::Ray::gpu() else { return eprintln!("no GPU adapter; skipped") };
    let mut tracer = phiray::Tracer::new(&gpu, &wall(), &phiray::Atlas::default()).expect("scene");
    tracer.set_mode(phiray::Mode::Rendered);
    tracer.set_samples(4);
    while !tracer.sample(&gpu, camera(), 24, 24).1 {}
    let before = tracer.radiance(&gpu);

    // The same tree with a Blur between the Exposure and the Composite.
    let mut tree = one_stop(-1.0);
    tree.graph.nodes.push(node(
        "CompositorNodeBlur",
        "Blur",
        vec![socket("Image", "Image", None)],
        vec![socket("Image", "Image", None)],
    ));
    tree.graph.links.retain(|l| l.from != "Exposure");
    tree.graph.links.push(link("Exposure", "Image", "Blur", "Image"));
    tree.graph.links.push(link("Blur", "Image", "Composite", "Image"));

    let run = tracer.composite(&gpu, &tree);
    assert_eq!(run.declined, vec!["CompositorNodeBlur".to_string()], "the Blur is named: {run:?}");
    assert!(run.line().contains("1 not run (CompositorNodeBlur)"), "{}", run.line());
    // The grade before it still happened, and the Blur passed the image on.
    let after = tracer.radiance(&gpu);
    for (a, b) in before.iter().zip(&after) {
        assert!((b[0] - a[0] * 0.5).abs() <= 1e-3 * a[0].abs().max(1.0), "{b:?} is half of {a:?}");
    }
}
