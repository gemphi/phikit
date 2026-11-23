//! A piece of the robot or the greenhouse as it is drawn: triangles in
//! the world, the way the surface faces at each of their corners, and
//! one colour. A corner faces the mean of the faces that meet at it, so
//! a curve is shaded round, but where one of them turns away more than
//! the crease angle the edge stays sharp, as a box's does.

use phiecs::comp::Matrix;
use phiusd::Mesh;

/// A triangle's three corners, or three ways its corners face; and the
/// cells its corners fall in.
type Tri = [[f32; 3]; 3];
type Cell3 = [[i32; 3]; 3];

/// Triangles in one colour, and the way each corner faces.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Piece {
    pub colour: [f32; 3],
    pub triangle: Vec<[[f32; 3]; 3]>,
    pub normal: Vec<[[f32; 3]; 3]>,
}

fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn unit(a: [f32; 3]) -> [f32; 3] {
    let l = (a[0] * a[0] + a[1] * a[1] + a[2] * a[2]).sqrt().max(f32::EPSILON);
    a.map(|v| v / l)
}

fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]]
}

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

/// A way turned by a matrix (not moved), kept one long.
fn turned(m: &Matrix, v: [f32; 3]) -> [f32; 3] {
    let (o, p) = (m.apply([0.0; 3]), m.apply(v));
    unit(sub(p, o))
}

/// A mesh placed in the world by a matrix, in a colour, its corners
/// shaded round but for creases sharper than `crease` degrees.
pub fn piece(mesh: &Mesh, world: &Matrix, colour: [f32; 3], crease: f32) -> Piece {
    let n = mesh.point.len();
    let index: Vec<[usize; 3]> = mesh.index.chunks_exact(3).map(|t| [t[0] as usize, t[1] as usize, t[2] as usize]).filter(|t| t.iter().all(|&i| i < n)).collect();
    let point: Vec<[f32; 3]> = mesh.point.iter().map(|p| world.apply(*p)).collect();
    // Corners at the same place share a way, however the mesh split them.
    let key = |p: [f32; 3]| p.map(|v| (v * 1e5).round() as i64);
    let mut sum: std::collections::HashMap<[i64; 3], [f32; 3]> = std::collections::HashMap::new();
    let face: Vec<[f32; 3]> = index.iter().map(|t| cross(sub(point[t[1]], point[t[0]]), sub(point[t[2]], point[t[0]]))).collect();
    for (t, f) in index.iter().zip(&face) {
        for &i in t {
            let s = sum.entry(key(point[i])).or_default();
            (0..3).for_each(|k| s[k] += f[k]);
        }
    }
    let sharp = crease.to_radians().cos();
    let normal = index.iter().zip(&face).map(|(t, f)| {
        let own = unit(*f);
        t.map(|i| {
            let round = unit(sum.get(&key(point[i])).copied().unwrap_or(own));
            match dot(round, own) >= sharp {
                true => round,
                false => own,
            }
        })
    }).collect();
    let triangle = index.iter().map(|t| t.map(|i| point[i])).collect();
    Piece { colour, triangle, normal }
}

/// Pieces moved by a matrix: the robot drawn once where it stood, and
/// carried to where it walks.
pub fn moved(piece: &[Piece], by: &Matrix) -> Vec<Piece> {
    piece.iter().map(|p| Piece {
        colour: p.colour,
        triangle: p.triangle.iter().map(|t| t.map(|v| by.apply(v))).collect(),
        normal: p.normal.iter().map(|t| t.map(|v| turned(by, v))).collect(),
    }).collect()
}

/// The same pieces with fewer triangles, for a picture where they are
/// small: every corner moved to the middle of its cell, `cells` cells
/// across their widest reach, and the triangles that fall into a point
/// or a line, or are drawn already, left out. Seen smaller than a cell a
/// point, nothing shows the difference.
pub fn coarse(piece: &[Piece], cells: f32) -> Vec<Piece> {
    let (mut low, mut high) = ([f32::MAX; 3], [f32::MIN; 3]);
    for v in piece.iter().flat_map(|p| p.triangle.iter().flatten()) {
        (0..3).for_each(|i| (low[i], high[i]) = (low[i].min(v[i]), high[i].max(v[i])));
    }
    let side = (0..3).map(|i| high[i] - low[i]).fold(0.0, f32::max) / cells.max(1.0);
    let cell = |v: [f32; 3]| [0, 1, 2].map(|i| ((v[i] - low[i]) / side.max(f32::EPSILON)).floor() as i32);
    let middle = |c: [i32; 3]| [0, 1, 2].map(|i| low[i] + (c[i] as f32 + 0.5) * side);
    let mut seen = std::collections::HashSet::new();
    piece.iter().map(|p| {
        let kept: Vec<(Cell3, Tri)> = p.triangle.iter().zip(&p.normal).map(|(t, n)| (t.map(cell), *n)).filter(|(c, _)| c[0] != c[1] && c[1] != c[2] && c[0] != c[2]).filter(|(c, _)| {
            let mut key = *c;
            key.sort();
            seen.insert(key)
        }).collect();
        Piece { colour: p.colour, triangle: kept.iter().map(|(c, _)| c.map(middle)).collect(), normal: kept.iter().map(|(_, n)| *n).collect() }
    }).collect()
}
