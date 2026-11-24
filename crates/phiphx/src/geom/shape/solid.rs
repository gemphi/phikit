//! Shapes with flat faces: the box (its edges rounded by a bevel), the
//! plane, and convex solids made as the hull of their corners (pyramid,
//! tetrahedron, octahedron, icosahedron).

use super::part::{ShapeMesh, unit};

/// A box `size` on each edge, its edges rounded to `bevel` in `ring` steps.
pub fn rounded_box(mesh: &mut ShapeMesh, size: f32, bevel: f32, ring: usize) {
    rounded_block(mesh, [size; 3], bevel, ring);
}

/// A box `size` along x, y and z, its edges rounded to `bevel` in `ring`
/// steps. Each face is a grid; every grid point is pulled onto the
/// rounded surface, so the faces stay flat and the edges and corners
/// turn smoothly.
pub fn rounded_block(mesh: &mut ShapeMesh, size: [f32; 3], bevel: f32, ring: usize) {
    let h = size.map(|s| s / 2.0);
    let b = bevel.clamp(0.0, h[0].min(h[1]).min(h[2]));
    let steps = match b > 0.0 {
        true => ring.max(1),
        false => 0,
    };
    let inner = h.map(|h| h - b);
    let line = |axis: usize| {
        let arc = |k: usize| inner[axis] + b * (std::f32::consts::FRAC_PI_2 * k as f32 / steps.max(1) as f32).sin();
        let mut line: Vec<f32> = (0..=steps).rev().map(|k| -arc(k)).collect();
        line.extend((0..=steps).map(arc));
        // The ends exactly ±h, as the neighbouring faces' planes are, so
        // the points on a shared edge are the same point bit for bit.
        let last = line.len() - 1;
        (line[0], line[last]) = (-h[axis], h[axis]);
        line
    };
    let lines = [line(0), line(1), line(2)];
    for axis in 0..3 {
        for sign in [1.0f32, -1.0] {
            let (u, v) = ((axis + 1) % 3, (axis + 2) % 3);
            grid(mesh, axis, sign, h[axis], [&lines[u], &lines[v]], &|p: [f32; 3]| {
                let c = [0, 1, 2].map(|i| p[i].clamp(-inner[i], inner[i]));
                let d = [p[0] - c[0], p[1] - c[1], p[2] - c[2]];
                let mut flat = [0.0; 3];
                flat[axis] = sign;
                let n = match d.iter().any(|v| *v != 0.0) && b > 0.0 {
                    true => unit(d),
                    false => flat,
                };
                ([c[0] + n[0] * b, c[1] + n[1] * b, c[2] + n[2] * b], n)
            });
        }
    }
}

/// A flat square `size` across, facing up.
pub fn plane(mesh: &mut ShapeMesh, size: f32) {
    let h = size / 2.0;
    grid(mesh, 1, 1.0, 0.0, [&[-h, h], &[-h, h]], &|p| (p, [0.0, 1.0, 0.0]));
}

/// One face of a box: the face looking along `sign` × axis `axis` at
/// distance `h`, a grid over `line` on the next two axes (one line each),
/// each point placed by `place` (position, normal). Quads wind
/// counter-clockwise from outside.
fn grid(mesh: &mut ShapeMesh, axis: usize, sign: f32, h: f32, line: [&[f32]; 2], place: &dyn Fn([f32; 3]) -> ([f32; 3], [f32; 3])) {
    let (u, v) = ((axis + 1) % 3, (axis + 2) % 3);
    let at = |i: usize, j: usize| {
        let mut p = [0.0; 3];
        p[axis] = sign * h;
        p[u] = line[0][i];
        p[v] = line[1][j];
        place(p)
    };
    for i in 0..line[0].len() - 1 {
        for j in 0..line[1].len() - 1 {
            if line[0][i] == line[0][i + 1] || line[1][j] == line[1][j + 1] {
                continue;
            }
            let quad = [at(i, j), at(i + 1, j), at(i + 1, j + 1), at(i, j + 1)];
            match sign > 0.0 {
                true => mesh.face(&quad),
                false => mesh.face(&[quad[3], quad[2], quad[1], quad[0]]),
            }
        }
    }
}

