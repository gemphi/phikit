//! The nine socket shapes, as outlines.
//!
//! Ported from Blender 5.2's `gpu_shader_2D_node_socket_frag.glsl`, which
//! draws every socket as one distance field in a `switch` over
//! `eNodeSocketDisplayShape`. That shader works in a unit square where the
//! socket's radius is `0.5`; the shapes here are that square's outlines at
//! the same proportions, scaled by the socket radius the card hands them
//! (`socket::DOT × unit`, Blender's `NODE_SOCKSIZE`).
//!
//! phiber drew every socket as a circle, because `BlendSocket` did not read
//! `display_shape` and `dot.rs` resolved a socket to one of two shapes. A
//! Geometry Nodes tree is largely unreadable that way: the diamond is what
//! says a socket takes a FIELD rather than a single value, and the dot
//! variants say the field is also given a value — a distinction Blender
//! makes with shape alone, never with colour.
//!
//! **The radii are Blender's, and they are equal-area, not equal-width.**
//! The shader sets `square_radius = diamond_radius = 0.5 / sqrt(2/π) ×
//! √½ = √π / 4 ≈ 0.4431`, which is the half-side of a square with the same
//! area as a circle of radius 0.5 (`4s² = π/4`). So a square socket is
//! visibly narrower than a circle one and reads as the same weight beside
//! it, which is the whole point of the constant and is lost on anyone who
//! substitutes 0.5.

use crate::phiphx::blend::shade::socket::{
    SOCK_DISPLAY_SHAPE_CIRCLE, SOCK_DISPLAY_SHAPE_CIRCLE_DOT, SOCK_DISPLAY_SHAPE_DIAMOND,
    SOCK_DISPLAY_SHAPE_DIAMOND_DOT, SOCK_DISPLAY_SHAPE_LINE, SOCK_DISPLAY_SHAPE_LIST,
    SOCK_DISPLAY_SHAPE_SQUARE, SOCK_DISPLAY_SHAPE_SQUARE_DOT, SOCK_DISPLAY_SHAPE_VOLUME_GRID,
};

/// `circle_radius` — the shader's, and the one every other radius is sized
/// against.
pub const CIRCLE: f32 = 0.5;

/// `square_radius` / `diamond_radius`: `0.5 / sqrt(2/π) × √½`, the half-side
/// of a square of the circle's own area. Written as the arithmetic rather
/// than as `0.4431` so it can be checked against the shader line.
pub const SQUARE: f32 = 0.443_113_46;

/// `SOCK_DISPLAY_SHAPE_LINE`'s rectangle: `square_radius × (0.75, 1.4)`.
pub const LINE_W: f32 = SQUARE * 0.75;
pub const LINE_H: f32 = SQUARE * 1.4;

/// How many segments a drawn circle is cut into. A socket is a few pixels
/// across, so sixteen is already past what the eye resolves.
const SEGMENTS: usize = 16;

/// The dot the three `_DOT` shapes carry at their centre, as a fraction of
/// the socket radius. The shader's `finalDotRadius` is the outline
/// thickness over the socket's size, which at Blender's own defaults is
/// about this; it is the one number here that is proportioned rather than
/// ported, because the shader derives it from a pixel size this painter
/// does not have.
pub const DOT_FRACTION: f32 = 0.28;

/// One socket's outline, in units of its radius, centred on the origin and
/// counter-clockwise. Multiply by the radius in pixels and offset to the
/// socket's place (`BNodeCard::socket_at`).
pub fn outline(display_shape: i32) -> Vec<(f32, f32)> {
    match display_shape {
        SOCK_DISPLAY_SHAPE_SQUARE | SOCK_DISPLAY_SHAPE_SQUARE_DOT => box_at(0.0, 0.0, SQUARE, SQUARE),
        SOCK_DISPLAY_SHAPE_DIAMOND | SOCK_DISPLAY_SHAPE_DIAMOND_DOT => diamond(),
        SOCK_DISPLAY_SHAPE_LINE => box_at(0.0, 0.0, LINE_W, LINE_H),
        // The two Geometry Nodes shapes are grids of rectangles in the
        // shader (a volume grid is four tiles, a list three stacked bars).
        // A single outline cannot be either, so each is drawn as the
        // rectangle that bounds it and `tiles` supplies the bars inside.
        SOCK_DISPLAY_SHAPE_VOLUME_GRID | SOCK_DISPLAY_SHAPE_LIST => box_at(0.0, 0.0, CIRCLE, CIRCLE),
        SOCK_DISPLAY_SHAPE_CIRCLE | SOCK_DISPLAY_SHAPE_CIRCLE_DOT => circle(CIRCLE),
        // `default:` in the shader is the circle, so a value from a newer
        // Blender is a circle here rather than nothing at all.
        _ => circle(CIRCLE),
    }
}

