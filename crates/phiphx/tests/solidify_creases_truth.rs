//! The Solidify's creases (`blend/modifier/solidify_creases.rs`) sharpen the
//! Subdivision after it where Blender's do: six rippled grids under Solidify
//! then Subdivision (`fixtures/make_solidify_creases.py`) — crease inner,
//! outer, rim, the grid's own edge and vertex creases carried to the shell,
//! flipped normals, the rim off — against the vertices and triangle counts
//! Blender dumped beside the file (`fixtures/solidify_creases_truth.txt`).
//!
//! The subdivided vertices are not in Blender's order, so each one is
//! matched to the nearest of the other side's, both ways.

use phiphx::phx::Phx;
use std::collections::HashMap;
use std::path::Path;

const FILE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/solidify_creases.blend");
const TRUTH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/solidify_creases_truth.txt");

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
fn solidified_creases_subdivide_where_blender_puts_them() {
    let model = Phx::open(Path::new(FILE)).expect("the solidify file opens");
    let want = truth();
    assert_eq!(want.len(), 6, "six grids in the truth");
    let mut report = Vec::new();
    let mut bad = Vec::new();
    for (name, (tris, theirs)) in &want {
        let ob = model.objects.iter().find(|o| &o.name == name).unwrap_or_else(|| panic!("{name}"));
        let ours = &ob.mesh.vertices;
        assert_eq!((ours.len(), ob.mesh.indices.len() / 3), (theirs.len(), *tris), "{name}: vertices, triangles");
        let worst = farthest(theirs, ours).max(farthest(ours, theirs));
        // The grid is 2 m across: 5e-6 of it. "Rim" and "No Rim" evaluate
        // the limit surface with creases still semi-sharp past `quality`
        // (0.7 outer is sharpness 4.9, the 0.9 vertex 8.1), which Blender's
        // OpenSubdiv patches honour and `subsurf.rs` drops (`isolated`); a
        // few vertices there sit up to 1.2e-2 off, mean 3e-5. "Flat Limit",
        // the same creases with the limit off, is exact: the creases the
        // Solidify writes are Blender's.
        let limit = match name.as_str() {
            "Rim" | "No Rim" => 0.02,
            _ => 1e-5,
        };
        if worst > limit {
            bad.push(format!("{name}: {worst:e}"));
        }
        report.push(format!("{name}: {worst:.2e}"));
    }
    report.sort();
    println!("{}", report.join("\n"));
    assert!(bad.is_empty(), "{bad:#?}");
}
