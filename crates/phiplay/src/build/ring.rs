//! A closed ring round the field, walked by the share of its length: what
//! the stands, the seats, the shells and their lights are all laid along.
//!
//! A stand's edge is the floor's rectangle grown by its distance from it —
//! straight along the sides, a quarter circle at each corner — so the
//! terraces turn the corners as the demo arena's did. A round venue (the
//! Cap) uses a circle instead.

use glam::Vec2;
use std::f32::consts::{FRAC_PI_2, TAU};

#[derive(Clone, Debug)]
pub struct BRing {
    points: Vec<Vec2>,
    /// Outward, one per point.
    normals: Vec<Vec2>,
    /// Length from the first point to each point; the last is the whole.
    lengths: Vec<f32>,
}

impl BRing {
    /// The rectangle of half-extents `half` grown by `d`: corners are
    /// quarter circles of radius `d`, each cut in `corner` segments.
    pub fn rounded(half: [f32; 2], d: f32, corner: usize) -> BRing {
        let mut points = Vec::new();
        let mut normals = Vec::new();
        // Counter-clockwise from the east side's foot, corner by corner.
        for (q, (cx, cy)) in [(half[0], half[1]), (-half[0], half[1]), (-half[0], -half[1]), (half[0], -half[1])].into_iter().enumerate() {
            let start = q as f32 * FRAC_PI_2;
            for k in 0..=corner {
                let a = start + FRAC_PI_2 * k as f32 / corner as f32;
                let n = Vec2::new(a.cos(), a.sin());
                points.push(Vec2::new(cx, cy) + n * d);
                normals.push(n);
            }
        }
        BRing::from(points, normals)
    }

    /// A circle of radius `r` in `segments`.
    pub fn circle(r: f32, segments: usize) -> BRing {
        let normals: Vec<Vec2> = (0..segments).map(|k| Vec2::from_angle(TAU * k as f32 / segments as f32)).collect();
        BRing::from(normals.iter().map(|n| *n * r).collect(), normals)
    }

    fn from(points: Vec<Vec2>, normals: Vec<Vec2>) -> BRing {
        let mut lengths = vec![0.0];
        for k in 0..points.len() {
            let next = points[(k + 1) % points.len()];
            lengths.push(lengths[k] + points[k].distance(next));
        }
        BRing { points, normals, lengths }
    }

    /// The ring's own points, counter-clockwise. Two rings built with the
    /// same corner count have their points in step, index for index — what
    /// a band between them is stitched along.
    pub fn points(&self) -> &[Vec2] {
        &self.points
    }

    /// The whole length round.
    pub fn length(&self) -> f32 {
        *self.lengths.last().unwrap_or(&0.0)
    }

    /// The point `s` of the way round (0–1, wrapping) and the outward
    /// normal there.
    pub fn at(&self, s: f32) -> (Vec2, Vec2) {
        let target = s.rem_euclid(1.0) * self.length();
        let k = self.lengths.partition_point(|&l| l <= target).saturating_sub(1).min(self.points.len() - 1);
        let span = (self.lengths[k + 1] - self.lengths[k]).max(1e-6);
        let t = (target - self.lengths[k]) / span;
        let j = (k + 1) % self.points.len();
        let p = self.points[k].lerp(self.points[j], t);
        let n = self.normals[k].lerp(self.normals[j], t).normalize_or_zero();
        (p, n)
    }

    /// `count` places evenly round the ring, as (point, outward normal).
    pub fn spaced(&self, count: usize) -> Vec<(Vec2, Vec2)> {
        (0..count).map(|k| self.at((k as f32 + 0.5) / count as f32)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_grown_rectangle_has_its_perimeter_and_faces_out() {
        let ring = BRing::rounded([14.0, 7.5], 2.0, 16);
        let want = 4.0 * (14.0 + 7.5) + TAU * 2.0;
        assert!((ring.length() - want).abs() < 0.05, "{} vs {want}", ring.length());
        for k in 0..100 {
            let (p, n) = ring.at(k as f32 / 100.0);
            assert!(p.dot(n) > 0.0, "outward at {k}");
            // Every point is 2 m off the rectangle.
            let off = Vec2::new((p.x.abs() - 14.0).max(0.0), (p.y.abs() - 7.5).max(0.0)).length();
            assert!((off - 2.0).abs() < 0.02, "{p} is {off} off");
        }
        let circle = BRing::circle(10.0, 256);
        assert!((circle.length() - TAU * 10.0).abs() < 0.01);
    }
}
