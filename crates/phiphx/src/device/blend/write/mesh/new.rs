//! A mesh as the writer takes it, and its welded, quad-paired form.
/// A mesh as the writer takes it: vertices and faces as vertex indices.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct BlendNewMesh {
    pub name: String,
    pub vertices: Vec<[f32; 3]>,
    pub faces: Vec<Vec<u32>>,
}

impl BlendNewMesh {
    /// A mesh from a triangle list as a viewport keeps one — vertices
    /// repeated per face, every face a triangle — welded and paired into
    /// quads the way `mesh.tris_convert_to_quads` does: two triangles that
    /// share an edge, lie in one plane and meet convexly become one face.
    /// A cube's twenty-four vertices and twelve triangles come back as
    /// Blender's eight and six.
    pub fn from_triangles(name: &str, vertices: &[[f32; 3]], indices: &[u32]) -> Self {
        // Weld: one vertex per distinct position.
        let mut welded: Vec<[f32; 3]> = Vec::new();
        let mut by_bits: std::collections::HashMap<[u32; 3], u32> = std::collections::HashMap::new();
        let remap: Vec<u32> = vertices
            .iter()
            .map(|v| {
                let key = [v[0].to_bits(), v[1].to_bits(), v[2].to_bits()];
                *by_bits.entry(key).or_insert_with(|| {
                    welded.push(*v);
                    (welded.len() - 1) as u32
                })
            })
            .collect();
        let tris: Vec<[u32; 3]> = indices
            .chunks_exact(3)
            .map(|t| [remap[t[0] as usize], remap[t[1] as usize], remap[t[2] as usize]])
            .filter(|t| t[0] != t[1] && t[1] != t[2] && t[0] != t[2])
            .collect();
        let normal = |t: &[u32; 3]| -> [f32; 3] {
            let (a, b, c) = (welded[t[0] as usize], welded[t[1] as usize], welded[t[2] as usize]);
            let (u, v) = ([b[0] - a[0], b[1] - a[1], b[2] - a[2]], [c[0] - a[0], c[1] - a[1], c[2] - a[2]]);
            let n = [u[1] * v[2] - u[2] * v[1], u[2] * v[0] - u[0] * v[2], u[0] * v[1] - u[1] * v[0]];
            let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt().max(1e-12);
            [n[0] / len, n[1] / len, n[2] / len]
        };
        // Directed edge → the triangle that has it, to find the neighbour
        // across each edge (which has it reversed).
        let mut owner: std::collections::HashMap<(u32, u32), usize> = std::collections::HashMap::new();
        for (i, t) in tris.iter().enumerate() {
            for k in 0..3 {
                owner.insert((t[k], t[(k + 1) % 3]), i);
            }
        }
        let mut used = vec![false; tris.len()];
        let mut faces: Vec<Vec<u32>> = Vec::new();
        for (i, t) in tris.iter().enumerate() {
            if used[i] {
                continue;
            }
            let n = normal(t);
            let mut quad = None;
            for k in 0..3 {
                let (b, c) = (t[k], t[(k + 1) % 3]);
                let a = t[(k + 2) % 3];
                let Some(&j) = owner.get(&(c, b)) else { continue };
                if j == i || used[j] {
                    continue;
                }
                let m = normal(&tris[j]);
                if n[0] * m[0] + n[1] * m[1] + n[2] * m[2] < 0.9999 {
                    continue;
                }
                let q = tris[j].iter().copied().find(|v| *v != b && *v != c).unwrap_or(b);
                if q == a {
                    continue;
                }
                // Convex: `q` lies across the shared edge from `a`.
                let side = |p: u32| {
                    let (pb, pc, pp) = (welded[b as usize], welded[c as usize], welded[p as usize]);
                    let (e, w) = ([pc[0] - pb[0], pc[1] - pb[1], pc[2] - pb[2]], [pp[0] - pb[0], pp[1] - pb[1], pp[2] - pb[2]]);
                    let x = [e[1] * w[2] - e[2] * w[1], e[2] * w[0] - e[0] * w[2], e[0] * w[1] - e[1] * w[0]];
                    x[0] * n[0] + x[1] * n[1] + x[2] * n[2]
                };
                if side(a) * side(q) >= 0.0 {
                    continue;
                }
                used[j] = true;
                quad = Some(vec![a, b, q, c]);
                break;
            }
            used[i] = true;
            faces.push(quad.unwrap_or_else(|| t.to_vec()));
        }
        Self { name: name.to_string(), vertices: welded, faces }
    }
}

/// The edges of `faces`, each once, and per corner the edge it starts.
pub(super) fn edges_of(faces: &[Vec<u32>]) -> (Vec<[u32; 2]>, Vec<u32>) {
    let mut edges: Vec<[u32; 2]> = Vec::new();
    let mut index = std::collections::HashMap::new();
    let mut corner_edges = Vec::new();
    for face in faces {
        for (n, &a) in face.iter().enumerate() {
            let b = face[(n + 1) % face.len()];
            let key = (a.min(b), a.max(b));
            let e = *index.entry(key).or_insert_with(|| {
                edges.push([a, b]);
                (edges.len() - 1) as u32
            });
            corner_edges.push(e);
        }
    }
    (edges, corner_edges)
}

pub(super) fn le_bytes<const N: usize>(le: bool, bytes: [u8; N]) -> [u8; N] {
    if le {
        bytes
    } else {
        let mut b = bytes;
        b.reverse();
        b
    }
}
