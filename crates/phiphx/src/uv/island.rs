//! The mesh cut into the pieces that unwrap separately.
//!
//! Blender unwraps **islands**: faces joined across edges that are not
//! seams. A seam is what makes a cut, which is why unwrapping a seamless
//! closed cube gives one island with no boundary at all — and a conformal
//! map needs a boundary, so that case is the one to watch rather than to
//! pretend about.

use super::BUvMesh;

/// One island: the faces in it.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct BUvIsland {
    pub face: Vec<usize>,
}

/// Every edge of `mesh` as (face, corner of its start) pairs, keyed by the
/// vertex pair in order.
fn edges(mesh: BUvMesh<'_>) -> Vec<((u32, u32), usize)> {
    let mut out = Vec::with_capacity(mesh.corner.len());
    for f in 0..mesh.faces() {
        let run = mesh.face(f);
        for k in 0..run.len() {
            let (a, b) = (run[k], run[(k + 1) % run.len()]);
            out.push(((a.min(b), a.max(b)), f));
        }
    }
    out
}

/// Split `mesh` into islands, cutting along `seam` (vertex pairs, either
/// order). Faces are joined only across an edge that two faces share and
/// that is not a seam.
pub fn split(mesh: BUvMesh<'_>, seam: &[(u32, u32)]) -> Vec<BUvIsland> {
    let cut: std::collections::HashSet<(u32, u32)> = seam.iter().map(|&(a, b)| (a.min(b), a.max(b))).collect();
    let mut by_edge: std::collections::HashMap<(u32, u32), Vec<usize>> = std::collections::HashMap::new();
    for (key, f) in edges(mesh) {
        by_edge.entry(key).or_default().push(f);
    }
    // Union-find over the faces.
    let mut up: Vec<usize> = (0..mesh.faces()).collect();
    fn root(up: &mut Vec<usize>, mut k: usize) -> usize {
        while up[k] != k {
            up[k] = up[up[k]];
            k = up[k];
        }
        k
    }
    for (key, faces) in &by_edge {
        if cut.contains(key) {
            continue;
        }
        for w in faces.windows(2) {
            let (a, b) = (root(&mut up, w[0]), root(&mut up, w[1]));
            if a != b {
                up[a] = b;
            }
        }
    }
    let mut group: std::collections::HashMap<usize, Vec<usize>> = std::collections::HashMap::new();
    for f in 0..mesh.faces() {
        let r = root(&mut up, f);
        group.entry(r).or_default().push(f);
    }
    let mut out: Vec<BUvIsland> = group.into_values().map(|mut face| {
        face.sort_unstable();
        BUvIsland { face }
    }).collect();
    out.sort_by_key(|i| i.face.first().copied().unwrap_or(0));
    out
}

/// Whether an island has a boundary — an edge used by one of its faces
/// only. A closed island has none, and a conformal map of it is not
/// defined: there is nothing for the solver to open out.
pub fn open(mesh: BUvMesh<'_>, part: &BUvIsland) -> bool {
    let mut count: std::collections::HashMap<(u32, u32), usize> = std::collections::HashMap::new();
    for &f in &part.face {
        let run = mesh.face(f);
        for k in 0..run.len() {
            let (a, b) = (run[k], run[(k + 1) % run.len()]);
            *count.entry((a.min(b), a.max(b))).or_default() += 1;
        }
    }
    count.values().any(|&n| n == 1)
}

#[cfg(test)]
mod tests {
    use super::super::tests::grid;
    use super::*;

    #[test]
    fn a_grid_with_no_seam_is_one_island() {
        let (pos, corner, starts) = grid(3);
        let mesh = BUvMesh { pos: &pos, corner: &corner, starts: &starts };
        let parts = split(mesh, &[]);
        assert_eq!(parts.len(), 1);
        assert_eq!(parts[0].face.len(), 9);
        assert!(open(mesh, &parts[0]), "a flat grid is all boundary");
    }

    /// A seam down the middle cuts the grid in two — which is the whole
    /// job of a seam, and the thing a reader that ignores them gets wrong
    /// in a way that looks like a solver bug.
    #[test]
    fn a_seam_cuts_the_grid_in_two() {
        let (pos, corner, starts) = grid(2);
        let mesh = BUvMesh { pos: &pos, corner: &corner, starts: &starts };
        // The vertical edges at i = 1: (1,4) and (4,7) on a 3 x 3 vertex grid.
        let parts = split(mesh, &[(1, 4), (4, 7)]);
        assert_eq!(parts.len(), 2, "{parts:?}");
        assert!(parts.iter().all(|p| p.face.len() == 2));
    }
}
