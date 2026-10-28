//! The Multiresolution modifier lands every vertex where Blender puts it
//! (`fixtures/make_multires.py`, `modifier/multires.rs`): a cube with three
//! levels of sculpted-in displacement at viewport levels 3, 2 and 1 (the
//! grids read at a stride below `totlvl`), a hexagonal prism whose six-sided
//! caps take one PTEX face per corner, and an open sheet with smooth and
//! kept corners. Counts must match exactly; positions within 1e-4 of the
//! object's size, but for the four smooth one-face corners of the sheet
//! (`smooth_corners`, below), where Blender's own normal is rounding noise.

use phiphx::phx::Phx;
use std::path::Path;

const FILE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/multires.blend");
const TRUTH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/multires.txt");

struct Truth {
    name: String,
    faces: usize,
    verts: Vec<[f32; 3]>,
}

fn truth() -> Vec<Truth> {
    std::fs::read_to_string(TRUTH)
        .expect("truth file")
        .lines()
        .filter_map(|l| {
            let f: Vec<&str> = l.split('\t').collect();
            let (count, faces) = (f.get(2)?.parse::<usize>().ok()?, f.get(3)?.parse::<usize>().ok()?);
            let nums: Vec<f32> = f.get(4)?.split(' ').filter_map(|x| x.parse().ok()).collect();
            let verts: Vec<[f32; 3]> = nums.chunks_exact(3).map(|c| [c[0], c[1], c[2]]).collect();
            (verts.len() == count).then(|| Truth { name: f[0].to_string(), faces, verts })
        })
        .collect()
}

fn dist(p: &[f32; 3], q: &[f32; 3]) -> f32 {
    (0..3).map(|i| (p[i] - q[i]).powi(2)).sum::<f32>().sqrt()
}

/// Each vertex of `a`'s distance to the nearest of `b`.
fn nearest(a: &[[f32; 3]], b: &[[f32; 3]]) -> Vec<f32> {
    a.iter().map(|p| b.iter().map(|q| dist(p, q)).fold(f32::MAX, f32::min)).collect()
}

fn size(v: &[[f32; 3]]) -> f32 {
    let (mut lo, mut hi) = ([f32::MAX; 3], [f32::MIN; 3]);
    for p in v {
        for i in 0..3 {
            lo[i] = lo[i].min(p[i]);
            hi[i] = hi[i].max(p[i]);
        }
    }
    dist(&lo, &hi)
}

#[test]
fn multires_displaces_every_vertex_where_blender_puts_it() {
    let model = Phx::open(Path::new(FILE)).expect("the multires file opens");
    let want = truth();
    assert_eq!(want.len(), 6, "six objects in the truth");
    let mut failed = Vec::new();
    for t in &want {
        let ob = model.objects.iter().find(|o| o.name == t.name).unwrap_or_else(|| panic!("{}", t.name));
        let ours = &ob.mesh.vertices;
        assert_eq!(ours.len(), t.verts.len(), "{}: vertex count", t.name);
        // Every evaluated face is a quad, drawn as two triangles.
        assert_eq!(ob.mesh.indices.len() / 3, 2 * t.faces, "{}: triangle count", t.name);
        // The base mesh's vertices come first in both, in the same order;
        // the rest are matched to the nearest.
        let skip = smooth_corners(&t.name);
        let keep = |i: &usize| !skip.contains(i);
        let kept = |d: Vec<f32>| -> Vec<f32> { d.into_iter().enumerate().filter(|(i, _)| keep(i)).map(|(_, d)| d).collect() };
        let there = kept(nearest(&t.verts, ours));
        let back = kept(nearest(ours, &t.verts));
        let worst = there.iter().chain(&back).copied().fold(0.0, f32::max);
        let mean = there.iter().sum::<f32>() / there.len() as f32;
        let same = (0..ob_corners(&t.name)).filter(keep).map(|i| dist(&ours[i], &t.verts[i])).fold(0.0, f32::max);
        let corner = skip.iter().map(|&i| dist(&ours[i], &t.verts[i])).fold(0.0, f32::max);
        let scale = size(&t.verts);
        eprintln!(
            "{}: {} verts, max {worst:.2e} mean {mean:.2e} base {same:.2e} smooth corners {corner:.2e} (size {scale:.2})",
            t.name,
            ours.len()
        );
        if worst >= 1e-4 * scale || same >= 1e-4 * scale {
            failed.push(format!("{}: a vertex {worst} from Blender's, a base vertex {same}", t.name));
        }
    }
    assert!(failed.is_empty(), "{failed:#?}");
    // The displacement is really there: the cube's first corner is not
    // where Catmull-Clark alone takes it, (-1/2, -1/2, -1/2).
    let cube = &want.iter().find(|t| t.name == "Cube").expect("Cube").verts;
    assert!(dist(&cube[0], &[-0.5; 3]) > 0.01, "the fixture carries a displacement");
}

/// Vertices of each object's base mesh.
fn ob_corners(name: &str) -> usize {
    match name {
        "Prism" => 12,
        "Sheet" | "Sheet Keep" => 16,
        _ => 8,
    }
}

/// The one-face corners "Sheet" smooths (`boundary_smooth` All), left out
/// of the comparison. There OpenSubdiv's Gregory end cap gives `dPdu` and
/// `dPdv` exactly opposite (`GregoryConverter`'s "irregular/smooth corner
/// case": `Ep`, `Em` on the two boundary edges), so the normal Blender's
/// `BKE_multires_construct_tangent_matrix` builds there is the normalised
/// cross product of float rounding: some unit vector across the boundary
/// chord, not a property of the surface. phiphx gets the two tangent axes
/// and drops the normal part (a zero cross product normalises to zero).
fn smooth_corners(name: &str) -> Vec<usize> {
    match name {
        "Sheet" => vec![0, 3, 12, 15],
        _ => Vec::new(),
    }
}
