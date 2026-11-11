//! Blender `subdiv_mesh.cc` with a displacement attached: which PTEX points
//! each subdivided vertex is evaluated at (`subdiv_foreach.cc`'s corner,
//! edge and inner traversals) and how their displacements combine.
//!
//! A vertex on a coarse corner or edge is visited once from every face
//! using it (`vert_every_corner` / `vert_every_edge`), its displacements
//! summed (`subdiv_accumulate_vert_displacement`) and divided by the count
//! (`evaluate_vert_and_apply_displacement_copy` / `_interpolate`); a vertex
//! inside a face is evaluated once (`subdiv_mesh_vert_inner`,
//! `eval_final_point`). Either way the displacement lands on the limit
//! position.

use super::disp::BDisplacement;
use rayon::prelude::*;

type F3 = [f32; 3];

/// One evaluation: the vertex, its displacement, and whether it is one of
/// several to average (corner and edge vertices) or the only one (inner).
type BVisit = (u32, F3, bool);

/// Every vertex `limit` holds, displaced: the grid at `level` over `faces`
/// (each coarse face's corner count).
pub(super) fn displace(disp: &BDisplacement, sizes: &[usize], level: u32, limit: &[F3]) -> Vec<F3> {
    let visits: Vec<Vec<BVisit>> = (0..sizes.len()).into_par_iter().map(|f| face_visits(disp, f, sizes[f], level)).collect();
    let mut sum = vec![[0.0f32; 3]; limit.len()];
    let mut count = vec![0u32; limit.len()];
    let mut inner: Vec<Option<F3>> = vec![None; limit.len()];
    for (v, d, accumulate) in visits.into_iter().flatten() {
        let v = v as usize;
        if v >= limit.len() {
            continue;
        }
        match accumulate {
            true => {
                sum[v] = std::array::from_fn(|i| sum[v][i] + d[i]);
                count[v] += 1;
            }
            false => inner[v] = Some(d),
        }
    }
    (0..limit.len())
        .map(|v| {
            let d = match (inner[v], count[v]) {
                (Some(d), _) => d,
                (None, 0) => [0.0; 3],
                (None, n) => {
                    let inv = 1.0 / n as f32;
                    std::array::from_fn(|i| sum[v][i] * inv)
                }
            };
            std::array::from_fn(|i| limit[v][i] + d[i])
        })
        .collect()
}

/// Everything face `f` evaluates, in Blender's order.
fn face_visits(disp: &BDisplacement, f: usize, n: usize, level: u32) -> Vec<BVisit> {
    let grids = disp.grids;
    let ptex = grids.ptex_offset[f];
    let resolution = (1usize << level) + 1;
    let mut out = Vec::new();
    let mut visit = |p: usize, u: f32, v: f32, accumulate: bool| {
        out.push((grids.vertex_at(p, u, v), disp.at(p, u, v), accumulate));
    };
    match n {
        4 => {
            let inv = 1.0 / (resolution - 1) as f32;
            // `subdiv_foreach_corner_vertices_regular_do`.
            for (u, v) in [(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)] {
                visit(ptex, u, v, true);
            }
            // `subdiv_foreach_edge_vertices_regular_do`.
            for corner in 0..4 {
                for i in 1..resolution - 1 {
                    let fac = i as f32 * inv;
                    let (u, v) = match corner {
                        0 => (fac, 0.0),
                        1 => (1.0, fac),
                        2 => (fac, 1.0),
                        _ => (0.0, fac),
                    };
                    visit(ptex, u, v, true);
                }
            }
            // `subdiv_foreach_inner_vertices_regular`.
            for y in 1..resolution - 1 {
                for x in 1..resolution - 1 {
                    visit(ptex, x as f32 * inv, y as f32 * inv, false);
                }
            }
        }
        _ => {
            let side = (resolution >> 1) + 1;
            let inv = 1.0 / (side - 1) as f32;
            // `subdiv_foreach_corner_vertices_special_do`.
            for corner in 0..n {
                visit(ptex + corner, 0.0, 0.0, true);
            }
            // `subdiv_foreach_edge_vertices_special_do`: the first half of
            // the edge (and its midpoint) in this corner's PTEX face, the
            // rest in the next corner's.
            for corner in 0..n {
                for i in 1..side {
                    visit(ptex + corner, i as f32 * inv, 0.0, true);
                }
                for i in 1..side - 1 {
                    visit(ptex + (corner + 1) % n, 0.0, 1.0 - i as f32 * inv, true);
                }
            }
            // `subdiv_foreach_inner_vertices_special`.
            visit(ptex, 1.0, 1.0, false);
            for corner in 0..n {
                for y in 1..side - 1 {
                    for x in 1..side {
                        visit(ptex + corner, x as f32 * inv, y as f32 * inv, false);
                    }
                }
            }
        }
    }
    out
}
