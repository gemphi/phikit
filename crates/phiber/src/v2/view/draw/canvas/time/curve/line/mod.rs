//! The line of one F-curve — `draw_fcurve_curve_keys`: the keys in view
//! joined as Blender joins them (a constant step's shelf, a linear
//! segment's straight run, a Bézier segment forward-differenced at the
//! resolution its pixel size earns), keys closer than three pixels folded
//! into their centre, and the end values held to the view's edges
//! (`FCURVE_EXTRAPOLATE_CONSTANT`) — or, for a curve with a modifier or
//! under two keys, `draw_fcurve_curve`: the curve evaluated every pixel
//! and a half across the view. Structure: the points. Stroking them is
//! `set.rs`.

pub mod set;

use crate::phiphx::blend::{BFCurve, BKey};

/// `calculate_bezt_draw_resolution`: a quarter point per pixel of extent.
pub const POINTS_PER_PIXEL: f32 = 0.25;
/// `add_bezt_vertices`: past 64 the forward difference drifts off the key.
pub const MAX_RESOLUTION: i32 = 64;
/// `draw_fcurve_curve_keys`: keys nearer than this on screen are folded.
pub const MIN_KEY_DISTANCE: f32 = 3.0;
/// `draw_fcurve_curve`: a sample every pixel and a half.
pub const PIXELS_PER_SAMPLE: f32 = 1.5;
/// `BEZT_IPO_CONST` / `_LIN` / `_BEZ`.
const IPO_CONST: u8 = 0;
const IPO_LIN: u8 = 1;
const IPO_BEZ: u8 = 2;

/// The points of a curve's line, in view units (frame, value), in order.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct BCurveLine {
    pub points: Vec<[f32; 2]>,
}

/// The screen extent of the keys folded so far.
struct BKeyBounds {
    x: (f32, f32),
    y: (f32, f32),
}

impl BKeyBounds {
    fn at(key: &BKey) -> Self {
        Self { x: (key.co[0], key.co[0]), y: (key.co[1], key.co[1]) }
    }

    /// `expand_key_bounds`: a Bézier segment's handles count towards it.
    fn expand(&mut self, left: &BKey, right: &BKey) {
        self.x.1 = right.co[0];
        let mut values = vec![right.co[1]];
        if left.ipo == IPO_BEZ {
            values.extend([right.h1[1], left.h2[1]]);
        }
        for value in values {
            self.y = (self.y.0.min(value), self.y.1.max(value));
        }
    }

    /// `calculate_pixel_distance`: width plus height, in pixels.
    fn pixels(&self, per: [f32; 2]) -> f32 {
        (self.x.1 - self.x.0) * per[0] + (self.y.1 - self.y.0) * per[1]
    }

    fn center(&self) -> [f32; 2] {
        [(self.x.0 + self.x.1) / 2.0, (self.y.0 + self.y.1) / 2.0]
    }
}

impl BCurveLine {
    /// `draw_fcurve`: a curve with a modifier, or without two keys to join,
    /// is sampled; the rest are drawn from their keys.
    pub fn for_view(curve: &BFCurve, first: f32, last: f32, per: [f32; 2]) -> Self {
        if curve.generator.is_some() || curve.keys.len() < 2 { Self::sampled(curve, first, last, per[0]) } else { Self::of(curve, first, last, per) }
    }

    /// `draw_fcurve_curve`: the curve evaluated at every sample across the
    /// view (`samplefreq = 1.5 / scale_x`), the far edge included.
    pub fn sampled(curve: &BFCurve, first: f32, last: f32, per_frame: f32) -> Self {
        if per_frame <= 0.0 {
            return Self::default();
        }
        let frequency = PIXELS_PER_SAMPLE / per_frame;
        let total = ((last - first) / frequency).round() as i32;
        if total <= 0 {
            return Self::default();
        }
        let mut points: Vec<[f32; 2]> = (0..total).map(|sample| first + sample as f32 * frequency).map(|x| [x, curve.evaluate(x)]).collect();
        points.push([last, curve.evaluate(last)]);
        Self { points }
    }

