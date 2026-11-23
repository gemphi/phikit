//! A face: somebody's picture, round, on a ground of its own; or,
//! for one with no picture, the letter their name begins with, drawn.

use super::super::core::light::surface;
use super::super::core::letter::letter;
use super::super::core::{PFit, PLight, PPaint, PRect};
use super::{GAP, ICON, INK};

/// Somebody's face.
pub trait PFace {
    /// Their picture, as a cell of a sheet: the sheet, how many it has
    /// in a row, and which it is. Nothing for one with no picture.
    fn cell(&self) -> Option<(&str, u32, u32)> {
        None
    }

    /// What they are called.
    fn name(&self) -> &str {
        ""
    }

    /// Draw it in a square, on a ground of its own: their picture,
    /// round; or, with none, the letter their name begins with, drawn
    /// as every icon is, as large as an icon and no larger than the
    /// square.
    fn face(&self, paint: &mut dyn PPaint, rect: PRect) {
        let (gap, side, ink) = (paint.size(GAP), paint.size(ICON).min(rect.size.width), paint.color(INK));
        surface(paint, rect.inset(-gap / 8.0), rect.size.width / 2.0 + gap / 8.0, PLight::Rest, "");
        match self.cell() {
            Some((sheet, column, index)) => paint.picture(sheet, rect, rect.size.width / 2.0, PFit::Cell { column, index }),
            None => paint.icon(&letter(self.name()), PRect::new(rect.center().x - side / 2.0, rect.center().y - side / 2.0, side, side), ink),
        }
    }
}

/// A cell of a sheet, or none, and a name.
impl PFace for (Option<(&str, u32, u32)>, &str) {
    fn cell(&self) -> Option<(&str, u32, u32)> {
        self.0
    }

    fn name(&self) -> &str {
        self.1
    }
}
