use crate::uv::project::*;

/// A unit cube's six faces, each its own quad.
fn cube() -> (Vec<[f32; 3]>, Vec<u32>, Vec<u32>) {
    let p = [
        [-1.0, -1.0, -1.0], [1.0, -1.0, -1.0], [1.0, 1.0, -1.0], [-1.0, 1.0, -1.0],
        [-1.0, -1.0, 1.0], [1.0, -1.0, 1.0], [1.0, 1.0, 1.0], [-1.0, 1.0, 1.0],
    ];
    let f: [[u32; 4]; 6] = [[0, 3, 2, 1], [4, 5, 6, 7], [0, 1, 5, 4], [2, 3, 7, 6], [1, 2, 6, 5], [0, 4, 7, 3]];
    let (mut corner, mut starts) = (Vec::new(), vec![0u32]);
    for q in f {
        corner.extend_from_slice(&q);
        starts.push(corner.len() as u32);
    }
    (p.to_vec(), corner, starts)
}

/// Cube projection puts every face square in the square: each face's
/// four UVs span the same box, because each is seen head on.
#[test]
fn cube_project_lays_each_face_flat() {
    let (pos, corner, starts) = cube();
    let mesh = BUvMesh { pos: &pos, corner: &corner, starts: &starts };
    let uv = project(mesh, BUnwrap::Cube);
    for f in 0..6 {
        let (a, b) = (starts[f] as usize, starts[f + 1] as usize);
        let (mut lo, mut hi) = ([f32::MAX; 2], [f32::MIN; 2]);
        for c in a..b {
            for k in 0..2 {
                lo[k] = lo[k].min(uv[c][k]);
                hi[k] = hi[k].max(uv[c][k]);
            }
        }
        assert!((hi[0] - lo[0] - 1.0).abs() < 1e-5 && (hi[1] - lo[1] - 1.0).abs() < 1e-5, "face {f} is a unit square: {lo:?} {hi:?}");
    }
}

/// Cylinder projection turns the angle into U, so a ring of points
/// around Z comes out spread across U and flat in V.
#[test]
fn cylinder_project_unrolls_the_angle() {
    let seg = 8;
    let mut pos = Vec::new();
    for i in 0..seg {
        let t = i as f32 / seg as f32 * std::f32::consts::TAU;
        pos.push([t.cos(), t.sin(), -1.0]);
        pos.push([t.cos(), t.sin(), 1.0]);
    }
    let (mut corner, mut starts) = (Vec::new(), vec![0u32]);
    for i in 0..seg as u32 {
        let j = (i + 1) % seg as u32;
        corner.extend_from_slice(&[2 * i, 2 * j, 2 * j + 1, 2 * i + 1]);
        starts.push(corner.len() as u32);
    }
    let mesh = BUvMesh { pos: &pos, corner: &corner, starts: &starts };
    let uv = project(mesh, BUnwrap::Cylinder);
    let (lo, hi) = uv.iter().fold((1.0f32, 0.0f32), |(l, h), c| (l.min(c[0]), h.max(c[0])));
    assert!(hi - lo > 0.8, "U runs most of the way round: {lo}..{hi}");
    // The two rings are at the two ends of V and nowhere between.
    let vs: Vec<f32> = uv.iter().map(|c| c[1]).collect();
    assert!(vs.iter().all(|v| (*v - 0.0).abs() < 1e-5 || (*v - 1.0).abs() < 1e-5), "V is the height: {vs:?}");
}
