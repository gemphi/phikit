//! Filling a group of closed outlines (`BKE_displist_fill`): what
//! Blender's sweep-line scanfill (`BLI_scanfill_calc_ex` with
//! `BLI_SCANFILL_CALC_HOLES`) produces, an even-odd fill of the outlines
//! projected on XY, done here by ear clipping with every hole bridged into
//! the outline around it. The vertices are the outlines' own and the
//! triangle count is the one any triangulation without new points has
//! (`n + 2·holes − 2` per filled region); which diagonals are cut is ours,
//! not scanfill's.

use glam::Vec2;

/// Triangles over `rings` (indices into the rings laid end to end),
/// counter-clockwise seen from +Z.
pub fn triangulate(rings: &[Vec<Vec2>]) -> Vec<[u32; 3]> {
    let mut starts = Vec::with_capacity(rings.len());
    let mut total = 0u32;
    for r in rings {
        starts.push(total);
        total += r.len() as u32;
    }
    let pts: Vec<Vec2> = rings.iter().flatten().copied().collect();
    let ring_idx = |k: usize| -> Vec<u32> { (0..rings[k].len() as u32).map(|i| starts[k] + i).collect() };
    let live: Vec<usize> = (0..rings.len()).filter(|&k| rings[k].len() >= 3).collect();
    // Nesting depth by the first point: even depth fills, odd is a hole.
    let contains = |outer: usize, inner: usize| point_in(&rings[outer], rings[inner][0]);
    let depth: Vec<usize> = live.iter().map(|&k| live.iter().filter(|&&o| o != k && contains(o, k)).count()).collect();
    let mut tris = Vec::new();
    for (a, &k) in live.iter().enumerate() {
        if depth[a] % 2 == 1 {
            continue;
        }
        let mut poly = ring_idx(k);
        orient(&mut poly, &pts, true);
        // The holes directly inside: one level deeper and inside this ring.
        let mut holes: Vec<Vec<u32>> = live
            .iter()
            .enumerate()
            .filter(|&(b, &h)| depth[b] == depth[a] + 1 && contains(k, h))
            .map(|(_, &h)| {
                let mut ring = ring_idx(h);
                orient(&mut ring, &pts, false);
                ring
            })
            .collect();
        holes.sort_by(|p, q| max_x(q, &pts).total_cmp(&max_x(p, &pts)));
        for hole in holes {
            bridge(&mut poly, &hole, &pts);
        }
        ear_clip(poly, &pts, &mut tris);
    }
    tris
}

fn max_x(ring: &[u32], pts: &[Vec2]) -> f32 {
    ring.iter().map(|&i| pts[i as usize].x).fold(f32::MIN, f32::max)
}

fn area2(ring: &[u32], pts: &[Vec2]) -> f32 {
    (0..ring.len()).map(|i| pts[ring[i] as usize].perp_dot(pts[ring[(i + 1) % ring.len()] as usize])).sum()
}

fn orient(ring: &mut [u32], pts: &[Vec2], ccw: bool) {
    if (area2(ring, pts) > 0.0) != ccw {
        ring.reverse();
    }
}

/// Even-odd point in polygon.
fn point_in(ring: &[Vec2], p: Vec2) -> bool {
    let n = ring.len();
    let mut inside = false;
    for i in 0..n {
        let (a, b) = (ring[i], ring[(i + n - 1) % n]);
        if (a.y > p.y) != (b.y > p.y) && p.x < (b.x - a.x) * (p.y - a.y) / (b.y - a.y) + a.x {
            inside = !inside;
        }
    }
    inside
}

fn cross(o: Vec2, a: Vec2, b: Vec2) -> f32 {
    (a - o).perp_dot(b - o)
}

fn in_triangle(p: Vec2, a: Vec2, b: Vec2, c: Vec2) -> bool {
    cross(a, b, p) >= 0.0 && cross(b, c, p) >= 0.0 && cross(c, a, p) >= 0.0
}

