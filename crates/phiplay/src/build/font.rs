//! A stroke face for numbers and capitals: each glyph a few lines and arcs
//! on a box one unit wide and 1.4 tall, drawn with a round pen. The same
//! strokes paint a shirt's number into its texture (`paint`) and build the
//! scoreboard's digits as geometry (`mesh`) — one face, two uses, and no
//! font file to carry into the browser.
//!
//! The shapes are an athletic block — round-ended, even weight, open
//! counters — the plainest numerals a broadcast can read across a court.

use super::mesh::BMesh;
use crate::field::BMark;
use glam::{Vec2, Vec3};

/// A glyph box's height, as a share of its width.
pub const TALL: f32 = 1.4;

const fn line(a: [f32; 2], b: [f32; 2]) -> BMark {
    BMark::Line { a, b }
}
const fn arc(c: [f32; 2], r: f32, from: f32, to: f32) -> BMark {
    BMark::Arc { c, r, from, to }
}

/// The strokes of `ch`, or none for a character the face lacks (a space).
pub fn strokes(ch: char) -> &'static [BMark] {
    match ch {
        '0' => &const { [arc([0.5, 1.0], 0.4, 0.0, 180.0), arc([0.5, 0.4], 0.4, 180.0, 360.0), line([0.1, 0.4], [0.1, 1.0]), line([0.9, 0.4], [0.9, 1.0])] },
        '1' => &const { [line([0.55, 0.0], [0.55, 1.4]), line([0.55, 1.4], [0.25, 1.15]), line([0.3, 0.0], [0.8, 0.0])] },
        '2' => &const { [arc([0.5, 1.02], 0.38, -30.0, 180.0), line([0.83, 0.83], [0.1, 0.0]), line([0.1, 0.0], [0.9, 0.0])] },
        '3' => &const { [arc([0.5, 1.06], 0.34, -90.0, 160.0), arc([0.5, 0.38], 0.38, -160.0, 90.0)] },
        '4' => &const { [line([0.7, 0.0], [0.7, 1.4]), line([0.7, 1.4], [0.08, 0.45]), line([0.08, 0.45], [0.92, 0.45])] },
        '5' => &const { [line([0.85, 1.4], [0.18, 1.4]), line([0.18, 1.4], [0.13, 0.8]), arc([0.5, 0.42], 0.4, -150.0, 140.0)] },
        '6' => &const { [arc([0.5, 0.42], 0.4, 0.0, 360.0), arc([0.95, 0.5], 0.85, 105.0, 172.0)] },
        '7' => &const { [line([0.1, 1.4], [0.9, 1.4]), line([0.9, 1.4], [0.35, 0.0])] },
        '8' => &const { [arc([0.5, 1.07], 0.32, 0.0, 360.0), arc([0.5, 0.38], 0.38, 0.0, 360.0)] },
        '9' => &const { [arc([0.5, 0.98], 0.4, 0.0, 360.0), arc([0.05, 0.9], 0.85, -75.0, -8.0)] },
        'A' => &const { [line([0.05, 0.0], [0.5, 1.4]), line([0.5, 1.4], [0.95, 0.0]), line([0.22, 0.5], [0.78, 0.5])] },
        'B' => &const { [line([0.12, 0.0], [0.12, 1.4]), line([0.12, 1.4], [0.55, 1.4]), arc([0.55, 1.07], 0.33, -90.0, 90.0), line([0.12, 0.74], [0.6, 0.74]), arc([0.6, 0.37], 0.37, -90.0, 90.0), line([0.12, 0.0], [0.6, 0.0])] },
        'C' => &const { [arc([0.52, 1.0], 0.42, 25.0, 180.0), line([0.1, 0.4], [0.1, 1.0]), arc([0.52, 0.4], 0.42, 180.0, 335.0)] },
        'D' => &const { [line([0.12, 0.0], [0.12, 1.4]), line([0.12, 1.4], [0.5, 1.4]), arc([0.5, 1.0], 0.4, 0.0, 90.0), line([0.9, 0.4], [0.9, 1.0]), arc([0.5, 0.4], 0.4, 270.0, 360.0), line([0.12, 0.0], [0.5, 0.0])] },
        'E' => &const { [line([0.88, 1.4], [0.12, 1.4]), line([0.12, 1.4], [0.12, 0.0]), line([0.12, 0.0], [0.88, 0.0]), line([0.12, 0.72], [0.72, 0.72])] },
        'F' => &const { [line([0.88, 1.4], [0.12, 1.4]), line([0.12, 1.4], [0.12, 0.0]), line([0.12, 0.72], [0.72, 0.72])] },
        'G' => &const { [arc([0.52, 1.0], 0.42, 25.0, 180.0), line([0.1, 0.4], [0.1, 1.0]), arc([0.52, 0.4], 0.42, 180.0, 360.0), line([0.94, 0.4], [0.94, 0.62]), line([0.94, 0.62], [0.55, 0.62])] },
        'H' => &const { [line([0.1, 0.0], [0.1, 1.4]), line([0.9, 0.0], [0.9, 1.4]), line([0.1, 0.72], [0.9, 0.72])] },
        'I' => &const { [line([0.5, 0.0], [0.5, 1.4]), line([0.25, 0.0], [0.75, 0.0]), line([0.25, 1.4], [0.75, 1.4])] },
        'J' => &const { [line([0.8, 1.4], [0.8, 0.4]), arc([0.45, 0.4], 0.35, 180.0, 360.0)] },
        'K' => &const { [line([0.12, 0.0], [0.12, 1.4]), line([0.9, 1.4], [0.12, 0.55]), line([0.4, 0.85], [0.92, 0.0])] },
        'L' => &const { [line([0.12, 1.4], [0.12, 0.0]), line([0.12, 0.0], [0.88, 0.0])] },
        'M' => &const { [line([0.06, 0.0], [0.1, 1.4]), line([0.1, 1.4], [0.5, 0.45]), line([0.5, 0.45], [0.9, 1.4]), line([0.9, 1.4], [0.94, 0.0])] },
        'N' => &const { [line([0.1, 0.0], [0.1, 1.4]), line([0.1, 1.4], [0.9, 0.0]), line([0.9, 0.0], [0.9, 1.4])] },
        'O' => &const { [arc([0.5, 1.0], 0.42, 0.0, 180.0), arc([0.5, 0.4], 0.42, 180.0, 360.0), line([0.08, 0.4], [0.08, 1.0]), line([0.92, 0.4], [0.92, 1.0])] },
        'P' => &const { [line([0.12, 0.0], [0.12, 1.4]), line([0.12, 1.4], [0.55, 1.4]), arc([0.55, 1.05], 0.35, -90.0, 90.0), line([0.12, 0.7], [0.55, 0.7])] },
        'Q' => &const { [arc([0.5, 1.0], 0.42, 0.0, 180.0), arc([0.5, 0.4], 0.42, 180.0, 360.0), line([0.08, 0.4], [0.08, 1.0]), line([0.92, 0.4], [0.92, 1.0]), line([0.6, 0.3], [0.95, -0.05])] },
        'R' => &const { [line([0.12, 0.0], [0.12, 1.4]), line([0.12, 1.4], [0.55, 1.4]), arc([0.55, 1.05], 0.35, -90.0, 90.0), line([0.12, 0.7], [0.55, 0.7]), line([0.5, 0.7], [0.9, 0.0])] },
        'S' => &const { [arc([0.5, 1.05], 0.35, 20.0, 270.0), arc([0.5, 0.35], 0.35, -160.0, 90.0)] },
        'T' => &const { [line([0.05, 1.4], [0.95, 1.4]), line([0.5, 1.4], [0.5, 0.0])] },
        'U' => &const { [line([0.1, 1.4], [0.1, 0.4]), arc([0.5, 0.4], 0.4, 180.0, 360.0), line([0.9, 0.4], [0.9, 1.4])] },
        'V' => &const { [line([0.05, 1.4], [0.5, 0.0]), line([0.5, 0.0], [0.95, 1.4])] },
        'W' => &const { [line([0.02, 1.4], [0.25, 0.0]), line([0.25, 0.0], [0.5, 0.9]), line([0.5, 0.9], [0.75, 0.0]), line([0.75, 0.0], [0.98, 1.4])] },
        'X' => &const { [line([0.08, 0.0], [0.92, 1.4]), line([0.08, 1.4], [0.92, 0.0])] },
        'Y' => &const { [line([0.05, 1.4], [0.5, 0.7]), line([0.95, 1.4], [0.5, 0.7]), line([0.5, 0.7], [0.5, 0.0])] },
        'Z' => &const { [line([0.1, 1.4], [0.9, 1.4]), line([0.9, 1.4], [0.1, 0.0]), line([0.1, 0.0], [0.9, 0.0])] },
        _ => &[],
    }
}

