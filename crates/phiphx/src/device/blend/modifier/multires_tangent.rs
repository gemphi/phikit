//! Limit-surface tangents at the vertices of the refined cage, as
//! OpenSubdiv's patches give them to `eval_limit_point_and_derivatives`
//! (`subdiv_eval.cc`), for the PTEX grid points multires evaluates.
//!
//! Only directions matter: `BKE_multires_construct_tangent_matrix`
//! normalises each axis. At a grid point the derivative along an edge is the
//! tangent of the vertex's 1-ring toward that edge:
//! - a regular vertex (four faces, or two on a boundary) sits on B-spline
//!   patches, whose derivative at a knot is the exact limit tangent — the
//!   same from any level's ring;
//! - an extraordinary one is the corner of a Gregory end cap built at the
//!   isolation level (`quality`) by OpenSubdiv's `GregoryConverter`
//!   (`far/catmarkPatchBuilder.cpp`): `CatmarkLimits::
//!   ComputeInteriorPointWeights` / `ComputeBoundaryPointWeights` over the
//!   ring *at that level*, which the ring here is subdivided to first.
//!
//! Creases are not taken into account (a smooth ring is assumed).

use super::super::topology::Topology;
use super::super::BlendEvalMesh;
use std::f64::consts::PI;

pub(super) type D3 = [f64; 3];

fn d3(p: [f32; 3]) -> D3 {
    [p[0] as f64, p[1] as f64, p[2] as f64]
}
fn add(a: D3, b: D3) -> D3 {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}
fn sc(a: D3, s: f64) -> D3 {
    [a[0] * s, a[1] * s, a[2] * s]
}
fn sum(ps: &[(D3, f64)]) -> D3 {
    ps.iter().fold([0.0; 3], |acc, &(p, w)| add(acc, sc(p, w)))
}

/// A vertex's 1-ring in OpenSubdiv's order: edge neighbours `e[i]`, and the
/// opposite corner `d[i]` of the face between `e[i]` and `e[i + 1]`. On a
/// boundary `e` has one more entry than `d`, the first and last on the
/// boundary edges.
#[derive(Clone)]
pub(super) struct BRing {
    pub v: D3,
    pub e: Vec<D3>,
    pub d: Vec<D3>,
    /// Cage vertex of each `e`.
    pub ids: Vec<u32>,
    pub boundary: bool,
}

impl BRing {
    /// The 1-ring of `v` in `cage` (all quads). `None` off a manifold.
    pub(super) fn of(t: &Topology, cage: &BlendEvalMesh, v: usize) -> Option<Self> {
        // (next, opposite, previous) of every face round `v`.
        let mut fans: Vec<(u32, u32, u32)> = Vec::new();
        for &(fi, c) in t.vert_corners(v) {
            let f = cage.faces.get(fi as usize);
            if f.len() != 4 {
                return None;
            }
            let c = c as usize;
            fans.push((f[(c + 1) % 4], f[(c + 2) % 4], f[(c + 3) % 4]));
        }
        if fans.is_empty() {
            return None;
        }
        let open = fans.iter().position(|a| !fans.iter().any(|b| b.2 == a.0));
        let mut used = vec![false; fans.len()];
        let mut at = open.unwrap_or(0);
        let p = |i: u32| d3(cage.vertices[i as usize]);
        let mut ring = BRing { v: p(v as u32), e: Vec::new(), d: Vec::new(), ids: Vec::new(), boundary: open.is_some() };
        loop {
            used[at] = true;
            let (n, o, pr) = fans[at];
            ring.ids.push(n);
            ring.e.push(p(n));
            ring.d.push(p(o));
            match fans.iter().position(|a| a.0 == pr) {
                Some(next) if !used[next] => at = next,
                Some(_) if open.is_none() => break,
                None if open.is_some() => {
                    ring.ids.push(pr);
                    ring.e.push(p(pr));
                    break;
                }
                _ => return None,
            }
        }
        used.iter().all(|&u| u).then_some(ring)
    }

    /// Faces round the vertex.
    fn faces(&self) -> usize {
        self.d.len()
    }

    /// Whether OpenSubdiv isolates it (`isRegular` of `GregoryConverter::
    /// Initialize`: `numFaces << isBoundary == 4`).
    pub(super) fn irregular(&self) -> bool {
        (self.faces() << self.boundary as usize) != 4
    }

