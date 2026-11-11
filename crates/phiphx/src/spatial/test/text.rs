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
