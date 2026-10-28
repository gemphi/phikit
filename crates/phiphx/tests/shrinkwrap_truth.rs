//! The Shrinkwrap modifier (`blend/modifier/shrinkwrap.rs`) puts every
//! vertex where Blender puts it: eleven grids over a sphere and a cube, one per
//! wrap method and mode (`fixtures/make_shrinkwrap.py`), against the
//! evaluated positions Blender itself dumped beside the file
//! (`fixtures/shrinkwrap_truth.txt`), vertex by vertex in object space.

use phiphx::phx::Phx;
use std::collections::HashMap;
use std::path::Path;

const FILE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/shrinkwrap.blend");
const TRUTH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/shrinkwrap_truth.txt");

fn truth() -> HashMap<String, Vec<[f32; 3]>> {
    std::fs::read_to_string(TRUTH)
        .expect("truth file")
        .lines()
        .filter_map(|l| {
            let mut it = l.split('\t');
            let (name, count, v) = (it.next()?, it.next()?.parse::<usize>().ok()?, it.next()?);
            let nums: Vec<f32> = v.split(' ').filter_map(|x| x.parse().ok()).collect();
            let verts: Vec<[f32; 3]> = nums.chunks_exact(3).map(|c| [c[0], c[1], c[2]]).collect();
            (verts.len() == count).then(|| (name.to_string(), verts))
        })
        .collect()
}

#[test]
fn shrinkwrapped_grids_land_where_blender_puts_them() {
    let model = Phx::open(Path::new(FILE)).expect("the shrinkwrap file opens");
    let want = truth();
    assert_eq!(want.len(), 11, "eleven wrapped grids in the truth");
    let mut report = Vec::new();
    let mut bad = Vec::new();
    for (name, theirs) in &want {
        let ours = &model.objects.iter().find(|o| &o.name == name).unwrap_or_else(|| panic!("{name}")).mesh.vertices;
        assert_eq!(ours.len(), theirs.len(), "{name}: vertex count");
        // Every grid is flat (z = 0) before its modifier: each one must move.
        assert!(theirs.iter().any(|v| v[2].abs() > 1e-3), "{name}: Blender left it flat");
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
    // The modifier does something: the grid is no longer flat.
    let on = &model.objects.iter().find(|o| o.name == "Surface On").unwrap().mesh.vertices;
    assert!(on.iter().any(|v| v[2].abs() > 0.1), "the grid wraps");
}
