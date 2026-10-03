//! A mesh's extreme points: the vertices farthest along each of a fixed
//! set of directions, found once per mesh and kept by the mesh's address.
//!
//! The outline of a selected object is the convex hull of its projected
//! vertices (`hull.rs`). Projecting every vertex each time the view moved
//! was the most expensive thing an orbit did on a large mesh: a floor of
//! two million vertices was projected and sorted on every mouse move. A
//! projection is linear, so the hull of the projected vertices is the hull
//! of the projected *3D hull* vertices, and a vertex farthest along some
//! direction is on the 3D hull. The farthest vertex along each of
//! [`DIRECTIONS`] directions, spread over the sphere, is a few hundred
//! points whose hull is within about a percent of the exact one — under
//! the outline's own width.

use crate::phiphx::scene::SceneMesh;
use crate::v2::scene::keep::BMeshKeep;
use glam::Vec3;
use std::sync::Arc;

/// How many directions: a Fibonacci sphere of this many.
pub const DIRECTIONS: usize = 192;
/// A mesh with no more vertices than this is used whole: its hull is exact
/// and costs less than the directions would.
pub const WHOLE: usize = 2 * DIRECTIONS;

static KEPT: BMeshKeep<Vec<[f32; 3]>> = BMeshKeep::new();

/// The extreme points of `mesh`, found once and kept while the mesh lives —
/// here on first use, or as the file loads (`scene/warm.rs`).
pub fn of(mesh: &Arc<SceneMesh>) -> Arc<Vec<[f32; 3]>> {
    KEPT.of(mesh, |m| find(&m.vertices))
}

/// Whether `mesh` has its points already.
pub fn ready(mesh: &Arc<SceneMesh>) -> bool {
    KEPT.has(mesh)
}

/// The directions, spread over the sphere (a Fibonacci lattice).
fn directions() -> Vec<Vec3> {
    let golden = std::f32::consts::PI * (3.0 - 5.0f32.sqrt());
    (0..DIRECTIONS)
        .map(|i| {
            let y = 1.0 - (i as f32 + 0.5) / DIRECTIONS as f32 * 2.0;
            let r = (1.0 - y * y).max(0.0).sqrt();
            let a = golden * i as f32;
            Vec3::new(a.cos() * r, y, a.sin() * r)
        })
        .collect()
}

/// The farthest vertex along each direction, each listed once.
pub fn find(vertices: &[[f32; 3]]) -> Vec<[f32; 3]> {
    if vertices.len() <= WHOLE {
        return vertices.to_vec();
    }
    let directions = directions();
    let mut best: Vec<(f32, usize)> = vec![(f32::NEG_INFINITY, 0); directions.len()];
    for (i, v) in vertices.iter().enumerate() {
        let v = Vec3::from(*v);
        for (d, slot) in directions.iter().zip(best.iter_mut()) {
            let along = d.dot(v);
            if along > slot.0 {
                *slot = (along, i);
            }
        }
    }
    let mut picked: Vec<usize> = best.into_iter().map(|(_, i)| i).collect();
    picked.sort_unstable();
    picked.dedup();
    picked.into_iter().map(|i| vertices[i]).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sphere(n: usize) -> Vec<[f32; 3]> {
        (0..n)
            .map(|i| {
                let (a, b) = (i as f32 * 0.618_034 * std::f32::consts::TAU, (i as f32 / n as f32 * 2.0 - 1.0).acos());
                [b.sin() * a.cos(), b.sin() * a.sin(), b.cos()]
            })
            .collect()
    }

    #[test]
    fn the_extreme_points_hull_matches_the_whole_mesh_s_within_the_outline_s_width() {
        let vertices = sphere(20_000);
        let points = find(&vertices);
        assert!(points.len() <= DIRECTIONS && points.len() > DIRECTIONS / 2, "{}", points.len());
        // Projected along any view, the hull of the extremes reaches as far
        // as the hull of every vertex, to a percent.
        for (yaw, pitch) in [(0.0f32, 0.0f32), (0.6, 0.3), (2.0, -1.0)] {
            let project = |v: &[f32; 3]| {
                let (sy, cy, sp, cp) = (yaw.sin(), yaw.cos(), pitch.sin(), pitch.cos());
                (v[0] * cy - v[1] * sy, v[0] * sp * sy + v[1] * sp * cy + v[2] * cp)
            };
            let reach = |pts: &[[f32; 3]]| pts.iter().map(project).fold((f32::MIN, f32::MIN), |m, p| (m.0.max(p.0), m.1.max(p.1)));
            let (whole, few) = (reach(&vertices), reach(&points));
            assert!((whole.0 - few.0).abs() < 0.01 && (whole.1 - few.1).abs() < 0.01, "{whole:?} vs {few:?}");
        }
        assert_eq!(find(&vertices[..100]).len(), 100, "a small mesh is used whole");
    }

    #[test]
    fn a_mesh_s_points_are_found_once() {
        let mesh = Arc::new(SceneMesh { vertices: sphere(1000), indices: Vec::new(), normals: crate::phiphx::scene::SceneNormals::PerFace(Vec::new()), corners: Default::default() });
        let (a, b) = (of(&mesh), of(&mesh));
        assert!(Arc::ptr_eq(&a, &b));
    }
}
