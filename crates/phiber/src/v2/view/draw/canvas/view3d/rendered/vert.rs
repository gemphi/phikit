//! Vertex sharing inside one mesh: one phiray vertex per (mesh vertex,
//! corner normal id), so a 20 M-triangle file needs no global hash map.

use std::collections::HashMap;

/// Reused across meshes; `start` clears it for the next one.
#[derive(Default)]
pub struct BRayVerts {
    /// Per mesh vertex: first (normal id, scene id) seen.
    first: Vec<(u32, u32)>,
    /// Further normal ids of the same vertex (split normals).
    more: HashMap<(u32, u32), u32>,
}

const NONE: (u32, u32) = (u32::MAX, u32::MAX);

impl BRayVerts {
    pub fn start(&mut self, count: usize) {
        self.first.clear();
        self.first.resize(count, NONE);
        self.more.clear();
    }

    /// Scene id of (vertex, normal id), made by `make` the first time.
    pub fn id(&mut self, v: u32, n: u32, make: impl FnOnce() -> u32) -> u32 {
        let slot = &mut self.first[v as usize];
        if slot.0 == n {
            return slot.1;
        }
        if *slot == NONE {
            *slot = (n, make());
            return slot.1;
        }
        *self.more.entry((v, n)).or_insert_with(make)
    }
}

#[cfg(test)]
mod verts_tests {
    use super::*;

    #[test]
    fn shares_per_vertex_and_normal() {
        let mut v = BRayVerts::default();
        v.start(2);
        let mut next = 0;
        let mut ids = Vec::new();
        for (a, n) in [(0, 0), (0, 0), (0, 5), (1, 0), (0, 5)] {
            ids.push(v.id(a, n, || { next += 1; next - 1 }));
        }
        assert_eq!(ids, [0, 0, 1, 2, 1]);
    }
}
