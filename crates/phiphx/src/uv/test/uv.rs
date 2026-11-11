use super::*;

/// A quad grid: `n x n` quads over `(n+1)²` vertices on the XY plane.
pub(super) fn grid(n: usize) -> (Vec<[f32; 3]>, Vec<u32>, Vec<u32>) {
    let mut pos = Vec::new();
    for j in 0..=n {
        for i in 0..=n {
            pos.push([i as f32 / n as f32, j as f32 / n as f32, 0.0]);
        }
    }
    let (mut corner, mut starts) = (Vec::new(), vec![0u32]);
    let stride = (n + 1) as u32;
    for j in 0..n as u32 {
        for i in 0..n as u32 {
            let a = j * stride + i;
            corner.extend_from_slice(&[a, a + 1, a + stride + 1, a + stride]);
            starts.push(corner.len() as u32);
        }
    }
    (pos, corner, starts)
}

#[test]
fn a_quad_fans_into_two_triangles() {
    let (pos, corner, starts) = grid(1);
    let mesh = BUvMesh { pos: &pos, corner: &corner, starts: &starts };
    assert_eq!(mesh.faces(), 1);
    assert_eq!(mesh.face(0), &[0, 1, 3, 2]);
    assert_eq!(mesh.triangles(), vec![[0, 1, 2], [0, 2, 3]]);
}
