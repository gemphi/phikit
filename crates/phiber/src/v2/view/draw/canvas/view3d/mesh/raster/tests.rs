//! The rasteriser's job is to let depth decide, not order.

use super::BFrame;
use gpui::Hsla;

/// A triangle with nothing textured on it.
const NO_UVS: [[f32; 2]; 3] = [[0.0; 2]; 3];

/// A full-canvas quad at one depth, as two triangles.
fn sheet(frame: &mut BFrame, z: f32, fill: Hsla) {
    for corners in [
        [(-0.5, -0.5, z), (0.5, -0.5, z), (0.5, 0.5, z)],
        [(-0.5, -0.5, z), (0.5, 0.5, z), (-0.5, 0.5, z)],
    ] {
        frame.triangle(corners, fill);
    }
}

fn pixel(frame: &BFrame, x: usize, y: usize) -> [u8; 4] {
    let at = (y * frame.width + x) * 4;
    frame.color[at..at + 4].try_into().unwrap()
}

fn red() -> Hsla {
    Hsla { h: 0.0, s: 1.0, l: 0.5, a: 1.0 }
}

fn blue() -> Hsla {
    Hsla { h: 0.666, s: 1.0, l: 0.5, a: 1.0 }
}

#[test]
fn the_nearer_surface_wins_whichever_is_drawn_first() {
    // Drawn far-then-near and near-then-far, the result is the same: the
    // near one. A painter's algorithm only gets this right in one order,
    // which is why a garment vanished depending on where its origin sat.
    let mut near_last = BFrame::new(8, 8);
    sheet(&mut near_last, 10.0, red());
    sheet(&mut near_last, 1.0, blue());

    let mut near_first = BFrame::new(8, 8);
    sheet(&mut near_first, 1.0, blue());
    sheet(&mut near_first, 10.0, red());

    assert_eq!(pixel(&near_last, 4, 4), pixel(&near_first, 4, 4));
    assert_eq!(pixel(&near_last, 4, 4)[0], 255, "the near sheet is blue");
}

#[test]
fn two_surfaces_that_cross_each_other_each_own_the_half_they_are_nearer_on() {
    // A sheet at a constant depth and a sheet tilted through it. No ordering
    // of two whole triangles can express this; only a per-pixel test can.
    let mut frame = BFrame::new(16, 16);
    sheet(&mut frame, 5.0, red());
    for corners in [
        [(-0.5, -0.5, 1.0), (0.5, -0.5, 1.0), (0.5, 0.5, 9.0)],
        [(-0.5, -0.5, 1.0), (0.5, 0.5, 9.0), (-0.5, 0.5, 9.0)],
    ] {
        frame.triangle(corners, blue());
    }
    // Near the top the tilted sheet is in front; near the bottom it is behind.
    assert_eq!(pixel(&frame, 8, 1)[0], 255, "tilted sheet is nearer at the top");
    assert_eq!(pixel(&frame, 8, 14)[2], 255, "flat sheet is nearer at the bottom");
}

#[test]
fn shading_varies_across_a_face_whose_corners_face_different_ways() {
    // A flat quad whose corner normals sweep from one side to the other —
    // what a smooth mesh carries. Taking one corner's normal for the whole
    // triangle made every curved surface read faceted; interpolating them
    // gives a gradient across the face.
    let mut frame = BFrame::new(16, 2);
    let lightness = |n: (f32, f32, f32), _uv: [f32; 2]| Hsla {
        h: 0.0,
        s: 0.0,
        l: n.0.abs().clamp(0.05, 0.95),
        a: 1.0,
    };
    // A quad, as two triangles, with the normal sweeping left to right.
    frame.shaded(
        [(-0.5, -0.5, 1.0), (0.5, -0.5, 1.0), (0.5, 0.5, 1.0)],
        [(0.1, 0.0, 0.0), (0.9, 0.0, 0.0), (0.9, 0.0, 0.0)],
        NO_UVS,
        lightness,
    );
    frame.shaded(
        [(-0.5, -0.5, 1.0), (0.5, 0.5, 1.0), (-0.5, 0.5, 1.0)],
        [(0.1, 0.0, 0.0), (0.9, 0.0, 0.0), (0.1, 0.0, 0.0)],
        NO_UVS,
        lightness,
    );
    let left = pixel(&frame, 1, 0)[0];
    let right = pixel(&frame, 14, 0)[0];
    assert!(left < right, "the face shades across: {left} then {right}");
    assert_ne!(left, 0, "both ends are covered");
}