/// Join `hole` into `poly` by a two-way cut from the hole's rightmost
/// point to an outline point it sees (Eberly's bridge).
fn bridge(poly: &mut Vec<u32>, hole: &[u32], pts: &[Vec2]) {
    let hm = (0..hole.len()).fold(0, |m, i| if pts[hole[i] as usize].x > pts[hole[m] as usize].x { i } else { m });
    let m = pts[hole[hm] as usize];
    // The nearest edge crossed by the ray to +X, and the point there.
    let n = poly.len();
    let mut best: Option<(f32, usize)> = None;
    for i in 0..n {
        let (a, b) = (pts[poly[i] as usize], pts[poly[(i + 1) % n] as usize]);
        if (a.y > m.y && b.y > m.y) || (a.y < m.y && b.y < m.y) {
            continue;
        }
        let x = match a.y == b.y {
            true => a.x.min(b.x),
            false => a.x + (m.y - a.y) * (b.x - a.x) / (b.y - a.y),
        };
        if x < m.x || best.is_some_and(|(bx, _)| bx <= x) {
            continue;
        }
        // The edge's end farther right is the candidate.
        let j = if a.x > b.x { i } else { (i + 1) % n };
        best = Some((x, j));
    }
    let Some((ix, mut pick)) = best else { return };
    let i_pt = Vec2::new(ix, m.y);
    let p = pts[poly[pick] as usize];
    // A reflex outline point inside (M, I, P) blocks the view: take the
    // one at the smallest angle to the ray.
    let mut angle = f32::MAX;
    for k in 0..n {
        let q = pts[poly[k] as usize];
        if poly[k] == poly[pick] || q == m {
            continue;
        }
        let (prev, next) = (pts[poly[(k + n - 1) % n] as usize], pts[poly[(k + 1) % n] as usize]);
        let reflex = cross(prev, q, next) <= 0.0;
        let tri = match cross(m, i_pt, p) >= 0.0 {
            true => in_triangle(q, m, i_pt, p),
            false => in_triangle(q, m, p, i_pt),
        };
        if reflex && tri {
            let d = q - m;
            let a = d.y.abs().atan2(d.x);
            if a < angle || (a == angle && d.length_squared() < (pts[poly[pick] as usize] - m).length_squared()) {
                angle = a;
                pick = k;
            }
        }
    }
    let mut joined = Vec::with_capacity(n + hole.len() + 2);
    joined.extend_from_slice(&poly[..=pick]);
    joined.extend((0..=hole.len()).map(|s| hole[(hm + s) % hole.len()]));
    joined.extend_from_slice(&poly[pick..]);
    *poly = joined;
}

/// Ear clipping of one counter-clockwise polygon.
fn ear_clip(mut poly: Vec<u32>, pts: &[Vec2], out: &mut Vec<[u32; 3]>) {
    let p = |i: u32| pts[i as usize];
    let mut guard = 0;
    while poly.len() > 3 {
        let n = poly.len();
        let ear = (0..n).find(|&i| {
            let (a, b, c) = (poly[(i + n - 1) % n], poly[i], poly[(i + 1) % n]);
            if cross(p(a), p(b), p(c)) <= 0.0 {
                return false;
            }
            !poly.iter().any(|&q| {
                let qp = p(q);
                qp != p(a) && qp != p(b) && qp != p(c) && in_triangle(qp, p(a), p(b), p(c))
            })
        });
        // A polygon with no clean ear left (collinear or touching points)
        // gives up its flattest corner.
        let i = ear.unwrap_or_else(|| {
            (0..n).max_by(|&x, &y| cross(p(poly[(x + n - 1) % n]), p(poly[x]), p(poly[(x + 1) % n])).total_cmp(&cross(p(poly[(y + n - 1) % n]), p(poly[y]), p(poly[(y + 1) % n])))).unwrap_or(0)
        });
        out.push([poly[(i + n - 1) % n], poly[i], poly[(i + 1) % n]]);
        poly.remove(i);
        guard += 1;
        if guard > 1_000_000 {
            return;
        }
    }
    if poly.len() == 3 {
        out.push([poly[0], poly[1], poly[2]]);
    }
}
