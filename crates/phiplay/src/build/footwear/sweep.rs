//! The three ways a shoe's pieces are made: a grid of points closed round
//! (an upper, a lining, a sole's wall), a tube swept along a path (piping,
//! a thong, a heel's stem) and a flat strip laid along a path (a strap, a
//! ribbon). Each comes out with smooth normals and its UVs along and
//! across it.

use super::super::mesh::BMesh;
use glam::{Vec2, Vec3};
use std::f32::consts::TAU;

/// Rows of points, each closed round on itself, joined row to row. With
/// `outward` the faces face away from the rows' middle; a lining faces in.
pub fn grid(rows: &[Vec<Vec3>], outward: bool) -> BMesh {
    sheet(rows, outward, true)
}

/// The same rows, closed round or left open at their ends (a patch).
pub fn sheet(rows: &[Vec<Vec3>], outward: bool, closed: bool) -> BMesh {
    let mut m = BMesh::default();
    let (nr, nc) = (rows.len(), rows.first().map_or(0, Vec::len));
    if nr < 2 || nc < 3 {
        return m;
    }
    for (i, row) in rows.iter().enumerate() {
        for (j, p) in row.iter().enumerate() {
            let along = if closed { row[(j + 1) % nc] - row[(j + nc - 1) % nc] } else { row[(j + 1).min(nc - 1)] - row[j.saturating_sub(1)] };
            let up = rows[(i + 1).min(nr - 1)][j] - rows[i.saturating_sub(1)][j];
            let mut n = along.cross(up);
            if !outward {
                n = -n;
            }
            m.vertex(*p, n, Vec2::new(j as f32 / nc as f32, i as f32 / (nr - 1) as f32));
        }
    }
    for i in 0..nr - 1 {
        for j in 0..if closed { nc } else { nc - 1 } {
            let (a, b) = ((i * nc + j) as u32, (i * nc + (j + 1) % nc) as u32);
            let (c, d) = (b + nc as u32, a + nc as u32);
            if outward {
                m.triangle(a, b, c);
                m.triangle(a, c, d);
            } else {
                m.triangle(a, c, b);
                m.triangle(a, d, c);
            }
        }
    }
    m
}

/// A closed row capped with a fan to `centre`, facing `n`.
pub fn cap(row: &[Vec3], centre: Vec3, n: Vec3) -> BMesh {
    let mut m = BMesh::default();
    let c = m.vertex(centre, n, Vec2::splat(0.5));
    let base = c + 1;
    for p in row {
        m.vertex(*p, n, Vec2::new(0.5, 0.5));
    }
    let k = row.len() as u32;
    // Wound so the fan faces `n`.
    let normal = (row[1] - centre).cross(row[0] - centre);
    let flip = normal.dot(n) > 0.0;
    for j in 0..k {
        let (a, b) = (base + j, base + (j + 1) % k);
        if flip { m.triangle(c, b, a) } else { m.triangle(c, a, b) }
    }
    m
}

/// Frames along a path, each turned as little as it can from the last.
fn frames(path: &[Vec3], closed: bool) -> Vec<(Vec3, Vec3, Vec3)> {
    let n = path.len();
    let tangent = |i: usize| {
        let (a, b) = if closed { (path[(i + n - 1) % n], path[(i + 1) % n]) } else { (path[i.saturating_sub(1)], path[(i + 1).min(n - 1)]) };
        (b - a).normalize_or_zero()
    };
    let t0 = tangent(0);
    let seed = if t0.z.abs() < 0.9 { Vec3::Z } else { Vec3::X };
    let mut normal = t0.cross(seed).normalize_or_zero();
    let mut out = Vec::with_capacity(n);
    for i in 0..n {
        let t = tangent(i);
        normal = (normal - t * normal.dot(t)).normalize_or_zero();
        out.push((t, normal, t.cross(normal)));
    }
    out
}