    /// The line of `curve` across the view `first ..= last`, with `per`
    /// pixels per frame and per value unit deciding what is worth a point.
    pub fn of(curve: &BFCurve, first: f32, last: f32, per: [f32; 2]) -> Self {
        let keys = &curve.keys;
        let (Some(head), Some(tail)) = (keys.first(), keys.last()) else { return Self::default() };
        let mut points = Vec::new();
        if head.co[0] > first {
            points.push([first, head.co[1]]);
        }
        // `get_bounding_bezt_index_range`: one key either side of the view.
        let from = keys.partition_point(|key| key.co[0] < first).saturating_sub(1);
        let to = keys.partition_point(|key| key.co[0] <= last).min(keys.len() - 1);
        points.push(keys[from].co);
        let mut bounds = BKeyBounds::at(&keys[from]);
        let mut skipped = false;
        for index in from + 1..=to {
            let (prev, key) = (&keys[index - 1], &keys[index]);
            bounds.expand(prev, key);
            let mut distance = bounds.pixels(per);
            if distance >= MIN_KEY_DISTANCE && skipped {
                // The folded keys become one point at their centre, and the
                // measure starts again from the key before this one.
                points.push(bounds.center());
                skipped = false;
                bounds = BKeyBounds::at(prev);
                bounds.expand(prev, key);
                distance = bounds.pixels(per);
            }
            if distance < MIN_KEY_DISTANCE {
                skipped = true;
                continue;
            }
            match prev.ipo {
                IPO_CONST => points.extend([prev.co, [key.co[0], prev.co[1]]]),
                IPO_LIN => points.push(prev.co),
                _ => Self::bezier(prev, key, per, &mut points),
            }
        }
        points.push(keys[to].co);
        if tail.co[0] < last {
            points.push([last, tail.co[1]]);
        }
        Self { points }
    }

