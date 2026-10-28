//! The Surface Deform modifier (`blend/modifier/surface_deform.rs`) puts
//! every bound vertex where Blender puts it: four grids bound to a
//! rippled sphere — plain, at strength 0.6, over an inverted vertex
//! group, and to a twisted copy whose twist was then doubled (the target's
//! evaluated mesh) — plus one never bound and one whose target was lost
//! (`fixtures/make_surface_deform.py`), against Blender's evaluated
//! positions from the saved file (`fixtures/surface_deform_truth.txt`,
//! `truth/blender_eval_dump.py`).

use phiphx::phx::Phx;
use std::collections::HashMap;
use std::path::Path;

const FILE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/surface_deform.blend");
const TRUTH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/surface_deform_truth.txt");

fn truth() -> HashMap<String, Vec<[f32; 3]>> {
    std::fs::read_to_string(TRUTH)
        .expect("truth file")
        .lines()
        .filter_map(|l| {
            let mut it = l.split('\t');
            let (kind, name, count, v) = (it.next()?, it.next()?, it.next()?.parse::<usize>().ok()?, it.next()?);
            let nums: Vec<f32> = v.split(' ').filter_map(|x| x.parse().ok()).collect();
            let verts: Vec<[f32; 3]> = nums.chunks_exact(3).map(|c| [c[0], c[1], c[2]]).collect();
            (kind == "verts" && verts.len() == count).then(|| (name.to_string(), verts))
        })
        .collect()
}

#[test]
fn bound_grids_follow_the_changed_target() {
    let model = Phx::open(Path::new(FILE)).expect("the surface deform file opens");
    let want = truth();
    let mut report = Vec::new();
    let mut bad = Vec::new();
    for name in ["Wrap", "Wrap Half", "Wrap Group", "Wrap Twist", "Unbound", "Lost"] {
        let theirs = &want[name];
        let ours = &model.objects.iter().find(|o| o.name == name).unwrap_or_else(|| panic!("{name}")).mesh.vertices;
        assert_eq!(ours.len(), theirs.len(), "{name}: vertex count");
        let errs: Vec<f32> = ours.iter().zip(theirs).map(|(a, b)| (0..3).map(|k| (a[k] - b[k]).powi(2)).sum::<f32>().sqrt()).collect();
        let max = errs.iter().copied().fold(0.0, f32::max);
        let mean = errs.iter().sum::<f32>() / errs.len() as f32;
        // The grid is 3 m across: 1e-4 of it.
        if max > 3e-4 {
            bad.push(format!("{name}: max {max:e}"));
        }
        report.push(format!("{name}: max {max:.2e} mean {mean:.2e}"));
    }
    report.sort();
    println!("{}", report.join("\n"));
    assert!(bad.is_empty(), "{bad:#?}");
    // The bound grids left the plane; the unbound and the lost one did not
    // (Blender disables a Surface Deform without a target).
    let flat = |name: &str| model.objects.iter().find(|o| o.name == name).unwrap().mesh.vertices.iter().all(|v| v[2].abs() < 1e-6);
    assert!(!flat("Wrap") && !flat("Wrap Twist"), "bound grids wrap the sphere");
    assert!(flat("Unbound") && flat("Lost"), "an unbound or targetless Surface Deform leaves the mesh");
}
