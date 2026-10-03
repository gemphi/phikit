//! Creases under the Subdivision Surface land where Blender 5.0 puts them:
//! every vertex of four creased cubes (`truth/blender_crease.py`) — a
//! crease sharp for two levels and a half at the third, a fractional one,
//! an infinitely sharp one, a creased vertex, three creased edges meeting
//! at a corner with the limit surface off, "Use Creases" off, sharpness
//! left over at the `quality` level dropped (a crease and a vertex), and
//! a smooth cube subdivided past its `quality` level. The file
//! is Blender 5.0's, so the creases are read from its attribute storage;
//! THYMUS (`rig_plays.rs`) reads them from 3.6's `CD_CREASE` layers.

use phiphx::phx::Phx;
use std::collections::HashMap;
use std::path::Path;

const FILE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/crease.blend");
const TRUTH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/truth/crease.txt");

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

/// The farthest any vertex of `a` is from the nearest of `b`.
fn farthest(a: &[[f32; 3]], b: &[[f32; 3]]) -> f32 {
    let d = |p: &[f32; 3], q: &[f32; 3]| (0..3).map(|i| (p[i] - q[i]).powi(2)).sum::<f32>().sqrt();
    a.iter().map(|p| b.iter().map(|q| d(p, q)).fold(f32::MAX, f32::min)).fold(0.0, f32::max)
}

#[test]
fn creased_cubes_subdivide_where_blender_puts_them() {
    let model = Phx::open(Path::new(FILE)).expect("the crease file opens");
    let want = truth();
    assert_eq!(want.len(), 7, "seven cubes in the truth");
    let smooth = &want["Off"];
    for (name, theirs) in &want {
        let ours = &model.objects.iter().find(|o| &o.name == name).unwrap_or_else(|| panic!("{name}")).mesh.vertices;
        assert_eq!(ours.len(), theirs.len(), "{name}: vertex count");
        let worst = farthest(theirs, ours).max(farthest(ours, theirs));
        // Past `quality` levels Blender evaluates the grid on OpenSubdiv's
        // Gregory end caps round each extraordinary vertex, which only
        // approximate the limit surface phiphx gives (not yet ported).
        let limit = if name == "Gregory" { 0.02 } else { 1e-4 };
        assert!(worst < limit, "{name}: a vertex {worst} from Blender's");
    }
    // The creases do something: Semi is not the smooth cube Off is.
    let semi: Vec<[f32; 3]> = want["Semi"].iter().map(|v| [v[0] + 12.0, v[1], v[2]]).collect();
    assert!(farthest(&semi, smooth) > 0.05, "the creases change the shape");
}
