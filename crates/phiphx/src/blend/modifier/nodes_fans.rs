//! The fan of faces round one vertex — the heart of Blender's
//! `normals_calc_corners` (`blenkernel/intern/mesh_normals.cc`).
//!
//! Each corner at a vertex is joined to its neighbours by the two edges of
//! its own face. `calc_connecting_edge_info` sorts those edges into
//! manifold (two corners, opposite windings) and sharp — marked sharp, used
//! by a sharp face, wound the same way as its neighbour, or used by a third
//! face — and `traverse_fan_local_corners` then walks the manifold ones in
//! both directions from a starting corner. Every corner the walk reaches is
//! one smooth fan and takes the fan's normal
//! (`accumulate_fan_normal`): the faces' normals weighted by each corner's
//! angle, or, for a fan of a single corner, that face's normal verbatim —
//! *"due to floating point precision it is required for the output to be
//! the same as previous versions"*.

use super::BlendEvalMesh;
use super::nodes_corners::{V3, dot, safe_acos_approx, sub};
use super::nodes_corners::normalize;
use std::collections::HashSet;

/// One corner of the vertex being fanned (`VertCornerInfo`).
struct Corner {
    face: usize,
    corner: usize,
    vert_prev: u32,
    vert_next: u32,
    edge_prev: usize,
    edge_next: usize,
}

/// `VertEdgeInfo`: what the fan may cross.
#[derive(Clone, Copy)]
enum Edge {
    Uninit,
    One { corner: usize, towards: bool },
    Two(usize, usize),
    Sharp,
}

/// `normals_calc_corners`'s body for one vertex: its corners, the edges
/// that join them, and one fan after another until every corner is done.
#[allow(clippy::too_many_arguments)]
pub(super) fn fan_vertex(
    m: &BlendEvalMesh,
    v: u32,
    faces: &[usize],
    starts: &[usize],
    face_normals: &[V3],
    sharp: &HashSet<(u32, u32)>,
    sharp_face: &dyn Fn(usize) -> bool,
    out: &mut [V3],
) {
    // `collect_corner_info` + `calc_local_edge_indices`: one local edge per
    // neighbouring vertex.
    let mut others: Vec<u32> = Vec::new();
    let mut local = |o: u32| match others.iter().position(|&x| x == o) {
        Some(i) => i,
        None => { others.push(o); others.len() - 1 }
    };
    let infos: Vec<Corner> = faces
        .iter()
        .map(|&f| {
            let face = m.faces.get(f);
            let i = face.iter().position(|&x| x == v).unwrap_or(0);
            let (vert_prev, vert_next) = (face[(i + face.len() - 1) % face.len()], face[(i + 1) % face.len()]);
            Corner { face: f, corner: starts[f] + i, vert_prev, vert_next, edge_prev: local(vert_prev), edge_next: local(vert_next) }
        })
        .collect();
    // `calc_connecting_edge_info`.
    let mut edges = vec![Edge::Uninit; others.len()];
    for (lc, info) in infos.iter().enumerate() {
        if sharp_face(info.face) {
            edges[info.edge_prev] = Edge::Sharp;
            edges[info.edge_next] = Edge::Sharp;
            continue;
        }
        add_corner(&mut edges[info.edge_prev], sharp.contains(&(v.min(info.vert_prev), v.max(info.vert_prev))), lc, true);
        add_corner(&mut edges[info.edge_next], sharp.contains(&(v.min(info.vert_next), v.max(info.vert_next))), lc, false);
    }
    let p = m.vertices[v as usize];
    let dirs: Vec<V3> = others.iter().map(|&o| normalize(sub(m.vertices[o as usize], p))).collect();
    let mut visited = vec![false; infos.len()];
    let mut start = 0;
    while start < infos.len() {
        let fan = traverse_fan(&infos, &edges, start);
        // `accumulate_fan_normal`.
        let n = match fan.as_slice() {
            [one] => face_normals[infos[*one].face],
            _ => {
                let mut n = [0.0f32; 3];
                for &lc in &fan {
                    let info = &infos[lc];
                    let w = safe_acos_approx(dot(dirs[info.edge_prev], dirs[info.edge_next]));
                    for k in 0..3 {
                        n[k] += face_normals[info.face][k] * w;
                    }
                }
                normalize(n)
            }
        };
        for &lc in &fan {
            out[infos[lc].corner] = n;
            visited[lc] = true;
        }
        while start < infos.len() && visited[start] {
            start += 1;
        }
    }
}

/// `add_corner_to_edge`: the first corner claims the edge (unless it is
/// marked sharp), the second makes it manifold if the windings oppose,
/// anything else makes it sharp.
fn add_corner(edge: &mut Edge, marked: bool, lc: usize, towards: bool) {
    *edge = match *edge {
        Edge::Uninit if marked => Edge::Sharp,
        Edge::Uninit => Edge::One { corner: lc, towards },
        Edge::One { corner, towards: t } if t != towards => Edge::Two(corner, lc),
        _ => Edge::Sharp,
    };
}

/// `traverse_fan_local_corners`: round the vertex one way over manifold
/// edges, then (unless the fan closed) the other way from the start.
fn traverse_fan(infos: &[Corner], edges: &[Edge], start: usize) -> Vec<usize> {
    let other = |e: Edge, cur: usize| match e {
        Edge::Two(a, b) if a == cur => Some(b),
        Edge::Two(a, b) if b == cur => Some(a),
        _ => None,
    };
    let mut fan = vec![start];
    let (mut cur, mut cyclic) = (start, false);
    while let Some(next) = other(edges[infos[cur].edge_next], cur) {
        if next == start {
            cyclic = true;
            break;
        }
        if fan.len() > infos.len() {
            break;
        }
        fan.push(next);
        cur = next;
    }
    fan.reverse();
    if cyclic {
        // The smallest corner index first, as Blender orders a closed fan.
        let first = (0..fan.len()).min_by_key(|&i| infos[fan[i]].corner).unwrap_or(0);
        fan.rotate_left(first);
        return fan;
    }
    cur = start;
    while let Some(next) = other(edges[infos[cur].edge_prev], cur) {
        if fan.len() > infos.len() {
            break;
        }
        fan.push(next);
        cur = next;
    }
    fan
}
