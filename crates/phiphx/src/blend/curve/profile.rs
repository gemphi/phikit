//! A custom bevel profile (`CurveProfile`, `curveprofile.cc`) sampled as
//! the bevel's quarter reads it: the path's handles found
//! (`calculate_path_handles`), the samples shared among its edges — evenly,
//! or the curved edges first, the remainder to the most curved — and each
//! edge's share taken along its Bézier (`create_samples`).
//!
//! Not done: even-length sampling (`PROF_SAMPLE_EVEN_LENGTHS`), which
//! falls back to the above.

use crate::blend::text::bevlist::forward_diff;
use glam::Vec2;

/// Handle types (`HD_*`).
const HD_FREE: u8 = 0;
const HD_AUTO: u8 = 1;
const HD_VECT: u8 = 2;

/// `PROF_SAMPLE_STRAIGHT_EDGES`.
pub const PROF_SAMPLE_STRAIGHT_EDGES: u32 = 1 << 2;

/// `CurveProfilePoint`: position, handle types and handle positions.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct BProfilePoint {
    pub co: Vec2,
    pub h1: u8,
    pub h2: u8,
    pub h1_loc: Vec2,
    pub h2_loc: Vec2,
}

/// `point_calculate_handle`: auto and vector handles from the neighbours
/// (an end mirrors its one neighbour).
fn point_handle(point: &mut BProfilePoint, prev: Option<Vec2>, next: Option<Vec2>) {
    if point.h1 == HD_FREE && point.h2 == HD_FREE {
        return;
    }
    let p = point.co;
    let (prev_loc, next_loc) = match (prev, next) {
        (None, Some(n)) => (2.0 * p - n, n),
        (Some(pr), None) => (pr, 2.0 * p - pr),
        (Some(pr), Some(n)) => (pr, n),
        (None, None) => return,
    };
    let (dvec_a, dvec_b) = (p - prev_loc, next_loc - p);
    let nonzero = |l: f32| if l == 0.0 { 1.0 } else { l };
    let (mut len_a, mut len_b) = (nonzero(dvec_a.length()), nonzero(dvec_b.length()));
    if point.h1 == HD_AUTO || point.h2 == HD_AUTO {
        let tvec = Vec2::new(dvec_b.x / len_b + dvec_a.x / len_a, dvec_b.y / len_b + dvec_a.y / len_a);
        let len = tvec.length() * 2.5614;
        if len != 0.0 {
            if point.h1 == HD_AUTO {
                len_a /= len;
                point.h1_loc = p + tvec * -len_a;
            }
            if point.h2 == HD_AUTO {
                len_b /= len;
                point.h2_loc = p + tvec * len_b;
            }
        }
    }
    if point.h1 == HD_VECT {
        point.h1_loc = p + dvec_a * (-1.0 / 3.0);
    }
    if point.h2 == HD_VECT {
        point.h2_loc = p + dvec_b * (1.0 / 3.0);
    }
}

/// `is_curved_edge`.
fn curved(path: &[BProfilePoint], i: usize) -> bool {
    path[i].h2 != HD_VECT || path[i + 1].h1 != HD_VECT
}

/// `angle_v2v2` (`normalize_v2`, then `angle_normalized_v2v2`).
fn angle(a: Vec2, b: Vec2) -> f32 {
    let unit = |v: Vec2| match v.dot(v) > 1.0e-35 {
        true => v * (1.0 / v.length()),
        false => Vec2::ZERO,
    };
    let (a, b) = (unit(a), unit(b));
    let half = |d: f32| (d / 2.0).clamp(-1.0, 1.0).asin();
    match a.dot(b) >= 0.0 {
        true => 2.0 * half(a.distance(b)),
        false => std::f32::consts::PI - 2.0 * half(a.distance(-b)),
    }
}

/// `create_samples`: `n_segments` points along the profile's path (its
/// last point left out).
pub fn create_samples(path: &[BProfilePoint], n_segments: usize, straight_edges: bool) -> Vec<Vec2> {
    let total = path.len();
    if total < 2 || n_segments == 0 {
        return Vec::new();
    }
    let mut path = path.to_vec();
    for i in 0..total {
        let prev = (i > 0).then(|| path[i - 1].co);
        let next = (i + 1 < total).then(|| path[i + 1].co);
        point_handle(&mut path[i], prev, next);
    }
    let edges = total - 1;
    // The edges most curved first (`sort_points_curvature`).
    let mut sorted: Vec<(usize, f32)> = (0..edges).map(|i| (i, angle(path[i].h2_loc - path[i].co, path[i + 1].co - path[i + 1].h1_loc))).collect();
    sorted.sort_by(|a, b| b.1.total_cmp(&a.1));
    let mut n_samples = vec![0usize; edges];
    let n_left = match (n_segments >= edges, straight_edges) {
        (false, _) => n_segments,
        (true, true) => {
            let n_common = n_segments / edges;
            n_samples.iter_mut().for_each(|n| *n = n_common);
            n_segments % edges
        }
        (true, false) => {
            let n_curved = match (0..edges).filter(|&i| curved(&path, i)).count() {
                0 => edges,
                c => c,
            };
            let n_left = n_segments - (edges - n_curved);
            let n_common = n_left / n_curved;
            if n_common > 0 {
                for i in 0..edges {
                    match curved(&path, i) || n_curved == edges {
                        true => n_samples[i] += n_common,
                        false => n_samples[i] = 1,
                    }
                }
            }
            n_left - n_common * n_curved
        }
    };
    for &(i, _) in sorted.iter().take(n_left) {
        n_samples[i] += 1;
    }
    let mut out = Vec::with_capacity(n_segments);
    for i in 0..edges {
        let (a, b, n) = (path[i], path[i + 1], n_samples[i]);
        if n == 0 {
            continue;
        }
        let xs = forward_diff(a.co.x, a.h2_loc.x, b.h1_loc.x, b.co.x, n);
        let ys = forward_diff(a.co.y, a.h2_loc.y, b.h1_loc.y, b.co.y, n);
        out.extend(xs.zip(ys).map(|(x, y)| Vec2::new(x, y)));
    }
    out
}