    /// `add_bezt_vertices` at `calculate_bezt_draw_resolution`: the segment
    /// sampled `resolution` times from its first key, the last sample short
    /// of the next key, which the caller adds.
    fn bezier(prev: &BKey, key: &BKey, per: [f32; 2], points: &mut Vec<[f32; 2]>) {
        let low = key.co[1].min(key.h2[1]).min(prev.co[1]).min(prev.h1[1]);
        let high = key.co[1].max(key.h2[1]).max(prev.co[1]).max(prev.h1[1]);
        let resolution = ((key.co[0] - prev.co[0]) * per[0] * POINTS_PER_PIXEL) as i32 + ((high - low) * per[1] * POINTS_PER_PIXEL) as i32;
        if resolution < 2 {
            points.push(prev.co);
            return;
        }
        let resolution = resolution.min(MAX_RESOLUTION);
        let (h1, h2) = BKey::corrected_handles(prev, key);
        for step in 0..resolution {
            let t = step as f32 / resolution as f32;
            points.push([
                BKey::cubic(prev.co[0], h1[0], h2[0], key.co[0], t),
                BKey::cubic(prev.co[1], h1[1], h2[1], key.co[1], t),
            ]);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(frame: f32, value: f32, ipo: u8) -> BKey {
        BKey::new([frame - 4.0, value], [frame, value], [frame + 4.0, value], ipo)
    }

    #[test]
    fn the_line_holds_its_ends_and_steps_or_runs_between_keys() {
        let mut curve = BFCurve::new("location", 0, vec![key(10.0, 1.0, IPO_CONST), key(20.0, 3.0, IPO_LIN), key(30.0, 2.0, IPO_LIN)]);
        let line = BCurveLine::of(&curve, 0.0, 40.0, [10.0, 10.0]);
        // Held from the view's edge, a shelf at 1 to frame 20, a straight run
        // 20 → 30, then held to the far edge.
        assert_eq!(line.points, vec![[0.0, 1.0], [10.0, 1.0], [10.0, 1.0], [20.0, 1.0], [20.0, 3.0], [30.0, 2.0], [40.0, 2.0]]);
        curve.keys.clear();
        assert_eq!(BCurveLine::of(&curve, 0.0, 40.0, [10.0, 10.0]), BCurveLine::default());
    }

    #[test]
    fn a_driver_is_sampled_across_the_view_every_pixel_and_a_half() {
        let mut inverted = BFCurve::new("influence", 0, vec![]);
        inverted.generator = Some(vec![1.0, -1.0]);
        // 4 units × 150 px = 600 px → 400 samples, then the far edge.
        let line = BCurveLine::for_view(&inverted, -2.0, 2.0, [150.0, 150.0]);
        assert_eq!(line.points.len(), 401);
        assert_eq!(line.points[0], [-2.0, 3.0]);
        assert_eq!(*line.points.last().unwrap(), [2.0, -1.0]);
        assert!((line.points[200][0]).abs() < 1e-5 && (line.points[200][1] - 1.0).abs() < 1e-5, "y = 1 − x through (0, 1): {:?}", line.points[200]);
        // A keyless driver with no modifier is a flat line at 0.
        let flat = BCurveLine::for_view(&BFCurve::new("influence", 0, vec![]), -2.0, 2.0, [150.0, 150.0]);
        assert!(flat.points.iter().all(|[_, value]| *value == 0.0));
        // Two plain keys still draw from the keys.
        let keyed = BFCurve::new("v", 0, vec![key(0.0, 1.0, IPO_LIN), key(10.0, 2.0, IPO_LIN)]);
        assert_eq!(BCurveLine::for_view(&keyed, 0.0, 10.0, [10.0, 10.0]), BCurveLine::of(&keyed, 0.0, 10.0, [10.0, 10.0]));
        assert_eq!(BCurveLine::sampled(&inverted, -2.0, 2.0, 0.0), BCurveLine::default(), "no scale, no samples");
    }

    #[test]
    fn a_bezier_segment_is_sampled_by_its_pixel_size_and_close_keys_fold() {
        let curve = BFCurve::new("location", 0, vec![key(0.0, 0.0, IPO_BEZ), key(40.0, 8.0, IPO_BEZ)]);
        // 40 frames × 10 px + 8 units × 10 px = 480 px → 120 points, capped at 64.
        let line = BCurveLine::of(&curve, 0.0, 40.0, [10.0, 10.0]);
        // The first key, 64 samples (the first of them on that key again,
        // as Blender's are), then the last key.
        assert_eq!(line.points.len(), 1 + 64 + 1);
        assert_eq!(line.points[0], [0.0, 0.0]);
        assert_eq!(line.points[1], [0.0, 0.0]);
        assert_eq!(*line.points.last().unwrap(), [40.0, 8.0]);
        let mid = line.points[1 + 32];
        assert!((mid[0] - 20.0).abs() < 0.5 && (mid[1] - 4.0).abs() < 0.5, "the halfway sample is near the middle: {mid:?}");
        // At a tenth of a pixel per frame the same segment is one straight
        // run: the first key (twice, as above) to the last.
        let coarse = BCurveLine::of(&curve, 0.0, 40.0, [0.1, 0.1]);
        assert_eq!(coarse.points, vec![[0.0, 0.0], [0.0, 0.0], [40.0, 8.0]]);
        // Keys a pixel apart fold: Blender adds the centre of the span that
        // finally clears three pixels — here 0 .. 30 — then the key before.
        let dense = BFCurve::new("v", 0, vec![key(0.0, 0.0, IPO_LIN), key(1.0, 0.0, IPO_LIN), key(2.0, 0.0, IPO_LIN), key(30.0, 0.0, IPO_LIN)]);
        let folded = BCurveLine::of(&dense, 0.0, 30.0, [1.0, 1.0]);
        assert_eq!(folded.points, vec![[0.0, 0.0], [15.0, 0.0], [2.0, 0.0], [30.0, 0.0]]);
    }
}
