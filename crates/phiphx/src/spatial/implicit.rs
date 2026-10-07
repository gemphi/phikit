//! Blender's `Add ▸ Metaball`: an implicit surface, polygonised.
//!
//! A metaball is not a mesh in the file — it is a set of blobs and a
//! threshold, and Blender tessellates the isosurface at display time
//! (`BKE_mball_polygonize`). The surface is built the same way here, over
//! **marching tetrahedra** rather than marching cubes: each cell of the
//! grid is cut into six tetrahedra, and a tetrahedron has only sixteen
//! sign patterns against the 256 a cube has. The table is small enough to
//! read, it cannot produce the ambiguous-face holes plain marching cubes
//! can, and the surface it gives is the same surface.
//!
//! The field is Blender's own `MB_BALL` falloff: each blob contributes
//! `rad² / dist²` and the surface is where the sum crosses the threshold.

/// One blob: where it sits and how big it is.
#[derive(Clone, Copy, Debug)]
pub struct Blob {
    pub at: [f32; 3],
    pub radius: f32,
}

/// The six tetrahedra a unit cell is cut into, by corner index
/// (`x + 2y + 4z`). Every tetrahedron shares the cell's main diagonal
/// 0–7, which is what makes the six of them a partition with no cracks.
const TETRA: [[usize; 4]; 6] = [[0, 5, 1, 7], [0, 1, 3, 7], [0, 3, 2, 7], [0, 2, 6, 7], [0, 6, 4, 7], [0, 4, 5, 7]];

/// Cell corner `k` as an offset.
const CORNER: [[u32; 3]; 8] = [[0, 0, 0], [1, 0, 0], [0, 1, 0], [1, 1, 0], [0, 0, 1], [1, 0, 1], [0, 1, 1], [1, 1, 1]];

/// The field at `p`: `Σ rad² / dist²`, Blender's `MB_BALL` falloff.
fn field(p: [f32; 3], blobs: &[Blob]) -> f32 {
    blobs
        .iter()
        .map(|b| {
            let d = [p[0] - b.at[0], p[1] - b.at[1], p[2] - b.at[2]];
            let d2 = d[0] * d[0] + d[1] * d[1] + d[2] * d[2];
            b.radius * b.radius / d2.max(1e-6)
        })
        .sum()
}

/// Where the surface crosses the segment `a`–`b`, by linear interpolation
/// of the field — which is what keeps the facets from staircasing.
fn cut(a: [f32; 3], fa: f32, b: [f32; 3], fb: f32, iso: f32) -> [f32; 3] {
    let t = ((iso - fa) / (fb - fa).abs().max(1e-9) * (fb - fa).signum()).clamp(0.0, 1.0);
    std::array::from_fn(|k| a[k] + t * (b[k] - a[k]))
}

