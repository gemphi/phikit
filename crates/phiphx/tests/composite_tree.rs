//! The scene's compositor tree read out of a `.blend` and run
//! (`fixtures/COMPOSITE.blend`, `truth/blender_composite.py`'s `chain`
//! case).
//!
//! The fixture's tree is Image → Brightness/Contrast → Hue Saturation Value
//! → Mix (Screen) → the tree's output, and the truth line is Blender's own
//! render of exactly that tree over exactly these pixels. So this measures
//! the whole path — where the tree lives in the file, which nodes those
//! are, what their sockets say, the order they run in — against Blender,
//! not just the arithmetic (`composite_truth.rs` does that).

use phiphx::blend::BlendFile;
use phiphx::comp::{BCompKind, BCompPixel};
use std::path::PathBuf;

const TRUTH: &str = include_str!("truth/composite.txt");
const TOL: f32 = 2e-4;

fn case(name: &str) -> Vec<BCompPixel> {
    let line = TRUTH.lines().find(|l| l.split_whitespace().next() == Some(name)).expect(name);
    line.split_whitespace().skip(1).map(|t| t.parse::<f32>().expect("number")).collect::<Vec<_>>().chunks_exact(4).map(|c| [c[0], c[1], c[2], c[3]]).collect()
}

#[test]
fn the_scenes_tree_is_read_and_runs_as_blenders_does() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/COMPOSITE.blend");
    let Ok(bytes) = std::fs::read(&path) else { return eprintln!("no COMPOSITE.blend; skipped") };
    let blend = BlendFile::parse_bytes(&bytes).expect("parse");
    let tree = blend.compositor().expect("the scene has a compositor tree");

    // It is on: a new scene has Post Processing ▸ Compositing set.
    assert!(tree.on, "the tree runs");
    assert!(tree.output.is_some(), "and ends somewhere");
    assert!(tree.declined().is_empty(), "every node in it runs: {:?}", tree.declined());

    // The nodes are the ones the fixture built, and nothing is Declined.
    let mut kinds: Vec<BCompKind> = tree.graph.nodes.iter().map(|n| BCompKind::of(&n.idname)).collect();
    kinds.sort_by_key(|k| format!("{k:?}"));
    kinds.dedup();
    for want in [BCompKind::Image, BCompKind::BrightContrast, BCompKind::HueSat, BCompKind::Mix] {
        assert!(kinds.contains(&want), "{want:?} is in the tree: {kinds:?}");
    }

    let plan = tree.plan().expect("a tree that runs has a plan");
    assert!(plan.len() >= 5, "the chain is five nodes deep: {}", plan.len());

    let (input, want) = (case("passthrough"), case("chain"));
    for (i, (&was, &expect)) in input.iter().zip(&want).enumerate() {
        let got = plan.pixel(was);
        assert!(
            (0..4).all(|k| (got[k] - expect[k]).abs() <= TOL * expect[k].abs().max(1.0)),
            "pixel {i}: {got:?} vs Blender's {expect:?} (from {was:?})"
        );
    }

    // And over a whole buffer, in place.
    let mut px: Vec<f32> = input.iter().flatten().copied().collect();
    plan.apply(4, 4, &mut px);
    for (k, v) in px.iter().enumerate() {
        let expect = want[k / 4][k % 4];
        assert!((v - expect).abs() <= TOL * expect.abs().max(1.0), "buffer {k}: {v} vs {expect}");
    }
}
