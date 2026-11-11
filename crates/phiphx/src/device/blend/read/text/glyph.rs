//! A glyph outline as Bézier splines (`blf_glyph_to_curves`,
//! `blf_glyph_curves.cc`): one cyclic `CU_BEZIER` spline per contour,
//! a Bézier point for every on-curve point and for every virtual one
//! between two conic controls, scaled by the font's `metrics.scale`.

use super::font::{BGlyph, BTag};
use super::{TBez, HD_ALIGN, HD_FREE, HD_VECT};
use glam::Vec3;

/// `VChar`: a character's splines and advance (`che->width`), in the
/// font's em space.
#[derive(Clone, Debug, Default)]
pub struct TChar {
    pub nurbs: Vec<Vec<TBez>>,
    pub width: f32,
}

/// `blf_character_to_curves` past the FreeType load.
pub fn char_curves(glyph: &BGlyph, scale: f32) -> TChar {
    TChar { nurbs: glyph.contours.iter().map(|c| contour(c, scale)).collect(), width: glyph.advance as f32 * scale }
}

fn v(x: f32, y: f32) -> Vec3 {
    Vec3::new(x, y, 0.0)
}

/// One contour's Bézier points.
fn contour(pts: &[(f64, f64, BTag)], scale: f32) -> Vec<TBez> {
    let n = pts.len();
    let eps_sq = 0.0001f32 * 0.0001;
    let x = |l: usize| pts[l].0 as f32;
    let y = |l: usize| pts[l].1 as f32;
    let mut out = Vec::new();
    for l in 0..n {
        let next = (l + 1) % n;
        let prev = (l + n - 1) % n;
        // Virtual conic on-curve points.
        if pts[l].2 == BTag::Conic && pts[next].2 == BTag::Conic {
            let dx = (pts[l].0 + pts[next].0) as f32 * scale / 2.0;
            let dy = (pts[l].1 + pts[next].1) as f32 * scale / 2.0;
            out.push(TBez {
                vec: [
                    v((dx + (2.0 * x(l)) * scale) / 3.0, (dy + (2.0 * y(l)) * scale) / 3.0),
                    v(dx, dy),
                    v((dx + (2.0 * x(next)) * scale) / 3.0, (dy + (2.0 * y(next)) * scale) / 3.0),
                ],
                h1: HD_ALIGN,
                h2: HD_ALIGN,
            });
        }
        if pts[l].2 != BTag::On {
            continue;
        }
        // The handles: a cubic control as it is, a conic one two thirds of
        // the way, a straight edge's a third of the way along it.
        let handle = |o: usize| match pts[o].2 {
            BTag::Cubic => (v(x(o) * scale, y(o) * scale), HD_FREE),
            BTag::Conic => (v((x(l) + 2.0 * x(o)) * scale / 3.0, (y(l) + 2.0 * y(o)) * scale / 3.0), HD_FREE),
            BTag::On => (v(x(l) * scale - (x(l) - x(o)) * scale / 3.0, y(l) * scale - (y(l) - y(o)) * scale / 3.0), HD_VECT),
        };
        let ((left, h1), (right, h2)) = (handle(prev), handle(next));
        let mut b = TBez { vec: [left, v(x(l) * scale, y(l) * scale), right], h1, h2 };
        // The handles that are aligned: neither a vector handle, the three
        // points on one line with distance between them, the handles on
        // opposite sides.
        let d2 = |a: Vec3, c: Vec3| a.truncate().distance_squared(c.truncate());
        let [a, m, c] = b.vec;
        if b.h1 != HD_VECT
            && b.h2 != HD_VECT
            && dist_squared_to_line(m, a, c) < 0.001 * 0.001
            && d2(a, m) > eps_sq
            && d2(m, c) > eps_sq
            && d2(a, c) > eps_sq
            && d2(a, c) > d2(a, m).max(d2(m, c))
        {
            b.h1 = HD_ALIGN;
            b.h2 = HD_ALIGN;
        }
        out.push(b);
    }
    out
}

/// `dist_squared_to_line_v2`: from `p` to the infinite line through `l1`, `l2`.
fn dist_squared_to_line(p: Vec3, l1: Vec3, l2: Vec3) -> f32 {
    let (p, l1, l2) = (p.truncate(), l1.truncate(), l2.truncate());
    let u = l2 - l1;
    let len = u.length_squared();
    let closest = match len {
        0.0 => l1,
        _ => l1 + u * ((p - l1).dot(u) / len),
    };
    p.distance_squared(closest)
}