/// Polygonise the blobs over a `res³` grid spanning `[-span, span]`, at
/// `iso`. Returns positions, triangle indices and smooth normals — the
/// field's own gradient, so the surface shades as the smooth thing it is.
pub fn polygonise(blobs: &[Blob], iso: f32, span: f32, res: u32) -> (Vec<(f32, f32, f32)>, Vec<u32>, Vec<(f32, f32, f32)>) {
    let step = 2.0 * span / res as f32;
    let at = |i: u32, j: u32, k: u32| [-span + i as f32 * step, -span + j as f32 * step, -span + k as f32 * step];
    let mut verts: Vec<(f32, f32, f32)> = Vec::new();
    let mut idx: Vec<u32> = Vec::new();
    for i in 0..res {
        for j in 0..res {
            for k in 0..res {
                let p: [[f32; 3]; 8] = std::array::from_fn(|c| at(i + CORNER[c][0], j + CORNER[c][1], k + CORNER[c][2]));
                let f: [f32; 8] = std::array::from_fn(|c| field(p[c], blobs));
                for tet in TETRA {
                    let inside = tet.iter().map(|&c| u32::from(f[c] >= iso)).fold(0u32, |m, b| (m << 1) | b);
                    // All in or all out: nothing crosses this tetrahedron.
                    if inside == 0 || inside == 0b1111 {
                        continue;
                    }
                    // The edges that cross are the ones whose ends differ.
                    let edges = [(0usize, 1usize), (0, 2), (0, 3), (1, 2), (1, 3), (2, 3)];
                    let mut hit: Vec<[f32; 3]> = Vec::with_capacity(4);
                    for (a, b) in edges {
                        let (ca, cb) = (tet[a], tet[b]);
                        if (f[ca] >= iso) != (f[cb] >= iso) {
                            hit.push(cut(p[ca], f[ca], p[cb], f[cb], iso));
                        }
                    }
                    if hit.len() < 3 {
                        continue;
                    }
                    let base = verts.len() as u32;
                    for h in &hit {
                        verts.push((h[0], h[1], h[2]));
                    }
                    // Three crossings is a triangle, four a quad.
                    idx.extend_from_slice(&[base, base + 1, base + 2]);
                    if hit.len() == 4 {
                        idx.extend_from_slice(&[base + 1, base + 3, base + 2]);
                    }
                }
            }
        }
    }
    // The gradient of the field, which is the surface's own normal — and
    // it points *into* the blob, so it is negated.
    let eps = step * 0.5;
    let mut norms: Vec<(f32, f32, f32)> = Vec::with_capacity(verts.len());
    for &(x, y, z) in &verts {
        let g = |d: [f32; 3]| field([x + d[0], y + d[1], z + d[2]], blobs) - field([x - d[0], y - d[1], z - d[2]], blobs);
        let n = [-g([eps, 0.0, 0.0]), -g([0.0, eps, 0.0]), -g([0.0, 0.0, eps])];
        let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt().max(1e-9);
        norms.push((n[0] / len, n[1] / len, n[2] / len));
    }
    // Each triangle was wound off the tetrahedron's edge order, which does
    // not say which way is out; the field's gradient does, so any triangle
    // facing the wrong way is flipped.
    for t in idx.chunks_exact_mut(3) {
        let (a, b, c) = (verts[t[0] as usize], verts[t[1] as usize], verts[t[2] as usize]);
        let (u, v) = ([b.0 - a.0, b.1 - a.1, b.2 - a.2], [c.0 - a.0, c.1 - a.1, c.2 - a.2]);
        let n = [u[1] * v[2] - u[2] * v[1], u[2] * v[0] - u[0] * v[2], u[0] * v[1] - u[1] * v[0]];
        let g = norms[t[0] as usize];
        if n[0] * g.0 + n[1] * g.1 + n[2] * g.2 < 0.0 {
            t.swap(1, 2);
        }
    }
    (verts, idx, norms)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// One blob is a ball: every vertex sits at the radius where the field
    /// crosses the threshold, and every triangle faces out.
    #[test]
    fn one_blob_is_a_ball() {
        let (v, i, n) = polygonise(&[Blob { at: [0.0; 3], radius: 0.7 }], 1.0, 1.6, 24);
        assert!(!v.is_empty() && i.len() % 3 == 0);
        // field = r²/d² = 1 at d = r, so the surface is the sphere of r.
        for &(x, y, z) in &v {
            let d = (x * x + y * y + z * z).sqrt();
            assert!((d - 0.7).abs() < 0.08, "{d} is about the radius");
        }
        for (k, &(nx, ny, nz)) in n.iter().enumerate() {
            let (x, y, z) = v[k];
            let dot = nx * x + ny * y + nz * z;
            assert!(dot > 0.0, "the normal points out of the blob");
        }
    }

    /// Two blobs close together are one surface, not two — which is the
    /// whole point of a metaball.
    #[test]
    fn two_near_blobs_fuse() {
        let near = [Blob { at: [-0.35, 0.0, 0.0], radius: 0.6 }, Blob { at: [0.35, 0.0, 0.0], radius: 0.6 }];
        // Midway between them the field is the sum of both, well over the
        // threshold, so there is solid there and the two have merged.
        assert!(super::field([0.0; 3], &near) > 1.0, "the waist is filled in");
        let far = [Blob { at: [-3.0, 0.0, 0.0], radius: 0.6 }, Blob { at: [3.0, 0.0, 0.0], radius: 0.6 }];
        assert!(super::field([0.0; 3], &far) < 1.0, "far apart they are two balls");
    }
}
