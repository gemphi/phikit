//! The grid of the animation window's `View2D` — `view2d_draw.cc`'s
//! `calculate_grid_step`, `get_divisor`, `calculate_grid_step_fractions`
//! and `get_parallel_lines_draw_steps`, as methods on the view.
//!
//! Structure: the step maths, for the frame axis and the value axis. The
//! lines, and the numbers beside them, are `set.rs`.

pub mod set;

use super::BTimeView;

/// `subframe_range`: the finest step a value axis shows is a hundredth.
const SUBFRAME_RANGE: i32 = 100;

impl BTimeView {
    /// `calculate_grid_step`: the view units between major lines, starting
    /// from `base` and dividing by 2, 3 or 5 (twos preferred — animators
    /// work on twos) while lines stay at least `min_distance` pixels apart,
    /// or doubling until they do. Never under 1.
    pub fn step(base: i32, pixels: f32, span: f32, min_distance: f32) -> i32 {
        if span.abs() < f32::EPSILON {
            return 1;
        }
        let per_unit = pixels / span;
        let mut distance = base.max(1);
        if per_unit * distance as f32 > min_distance {
            while distance > 1 {
                let next = distance / Self::divisor(distance);
                if per_unit * (next as f32) < min_distance {
                    break;
                }
                distance = next;
            }
        } else {
            while per_unit * (distance as f32) < min_distance && distance < (1 << 30) {
                distance *= 2;
            }
        }
        distance
    }

    /// `calculate_grid_step_fractions`: the same, but allowed under 1 by
    /// stepping a view a hundred times larger and scaling back.
    pub fn fraction_step(base: i32, pixels: f32, span: f32, min_distance: f32) -> f32 {
        let distance = Self::step(base, pixels, span, min_distance);
        if distance > 1 {
            return distance as f32;
        }
        Self::step(SUBFRAME_RANGE, pixels, span * SUBFRAME_RANGE as f32, min_distance) as f32 / SUBFRAME_RANGE as f32
    }

    /// `get_divisor`: 2, 3 or 5 — a division that lands on 2 first, then the
    /// first that divides cleanly, else the distance itself (next step is 1).
    pub fn divisor(distance: i32) -> i32 {
        let clean = |d: i32| distance % d == 0;
        if let Some(d) = [2, 3, 5].into_iter().find(|&d| clean(d) && distance / d == 2) {
            return d;
        }
        [2, 3, 5].into_iter().find(|&d| clean(d)).unwrap_or(distance)
    }

    /// The units between major lines across `width` pixels, from the axis's
    /// base — whole frames in the Dope Sheet (`view2d_draw_lines_x_frames`
    /// without fractions), fractions allowed in the Graph and Drivers
    /// editors (`show_fractions`).
    pub fn grid_step(&self, width: f32, min_distance: f32) -> f32 {
        let span = self.last - self.first;
        if self.values.is_some() { Self::fraction_step(self.base(), width, span, min_distance) } else { Self::step(self.base(), width, span, min_distance) as f32 }
    }

    /// The values between major lines down `height` pixels
    /// (`view2d_draw_lines_y(v2d, show_fractions = true, base)`).
    pub fn value_step(&self, height: f32, base: i32, min_distance: f32) -> f32 {
        let (low, high) = self.values.unwrap_or((0.0, 1.0));
        Self::fraction_step(base, height, high - low, min_distance)
    }

    /// `view2d_draw_lines`: the minor lines sit a divisor inside the major
    /// step, and are drawn only past a fifth of the minimum distance.
    pub fn minor_step(major: f32, per_unit: f32, min_distance: f32) -> Option<f32> {
        let whole = if major > 1.0 { major.round() } else { (major * SUBFRAME_RANGE as f32).round() } as i32;
        let minor = major / Self::divisor(whole.max(1)) as f32;
        (per_unit * minor > min_distance / 5.0 && minor < major).then_some(minor)
    }

    /// `get_parallel_lines_draw_steps`: the first line at or after `first`,
    /// and every line after it up to `last`.
    pub fn lines_between(first: f32, last: f32, step: f32) -> impl Iterator<Item = f32> {
        let begin = (first / step).ceil() * step;
        let count = if first < last && begin <= last { ((last - begin) / step).floor().max(0.0) as i32 + 1 } else { 0 };
        (0..count).map(move |i| begin + i as f32 * step)
    }

