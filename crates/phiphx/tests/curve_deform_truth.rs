//! The Curve modifier bends where Blender bends: every vertex of six arrayed
//! bars (`fixtures/make_curve_deform.py`), one per deform axis — ±X, ±Y,
//! ±Z — over open and cyclic Bézier curves and a NURBS one, with the bounds
//! from the bar's own box or Blender's default `0..1`, `CU_STRETCH`, the
//! path radius on and off, a bar running past the curve's open end, and a
//! vertex group ramp, straight and inverted. The truth is Blender's own
//! evaluated mesh, dumped beside the fixture.

use phiphx::phx::Phx;
use std::collections::HashMap;
use std::path::Path;

const FILE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/curve_deform.blend");
const TRUTH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/curve_deform.txt");

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

/// The object's size: its evaluated box's diagonal.
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
fn arrayed_bars_bend_along_their_curves_as_blender_bends_them() {
    let model = Phx::open(Path::new(FILE)).expect("the curve deform file opens");
    let want = truth();
    assert_eq!(want.len(), 6, "six bars in the truth");
    for (name, theirs) in &want {
        let ours = &model.objects.iter().find(|o| &o.name == name).unwrap_or_else(|| panic!("{name}")).mesh.vertices;
        assert_eq!(ours.len(), theirs.len(), "{name}: vertex count");
        let errors: Vec<f32> = ours.iter().zip(theirs).map(|(a, b)| dist(a, b)).collect();
        let max = errors.iter().copied().fold(0.0, f32::max);
        let mean = errors.iter().sum::<f32>() / errors.len() as f32;
        let scale = size(theirs);
        println!("{name}: max {max:.3e}, mean {mean:.3e}, size {scale:.2}");
        assert!(max < 1e-4 * scale, "{name}: a vertex {max} from Blender's (object {scale} across)");
    }
    // The bend does something: the +X bar is no longer straight.
    let bar = &want["PosX"];
    let ys: Vec<f32> = bar.iter().map(|v| v[1]).collect();
    let spread = ys.iter().copied().fold(f32::MIN, f32::max) - ys.iter().copied().fold(f32::MAX, f32::min);
    assert!(spread > 1.0, "the bar follows the S");
}