/// A square pyramid: base `size` across, apex `height` above it, centred.
pub fn pyramid(size: f32, height: f32) -> Vec<[f32; 3]> {
    let (h, y) = (size / 2.0, height / 2.0);
    let mut corners: Vec<[f32; 3]> = [(-1.0, -1.0), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)]
        .iter()
        .map(|(x, z)| [x * h, -y, z * h])
        .collect();
    corners.push([0.0, y, 0.0]);
    corners
}

/// The regular solid with `corners` corners (4, 6 or 12) on a sphere `size` across.
pub fn regular(corners: usize, size: f32) -> Vec<[f32; 3]> {
    let phi = (1.0 + 5f32.sqrt()) / 2.0;
    let corner: Vec<[f32; 3]> = match corners {
        4 => signs().filter(|s| s[0] * s[1] * s[2] > 0.0).collect(),
        6 => (0..3).flat_map(|a| [1.0f32, -1.0].map(|s| axis_point(a, s))).collect(),
        _ => (0..3)
            .flat_map(|a| signs2().map(move |(s, t)| rotate([0.0, s, t * phi], a)))
            .collect(),
    };
    corner.into_iter().map(|p| unit(p).map(|v| v * size / 2.0)).collect()
}

fn signs() -> impl Iterator<Item = [f32; 3]> {
    (0..8).map(|i| [0, 1, 2].map(|b| match (i >> b) & 1 {
        0 => 1.0,
        _ => -1.0,
    }))
}

fn signs2() -> impl Iterator<Item = (f32, f32)> {
    [(1.0, 1.0), (1.0, -1.0), (-1.0, 1.0), (-1.0, -1.0)].into_iter()
}

fn axis_point(a: usize, s: f32) -> [f32; 3] {
    let mut p = [0.0; 3];
    p[a] = s;
    p
}

/// `p` with its coordinates turned `a` places (x y z → y z x …).
fn rotate(p: [f32; 3], a: usize) -> [f32; 3] {
    [p[(3 - a) % 3], p[(4 - a) % 3], p[(5 - a) % 3]]
}

/// The convex hull of a few `corners` as flat faces: every plane through
/// three corners with all others behind it is a face, its corners in
/// counter-clockwise order seen from outside.
pub fn hull(mesh: &mut ShapeMesh, corners: &[[f32; 3]]) {
    let n = corners.len();
    let scale = corners.iter().flatten().fold(0.0f32, |m, v| m.max(v.abs()));
    let eps = scale * 1e-4;
    let mut done: Vec<Vec<usize>> = Vec::new();
    for i in 0..n {
        for j in i + 1..n {
            for k in j + 1..n {
                let normal = unit(cross(sub(corners[j], corners[i]), sub(corners[k], corners[i])));
                let side = |p: [f32; 3]| dot(normal, sub(p, corners[i]));
                let ahead = (0..n).filter(|&m| side(corners[m]) > eps).count();
                let behind = (0..n).filter(|&m| side(corners[m]) < -eps).count();
                let normal = match (ahead, behind) {
                    (0, _) => normal,
                    (_, 0) => normal.map(|v| -v),
                    _ => continue,
                };
                let mut face: Vec<usize> = (0..n).filter(|&m| side(corners[m]).abs() <= eps).collect();
                if done.contains(&face) {
                    continue;
                }
                done.push(face.clone());
                let centre = face.iter().fold([0.0; 3], |c, &m| add(c, corners[m])).map(|v| v / face.len() as f32);
                let (e1, e2) = (unit(sub(corners[face[0]], centre)), cross(normal, unit(sub(corners[face[0]], centre))));
                face.sort_by(|&a, &b| {
                    let angle = |m: usize| dot(sub(corners[m], centre), e2).atan2(dot(sub(corners[m], centre), e1));
                    angle(a).total_cmp(&angle(b))
                });
                let polygon: Vec<_> = face.iter().map(|&m| (corners[m], normal)).collect();
                mesh.face(&polygon);
            }
        }
    }
}

fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}
fn add(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}
fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}
fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]]
}
