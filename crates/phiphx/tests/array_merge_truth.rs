//! The Array's "Merge" (`blend/modifier/array_merge.rs`, `merge_verts.rs`)
//! leaves the vertices and faces Blender's does: strips arrayed so their
//! copies touch, stop just short, collapse onto each other in whole or in
//! part, or stay apart (`fixtures/make_array_merge.py`), against the vertices
//! and triangle counts Blender dumped beside the file
//! (`fixtures/array_merge_truth.txt`), matched nearest to nearest.
//!
//! The same file carries a 192-corner n-gon: every face is read whatever its
//! size. A 3.6 file reads its faces another way (`mesh/block.rs`,
//! `mesh/legacy.rs`), which capped them at 99 corners; BC-15's wheels are
//! checked where the file is on the machine.

use phiphx::phx::Phx;
use std::collections::HashMap;
use std::path::Path;

const FILE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/array_merge.blend");
const TRUTH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/array_merge_truth.txt");

/// `name → (triangles, vertices)`.
fn truth() -> HashMap<String, (usize, Vec<[f32; 3]>)> {
    std::fs::read_to_string(TRUTH)
        .expect("truth file")
        .lines()
        .filter_map(|l| {
            let mut it = l.split('\t');
            let (name, count, tris, v) = (it.next()?, it.next()?.parse::<usize>().ok()?, it.next()?.parse().ok()?, it.next()?);
            let nums: Vec<f32> = v.split(' ').filter_map(|x| x.parse().ok()).collect();
            let verts: Vec<[f32; 3]> = nums.chunks_exact(3).map(|c| [c[0], c[1], c[2]]).collect();
            (verts.len() == count).then(|| (name.to_string(), (tris, verts)))
        })
        .collect()
}

/// The farthest any vertex of `a` is from the nearest of `b`.
fn farthest(a: &[[f32; 3]], b: &[[f32; 3]]) -> f32 {
    let d = |p: &[f32; 3], q: &[f32; 3]| (0..3).map(|i| (p[i] - q[i]).powi(2)).sum::<f32>().sqrt();
    a.iter().map(|p| b.iter().map(|q| d(p, q)).fold(f32::MAX, f32::min)).fold(0.0, f32::max)
}

#[test]
fn merged_arrays_and_a_big_ngon_come_out_as_blenders() {
    let model = Phx::open(Path::new(FILE)).expect("the array file opens");
    let want = truth();
    assert_eq!(want.len(), 6, "five arrays and the hub in the truth");
    let mut bad = Vec::new();
    for (name, (tris, theirs)) in &want {
        let ob = model.objects.iter().find(|o| &o.name == name).unwrap_or_else(|| panic!("{name}"));
        let ours = &ob.mesh.vertices;
        let counts = (ours.len(), ob.mesh.indices.len() / 3);
        let worst = farthest(theirs, ours).max(farthest(ours, theirs));
        println!("{name}: {counts:?} against {:?}, {worst:.2e}", (theirs.len(), tris));
        if counts != (theirs.len(), *tris) || worst > 1e-5 {
            bad.push(format!("{name}: {counts:?} against {:?}, {worst:e}", (theirs.len(), tris)));
        }
    }
    assert!(bad.is_empty(), "{bad:#?}");
    // Merging does something: "Apart" keeps all five copies' vertices.
    assert!(want["Touching"].1.len() < want["Apart"].1.len());
}

/// BC-15 (a 3.6 file): one wheel's 192-corner hub cap, two on the rear
/// pair. Blender: 193,066 and 386,132 triangles. A production file, so
/// `--ignored`, as `curve_mesh_truth.rs` checks its roofs.
#[test]
#[ignore]
fn bc15_wheels_keep_their_hub_caps() {
    let path = Path::new(r"C:\Users\phiac\Downloads\BC-15.blend");
    if !path.exists() {
        eprintln!("skipped: {} is not on this machine", path.display());
        return;
    }
    let model = Phx::open(path).expect("BC-15 opens");
    for (name, tris) in [("Front_Wheel", 193_066), ("Rear_Wheels", 386_132)] {
        let ob = model.objects.iter().find(|o| o.name.rsplit('/').next() == Some(name)).unwrap_or_else(|| panic!("{name}"));
        assert_eq!(ob.mesh.indices.len() / 3, tris, "{name}");
    }
}