/// How far `p` (in glyph units) is from `ch`'s nearest stroke.
pub fn distance(ch: char, p: [f32; 2]) -> f32 {
    strokes(ch).iter().map(|m| m.distance(p)).fold(f32::MAX, f32::min)
}

/// Whether a pen `weight` wide covers `p` in `text`, set left to right
/// with `gap` between glyphs and centred on the origin — `p`, `weight` and
/// `gap` all in units of one glyph's width.
pub fn covers(text: &str, p: [f32; 2], weight: f32, gap: f32) -> bool {
    let n = text.chars().count() as f32;
    let width = n + (n - 1.0).max(0.0) * gap;
    let x0 = -width / 2.0;
    text.chars().enumerate().any(|(i, ch)| {
        let left = x0 + i as f32 * (1.0 + gap);
        distance(ch, [p[0] - left, p[1] + TALL / 2.0]) < weight / 2.0
    })
}

/// `text` as flat geometry in the x–z plane facing −y, `height` metres
/// tall, centred on the origin: each stroke a band of `weight` (a share of
/// the glyph width) with round-ish ends.
pub fn mesh(text: &str, height: f32, weight: f32, gap: f32) -> BMesh {
    let unit = height / TALL;
    let n = text.chars().count() as f32;
    let width = n + (n - 1.0).max(0.0) * gap;
    let mut m = BMesh::default();
    for (i, ch) in text.chars().enumerate() {
        let left = -width / 2.0 + i as f32 * (1.0 + gap);
        let to = |p: Vec2| Vec3::new((p.x + left) * unit, 0.0, (p.y - TALL / 2.0) * unit);
        for stroke in strokes(ch) {
            let path: Vec<Vec2> = match *stroke {
                BMark::Line { a, b } => vec![Vec2::from(a), Vec2::from(b)],
                BMark::Arc { c, r, from, to: end } => {
                    let span = if end - from >= 360.0 { 360.0 } else { (end - from).rem_euclid(360.0) };
                    let steps = (span / 12.0).ceil().max(2.0) as usize;
                    (0..=steps).map(|k| Vec2::from(c) + Vec2::from_angle((from + span * k as f32 / steps as f32).to_radians()) * r).collect()
                }
                _ => Vec::new(),
            };
            for w in path.windows(2) {
                let d = (w[1] - w[0]).normalize_or_zero();
                let side = Vec2::new(-d.y, d.x) * weight / 2.0;
                // Overrun each band by half the pen so joints and ends close.
                let (a, b) = (w[0] - d * weight / 2.0, w[1] + d * weight / 2.0);
                m.quad(to(a - side), to(b - side), to(b + side), to(a + side));
            }
        }
    }
    m
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_digit_and_capital_has_strokes_inside_its_box() {
        for ch in ('0'..='9').chain('A'..='Z') {
            let s = strokes(ch);
            assert!(!s.is_empty(), "{ch}");
            for m in s {
                if let BMark::Line { a, b } = *m {
                    for p in [a, b] {
                        assert!((-0.1..=1.1).contains(&p[0]) && (-0.1..=1.45).contains(&p[1]), "{ch}: {p:?}");
                    }
                }
            }
        }
        // An 8 is inked at its waist and open in its counters.
        assert!(distance('8', [0.5, 0.76]) < 0.05);
        assert!(distance('8', [0.5, 1.07]) > 0.25 && distance('8', [0.5, 0.38]) > 0.3);
        assert!(covers("23", [-0.6, 0.0], 0.18, 0.15) || covers("23", [0.6, 0.0], 0.18, 0.15));
        assert!(mesh("HOME", 0.5, 0.16, 0.2).triangle_count() > 20);
    }
}
