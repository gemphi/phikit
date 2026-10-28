//! The limit surface at PTEX grid points: Blender's
//! `eval_limit_point_and_derivatives` (`subdiv_eval.cc`) for the points the
//! multires grid lands on — positions from the Subdivision Surface's limit
//! evaluation (`subsurf.rs`), derivative directions from the vertex rings
//! (`multires_tangent.rs`).

use super::grid::BGrids;
use super::tangent::{BRing, D3};
use super::super::topology::Topology;
use super::super::BlendEvalMesh;

/// What the derivatives are read from.
pub(super) struct BLimit<'a> {
    pub grids: &'a BGrids,
    /// The cage refined to the grid's level, and its adjacency.
    pub cage: &'a BlendEvalMesh,
    pub topology: &'a Topology,
    /// Every cage vertex's limit position.
    pub limit: &'a [[f32; 3]],
    /// Grid level and OpenSubdiv's isolation level (`quality`).
    pub level: u32,
    pub quality: u32,
    /// `boundary_smooth` Keep Corners.
    pub keep_corners: bool,
}

fn f32s(a: D3) -> [f32; 3] {
    [a[0] as f32, a[1] as f32, a[2] as f32]
}

fn neg(a: [f32; 3]) -> [f32; 3] {
    [-a[0], -a[1], -a[2]]
}

impl BLimit<'_> {
    /// `dPdu`, `dPdv` at PTEX `(ptex, u, v)`, up to a positive scale each.
    /// A quad's PTEX face is turned onto the corner grid holding the point
    /// (`rotate_quad_to_corner`: corner 1 has `s = 2v`, `t = 2(1 − u)`, …),
    /// so its `u` and `v` derivatives are that grid's `±s`, `±t` ones.
    pub(super) fn derivatives(&self, ptex: usize, u: f32, v: f32) -> ([f32; 3], [f32; 3]) {
        let (l, s, t, quad) = self.grids.locate(ptex, u, v);
        let ds = self.along(l, s, t, true);
        let dt = self.along(l, s, t, false);
        match quad {
            Some(1) => (neg(dt), ds),
            Some(2) => (neg(ds), neg(dt)),
            Some(3) => (dt, neg(ds)),
            _ => (ds, dt),
        }
    }

    /// The derivative along the grid's `s` (or `t`) axis at `(s, t)`: the
    /// tangent toward the next grid point that way, or away from the
    /// previous one on the grid's far side.
    fn along(&self, l: usize, s: usize, t: usize, axis_s: bool) -> [f32; 3] {
        let r = self.grids.r;
        let at = self.grids.vertex(l, s, t);
        let coord = if axis_s { s } else { t };
        let (step, sign) = match coord < r {
            true => (coord + 1, 1.0),
            false => (coord - 1, -1.0),
        };
        let toward = match axis_s {
            true => self.grids.vertex(l, step, t),
            false => self.grids.vertex(l, s, step),
        };
        let d = self.tangent(at, toward);
        [d[0] * sign, d[1] * sign, d[2] * sign]
    }

    /// The tangent at cage vertex `at` toward its neighbour `toward`; an
    /// extraordinary vertex's from its ring at the isolation level. Off a
    /// manifold, the chord between the two limit points stands in.
    fn tangent(&self, at: u32, toward: u32) -> [f32; 3] {
        let chord = || {
            let (a, b) = (self.limit[at as usize], self.limit[toward as usize]);
            [b[0] - a[0], b[1] - a[1], b[2] - a[2]]
        };
        let Some(mut ring) = BRing::of(self.topology, self.cage, at as usize) else { return chord() };
        let Some(j) = ring.ids.iter().position(|&i| i == toward) else { return chord() };
        if ring.irregular() {
            for _ in self.level..self.quality {
                ring = ring.refined(self.keep_corners);
            }
        }
        f32s(ring.tangent(j, self.keep_corners))
    }
}
