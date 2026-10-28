//! `geometry::mesh_merge_verts` (`mesh_merge_verts.cc`, `create_merged_mesh`)
//! with `do_mix_data` off, as `MOD_array.cc` calls it: every vertex the map
//! sends elsewhere is dropped, and its corners land on the target, which
//! keeps its own position and data (`merge_customdata_all`).
//!
//! The faces touched follow the weld's rules. An edge whose two ends land on
//! one vertex collapses and takes its corner with it
//! (`weld_poly_loop_ctx_setup_collapsed_and_split`); a face left with under
//! three corners goes. A face that comes back to a vertex it already passed
//! splits there (`weld_poly_split_recursive`): a two-corner piece vanishes, a
//! bigger one becomes a face of its own, after all the others. Of faces left
//! with the same edges only the first stays (`weld_poly_find_doubles`). The
//! corners that became one mix their UVs and colours (`mix_groups`).

use super::BlendEvalMesh;
use std::collections::HashSet;

/// A face being rebuilt: per corner, its vertex (an index before the merge)
/// and the source corners that became it.
type Ring = Vec<(u32, Vec<usize>)>;

/// Drop the corners of collapsed edges, each into the corner after it.
fn collapse(ring: Ring) -> Option<Ring> {
    let n = ring.len();
    let (mut out, mut carry): (Ring, Vec<usize>) = (Vec::with_capacity(n), Vec::new());
    for k in 0..n {
        let (v, mut src) = ring[k].clone();
        match v == ring[(k + 1) % n].0 {
            true => carry.append(&mut src),
            false => {
                src.append(&mut carry);
                out.push((v, src));
            }
        }
    }
    if out.len() < 3 {
        return None;
    }
    // A run that wraps round ends on the first corner.
    out[0].1.append(&mut carry);
    Some(out)
}

/// `weld_poly_split_recursive`: cut `ring` where a merged vertex repeats.
/// Each piece split off is pushed to `pieces` before its own pieces (`None`
/// when it collapsed, as Blender leaves a collapsed `WeldPoly` in place).
fn split(mut ring: Ring, ctx: &[bool], pieces: &mut Vec<Option<Ring>>) -> Option<Ring> {
    'again: loop {
        let n = ring.len();
        for i in 0..n {
            if !ctx[ring[i].0 as usize] {
                continue;
            }
            let Some(j) = (i + 1..n).find(|&j| ring[j].0 == ring[i].0) else { continue };
            let a: Ring = ring[i..j].to_vec();
            let b: Ring = ring[j..].iter().chain(&ring[..i]).cloned().collect();
            match (j - i, n - (j - i)) {
                (2, 2) => return None,
                (2, _) => ring = b,
                (_, 2) => ring = a,
                _ => {
                    let at = pieces.len();
                    pieces.push(None);
                    let piece = split(a, ctx, pieces);
                    pieces[at] = piece;
                    ring = b;
                }
            }
            continue 'again;
        }
        return Some(ring);
    }
}

/// A face's edges, sorted: what `poly_find_doubles` compares.
fn edges_of(ring: &Ring) -> Vec<(u32, u32)> {
    let n = ring.len();
    let mut e: Vec<(u32, u32)> = (0..n).map(|k| (ring[k].0, ring[(k + 1) % n].0)).map(|(a, b)| (a.min(b), a.max(b))).collect();
    e.sort_unstable();
    e
}

/// The mean of `src`'s values in `data`.
fn mix<const N: usize>(data: &[[f32; N]], src: &[usize]) -> [f32; N] {
    let mut sum = [0.0; N];
    for &c in src {
        for (s, x) in sum.iter_mut().zip(data[c]) {
            *s += x;
        }
    }
    sum.map(|s| s / src.len() as f32)
}

