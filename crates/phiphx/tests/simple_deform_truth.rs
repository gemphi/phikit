//! The Simple Deform modifier (`blend/modifier/simple_deform.rs`) puts
//! every vertex where Blender puts it: twelve subdivided cubes, one per
//! mode, axis, limit, lock, origin and vertex-group setting
//! (`fixtures/make_simple_deform.py`), against the evaluated positions
//! Blender dumped from the saved file (`fixtures/simple_deform_truth.txt`,
//! `truth/blender_eval_dump.py`), vertex by vertex in object space.

use phiphx::phx::Phx;
use std::collections::HashMap;
use std::path::Path;

const FILE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/simple_deform.blend");
const TRUTH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/simple_deform_truth.txt");

/// The `verts` lines: name → Blender's evaluated vertices.
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
fn every_mode_lands_where_blender_puts_it() {
    let model = Phx::open(Path::new(FILE)).expect("the simple deform file opens");
    let want = truth();
    assert_eq!(want.len(), 12, "twelve deformed cubes in the truth");
    let mut report = Vec::new();
    let mut bad = Vec::new();
    for (name, theirs) in &want {
        let ours = &model.objects.iter().find(|o| &o.name == name).unwrap_or_else(|| panic!("{name}")).mesh.vertices;
        assert_eq!(ours.len(), theirs.len(), "{name}: vertex count");
        // Every cube is 2 m on a side before its modifier; Twist, Bend,
        // Taper and Stretch each move something off the cube.
        assert!(theirs.iter().any(|v| v.iter().any(|c| (c.abs() - 1.0).abs() > 1e-3 && c.abs() > 1e-3)), "{name}: Blender left the cube");
        let errs: Vec<f32> = ours.iter().zip(theirs).map(|(a, b)| (0..3).map(|k| (a[k] - b[k]).powi(2)).sum::<f32>().sqrt()).collect();
        let max = errs.iter().copied().fold(0.0, f32::max);
        let mean = errs.iter().sum::<f32>() / errs.len() as f32;
        if max > 2e-4 {
            bad.push(format!("{name}: max {max:e}"));
        }
        report.push(format!("{name}: max {max:.2e} mean {mean:.2e}"));
    }
    report.sort();
    println!("{}", report.join("\n"));
    assert!(bad.is_empty(), "{bad:#?}");
}

#[test]
fn the_modifier_moves_without_adding_vertices() {
    let model = Phx::open(Path::new(FILE)).expect("the simple deform file opens");
    let twist = model.objects.iter().find(|o| o.name == "Twist Z").expect("Twist Z");
    assert_eq!(twist.mesh.vertices.len(), 152, "a cube with every edge cut four times: 8 + 12 * 4 + 6 * 16");
    // A twist about Z leaves z alone and turns x, y: a top corner is no
    // longer at (±1, ±1).
    assert!(twist.mesh.vertices.iter().any(|v| v[2] > 0.99 && (v[0].abs() - 1.0).abs() > 0.05));
}
