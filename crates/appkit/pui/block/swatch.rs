//! A swatch: a colour to choose, round, lit as a surface is, in a ring
//! of soft light, the chosen one in strong light. A row of them spans a
//! width, a little smaller (down to three quarters) where that keeps them
//! all on one row, and runs on to more rows where it does not.

use crate::pui::core::light::{label, surface};
use crate::pui::core::{PColor, PLight, PPaint, PPoint, PRect};

/// The theme's tokens: a swatch's side, and the ring of light round it.
pub const SIZE: &str = "swatch.size";
pub const RING: &str = "swatch.ring";

/// Colours to choose from.
pub trait PSwatch {
    /// Each colour, and whether it is the one chosen.
    fn colour(&self) -> Vec<(PColor, bool)>;

    /// Draw them from a place across a width: where each stands, and
    /// where the last row ends.
    fn draw_in(&self, paint: &mut dyn PPaint, at: PPoint, wide: f32) -> (Vec<PRect>, f32) {
        let list = self.colour();
        let (side, ring) = (paint.size(SIZE), paint.size(RING));
        let count = list.len().max(1) as f32;
        let fit = (wide + ring) / count - ring * 3.0;
        let side = match fit >= side * 0.75 {
            true => side.min(fit),
            false => side,
        };
        let cell = side + ring * 2.0;
        let across = ((wide + ring) / (cell + ring)).floor().clamp(1.0, count) as usize;
        let step = match across {
            1 => 0.0,
            n => (wide - cell) / (n - 1) as f32,
        };
        let rect: Vec<PRect> = (0..list.len()).map(|i| PRect::new(at.x + ring + (i % across) as f32 * step, at.y + ring + (i / across) as f32 * (cell + ring), side, side)).collect();
        for (r, (colour, on)) in rect.iter().zip(&list) {
            let strength = match on {
                true => PLight::Strong,
                false => PLight::Soft,
            };
            label(paint, r.inset(-ring), side / 2.0 + ring, strength, false, "");
            paint.round(*r, side / 2.0, *colour);
            surface(paint, *r, side / 2.0, PLight::Rest, "");
        }
        let rows = list.len().div_ceil(across) as f32;
        (rect, at.y + rows * (cell + ring) - ring)
    }
}

impl PSwatch for [(PColor, bool)] {
    fn colour(&self) -> Vec<(PColor, bool)> {
        self.to_vec()
    }
}
