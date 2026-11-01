//! The outline of glass, as a distance field: negative inside, zero on the
//! edge. The mask, the lens and the edge light all read it, and two shapes
//! merge by blending their distances.

/// A rounded rectangle in canvas pixels.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GlassShape {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub corner_radius: f32,
}

impl GlassShape {
    pub fn new(x: f32, y: f32, width: f32, height: f32, corner_radius: f32) -> Self {
        Self { x, y, width, height, corner_radius: corner_radius.clamp(0.0, width.min(height) * 0.5) }
    }

    /// Signed distance from a point to this outline.
    pub fn distance(&self, px: f32, py: f32) -> f32 {
        let (hw, hh) = (self.width * 0.5, self.height * 0.5);
        let r = self.corner_radius;
        let qx = (px - self.x - hw).abs() - hw + r;
        let qy = (py - self.y - hh).abs() - hh + r;
        let outside = (qx.max(0.0).powi(2) + qy.max(0.0).powi(2)).sqrt();
        outside + qx.max(qy).min(0.0) - r
    }
}

/// The distance to several shapes at once. Within `spacing` of each other they
/// blend into one outline (a smooth minimum); at zero they stay apart.
pub(super) fn union_distance(shapes: &[GlassShape], spacing: f32, px: f32, py: f32) -> f32 {
    shapes.iter().map(|s| s.distance(px, py)).fold(f32::MAX, |a, b| match spacing > 0.0 {
        true => {
            let h = (spacing - (a - b).abs()).max(0.0) / spacing;
            a.min(b) - h * h * spacing * 0.25
        }
        false => a.min(b),
    })
}

/// A pixel rectangle: `x0..x1`, `y0..y1`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GlassBounds {
    pub x0: usize,
    pub y0: usize,
    pub x1: usize,
    pub y1: usize,
}

impl GlassBounds {
    /// The box around `shapes`, grown by `margin` and cut to `limit`, or
    /// `None` when nothing is left.
    pub(super) fn around(shapes: &[GlassShape], margin: f32, limit: GlassBounds) -> Option<GlassBounds> {
        let fold = |seed: f32, pick: fn(f32, f32) -> f32, side: &dyn Fn(&GlassShape) -> f32| {
            shapes.iter().map(side).fold(seed, pick)
        };
        let x0 = fold(f32::MAX, f32::min, &|s| s.x) - margin;
        let y0 = fold(f32::MAX, f32::min, &|s| s.y) - margin;
        let x1 = fold(f32::MIN, f32::max, &|s| s.x + s.width) + margin;
        let y1 = fold(f32::MIN, f32::max, &|s| s.y + s.height) + margin;
        let cut = GlassBounds {
            x0: (x0.floor().max(0.0) as usize).max(limit.x0),
            y0: (y0.floor().max(0.0) as usize).max(limit.y0),
            x1: (x1.ceil().max(0.0) as usize).min(limit.x1),
            y1: (y1.ceil().max(0.0) as usize).min(limit.y1),
        };
        (cut.x0 < cut.x1 && cut.y0 < cut.y1).then_some(cut)
    }

    pub fn width(&self) -> usize {
        self.x1 - self.x0
    }

    pub fn height(&self) -> usize {
        self.y1 - self.y0
    }
}
