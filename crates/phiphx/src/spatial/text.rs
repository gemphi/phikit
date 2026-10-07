//! Blender's `Add ▸ Text`: a string as a solid mesh.
//!
//! A Text object in Blender is a **font curve** — the glyphs' outlines are
//! the curve, and `Curve.ext1` (Geometry ▸ Extrude) gives it depth. The
//! outlines are baked (`glyph.rs`); what is left is the part that is the
//! same for any outline font:
//!
//! 1. lay the glyphs out along the baseline by their advance widths;
//! 2. turn each glyph's contours into a triangulated face, with the inner
//!    contours cut out as holes — the counter of an `o`, both of a `B`;
//! 3. give the face a front and a back and join them with a wall.
//!
//! The triangulation is ear clipping with the holes **bridged** into the
//! outer contour first: the rightmost point of a hole is joined to the
//! outer edge directly to its right by a pair of coincident edges, which
//! turns a contour-with-holes into one simple polygon that ear clipping
//! can take. It is the standard construction and it is why a letter's
//! counter comes out as a hole rather than as a filled blob.

use super::glyph::{FIRST, GLYPH};

type P2 = (f32, f32);

/// `text` as a mesh: positions, triangle indices and normals, centred on
/// the origin, one em tall and `depth` em deep.
pub fn mesh(text: &str, depth: f32) -> (Vec<(f32, f32, f32)>, Vec<u32>, Vec<(f32, f32, f32)>) {
    let mut verts: Vec<(f32, f32, f32)> = Vec::new();
    let mut idx: Vec<u32> = Vec::new();
    let mut norms: Vec<(f32, f32, f32)> = Vec::new();
    let half = depth * 0.5;
    let mut pen = 0.0f32;
    for ch in text.chars() {
        let Some(g) = glyph_of(ch) else { continue };
        let shape: Vec<Vec<P2>> = g.contour.iter().map(|c| c.iter().map(|&(x, y)| (x + pen, y)).collect()).collect();
        for (poly, tris) in pieces(&shape) {
            // Front cap, +Z, and the back cap wound the other way.
            let base = verts.len() as u32;
            for p in &poly {
                verts.push((p.0, p.1, half));
                norms.push((0.0, 0.0, 1.0));
            }
            let back = verts.len() as u32;
            for p in &poly {
                verts.push((p.0, p.1, -half));
                norms.push((0.0, 0.0, -1.0));
            }
            for t in tris.chunks_exact(3) {
                idx.extend_from_slice(&[base + t[0], base + t[1], base + t[2]]);
                idx.extend_from_slice(&[back + t[0], back + t[2], back + t[1]]);
            }
        }
        // The wall, one quad per contour edge, with its own normal so the
        // side stays crisp against the cap. Over the glyph's own contours,
        // not the bridged polygons — a bridge is a pair of coincident
        // edges and walling it would put a blade through the letter.
        for contour in &shape {
            for k in 0..contour.len() {
                let (a, b) = (contour[k], contour[(k + 1) % contour.len()]);
                let (dx, dy) = (b.0 - a.0, b.1 - a.1);
                let len = (dx * dx + dy * dy).sqrt().max(1e-9);
                let n = (dy / len, -dx / len, 0.0);
                let w = verts.len() as u32;
                for (p, z) in [(a, half), (b, half), (b, -half), (a, -half)] {
                    verts.push((p.0, p.1, z));
                    norms.push(n);
                }
                idx.extend_from_slice(&[w, w + 1, w + 2, w, w + 2, w + 3]);
            }
        }
        pen += g.advance;
    }
    // Centre it: Blender's Text sits with its origin at the start of the
    // baseline, but a primitive in a row wants its middle at the origin.
    let shift = pen * 0.5;
    for v in &mut verts {
        v.0 -= shift;
        v.1 -= 0.32;
    }
    (verts, idx, norms)
}

fn glyph_of(ch: char) -> Option<&'static super::glyph::Glyph> {
    let c = ch as u32;
    (c >= FIRST as u32).then(|| GLYPH.get((c - FIRST as u32) as usize)).flatten()
}

/// Signed area; positive is counter-clockwise.
fn area(poly: &[P2]) -> f32 {
    let mut a = 0.0;
    for k in 0..poly.len() {
        let (p, q) = (poly[k], poly[(k + 1) % poly.len()]);
        a += p.0 * q.1 - q.0 * p.1;
    }
    a * 0.5
}

/// Whether `p` is inside `poly` (even-odd ray casting).
fn contains(poly: &[P2], p: P2) -> bool {
    let mut inside = false;
    for k in 0..poly.len() {
        let (a, b) = (poly[k], poly[(k + 1) % poly.len()]);
        if (a.1 > p.1) != (b.1 > p.1) && p.0 < (b.0 - a.0) * (p.1 - a.1) / (b.1 - a.1 + f32::EPSILON) + a.0 {
            inside = !inside;
        }
    }
    inside
}

