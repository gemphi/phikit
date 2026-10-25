//! A field's markings as data: what a sport paints on its court or pitch,
//! in metres from the centre spot, +x along the length toward the east end.
//!
//! The sports write their markings once, from their rulebooks, as these
//! shapes; the field texture, the goals' placement and the game's own
//! geometry (is the ball in the key? past the three-point line?) all read the
//! same list. A mark drawn in one place and tested in another is how a court
//! ends up painted to one rule and refereed to another.

/// One painted shape. Line widths are the sport's (`BFieldSpec::line`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BMark {
    /// A straight line from `a` to `b`.
    Line { a: [f32; 2], b: [f32; 2] },
    /// An arc about `c` of radius `r`, counter-clockwise from `from` to
    /// `to` (degrees, 0 = +x).
    Arc { c: [f32; 2], r: f32, from: f32, to: f32 },
    /// A spot (a filled disc) — the penalty mark, the centre spot.
    Spot { c: [f32; 2], r: f32 },
    /// An axis-aligned rectangle's outline.
    Frame { min: [f32; 2], max: [f32; 2] },
}

/// A painted area under the lines — a basketball key, a netball third.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BPaint {
    pub min: [f32; 2],
    pub max: [f32; 2],
    /// Which of the theme's colours fills it (`theme::BRole`).
    pub role: crate::theme::BRole,
}

/// The field: its playing area, the run-off round it, the line width, and
/// what is painted.
#[derive(Clone, Copy, Debug)]
pub struct BFieldSpec {
    /// Half the playing area's length (x) and width (y), in metres.
    pub half: [f32; 2],
    /// How far the floor runs past the lines before the boards or seats.
    pub runoff: f32,
    /// Line width, metres.
    pub line: f32,
    pub marks: &'static [BMark],
    pub paint: &'static [BPaint],
}

impl BFieldSpec {
    /// The whole floor's half-extent: the playing area and its run-off.
    pub fn floor_half(&self) -> [f32; 2] {
        [self.half[0] + self.runoff, self.half[1] + self.runoff]
    }

    /// Whether `p` lies on the playing area. The boundary line counts as
    /// in: all five rulebooks agree on that.
    pub fn contains(&self, p: [f32; 2]) -> bool {
        p[0].abs() <= self.half[0] && p[1].abs() <= self.half[1]
    }
}

impl BMark {
    /// The distance from `p` to the mark's painted centre line — what a
    /// rasteriser compares with half the line width.
    pub fn distance(&self, p: [f32; 2]) -> f32 {
        let len = |v: [f32; 2]| (v[0] * v[0] + v[1] * v[1]).sqrt();
        match *self {
            BMark::Line { a, b } => {
                let ab = [b[0] - a[0], b[1] - a[1]];
                let ap = [p[0] - a[0], p[1] - a[1]];
                let t = ((ap[0] * ab[0] + ap[1] * ab[1]) / (ab[0] * ab[0] + ab[1] * ab[1]).max(1e-9)).clamp(0.0, 1.0);
                len([ap[0] - ab[0] * t, ap[1] - ab[1] * t])
            }
            BMark::Arc { c, r, from, to } => {
                let d = [p[0] - c[0], p[1] - c[1]];
                let angle = d[1].atan2(d[0]).to_degrees().rem_euclid(360.0);
                let (from, span) = (from.rem_euclid(360.0), (to - from).rem_euclid(360.0));
                let span = if span == 0.0 && to != from { 360.0 } else { span };
                if (angle - from).rem_euclid(360.0) <= span {
                    (len(d) - r).abs()
                } else {
                    // Past either end: the distance to the nearer end point.
                    let end = |deg: f32| {
                        let a = deg.to_radians();
                        len([p[0] - (c[0] + r * a.cos()), p[1] - (c[1] + r * a.sin())])
                    };
                    end(from).min(end(from + span))
                }
            }
            // Negative inside: a spot is filled, not outlined.
            BMark::Spot { c, r } => len([p[0] - c[0], p[1] - c[1]]) - r,
            BMark::Frame { min, max } => {
                let lines = [
                    BMark::Line { a: min, b: [max[0], min[1]] },
                    BMark::Line { a: [max[0], min[1]], b: max },
                    BMark::Line { a: max, b: [min[0], max[1]] },
                    BMark::Line { a: [min[0], max[1]], b: min },
                ];
                lines.iter().map(|l| l.distance(p)).fold(f32::MAX, f32::min)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_mark_measures_to_its_own_line() {
        let line = BMark::Line { a: [0.0, 0.0], b: [2.0, 0.0] };
        assert!((line.distance([1.0, 0.5]) - 0.5).abs() < 1e-6);
        assert!((line.distance([3.0, 0.0]) - 1.0).abs() < 1e-6, "past the end, to the end");
        let half = BMark::Arc { c: [0.0, 0.0], r: 1.0, from: -90.0, to: 90.0 };
        assert!(half.distance([1.0, 0.0]) < 1e-6, "on the arc");
        assert!((half.distance([-1.0, 0.0]) - 2f32.sqrt()).abs() < 1e-5, "the missing half: nearest end");
        let ring = BMark::Arc { c: [0.0, 0.0], r: 2.0, from: 0.0, to: 360.0 };
        assert!(ring.distance([-2.0, 0.0]) < 1e-6, "a full circle has no missing side");
        assert!(BMark::Spot { c: [0.0, 0.0], r: 0.1 }.distance([0.05, 0.0]) < 0.0, "inside a spot");
    }
}