    /// The frame lines in view.
    pub fn lines(&self, step: f32) -> impl Iterator<Item = f32> {
        Self::lines_between(self.first, self.last, step)
    }

    /// The value lines in view; none without a value axis.
    pub fn value_lines(&self, step: f32) -> impl Iterator<Item = f32> {
        let (low, high) = self.values.unwrap_or((0.0, 0.0));
        Self::lines_between(low, high, step)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn view(start: i32, end: i32, fps: f32) -> BTimeView {
        BTimeView::of_range(start, end, fps)
    }

    #[test]
    fn the_grid_step_divides_the_fps_on_twos_first_and_doubles_when_crowded() {
        // 24 fps over 270 frames in 1400 px: 5.2 px/frame; 24 → 12 (62 px)
        // → 6 would be 31 px, under 45, so 12 stays.
        assert_eq!(view(1, 250, 24.0).grid_step(1400.0, 45.0), 12.0);
        // The same range in 300 px: 1.1 px/frame; 24 → 48 (53 px).
        assert_eq!(view(1, 250, 24.0).grid_step(300.0, 45.0), 48.0);
        assert_eq!(BTimeView::divisor(4), 2, "4 / 2 = 2: twos win");
        assert_eq!(BTimeView::divisor(15), 3);
        assert_eq!(BTimeView::divisor(7), 7, "a prime steps down to 1");
        assert_eq!(BTimeView::step(10, 100.0, 0.0, 45.0), 1, "an empty view steps by one");
        // The Drivers editor: −2 .. 2 in 600 px steps in tens with
        // fractions — 150 px/unit; 1 → 0.5 (75 px) → 0.25 would be 37 px.
        let mut drivers = view(1, 250, 24.0);
        drivers.first = -2.0;
        drivers.last = 2.0;
        drivers.values = Some((-2.0, 2.0));
        drivers.drivers = true;
        assert_eq!(drivers.grid_step(600.0, 45.0), 0.5);
    }

    #[test]
    fn the_value_step_goes_under_one_in_hundredths() {
        // Blender's default Graph view: −9..10 over 400 px is 21 px per
        // unit; base 10 divides by 5 to 2, which is 42 px — under 45 — so
        // 10 stays, and its minors sit every 2.
        let mut v = view(1, 250, 24.0);
        v.values = Some((-9.0, 10.0));
        assert_eq!(v.value_step(400.0, 10, 45.0), 10.0);
        assert_eq!(v.value_lines(10.0).collect::<Vec<_>>(), vec![0.0, 10.0]);
        assert_eq!(BTimeView::minor_step(10.0, 21.0, 45.0), Some(2.0));
        // A range of 0..0.21 over 400 px: 1 is too coarse, so the hundredths
        // view steps 100 → 50 → 25 → 5, and 5 hundredths is 0.05.
        v.values = Some((0.0, 0.21));
        let step = v.value_step(400.0, 10, 45.0);
        assert!((step - 0.05).abs() < 1e-6, "{step}");
        assert_eq!(v.value_lines(step).count(), 5);
        assert_eq!(BTimeView::minor_step(5.0, 21.0, 45.0), Some(1.0), "5 splits by 5 into 21 px minors");
        assert_eq!(BTimeView::minor_step(1.0, 4.0, 45.0), None, "a 2 px minor is under a fifth of 45");
    }

    #[test]
    fn lines_start_on_a_multiple_and_an_inverted_range_has_none() {
        assert_eq!(BTimeView::lines_between(-9.0, 50.0, 24.0).collect::<Vec<_>>(), vec![0.0, 24.0, 48.0]);
        assert_eq!(BTimeView::lines_between(5.0, 1.0, 1.0).count(), 0);
        let v = view(1, 140, 24.0);
        assert_eq!(v.lines(24.0).take(3).collect::<Vec<_>>(), vec![0.0, 24.0, 48.0]);
        assert_eq!(v.value_lines(1.0).count(), 0, "the Dope Sheet has no value axis");
    }
}
