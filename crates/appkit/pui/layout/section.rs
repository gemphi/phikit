//! A section on a ground of its own: the one way every app groups what
//! belongs together. A ground is not decoration: what stands on one
//! ground is one thing, a row with its buttons, a list under its
//! heading, a pane. An app does not draw its own grounds for that; it
//! says what the section draws, here.
//!
//! What a section draws is not known until it is drawn, so it is drawn
//! first with a painter that draws nothing and measures as the real one
//! does, the ground is laid as far as it went, and it is drawn again.
//! Something drawn in one pass, that knows where its sections begin and
//! end, lays each ground itself (`ground`).

use crate::pui::core::light::label;
use crate::pui::core::{PColor, PLight, PPaint, PPoint, PRect};
use crate::theme::PTheme;

/// The theme's tokens for a section's ground, the same for every section
/// of every part: the room it has beside and over what stands on it, and
/// how round it is. A theme without them has half a gap beside, a
/// quarter over, and a row's roundness.
pub const GAP: &str = "gap";
pub const RADIUS: &str = "row.radius";
pub const SIDE: &str = "section.side";
pub const TOP: &str = "section.top";
pub const ROUND: &str = "section.radius";

/// A painter that draws nothing and measures as another does: with it
/// something is laid out to learn how long it is.
pub struct PDry<'a>(pub &'a dyn PPaint);

impl PPaint for PDry<'_> {
    fn theme(&self) -> &dyn PTheme {
        self.0.theme()
    }

    fn measure(&self, text: &str) -> f32 {
        self.0.measure(text)
    }

    fn fill(&mut self, _: PRect, _: PColor) {}

    fn text(&mut self, _: PPoint, _: &str, _: PColor) {}

    fn icon(&mut self, _: &str, _: PRect, _: PColor) {}
}

/// Lay the ground of a section that begins at `at`, is `wide`, and ends
/// at `end`: a little wider and higher than what stands on it, lit from
/// the right, against the pane it stands in. Nothing for a section
/// with nothing in it.
pub fn ground(paint: &mut dyn PPaint, at: PPoint, wide: f32, end: f32) {
    let (gap, theme) = (paint.size(GAP), paint.theme());
    let (side, top) = (theme.size(SIDE).unwrap_or(gap / 2.0), theme.size(TOP).unwrap_or(gap / 4.0));
    let radius = theme.size(ROUND).unwrap_or(paint.size(RADIUS));
    if end > at.y {
        label(paint, PRect::new(at.x - side, at.y - top, wide + side * 2.0, end - at.y + top * 2.0), radius, PLight::Soft, true, "");
    }
}

/// Draw a section on a ground of its own. `draw` draws it with the
/// painter it is given and says how far down it went.
pub fn section(paint: &mut dyn PPaint, at: PPoint, wide: f32, draw: &mut dyn FnMut(&mut dyn PPaint) -> f32) -> f32 {
    let end = draw(&mut PDry(&*paint));
    ground(paint, at, wide, end);
    draw(paint)
}