/// A tube along `path`, `radius(s)` thick at the share `s` of its length.
pub fn tube(path: &[Vec3], closed: bool, sides: usize, radius: impl Fn(f32) -> f32) -> BMesh {
    let mut m = BMesh::default();
    let n = path.len();
    if n < 2 {
        return m;
    }
    let lengths: Vec<f32> = std::iter::once(0.0).chain(path.windows(2).scan(0.0, |acc, w| { *acc += w[0].distance(w[1]); Some(*acc) })).collect();
    let total = lengths[n - 1].max(1e-6);
    for (i, (_, nrm, bin)) in frames(path, closed).into_iter().enumerate() {
        let r = radius(lengths[i] / total);
        for k in 0..sides {
            let a = TAU * k as f32 / sides as f32;
            let d = nrm * a.cos() + bin * a.sin();
            m.vertex(path[i] + d * r, d, Vec2::new(k as f32 / sides as f32, lengths[i] / total));
        }
    }
    let rings = if closed { n } else { n - 1 };
    for i in 0..rings {
        let (r0, r1) = (i * sides, ((i + 1) % n) * sides);
        for k in 0..sides {
            let (a, b) = ((r0 + k) as u32, (r0 + (k + 1) % sides) as u32);
            let (c, d) = ((r1 + (k + 1) % sides) as u32, (r1 + k) as u32);
            m.triangle(a, b, c);
            m.triangle(a, c, d);
        }
    }
    if !closed {
        for (end, i) in [(true, 0usize), (false, n - 1)] {
            let ring: Vec<Vec3> = (0..sides).map(|k| Vec3::from(m.positions[i * sides + k])).collect();
            let axis = (path[if end { 0 } else { n - 1 }] - path[if end { 1 } else { n - 2 }]).normalize_or_zero();
            let capped = cap(&ring, path[i], axis);
            m.merge(&capped, glam::Mat4::IDENTITY);
        }
    }
    m
}

/// A strap along `path`, `width` across, lying along `across` (one
/// direction per point, in the surface it lies on), facing `out`, a hair
/// thick so it shows from both sides.
pub fn strip(path: &[Vec3], across: &[Vec3], out: &[Vec3], width: f32, closed: bool) -> BMesh {
    let mut m = BMesh::default();
    let n = path.len();
    if n < 2 {
        return m;
    }
    const THICK: f32 = 0.0012;
    // Wound so the first face looks along `out`, whichever way `across`
    // was given.
    let turned = (path[1] - path[0]).cross(across[0]).dot(out[0]) < 0.0;
    for (face, sign) in [(0u32, 1.0f32), (1, -1.0)] {
        let base = m.positions.len() as u32;
        for i in 0..n {
            let lift = out[i] * (THICK * 0.5 * sign);
            for e in [-0.5f32, 0.5] {
                m.vertex(path[i] + across[i] * (e * width) + lift, out[i] * sign, Vec2::new(e + 0.5, i as f32 / (n - 1) as f32));
            }
        }
        let segs = if closed { n } else { n - 1 };
        for i in 0..segs {
            let (a, b) = (base + 2 * i as u32, base + 2 * i as u32 + 1);
            let j = ((i + 1) % n) as u32;
            let (c, d) = (base + 2 * j + 1, base + 2 * j);
            if (face == 0) != turned {
                m.triangle(a, d, c);
                m.triangle(a, c, b);
            } else {
                m.triangle(a, c, d);
                m.triangle(a, b, c);
            }
        }
    }
    m
}

/// `m` mirrored across x = 0 — the right shoe from the left — its faces
/// turned back so they still face out.
pub fn mirrored(m: &BMesh) -> BMesh {
    let mut out = m.clone();
    for p in &mut out.positions {
        p[0] = -p[0];
    }
    for n in &mut out.normals {
        n[0] = -n[0];
    }
    for t in out.indices.chunks_mut(3) {
        t.swap(1, 2);
    }
    out
}