/// One glyph's contours as filled pieces: each outer contour, with its own
/// holes bridged in, and that piece's triangles.
///
/// Which contour is an outline and which a hole is **nesting**, not size.
/// Taking the largest as the outline and the rest as holes is right for an
/// `o` and wrong for a `%`, which is three separate pieces — a slash and
/// two rings — and came out with a negative area because two of its three
/// outlines were being cut out of the third. A contour inside an odd
/// number of others is a hole; otherwise it is an outline, and it takes
/// the holes that sit inside it.
fn pieces(shape: &[Vec<P2>]) -> Vec<(Vec<P2>, Vec<u32>)> {
    let depth: Vec<usize> = shape
        .iter()
        .map(|c| shape.iter().filter(|o| !std::ptr::eq(o.as_slice(), c.as_slice()) && contains(o, c[0])).count())
        .collect();
    let mut out = Vec::new();
    for (k, contour) in shape.iter().enumerate() {
        if depth[k] % 2 != 0 {
            continue;
        }
        let mut poly = contour.clone();
        if area(&poly) < 0.0 {
            poly.reverse();
        }
        // Its own holes: the odd-depth contours inside it, rightmost
        // first, so a bridge never crosses a hole still to come.
        let mut holes: Vec<Vec<P2>> = shape
            .iter()
            .enumerate()
            .filter(|(j, h)| depth[*j] % 2 == 1 && contains(contour, h[0]))
            .map(|(_, h)| h.clone())
            .collect();
        holes.sort_by(|a, b| rightmost(b).0.total_cmp(&rightmost(a).0));
        for mut hole in holes {
            if area(&hole) > 0.0 {
                hole.reverse();
            }
            bridge(&mut poly, &hole);
        }
        let tris = ear_clip(&poly);
        out.push((poly, tris));
    }
    out
}

fn rightmost(poly: &[P2]) -> P2 {
    poly.iter().copied().fold((f32::MIN, 0.0), |best, p| if p.0 > best.0 { p } else { best })
}

/// Join `hole` into `poly` at the outer vertex nearest to the hole's
/// rightmost point, by a pair of coincident edges.
fn bridge(poly: &mut Vec<P2>, hole: &[P2]) {
    let start = hole.iter().enumerate().max_by(|a, b| a.1.0.total_cmp(&b.1.0)).map(|(i, _)| i).unwrap_or(0);
    let m = hole[start];
    // The outer vertex to the right of `m` and nearest it. Taking the
    // nearest *visible* one is the textbook rule; nearest-to-the-right is
    // the same vertex for a glyph's counter and is a great deal shorter.
    let at = poly
        .iter()
        .enumerate()
        .filter(|(_, p)| p.0 >= m.0)
        .min_by(|a, b| dist2(*a.1, m).total_cmp(&dist2(*b.1, m)))
        .or_else(|| poly.iter().enumerate().min_by(|a, b| dist2(*a.1, m).total_cmp(&dist2(*b.1, m))))
        .map(|(i, _)| i)
        .unwrap_or(0);
    let mut joined: Vec<P2> = poly[..=at].to_vec();
    for k in 0..hole.len() {
        joined.push(hole[(start + k) % hole.len()]);
    }
    joined.push(m);
    joined.push(poly[at]);
    joined.extend_from_slice(&poly[at + 1..]);
    *poly = joined;
}

fn dist2(a: P2, b: P2) -> f32 {
    (a.0 - b.0) * (a.0 - b.0) + (a.1 - b.1) * (a.1 - b.1)
}

fn cross(o: P2, a: P2, b: P2) -> f32 {
    (a.0 - o.0) * (b.1 - o.1) - (a.1 - o.1) * (b.0 - o.0)
}

fn inside(p: P2, a: P2, b: P2, c: P2) -> bool {
    let (d1, d2, d3) = (cross(p, a, b), cross(p, b, c), cross(p, c, a));
    let neg = d1 < 0.0 || d2 < 0.0 || d3 < 0.0;
    let pos = d1 > 0.0 || d2 > 0.0 || d3 > 0.0;
    !(neg && pos)
}

