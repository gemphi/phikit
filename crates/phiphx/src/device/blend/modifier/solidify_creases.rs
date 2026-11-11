//! The creases the simple Solidify leaves for a Subdivision after it
//! (`MOD_solidify_extrude.cc`, `solidify_extrude_modifier_do`).
//!
//! The shell is a copy of the whole mesh (`vert_interp.copy` /
//! `edge_interp.copy` into the second half), so every edge and vertex crease
//! the mesh had is on the copy too. With the rim on, the three
//! `crease_*` settings are then written onto the edges it made: `crease_rim`
//! onto each edge joining a rim vertex to its copy, `crease_outer` added to
//! the open edge of the original surface and `crease_inner` to its copy,
//! each sum held to 1 ("without wrapping").

use super::super::BlendEvalMesh;
use std::collections::HashMap;

/// `SolidifyModifierData.crease_inner` / `crease_outer` / `crease_rim`.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct BSolidifyCreases {
    pub inner: f32,
    pub outer: f32,
    pub rim: f32,
}

/// `(a, b)` with `a < b`, as `edge_creases` keys its edges.
fn key(a: u32, b: u32) -> (u32, u32) {
    match a < b {
        true => (a, b),
        false => (b, a),
    }
}

/// `*cr = min(*cr + add, 1)` on edge `(a, b)`, adding the edge when it had
/// no crease (a missing entry is a crease of 0).
fn add(mesh: &mut BlendEvalMesh, at: &mut HashMap<(u32, u32), usize>, a: u32, b: u32, add: f32) {
    let k = key(a, b);
    match at.get(&k) {
        Some(&i) => {
            let tcr = mesh.edge_creases[i].2 + add;
            mesh.edge_creases[i].2 = match tcr > 1.0 {
                true => 1.0,
                false => tcr,
            };
        }
        None => {
            at.insert(k, mesh.edge_creases.len());
            mesh.edge_creases.push((k.0, k.1, add.min(1.0)));
        }
    }
}

/// Creases on the solidified `mesh`, whose first `base` vertices are the
/// original surface and the next `base` its shell. `open` is the original
/// surface's open edges (the rim's), empty without a rim — Blender writes
/// the three rim creases only `if (do_rim)`.
pub(super) fn shell_creases(mesh: &mut BlendEvalMesh, base: u32, open: &[(u32, u32)], creases: BSolidifyCreases) {
    // `edge_interp.copy(0, edges_num, edges_num)`: the shell's edges carry
    // the creases of the edges they copy.
    let copied: Vec<(u32, u32, f32)> = mesh.edge_creases.iter().map(|&(a, b, c)| (a + base, b + base, c)).collect();
    mesh.edge_creases.extend(copied);
    // `vert_interp.copy(0, verts_num, verts_num)`.
    match mesh.vert_creases.len() == base as usize {
        true => mesh.vert_creases.extend_from_within(..),
        false => mesh.vert_creases.clear(),
    }
    if open.is_empty() || creases == BSolidifyCreases::default() {
        return;
    }
    let mut at: HashMap<(u32, u32), usize> = HashMap::new();
    for (i, &(a, b, _)) in mesh.edge_creases.iter().enumerate() {
        at.entry(key(a, b)).or_insert(i);
    }
    if creases.rim != 0.0 {
        // The rim's edges, one per rim vertex (`new_vert_arr`): its crease
        // is set, not added, and no other edge joins a vertex to its copy.
        let mut rim_verts: Vec<u32> = open.iter().flat_map(|&(a, b)| [a, b]).collect();
        rim_verts.sort_unstable();
        rim_verts.dedup();
        for v in rim_verts {
            at.insert((v, v + base), mesh.edge_creases.len());
            mesh.edge_creases.push((v, v + base, creases.rim));
        }
    }
    for &(a, b) in open {
        if creases.outer != 0.0 {
            add(mesh, &mut at, a, b, creases.outer);
        }
        if creases.inner != 0.0 {
            add(mesh, &mut at, a + base, b + base, creases.inner);
        }
    }
}
