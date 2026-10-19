//! A link as a tube: Blender's noodle lifted into the third axis.
//!
//! On the floor plane the curve is exactly the flat editor's — the cubic with
//! Blender's `calculate_inner_link_bezier_points` handles (`paint::handles`),
//! so a link seen from straight above is the link the flat canvas draws. The
//! third axis is the spatial view's own: the tube leaves one node's rim and
//! arrives at the other's, and rises `TUBE_LIFT` of a header above the higher
//! of the two at its middle, so a tube crossing a node it does not touch reads
//! as passing over it rather than through it.
//!
//! It is one of the two routings the proposal left open ("orthogonal with
//! rounded corners, or Blender's curve lifted into 3D — pick when the first
//! version is on screen"). This is the curve, because it is the one with a
//! source; the other is still a job in `jobs.md`.

use super::layout::BTubeEnd;
use super::set::{TUBE_LIFT, TUBE_SEGMENTS};
use super::super::paint::{CURVING, handles};

impl BTubeEnd {
    /// The tube's spine, `TUBE_SEGMENTS + 1` points from `from` to `to`.
    pub fn spine(&self, header: f32) -> Vec<[f32; 3]> {
        let p0 = (self.from[0], self.from[1]);
        let p3 = (self.to[0], self.to[1]);
        let (c1, c2) = handles(p0, p3, CURVING);
        let lift = self.from[2].max(self.to[2]) + header * TUBE_LIFT;
        (0..=TUBE_SEGMENTS)
            .map(|i| {
                let t = i as f32 / TUBE_SEGMENTS as f32;
                let (x, y) = cubic(p0, c1, c2, p3, t);
                // The height eases from one rim to the other and a smooth
                // hump is added over it: zero at both ends, the lift at the
                // middle.
                let along = self.from[2] + (self.to[2] - self.from[2]) * smooth(t);
                let hump = (lift - along) * (4.0 * t * (1.0 - t));
                [x, y, along + hump]
            })
            .collect()
    }
}

/// The cubic bezier at `t`.
fn cubic(p0: (f32, f32), c1: (f32, f32), c2: (f32, f32), p3: (f32, f32), t: f32) -> (f32, f32) {
    let u = 1.0 - t;
    let w = [u * u * u, 3.0 * u * u * t, 3.0 * u * t * t, t * t * t];
    (
        w[0] * p0.0 + w[1] * c1.0 + w[2] * c2.0 + w[3] * p3.0,
        w[0] * p0.1 + w[1] * c1.1 + w[2] * c2.1 + w[3] * p3.1,
    )
}

/// Smoothstep: flat at both ends, so the tube leaves and arrives level.
fn smooth(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_tube_starts_and_ends_on_its_rims_and_rises_between() {
        let end = BTubeEnd { from: [0.0, 0.0, 18.0], to: [300.0, 40.0, 72.0] };
        let spine = end.spine(30.0);
        assert_eq!(spine.len(), TUBE_SEGMENTS + 1);
        assert_eq!(spine[0], end.from);
        let last = spine[TUBE_SEGMENTS];
        assert!((last[0] - 300.0).abs() < 1e-3 && (last[1] - 40.0).abs() < 1e-3 && (last[2] - 72.0).abs() < 1e-3, "{last:?}");
        let peak = spine.iter().map(|p| p[2]).fold(f32::MIN, f32::max);
        assert!(peak > 72.0, "it rises above the higher rim: {peak}");
        assert!((peak - (72.0 + 30.0 * TUBE_LIFT)).abs() < 1.0, "to about the lift: {peak}");
    }

    #[test]
    fn seen_from_above_the_tube_is_the_flat_noodle() {
        // The floor coordinates are the same cubic the flat painter strokes;
        // at t = 0.5 that is the bezier midpoint with Blender's handles.
        let end = BTubeEnd { from: [0.0, 0.0, 0.0], to: [200.0, 100.0, 0.0] };
        let mid = end.spine(30.0)[TUBE_SEGMENTS / 2];
        let (c1, c2) = handles((0.0, 0.0), (200.0, 100.0), CURVING);
        let expect = cubic((0.0, 0.0), c1, c2, (200.0, 100.0), 0.5);
        assert!((mid[0] - expect.0).abs() < 1e-3 && (mid[1] - expect.1).abs() < 1e-3);
    }
}
