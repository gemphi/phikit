//! The Array's "Merge" (`MOD_ARR_MERGE`, `arrayModifier_doArray` in
//! `MOD_array.cc`): each copy's vertices within `merge_dist` of the copy
//! before it are sent onto it, and with "First and Last" (`MOD_ARR_MERGEFINAL`)
//! the first copy's onto the last; then `geometry::mesh_merge_verts` merges
//! them (`merge_verts.rs`).
//!
//! The offsets phiphx reads are translations, so `offset_has_scale` is false
//! and, as in Blender, only the second copy is searched: every later copy's
//! map is the one before it, moved along by a copy (when the vertex it
//! leads to is still close enough).

use super::BlendEvalMesh;
use super::super::merge_verts::merge_verts;

/// `SortVertsElem`: a vertex and the sum of its coordinates.
struct SortVert {
    vertex: usize,
    sum: f32,
}

fn sorted(p: &[[f32; 3]], start: usize, len: usize) -> Vec<SortVert> {
    let mut out: Vec<SortVert> = (start..start + len).map(|v| SortVert { vertex: v, sum: p[v][0] + p[v][1] + p[v][2] }).collect();
    out.sort_by(|a, b| a.sum.total_cmp(&b.sum));
    out
}

fn len_sq(a: [f32; 3], b: [f32; 3]) -> f32 {
    (0..3).map(|k| (a[k] - b[k]) * (a[k] - b[k])).sum()
}

/// `compare_len_v3v3`.
fn within(a: [f32; 3], b: [f32; 3], dist: f32) -> bool {
    len_sq(a, b) <= dist * dist
}

/// Follow `target`'s own mapping while it stays within `dist` of `from`;
/// `-1` once it strays (the loop both callers in `MOD_array.cc` share).
fn follow(map: &[i32], p: &[[f32; 3]], from: usize, mut target: i32, dist: f32) -> i32 {
    while target != -1 && !matches!(map[target as usize], t if t == -1 || t == target) {
        target = match within(p[from], p[map[target as usize] as usize], dist) {
            true => map[target as usize],
            false => -1,
        };
    }
    target
}

/// `dm_mvert_map_doubles`: for every source vertex not mapped yet, the
/// nearest target vertex within `dist`, both sides sorted by the sum of
/// their coordinates so only a window of `√3 · dist` of it is searched.
fn map_doubles(map: &mut [i32], p: &[[f32; 3]], target: (usize, usize), source: (usize, usize), dist: f32) {
    let dist3 = (3f32.sqrt() + 0.00005) * dist;
    let targets = sorted(p, target.0, target.1);
    let sources = sorted(p, source.0, source.1);
    let mut low = 0;
    let mut completed = false;
    for s in &sources {
        if map[s.vertex] != -1 {
            continue;
        }
        if completed {
            map[s.vertex] = -1;
            continue;
        }
        while low < targets.len() && targets[low].sum < s.sum - dist3 {
            low += 1;
        }
        if low >= targets.len() {
            map[s.vertex] = -1;
            completed = true;
            continue;
        }
        let (mut best, mut best_sq) = (-1, dist * dist);
        for t in targets[low..].iter().take_while(|t| t.sum <= s.sum + dist3) {
            let d = len_sq(p[s.vertex], p[t.vertex]);
            if d <= best_sq {
                best_sq = d;
                best = follow(map, p, s.vertex, t.vertex as i32, dist);
            }
        }
        map[s.vertex] = best;
    }
}

/// Merge `mesh`, `count` copies of `chunk` vertices each, as `MOD_ARR_MERGE`
/// does (`merge_final`: `MOD_ARR_MERGEFINAL`).
pub(super) fn array_merge(mesh: &mut BlendEvalMesh, chunk: usize, count: usize, dist: f32, merge_final: bool) {
    let total = chunk * count;
    if count < 2 || mesh.vertices.len() != total {
        return;
    }
    let p = &mesh.vertices;
    let mut map = vec![-1i32; total];
    for c in 1..count {
        match c >= 2 {
            // "Mapping chunk 3 to chunk 2 is a translation of mapping 2 to 1".
            true => {
                for k in 0..chunk {
                    let (this, prev) = (c * chunk + k, (c - 1) * chunk + k);
                    map[this] = match map[prev] {
                        -1 => -1,
                        t => follow(&map, p, this, t + chunk as i32, dist),
                    };
                }
            }
            false => map_doubles(&mut map, p, ((c - 1) * chunk, chunk), (c * chunk, chunk), dist),
        }
    }
    if merge_final {
        map_doubles(&mut map, p, ((count - 1) * chunk, chunk), (0, chunk), dist);
    }
    // "We have to follow chains of doubles", in place, as Blender does.
    for i in 0..total {
        let mut new_i = map[i];
        if new_i == -1 {
            continue;
        }
        while !matches!(map[new_i as usize], t if t == -1 || t == new_i) {
            new_i = map[new_i as usize];
        }
        map[i] = match i as i32 == new_i {
            true => -1,
            false => new_i,
        };
    }
    let dest: Vec<Option<u32>> = map.iter().map(|&t| (t != -1).then_some(t as u32)).collect();
    merge_verts(mesh, &dest);
}
