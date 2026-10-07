//! Least-squares conformal maps — Blender's `param_lscm_*`, which is
//! Unwrap ▸ Conformal and the second half of Unwrap ▸ Angle Based.
//!
//! Lévy et al.'s formulation. Flatten each triangle into its own plane,
//! and ask of the map `u = U + iV` that each triangle's complex derivative
//! be the same in both directions — the Cauchy–Riemann condition, which is
//! exactly "angles are preserved". Per triangle that is one complex
//! equation,
//!
//! ```text
//! W₁ u₁ + W₂ u₂ + W₃ u₃ = 0,   Wⱼ = (x_{j+2} − x_{j+1}) + i (y_{j+2} − y_{j+1})
//! ```
//!
//! and over the mesh it is a tall system in `2V` real unknowns that is
//! solved in the least-squares sense. It has no unique answer on its own:
//! a conformal map composed with any similarity is still conformal, so the
//! system has a four-dimensional null space and **at least two vertices
//! must be fixed**. Those are the pins. Blender uses the user's when there
//! are any and picks two itself when there are not; both are here.
//!
//! The normal equations are solved by conjugate gradients, which never
//! forms `AᵀA` — each iteration is one `A x` and one `Aᵀ y` over the rows,
//! so the cost is the triangle count and the memory is the mesh.

use super::island::{self, BUvIsland};
use super::{BUvMesh, BUvPin};

/// One row of the system: `Σ coef · unknown = rhs`.
struct Row {
    at: Vec<(usize, f32)>,
    rhs: f32,
}