#[test]
fn a_flat_face_shades_evenly() {
    // The same normal at every corner is a flat face, and must stay flat.
    let mut frame = BFrame::new(16, 2);
    let flat = |n: (f32, f32, f32), _uv: [f32; 2]| Hsla {
        h: 0.0,
        s: 0.0,
        l: n.0.abs().clamp(0.05, 0.95),
        a: 1.0,
    };
    for corners in [
        [(-0.5, -0.5, 1.0), (0.5, -0.5, 1.0), (0.5, 0.5, 1.0)],
        [(-0.5, -0.5, 1.0), (0.5, 0.5, 1.0), (-0.5, 0.5, 1.0)],
    ] {
        frame.shaded(corners, [(0.5, 0.0, 0.0); 3], NO_UVS, flat);
    }
    assert_eq!(pixel(&frame, 1, 0), pixel(&frame, 14, 0));
}

#[test]
fn a_textured_face_varies_across_itself() {
    use crate::phiphx::scene::SceneTexture;

    // A 2x1 image, black then white, stretched across a quad. Sampling once
    // at the UV centroid gave the whole face one colour — which is what made
    // a textured model read as flat patches rather than as missing texture.
    let image = SceneTexture {
        width: 2,
        height: 1,
        rgba: std::sync::Arc::new(vec![0, 0, 0, 255, 255, 255, 255, 255]),
    };
    let mut frame = BFrame::new(16, 2);
    let shade = |_n: (f32, f32, f32), uv: [f32; 2]| {
        let rgba = image.sample_smooth(uv);
        Hsla { h: 0.0, s: 0.0, l: rgba[0].clamp(0.0, 1.0), a: 1.0 }
    };
    // The quad's UVs run 0 to 1 left to right.
    frame.shaded(
        [(-0.5, -0.5, 1.0), (0.5, -0.5, 1.0), (0.5, 0.5, 1.0)],
        [(0.0, 0.0, 1.0); 3],
        [[0.0, 1.0], [1.0, 1.0], [1.0, 0.0]],
        shade,
    );
    frame.shaded(
        [(-0.5, -0.5, 1.0), (0.5, 0.5, 1.0), (-0.5, 0.5, 1.0)],
        [(0.0, 0.0, 1.0); 3],
        [[0.0, 1.0], [1.0, 0.0], [0.0, 0.0]],
        shade,
    );
    let left = pixel(&frame, 1, 0)[0];
    let right = pixel(&frame, 14, 0)[0];
    assert!(left < right, "the texture reads across the face: {left} then {right}");
    assert!(right - left > 60, "and by a visible amount");
}

#[test]
fn nothing_drawn_stays_transparent() {
    let frame = BFrame::new(4, 4);
    assert!(frame.color.iter().all(|&b| b == 0));
}

#[test]
fn a_degenerate_triangle_covers_nothing() {
    let mut frame = BFrame::new(4, 4);
    frame.triangle([(0.0, 0.0, 1.0); 3], red());
    assert!(frame.color.iter().all(|&b| b == 0));
}

#[test]
fn downsampling_averages_the_block_it_replaces() {
    // Half the canvas covered at 2x becomes a half-strength edge at 1x, which
    // is the antialiasing the vector paths used to give for free.
    let mut frame = BFrame::new(4, 2);
    for corners in [
        [(-0.5, -0.5, 1.0), (0.0, -0.5, 1.0), (0.0, 0.5, 1.0)],
        [(-0.5, -0.5, 1.0), (0.0, 0.5, 1.0), (-0.5, 0.5, 1.0)],
    ] {
        frame.triangle(corners, red());
    }
    let small = frame.downsample(2);
    assert_eq!((small.width, small.height), (2, 1));
    assert_eq!(pixel(&small, 0, 0)[3], 255, "the covered half is opaque");
    assert_eq!(pixel(&small, 1, 0)[3], 0, "the uncovered half is clear");
}
