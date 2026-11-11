//! Flat mesh adjacency for the subdivision evaluator: edges from a sorted
//! corner list (no hashing), per-corner edge ids and CSR vertex → edge /
//! vertex → corner tables.

use super::BlendEvalMesh;

pub(crate) struct Topology {
    /// Edge endpoints (`a < b`) and how many faces use each edge.
    pub edges: Vec<(u32, u32)>,
    pub edge_nfaces: Vec<u32>,
    /// First two faces of each edge.
    pub edge_faces: Vec<[u32; 2]>,
    /// Edge from corner `c` to the next corner of its face.
    pub corner_edge: Vec<u32>,
    pub face_start: Vec<usize>,
    v_edge_off: Vec<u32>,
    v_edges: Vec<u32>,
    v_corner_off: Vec<u32>,
    /// `(face, corner index within the face)`.
    v_corners: Vec<(u32, u32)>,
}

fn csr(n: usize, pairs: impl Iterator<Item = (u32, (u32, u32))> + Clone) -> (Vec<u32>, Vec<(u32, u32)>) {
    let mut off = vec![0u32; n + 1];
    for (v, _) in pairs.clone() {
        off[v as usize + 1] += 1;
    }
    for i in 0..n {
        off[i + 1] += off[i];
    }
    let mut fill = off.clone();
    let mut out = vec![(0, 0); off[n] as usize];
    for (v, x) in pairs {
        out[fill[v as usize] as usize] = x;
        fill[v as usize] += 1;
    }
    (off, out)
}

impl Topology {
    pub(crate) fn of(m: &BlendEvalMesh) -> Self {
        let n = m.vertices.len();
        let mut face_start = Vec::with_capacity(m.faces.len() + 1);
        let mut keys: Vec<(u32, u32, u32, u32)> = Vec::new(); // (a, b, face, corner)
        let mut corner = 0usize;
        for (fi, f) in m.faces.iter().enumerate() {
            face_start.push(corner);
            for (i, &a) in f.iter().enumerate() {
                let b = f[(i + 1) % f.len()];
                keys.push((a.min(b), a.max(b), fi as u32, corner as u32));
                corner += 1;
            }
        }
        face_start.push(corner);
        keys.sort_unstable();
        let (mut edges, mut edge_nfaces, mut edge_faces) = (Vec::new(), Vec::new(), Vec::new());
        let mut corner_edge = vec![0u32; corner];
        for k in &keys {
            if edges.last() != Some(&(k.0, k.1)) {
                edges.push((k.0, k.1));
                edge_nfaces.push(0);
                edge_faces.push([k.2, k.2]);
            }
            let e = edges.len() - 1;
            if edge_nfaces[e] == 1 {
                edge_faces[e][1] = k.2;
            }
            edge_nfaces[e] += 1;
            corner_edge[k.3 as usize] = e as u32;
        }
        let ends = edges.iter().enumerate().flat_map(|(e, &(a, b))| [(a, (e as u32, 0)), (b, (e as u32, 0))]);
        let (v_edge_off, ve) = csr(n, ends);
        let corners: Vec<(u32, (u32, u32))> = (0..m.faces.len())
            .flat_map(|fi| m.faces.get(fi).iter().enumerate().map(move |(c, &v)| (v, (fi as u32, c as u32))))
            .collect();
        let corners = corners.into_iter();
        let (v_corner_off, v_corners) = csr(n, corners);
        Self {
            edges,
            edge_nfaces,
            edge_faces,
            corner_edge,
            face_start,
            v_edge_off,
            v_edges: ve.into_iter().map(|x| x.0).collect(),
            v_corner_off,
            v_corners,
        }
    }

    pub(crate) fn vert_edges(&self, v: usize) -> &[u32] {
        &self.v_edges[self.v_edge_off[v] as usize..self.v_edge_off[v + 1] as usize]
    }

    pub(crate) fn vert_corners(&self, v: usize) -> &[(u32, u32)] {
        &self.v_corners[self.v_corner_off[v] as usize..self.v_corner_off[v + 1] as usize]
    }

    pub(crate) fn other(&self, e: u32, v: usize) -> u32 {
        let (a, b) = self.edges[e as usize];
        if a as usize == v { b } else { a }
    }

    /// The edge between `a` and `b`, if there is one (`edges` is sorted).
    pub(crate) fn edge_of(&self, a: u32, b: u32) -> Option<usize> {
        self.edges.binary_search(&(a.min(b), a.max(b))).ok()
    }

    pub(crate) fn vert_count(&self) -> usize {
        self.v_edge_off.len() - 1
    }

}
