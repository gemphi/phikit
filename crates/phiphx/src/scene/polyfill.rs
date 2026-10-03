//! Face triangulation, following Blender's `mesh_calc_tessellation_for_face`:
//! triangles as-is, quads split (0,1,2)+(0,2,3), n-gons ear-clipped in 2D
//! after projecting onto the face's dominant plane
//! (`axis_dominant_v3_to_m3_negate` + `BLI_polyfill_calc`). Returns local
//! corner indices, so concave n-gons no longer produce overlapping fans.

type P2 = [f32; 2];

/// Newell normal of a polygon (robust for non-planar / concave faces).
fn newell(p: &[[f32; 3]]) -> [f32; 3] {
    let mut n = [0.0f32; 3];
    for i in 0..p.len() {
        let (a, b) = (p[i], p[(i + 1) % p.len()]);
        n[0] += (a[1] - b[1]) * (a[2] + b[2]);
        n[1] += (a[2] - b[2]) * (a[0] + b[0]);
        n[2] += (a[0] - b[0]) * (a[1] + b[1]);
    }
    n
}

/// Drop the dominant axis; keep winding counter-clockwise in 2D.
fn project(p: &[[f32; 3]]) -> Vec<P2> {
    let n = newell(p);
    let (ax, ay, az) = (n[0].abs(), n[1].abs(), n[2].abs());
    let (i, j, sign) = if az >= ax && az >= ay {
        (0, 1, n[2])
    } else if ax >= ay {
        (1, 2, n[0])
    } else {
        (2, 0, n[1])
    };
    let flip = if sign < 0.0 { -1.0 } else { 1.0 };
    p.iter().map(|v| [v[i] * flip, v[j]]).collect()
}

fn cross(o: P2, a: P2, b: P2) -> f32 {
    (a[0] - o[0]) * (b[1] - o[1]) - (a[1] - o[1]) * (b[0] - o[0])
}

fn inside(p: P2, a: P2, b: P2, c: P2) -> bool {
    cross(a, b, p) >= 0.0 && cross(b, c, p) >= 0.0 && cross(c, a, p) >= 0.0
}

/// Ear clipping over a CCW 2D polygon. Falls back to a fan if no ear is
/// found (degenerate / self-intersecting input), so every corner is covered.
fn ear_clip(pts: &[P2]) -> Vec<[usize; 3]> {
    let mut ring: Vec<usize> = (0..pts.len()).collect();
    let mut out = Vec::with_capacity(pts.len().saturating_sub(2));
    let mut guard = 0;
    while ring.len() > 3 && guard < pts.len() * pts.len() {
        guard += 1;
        let n = ring.len();
        let ear = (0..n).find(|&k| {
            let (a, b, c) = (ring[(k + n - 1) % n], ring[k], ring[(k + 1) % n]);
            cross(pts[a], pts[b], pts[c]) > 0.0
                && !ring
                    .iter()
                    .filter(|&&r| r != a && r != b && r != c)
                    .any(|&r| inside(pts[r], pts[a], pts[b], pts[c]))
        });
        match ear {
            Some(k) => {
                out.push([ring[(k + n - 1) % n], ring[k], ring[(k + 1) % n]]);
                ring.remove(k);
            }
            None => break,
        }
    }
    for i in 1..ring.len().saturating_sub(1) {
        out.push([ring[0], ring[i], ring[i + 1]]);
    }
    out
}

/// Triangulate one face given its corner positions (Blender corner order).
pub(crate) fn triangulate(positions: &[[f32; 3]]) -> Vec<[usize; 3]> {
    match positions.len() {
        0..=2 => Vec::new(),
        3 => vec![[0, 1, 2]],
        4 => vec![[0, 1, 2], [0, 2, 3]],
        _ => ear_clip(&project(positions)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn area(p: &[[f32; 3]], tris: &[[usize; 3]]) -> f32 {
        tris.iter()
            .map(|t| {
                let (a, b, c) = (p[t[0]], p[t[1]], p[t[2]]);
                0.5 * ((b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0])).abs()
            })
            .sum()
    }

    #[test]
    fn concave_l_shape_is_covered_exactly_once() {
        // L-shaped hexagon in XY, area 3; a fan from corner 0 would overlap.
        let l = [
            [0.0, 0.0, 0.0], [2.0, 0.0, 0.0], [2.0, 1.0, 0.0],
            [1.0, 1.0, 0.0], [1.0, 2.0, 0.0], [0.0, 2.0, 0.0],
        ];
        let tris = triangulate(&l);
        assert_eq!(tris.len(), 4);
        assert!((area(&l, &tris) - 3.0).abs() < 1e-5);
    }

    #[test]
    fn clockwise_and_vertical_faces_still_triangulate() {
        let cw: Vec<[f32; 3]> = [[0.0, 0.0], [0.0, 1.0], [1.0, 1.0], [2.0, 0.5], [1.0, 0.0]]
            .iter()
            .map(|p| [p[0], 0.0, p[1]])
            .collect();
        assert_eq!(triangulate(&cw).len(), 3);
    }

    #[test]
    fn quads_use_blenders_fixed_split() {
        let q = [[0.0; 3], [1.0, 0.0, 0.0], [1.0, 1.0, 0.0], [0.0, 1.0, 0.0]];
        assert_eq!(triangulate(&q), vec![[0, 1, 2], [0, 2, 3]]);
    }
}