/// Flatten a triangle into its own plane: `a` at the origin, `b` along x.
fn flat(a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> [[f32; 2]; 3] {
    let e = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let f = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
    let len = (e[0] * e[0] + e[1] * e[1] + e[2] * e[2]).sqrt();
    if len < 1e-12 {
        return [[0.0; 2]; 3];
    }
    let ex = [e[0] / len, e[1] / len, e[2] / len];
    let dot = ex[0] * f[0] + ex[1] * f[1] + ex[2] * f[2];
    let perp = [f[0] - dot * ex[0], f[1] - dot * ex[1], f[2] - dot * ex[2]];
    let h = (perp[0] * perp[0] + perp[1] * perp[1] + perp[2] * perp[2]).sqrt();
    [[0.0, 0.0], [len, 0.0], [dot, h]]
}

/// Solve the island into `uv` (one per corner of the whole mesh).
pub fn solve(mesh: BUvMesh<'_>, part: &BUvIsland, pin: &[BUvPin], uv: &mut [[f32; 2]]) {
    // The island's own vertices, numbered for the system.
    let mut index: std::collections::HashMap<u32, usize> = std::collections::HashMap::new();
    let mut vert: Vec<u32> = Vec::new();
    for &f in &part.face {
        for &c in mesh.face(f) {
            if !index.contains_key(&c) {
                index.insert(c, vert.len());
                vert.push(c);
            }
        }
    }
    if vert.len() < 3 {
        return;
    }
    // Which of the island's vertices are fixed, and to what. A pin names a
    // corner; its vertex is what the system can fix.
    let mut fixed: Vec<(usize, [f32; 2])> = Vec::new();
    for p in pin {
        if let Some(&c) = mesh.corner.get(p.corner)
            && let Some(&k) = index.get(&c)
            && !fixed.iter().any(|(j, _)| *j == k)
        {
            fixed.push((k, p.uv));
        }
    }
    if fixed.len() < 2 {
        // Blender picks two itself: the pair of the island's vertices
        // furthest apart, laid along U at their own distance, so the
        // island comes out at roughly its own scale rather than at
        // whatever the solver's null space happened to land on.
        let (mut best, mut pair) = (-1.0f32, (0usize, 1usize));
        for a in 0..vert.len() {
            for b in a + 1..vert.len() {
                let (p, q) = (mesh.pos[vert[a] as usize], mesh.pos[vert[b] as usize]);
                let d = (p[0] - q[0]).powi(2) + (p[1] - q[1]).powi(2) + (p[2] - q[2]).powi(2);
                if d > best {
                    best = d;
                    pair = (a, b);
                }
            }
        }
        let span = best.max(1e-12).sqrt();
        fixed.retain(|(k, _)| *k != pair.0 && *k != pair.1);
        fixed.push((pair.0, [0.0, 0.0]));
        fixed.push((pair.1, [span, 0.0]));
    }
    // A closed island has no boundary to open out, and the conformal
    // system over it is degenerate. Blender's own answer is as poor; this
    // one at least says so by falling back to the island's dominant plane
    // rather than returning noise.
    if !island::open(mesh, part) {
        plane_project(mesh, part, uv);
        return;
    }

    let n = vert.len();
    let free: Vec<usize> = (0..n).filter(|k| !fixed.iter().any(|(j, _)| j == k)).collect();
    let mut slot = vec![usize::MAX; n];
    for (s, &k) in free.iter().enumerate() {
        slot[k] = s;
    }
    let value = |k: usize| fixed.iter().find(|(j, _)| *j == k).map(|(_, v)| *v);

    let mut rows: Vec<Row> = Vec::new();
    for &f in &part.face {
        let run = mesh.face(f);
        for k in 1..run.len().saturating_sub(1) {
            let tri = [run[0], run[k], run[k + 1]];
            let p = flat(mesh.pos[tri[0] as usize], mesh.pos[tri[1] as usize], mesh.pos[tri[2] as usize]);
            let area = ((p[1][0] - p[0][0]) * (p[2][1] - p[0][1]) - (p[2][0] - p[0][0]) * (p[1][1] - p[0][1])).abs();
            if area < 1e-14 {
                continue;
            }
            let s = 1.0 / area.sqrt();
            // Wⱼ = (x_{j+2} − x_{j+1}) + i (y_{j+2} − y_{j+1}).
            let w: [[f32; 2]; 3] = std::array::from_fn(|j| {
                let (a, b) = ((j + 2) % 3, (j + 1) % 3);
                [(p[a][0] - p[b][0]) * s, (p[a][1] - p[b][1]) * s]
            });
            // Real part: Σ (Wx U − Wy V) = 0. Imaginary: Σ (Wy U + Wx V) = 0.
            for imag in [false, true] {
                let mut at = Vec::with_capacity(6);
                let mut rhs = 0.0;
                for j in 0..3 {
                    let k = index[&tri[j]];
                    let (cu, cv) = if imag { (w[j][1], w[j][0]) } else { (w[j][0], -w[j][1]) };
                    match value(k) {
                        Some(v) => rhs -= cu * v[0] + cv * v[1],
                        None => {
                            at.push((2 * slot[k], cu));
                            at.push((2 * slot[k] + 1, cv));
                        }
                    }
                }
                if !at.is_empty() {
                    rows.push(Row { at, rhs });
                }
            }
        }
    }
    let x = least_squares(&rows, 2 * free.len());
    // Back onto the corners. Every corner of the island takes its own
    // vertex's UV, which is what makes a seam's two sides separate: they
    // are different vertices only where the mesh says so.
    for &f in &part.face {
        let (a, b) = (mesh.starts[f] as usize, mesh.starts[f + 1] as usize);
        for c in a..b {
            let k = index[&mesh.corner[c]];
            uv[c] = match value(k) {
                Some(v) => v,
                None => [x[2 * slot[k]], x[2 * slot[k] + 1]],
            };
        }
    }
}

/// The island flattened onto its own dominant plane — the honest answer
/// for a closed island, which has no boundary for a conformal map to open.
fn plane_project(mesh: BUvMesh<'_>, part: &BUvIsland, uv: &mut [[f32; 2]]) {
    let mut n = [0.0f32; 3];
    for &f in &part.face {
        let run = mesh.face(f);
        for k in 1..run.len().saturating_sub(1) {
            let (a, b, c) = (mesh.pos[run[0] as usize], mesh.pos[run[k] as usize], mesh.pos[run[k + 1] as usize]);
            let (u, v) = ([b[0] - a[0], b[1] - a[1], b[2] - a[2]], [c[0] - a[0], c[1] - a[1], c[2] - a[2]]);
            n[0] += u[1] * v[2] - u[2] * v[1];
            n[1] += u[2] * v[0] - u[0] * v[2];
            n[2] += u[0] * v[1] - u[1] * v[0];
        }
    }
    let axis = (0..3).max_by(|a, b| n[*a].abs().total_cmp(&n[*b].abs())).unwrap_or(2);
    let (i, j) = match axis {
        0 => (1, 2),
        1 => (0, 2),
        _ => (0, 1),
    };
    for &f in &part.face {
        let (a, b) = (mesh.starts[f] as usize, mesh.starts[f + 1] as usize);
        for c in a..b {
            let p = mesh.pos[mesh.corner[c] as usize];
            uv[c] = [p[i], p[j]];
        }
    }
}

/// `min ‖A x − b‖` by conjugate gradients on `AᵀA x = Aᵀ b`, with `A`
/// never formed: each step is one pass over the rows each way.
fn least_squares(rows: &[Row], unknowns: usize) -> Vec<f32> {
    let mut x = vec![0.0f32; unknowns];
    if unknowns == 0 || rows.is_empty() {
        return x;
    }
    let apply = |x: &[f32]| -> Vec<f32> { rows.iter().map(|r| r.at.iter().map(|(k, c)| c * x[*k]).sum()).collect() };
    let adjoint = |y: &[f32]| -> Vec<f32> {
        let mut out = vec![0.0f32; unknowns];
        for (r, row) in rows.iter().enumerate() {
            for (k, c) in &row.at {
                out[*k] += c * y[r];
            }
        }
        out
    };
    let b: Vec<f32> = rows.iter().map(|r| r.rhs).collect();
    let ax = apply(&x);
    let resid: Vec<f32> = b.iter().zip(&ax).map(|(b, a)| b - a).collect();
    let mut g = adjoint(&resid);
    let mut p = g.clone();
    let mut gg: f32 = g.iter().map(|v| v * v).sum();
    let tol = gg * 1e-12;
    for _ in 0..(4 * unknowns).clamp(64, 4000) {
        if gg <= tol || !gg.is_finite() {
            break;
        }
        let ap = apply(&p);
        let pp: f32 = ap.iter().map(|v| v * v).sum();
        if pp <= 1e-30 {
            break;
        }
        let alpha = gg / pp;
        for k in 0..unknowns {
            x[k] += alpha * p[k];
        }
        let atap = adjoint(&ap);
        for k in 0..unknowns {
            g[k] -= alpha * atap[k];
        }
        let next: f32 = g.iter().map(|v| v * v).sum();
        let beta = next / gg;
        for k in 0..unknowns {
            p[k] = g[k] + beta * p[k];
        }
        gg = next;
    }
    x
}

#[cfg(test)]
mod tests {
    use super::super::tests::grid;
    use super::super::{BUnwrap, BUvPin, unwrap};
    use super::*;

    /// The angle a triangle's UVs make at each corner, against the angle
    /// the mesh makes there: a conformal map keeps them.
    fn angle_error(mesh: BUvMesh<'_>, uv: &[[f32; 2]]) -> f32 {
        let mut worst = 0.0f32;
        for f in 0..mesh.faces() {
            let run = mesh.face(f);
            for k in 1..run.len().saturating_sub(1) {
                let tri = [0, k, k + 1];
                let at = mesh.starts[f] as usize;
                for j in 0..3 {
                    let (a, b, c) = (tri[j], tri[(j + 1) % 3], tri[(j + 2) % 3]);
                    let ang3 = angle3(mesh.pos[mesh.corner[at + a] as usize], mesh.pos[mesh.corner[at + b] as usize], mesh.pos[mesh.corner[at + c] as usize]);
                    let ang2 = angle2(uv[at + a], uv[at + b], uv[at + c]);
                    worst = worst.max((ang3 - ang2).abs());
                }
            }
        }
        worst
    }

    fn angle3(o: [f32; 3], a: [f32; 3], b: [f32; 3]) -> f32 {
        let u = [a[0] - o[0], a[1] - o[1], a[2] - o[2]];
        let v = [b[0] - o[0], b[1] - o[1], b[2] - o[2]];
        let d = u[0] * v[0] + u[1] * v[1] + u[2] * v[2];
        let lu = (u[0] * u[0] + u[1] * u[1] + u[2] * u[2]).sqrt();
        let lv = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
        (d / (lu * lv).max(1e-12)).clamp(-1.0, 1.0).acos()
    }

    fn angle2(o: [f32; 2], a: [f32; 2], b: [f32; 2]) -> f32 {
        let u = [a[0] - o[0], a[1] - o[1]];
        let v = [b[0] - o[0], b[1] - o[1]];
        let d = u[0] * v[0] + u[1] * v[1];
        let lu = (u[0] * u[0] + u[1] * u[1]).sqrt();
        let lv = (v[0] * v[0] + v[1] * v[1]).sqrt();
        (d / (lu * lv).max(1e-12)).clamp(-1.0, 1.0).acos()
    }

    /// A flat grid is already flat: the conformal map of it is the grid
    /// itself, so every angle survives exactly. If this is wrong nothing
    /// else can be right.
    #[test]
    fn a_flat_grid_unwraps_to_itself() {
        let (pos, corner, starts) = grid(4);
        let mesh = BUvMesh { pos: &pos, corner: &corner, starts: &starts };
        let uv = unwrap(mesh, BUnwrap::Conformal, &[], &[]);
        assert_eq!(uv.len(), corner.len());
        assert!(angle_error(mesh, &uv) < 1e-3, "every angle kept: {}", angle_error(mesh, &uv));
    }

    /// A cylinder cut open along one seam — a real curved surface — comes
    /// back with its angles within a couple of degrees, which is what a
    /// conformal map promises (and area, which it does not, is free to
    /// stretch).
    #[test]
    fn an_open_cylinder_keeps_its_angles() {
        let (rings, seg) = (6usize, 12usize);
        let mut pos = Vec::new();
        for j in 0..=rings {
            for i in 0..=seg {
                let t = i as f32 / seg as f32 * std::f32::consts::TAU;
                pos.push([t.cos(), t.sin(), j as f32 / rings as f32 * 2.0]);
            }
        }
        let (mut corner, mut starts) = (Vec::new(), vec![0u32]);
        let stride = (seg + 1) as u32;
        for j in 0..rings as u32 {
            for i in 0..seg as u32 {
                let a = j * stride + i;
                corner.extend_from_slice(&[a, a + 1, a + stride + 1, a + stride]);
                starts.push(corner.len() as u32);
            }
        }
        let mesh = BUvMesh { pos: &pos, corner: &corner, starts: &starts };
        let uv = unwrap(mesh, BUnwrap::Conformal, &[], &[]);
        let worst = angle_error(mesh, &uv).to_degrees();
        assert!(worst < 3.0, "the angles survive the unwrap: worst {worst:.2} degrees");
    }

    /// Pins hold. Two corners pinned where the user put them, and the
    /// solved UVs land on them — which is the only thing that makes a pin
    /// more than a different colour of dot.
    #[test]
    fn a_pinned_corner_stays_where_it_was_put() {
        let (pos, corner, starts) = grid(3);
        let mesh = BUvMesh { pos: &pos, corner: &corner, starts: &starts };
        let pins = [BUvPin { corner: 0, uv: [0.25, 0.75] }, BUvPin { corner: 2, uv: [0.9, 0.1] }];
        let islands = super::super::island::split(mesh, &[]);
        let mut uv = vec![[0.0f32; 2]; corner.len()];
        super::solve(mesh, &islands[0], &pins, &mut uv);
        for p in pins {
            assert!((uv[p.corner][0] - p.uv[0]).abs() < 1e-5 && (uv[p.corner][1] - p.uv[1]).abs() < 1e-5, "{:?} vs {:?}", uv[p.corner], p.uv);
        }
        // And the rest moved with them rather than ignoring them: the
        // solved map is the one the two pins pick out of the similarity
        // family, so it is not the identity any more.
        assert!(uv.iter().any(|c| c[0] > 0.9 || c[1] > 0.75), "the island followed the pins: {uv:?}");
    }
}