/// Merge per `dest`: `dest[v] = Some(t)` sends `v` onto `t`, which is not
/// itself sent anywhere ("must not contain any chained mapping").
pub(super) fn merge_verts(mesh: &mut BlendEvalMesh, dest: &[Option<u32>]) {
    let n = mesh.vertices.len();
    if dest.len() != n || dest.iter().all(Option::is_none) {
        return;
    }
    // `weld_vert_ctx_alloc_and_setup`: a merged vertex and its target.
    let mut ctx = vec![false; n];
    for (v, d) in dest.iter().enumerate() {
        if let Some(t) = *d {
            ctx[v] = true;
            ctx[t as usize] = true;
        }
    }
    let mut final_map = vec![0u32; n];
    let mut kept = 0;
    for v in (0..n).filter(|&v| dest[v].is_none()) {
        final_map[v] = kept;
        kept += 1;
    }
    for v in 0..n {
        if let Some(t) = dest[v] {
            final_map[v] = final_map[t as usize];
        }
    }
    let alive = |v: &usize| dest[*v].is_none();
    mesh.vertices = (0..n).filter(alive).map(|v| mesh.vertices[v]).collect();
    if mesh.dverts.len() == n {
        mesh.dverts = (0..n).filter(alive).map(|v| std::mem::take(&mut mesh.dverts[v])).collect();
    }
    if mesh.vert_creases.len() == n {
        mesh.vert_creases = (0..n).filter(alive).map(|v| mesh.vert_creases[v]).collect();
    }

    // Faces: each original one rebuilt (or kept as it was), then the pieces.
    let mut rings: Vec<(usize, Option<Ring>)> = Vec::new();
    let mut pieces: Vec<(usize, Option<Ring>)> = Vec::new();
    for f in 0..mesh.faces.len() {
        let (start, face) = (mesh.faces.start(f), mesh.faces.get(f));
        let ring: Ring = face.iter().enumerate().map(|(k, &v)| (v, vec![start + k])).collect();
        if !face.iter().any(|&v| ctx[v as usize]) {
            rings.push((f, Some(ring)));
            continue;
        }
        let ring: Ring = ring.into_iter().map(|(v, src)| (dest[v as usize].unwrap_or(v), src)).collect();
        let mut split_off = Vec::new();
        let main = collapse(ring).and_then(|r| split(r, &ctx, &mut split_off));
        rings.push((f, main));
        pieces.extend(split_off.into_iter().map(|p| (f, p)));
    }
    // `weld_poly_find_doubles`, over the faces the merge touched.
    let mut seen: HashSet<Vec<(u32, u32)>> = HashSet::new();
    for (f, ring) in rings.iter_mut().chain(pieces.iter_mut()) {
        let touched = mesh.faces.get(*f).iter().any(|&v| ctx[v as usize]);
        if let Some(r) = ring.as_ref().filter(|_| touched) {
            if !seen.insert(edges_of(r)) {
                *ring = None;
            }
        }
    }
    let uvs = mesh.loop_uvs.len() >= mesh.faces.corners();
    let colors = mesh.loop_colors.len() >= mesh.faces.corners();
    let mut out = BlendEvalMesh { vertices: std::mem::take(&mut mesh.vertices), ..Default::default() };
    for (f, ring) in rings.into_iter().chain(pieces) {
        let Some(ring) = ring else { continue };
        out.faces.push(&ring.iter().map(|(v, _)| final_map[*v as usize]).collect::<Vec<_>>());
        if uvs {
            out.loop_uvs.extend(ring.iter().map(|(_, src)| mix(&mesh.loop_uvs, src)));
        }
        if colors {
            out.loop_colors.extend(ring.iter().map(|(_, src)| mix(&mesh.loop_colors, src)));
        }
        out.face_materials.extend(mesh.face_materials.get(f).copied());
        out.face_smooth.extend(mesh.face_smooth.get(f).copied());
    }
    mesh.vertices = out.vertices;
    mesh.faces = out.faces;
    mesh.loop_uvs = out.loop_uvs;
    mesh.loop_colors = out.loop_colors;
    mesh.face_materials = out.face_materials;
    mesh.face_smooth = out.face_smooth;
    // Edges: a collapsed one goes, of doubles the first stays with its data.
    let remap = |a: u32, b: u32| {
        let (a, b) = (final_map[a as usize], final_map[b as usize]);
        (a.min(b), a.max(b))
    };
    let mut seen = HashSet::new();
    mesh.edge_creases.retain_mut(|e| {
        (e.0, e.1) = remap(e.0, e.1);
        e.0 != e.1 && seen.insert((e.0, e.1))
    });
    let mut seen = HashSet::new();
    mesh.loose_edges.retain_mut(|e| {
        *e = remap(e.0, e.1);
        e.0 != e.1 && seen.insert(*e)
    });
}