/// The bars drawn inside `display_shape`, in units of the radius — empty for
/// the seven shapes that have none.
///
/// `SOCK_DISPLAY_SHAPE_VOLUME_GRID` is the shader's mirrored `rect_corner`
/// pattern, which comes out as a 2 × 2 grid of tiles; `..._LIST` is its
/// three stacked bars (`rect_side_length = (0.5, 0.25)`).
pub fn tiles(display_shape: i32) -> Vec<Vec<(f32, f32)>> {
    let q = CIRCLE / 2.0;
    match display_shape {
        SOCK_DISPLAY_SHAPE_VOLUME_GRID => {
            let s = q * 0.62;
            [(-q, -q), (q, -q), (-q, q), (q, q)].iter().map(|&(x, y)| box_at(x, y, s, s)).collect()
        }
        SOCK_DISPLAY_SHAPE_LIST => {
            let h = CIRCLE / 6.0;
            [-2.0 * h, 0.0, 2.0 * h].iter().map(|&y| box_at(0.0, y, CIRCLE * 0.8, h * 0.62)).collect()
        }
        _ => Vec::new(),
    }
}

/// Whether this shape carries the centre dot (the three `_DOT` variants).
/// Blender draws it in the OUTLINE colour, not the socket's, so a dotted
/// socket reads as the same colour with a hole in it.
pub fn dotted(display_shape: i32) -> bool {
    matches!(
        display_shape,
        SOCK_DISPLAY_SHAPE_CIRCLE_DOT | SOCK_DISPLAY_SHAPE_SQUARE_DOT | SOCK_DISPLAY_SHAPE_DIAMOND_DOT
    )
}

/// A circle of `r`, counter-clockwise.
pub fn circle(r: f32) -> Vec<(f32, f32)> {
    (0..SEGMENTS)
        .map(|i| {
            let a = std::f32::consts::TAU * i as f32 / SEGMENTS as f32;
            (r * a.cos(), r * a.sin())
        })
        .collect()
}

/// An axis-aligned rectangle centred on `(cx, cy)`, counter-clockwise.
fn box_at(cx: f32, cy: f32, hw: f32, hh: f32) -> Vec<(f32, f32)> {
    vec![(cx - hw, cy - hh), (cx + hw, cy - hh), (cx + hw, cy + hh), (cx - hw, cy + hh)]
}

/// The square turned 45°, which is what `abs(rotate_45(co))` makes of it:
/// its points sit at `SQUARE × √2` along the axes, so it has the circle's
/// area and is WIDER than the square it came from.
fn diamond() -> Vec<(f32, f32)> {
    let d = SQUARE * std::f32::consts::SQRT_2;
    vec![(d, 0.0), (0.0, d), (-d, 0.0), (0.0, -d)]
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Twice the signed area of a ring; positive when counter-clockwise.
    fn area2(ring: &[(f32, f32)]) -> f32 {
        ring.iter()
            .enumerate()
            .map(|(i, &(x0, y0))| {
                let (x1, y1) = ring[(i + 1) % ring.len()];
                x0 * y1 - x1 * y0
            })
            .sum()
    }

    #[test]
    fn the_square_and_the_diamond_have_the_circles_area() {
        // The shader's radii are equal-area, which is the one thing a reader
        // substituting 0.5 would get wrong. π/4 is the circle's at r = 0.5.
        let want = std::f32::consts::PI / 4.0;
        for shape in [SOCK_DISPLAY_SHAPE_SQUARE, SOCK_DISPLAY_SHAPE_DIAMOND] {
            let got = area2(&outline(shape)) / 2.0;
            assert!((got - want).abs() < 1e-3, "shape {shape}: {got} vs {want}");
        }
        // And the diamond is wider than the square, not narrower.
        let widest = |s| outline(s).iter().fold(0.0_f32, |m, p| m.max(p.0.abs()));
        assert!(widest(SOCK_DISPLAY_SHAPE_DIAMOND) > widest(SOCK_DISPLAY_SHAPE_SQUARE));
    }

    #[test]
    fn every_shape_draws_something_counter_clockwise_and_an_unknown_one_is_a_circle() {
        for shape in 0..=8 {
            let ring = outline(shape);
            assert!(ring.len() >= 3, "shape {shape} has no outline");
            assert!(area2(&ring) > 0.0, "shape {shape} winds the wrong way");
        }
        assert_eq!(outline(99), outline(SOCK_DISPLAY_SHAPE_CIRCLE), "the shader's `default:` is the circle");
        assert_eq!(tiles(SOCK_DISPLAY_SHAPE_VOLUME_GRID).len(), 4);
        assert_eq!(tiles(SOCK_DISPLAY_SHAPE_LIST).len(), 3);
        assert!(tiles(SOCK_DISPLAY_SHAPE_CIRCLE).is_empty());
        assert!(dotted(SOCK_DISPLAY_SHAPE_DIAMOND_DOT) && !dotted(SOCK_DISPLAY_SHAPE_DIAMOND));
    }
}