    /// The ring one Catmull-Clark level finer (face, edge and vertex points
    /// of the faces round the vertex; a boundary vertex by the crease rule,
    /// its boundary edges by their midpoints; a corner kept if `corner`).
    pub(super) fn refined(&self, corner: bool) -> Self {
        let (v, n) = (self.v, self.faces());
        let ne = self.e.len();
        let f: Vec<D3> = (0..n).map(|i| sc(add(add(v, self.e[i]), add(self.d[i], self.e[(i + 1) % ne])), 0.25)).collect();
        let e: Vec<D3> = (0..ne)
            .map(|i| match (self.boundary, i) {
                (true, 0) => sc(add(v, self.e[0]), 0.5),
                (true, i) if i == n => sc(add(v, self.e[n]), 0.5),
                _ => sc(add(add(v, self.e[i]), add(f[(i + n - 1) % n], f[i])), 0.25),
            })
            .collect();
        let nv = match (self.boundary, corner && n == 1) {
            (true, true) => v,
            (true, false) => sc(add(add(self.e[0], self.e[n]), sc(v, 6.0)), 0.125),
            (false, _) => {
                let k = n as f64;
                let s = self.e.iter().chain(f.iter()).fold([0.0; 3], |a, &p| add(a, p));
                add(sc(v, (k - 2.0) / k), sc(s, 1.0 / (k * k)))
            }
        };
        BRing { v: nv, e, d: f, ids: self.ids.clone(), boundary: self.boundary }
    }

    /// The tangent toward edge `j` (`Ep − P` of the patch whose `Ep` runs
    /// along it), up to a positive scale. `sharp_corner`: a one-face corner
    /// kept sharp (`boundary_smooth` Keep Corners).
    pub(super) fn tangent(&self, j: usize, sharp_corner: bool) -> D3 {
        match (self.boundary, self.faces()) {
            (false, n) => self.interior(j, n),
            (true, 1) if sharp_corner => add(self.e[j], sc(self.v, -1.0)),
            (true, 1) => add(self.e[j], sc(self.e[1 - j.min(1)], -1.0)),
            (true, k) => self.boundary_tangent(j, k),
        }
    }

    /// `ComputeInteriorPointWeights`: each ring point weighted by the
    /// cosines of the faces' angles, the whole rotated to edge `j`.
    fn interior(&self, j: usize, n: usize) -> D3 {
        let ang = 2.0 * PI / n as f64;
        let mut t = [0.0; 3];
        for i in 0..n {
            let (ip, inx) = ((i + n - 1) % n, (i + 1) % n);
            let w = ((ang * ((i + n - j) % n) as f64).cos(), self.e[i]);
            let m = sum(&[(self.e[ip], 2.0), (self.d[ip], 1.0), (w.1, 4.0), (self.d[i], 1.0), (self.e[inx], 2.0)]);
            t = add(t, sc(m, w.0));
        }
        t
    }

    /// `ComputeBoundaryPointWeights`: the boundary tangent `(e₀ − e_k)/6`
    /// and the interior one of Loop and Schaefer, mixed by the edge's angle
    /// `jπ/k`; on the boundary edges themselves, the boundary tangent alone.
    fn boundary_tangent(&self, j: usize, k: usize) -> D3 {
        let along = sc(add(self.e[0], sc(self.e[k], -1.0)), 1.0 / 6.0);
        match j {
            0 => return along,
            j if j == k => return sc(along, -1.0),
            _ => {}
        }
        let kf = k as f64;
        let theta = PI / kf;
        let (c, s) = (theta.cos(), theta.sin());
        let div3kc = 1.0 / (3.0 * kf + c);
        let gamma = -4.0 * s * div3kc;
        let alpha0 = -((1.0 + 2.0 * c) * (1.0 + c).sqrt()) * div3kc / (1.0 - c).sqrt();
        let mut tan = sum(&[(self.v, gamma), (self.e[0], alpha0), (self.e[k], alpha0), (self.d[0], s * div3kc)]);
        for i in 1..k {
            let (si, si1) = ((theta * i as f64).sin(), (theta * (i + 1) as f64).sin());
            tan = add(tan, sum(&[(self.e[i], 4.0 * si * div3kc), (self.d[i], (si + si1) * div3kc)]));
        }
        let a = theta * j as f64;
        add(sc(tan, a.sin() / 3.0), sc(along, a.cos()))
    }
}
