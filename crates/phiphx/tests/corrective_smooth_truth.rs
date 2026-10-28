//! Corrective Smooth (`modifier/corrective_smooth.rs`) puts every vertex
//! where Blender puts it: eight bent tubes (`fixtures/make_corrective_smooth.py`)
//! — the pinch a plain Armature leaves, the default smooth, Length Weight
//! with a scale, a vertex group with Pin Boundaries and the same inverted,
//! Only Smooth, a BIND rest bound at another pose, and the modifier ahead of
//! the Armature — compared vertex by vertex with Blender's own evaluation
//! (`fixtures/corrective_smooth.tsv`).

use phiphx::phx::Phx;
use std::collections::HashMap;
use std::path::Path;

const FILE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/corrective_smooth.blend");
const TRUTH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/corrective_smooth.tsv");

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

fn dist(p: &[f32; 3], q: &[f32; 3]) -> f32 {
    (0..3).map(|i| (p[i] - q[i]).powi(2)).sum::<f32>().sqrt()
}

/// The largest and the mean distance between same-index vertices.
fn errors(a: &[[f32; 3]], b: &[[f32; 3]]) -> (f32, f32) {
    let d: Vec<f32> = a.iter().zip(b).map(|(p, q)| dist(p, q)).collect();
    (d.iter().copied().fold(0.0, f32::max), d.iter().sum::<f32>() / d.len().max(1) as f32)
}

#[test]
fn corrective_smooth_matches_blender_vertex_for_vertex() {
    let model = Phx::open(Path::new(FILE)).expect("the corrective smooth file opens");
    let want = truth();
    assert_eq!(want.len(), 8, "eight tubes in the truth");
    // The tube is 2 m tall: 1e-4 of its size.
    let limit = 2e-4;
    for (name, theirs) in &want {
        let ours = &model.objects.iter().find(|o| &o.name == name).unwrap_or_else(|| panic!("{name}")).mesh.vertices;
        assert_eq!(ours.len(), theirs.len(), "{name}: vertex count");
        let (max, mean) = errors(ours, theirs);
        eprintln!("{name:8} max {max:.3e}  mean {mean:.3e}");
        assert!(max < limit, "{name}: a vertex {max} from Blender's");
    }
    // The modifier does something: the default smooth undoes the pinch.
    let (moved, _) = errors(&want["Simple"], &want["Plain"]);
    assert!(moved > 0.02, "the smooth changes the bent tube ({moved})");
    let (bound, _) = errors(&want["Bind"], &want["Simple"]);
    assert!(bound > 1e-3, "the bind rest is not the ORCO rest ({bound})");
}