/// Ear clipping over a simple counter-clockwise polygon.
fn ear_clip(poly: &[P2]) -> Vec<u32> {
    let n = poly.len();
    if n < 3 {
        return Vec::new();
    }
    let mut live: Vec<u32> = (0..n as u32).collect();
    let mut out = Vec::with_capacity((n - 2) * 3);
    let mut guard = 0;
    while live.len() > 3 && guard < n * n + 64 {
        guard += 1;
        let mut cut = None;
        for k in 0..live.len() {
            let (ia, ib, ic) = (live[(k + live.len() - 1) % live.len()], live[k], live[(k + 1) % live.len()]);
            let (a, b, c) = (poly[ia as usize], poly[ib as usize], poly[ic as usize]);
            // Convex, and no other live vertex inside it: an ear.
            if cross(a, b, c) <= 0.0 {
                continue;
            }
            if live.iter().any(|&j| j != ia && j != ib && j != ic && inside(poly[j as usize], a, b, c)) {
                continue;
            }
            out.extend_from_slice(&[ia, ib, ic]);
            cut = Some(k);
            break;
        }
        match cut {
            Some(k) => {
                live.remove(k);
            }
            // No ear found: the polygon is not simple (a bridge crossed
            // something). Fan the rest rather than looping for ever — a
            // glyph drawn slightly wrong beats a hang.
            None => break,
        }
    }
    for k in 1..live.len().saturating_sub(1) {
        out.extend_from_slice(&[live[0], live[k], live[k + 1]]);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The baked font is there and says what a font says.
    #[test]
    fn the_font_is_baked() {
        assert_eq!(GLYPH.len(), 95, "ASCII 32..=126");
        let a = glyph_of('A').expect("A");
        assert_eq!(a.contour.len(), 2, "A has a counter");
        assert_eq!(glyph_of('T').expect("T").contour.len(), 1, "T has none");
        assert_eq!(glyph_of('B').expect("B").contour.len(), 3, "B has two");
        assert!(glyph_of(' ').expect("space").contour.is_empty() && glyph_of(' ').unwrap().advance > 0.0);
    }

    /// Ear clipping over a square gives two triangles covering it.
    #[test]
    fn a_square_is_two_triangles() {
        let sq = [(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)];
        let t = ear_clip(&sq);
        assert_eq!(t.len(), 6);
        let total: f32 = t
            .chunks_exact(3)
            .map(|k| cross(sq[k[0] as usize], sq[k[1] as usize], sq[k[2] as usize]).abs() * 0.5)
            .sum();
        assert!((total - 1.0).abs() < 1e-5, "the triangles cover the square: {total}");
    }

    /// A square with a square hole triangulates to the area of the ring,
    /// not of the whole square — which is the only thing that says the
    /// hole was actually cut out.
    #[test]
    fn a_hole_is_cut_out_not_filled_in() {
        let outer = vec![(0.0, 0.0), (3.0, 0.0), (3.0, 3.0), (0.0, 3.0)];
        let hole = vec![(1.0, 1.0), (1.0, 2.0), (2.0, 2.0), (2.0, 1.0)];
        let parts = pieces(&[outer, hole]);
        assert_eq!(parts.len(), 1, "one filled piece");
        let (poly, tris) = &parts[0];
        let total: f32 = tris
            .chunks_exact(3)
            .map(|k| cross(poly[k[0] as usize], poly[k[1] as usize], poly[k[2] as usize]) * 0.5)
            .sum();
        assert!((total - 8.0).abs() < 1e-3, "9 minus the 1 of the hole: {total}");
    }

    /// Blender's default Text object reads "Text". It comes out as a solid
    /// with depth, every normal a unit, and wider than it is tall.
    #[test]
    fn the_default_text_is_a_solid() {
        let (v, i, n) = mesh("Text", 0.2);
        assert!(i.len() >= 300 && i.len() % 3 == 0, "{} indices", i.len());
        assert_eq!(n.len(), v.len());
        for &(nx, ny, nz) in &n {
            assert!(((nx * nx + ny * ny + nz * nz).sqrt() - 1.0).abs() < 1e-3);
        }
        let (mut lo, mut hi) = ([f32::MAX; 3], [f32::MIN; 3]);
        for &(x, y, z) in &v {
            for (k, c) in [x, y, z].into_iter().enumerate() {
                lo[k] = lo[k].min(c);
                hi[k] = hi[k].max(c);
            }
        }
        assert!((hi[2] - lo[2] - 0.2).abs() < 1e-5, "extruded to the depth asked for");
        assert!(hi[0] - lo[0] > hi[1] - lo[1], "four letters are wider than they are tall");
        assert!(lo[0] < 0.0 && hi[0] > 0.0, "centred on the origin");
    }

    /// Every glyph of the font triangulates without hanging and with a
    /// positive area — the one sweep that would catch a bridge that
    /// crosses, in whichever letter it happens to be.
    #[test]
    fn every_glyph_triangulates() {
        for c in 33u8..127 {
            let ch = c as char;
            let g = glyph_of(ch).expect("glyph");
            if g.contour.is_empty() {
                continue;
            }
            let shape: Vec<Vec<P2>> = g.contour.iter().map(|c| c.to_vec()).collect();
            let parts = pieces(&shape);
            assert!(!parts.is_empty(), "{ch:?} has a filled piece");
            let mut total = 0.0;
            for (poly, tris) in &parts {
                assert!(!tris.is_empty(), "{ch:?} triangulates");
                total += tris
                    .chunks_exact(3)
                    .map(|k| cross(poly[k[0] as usize], poly[k[1] as usize], poly[k[2] as usize]) * 0.5)
                    .sum::<f32>();
            }
            assert!(total > 0.0, "{ch:?} has a positive area: {total}");
        }
    }
}
