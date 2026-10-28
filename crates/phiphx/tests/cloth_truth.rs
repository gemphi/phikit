//! The Cloth modifier (`blend/modifier/cloth.rs`) shows what Blender shows
//! at the saved frame, from the cache alone: six sheets whose caches put
//! frame 8 at an exact cached frame, before the start, past the end, between
//! two cached frames, at the start frame, and on disk beside the file
//! (`fixtures/make_cloth.py`), against Blender's evaluated positions from
//! the saved file (`fixtures/cloth_truth.txt`, `truth/blender_eval_dump.py`).

use phiphx::phx::Phx;
use std::collections::HashMap;
use std::path::Path;

const FILE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/cloth.blend");
const TRUTH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/cloth_truth.txt");

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
fn every_sheet_shows_its_cached_frame() {
    let model = Phx::open(Path::new(FILE)).expect("the cloth file opens");
    let want = truth();
    let mut report = Vec::new();
    let mut bad = Vec::new();
    for name in ["Sheet", "Early", "Late", "Gap", "Start", "Disk"] {
        let theirs = &want[name];
        let ours = &model.objects.iter().find(|o| o.name == name).unwrap_or_else(|| panic!("{name}")).mesh.vertices;
        assert_eq!(ours.len(), theirs.len(), "{name}: vertex count");
        let errs: Vec<f32> = ours.iter().zip(theirs).map(|(a, b)| (0..3).map(|k| (a[k] - b[k]).powi(2)).sum::<f32>().sqrt()).collect();
        let max = errs.iter().copied().fold(0.0, f32::max);
        let mean = errs.iter().sum::<f32>() / errs.len() as f32;
        // The sheet is 2 m across: 1e-4 of it.
        if max > 2e-4 {
            bad.push(format!("{name}: max {max:e}"));
        }
        report.push(format!("{name}: max {max:.2e} mean {mean:.2e}"));
    }
    report.sort();
    println!("{}", report.join("\n"));
    assert!(bad.is_empty(), "{bad:#?}");
    // The sheets with a cached frame fell; the ones before or at their
    // start frame are the flat grid as saved.
    let flat = |name: &str| model.objects.iter().find(|o| o.name == name).unwrap().mesh.vertices.iter().all(|v| v[2].abs() < 1e-6);
    assert!(flat("Early") && flat("Start"), "before or at the start frame the mesh passes through");
    assert!(!flat("Sheet") && !flat("Late") && !flat("Gap") && !flat("Disk"), "a cached frame moves the sheet");
}
