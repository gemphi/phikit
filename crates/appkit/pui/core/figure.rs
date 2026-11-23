//! A figure: a number a person reads, on a ground of its own, so that
//! it is told from the words around it: an amount of money, a count of
//! what waits, a price. Every app draws its figures with this, so they
//! look alike and change with the look together.
//!
//! The ground is a label's light, from the side. A figure that stands
//! on something already lit runs against it (`against`), so the two
//! grounds are told apart. Its measures are the theme's: how high it
//! is, the room beside its text, and how round its corners. It is
//! never narrower than it is high, so one digit stands on a round
//! ground. In a theme that gives it no height it is its text alone.

use super::light::{self, PLight};
use super::{PPaint, PPoint, PRect, PSize};

/// The theme's tokens for a figure: how high its ground is, the room
/// at each side of its text, how round its corners, and what a flat
/// theme fills it with.
pub const HIGH: &str = "figure.height";
pub const INSET: &str = "figure.inset";
pub const RADIUS: &str = "figure.radius";
pub const FILL: &str = "figure.fill";
/// How high a line of text is.
pub const LINE: &str = "text.line";

/// How wide and how high a figure with that text is: nothing for no
/// text.
pub fn size(paint: &dyn PPaint, text: &str) -> PSize {
    let (high, inset) = (paint.size(HIGH), paint.size(INSET));
    match text.is_empty() {
        true => PSize::default(),
        false => PSize { width: (paint.measure(text) + inset * 2.0).max(high), height: high.max(paint.size(LINE)) },
    }
}

/// Draw a figure that ends at a place, its middle at that height.
/// Where it stands; nothing is drawn for no text.
pub fn end(paint: &mut dyn PPaint, at: PPoint, text: &str, against: bool) -> PRect {
    let size = size(paint, text);
    draw(paint, PRect::new(at.x - size.width, at.y - size.height / 2.0, size.width, size.height), text, against)
}

/// Draw a figure that starts at a place, its middle at that height.
pub fn start(paint: &mut dyn PPaint, at: PPoint, text: &str, against: bool) -> PRect {
    let size = size(paint, text);
    draw(paint, PRect::new(at.x, at.y - size.height / 2.0, size.width, size.height), text, against)
}

/// The ground, where the theme gives a figure one, and the text in its
/// middle.
fn draw(paint: &mut dyn PPaint, rect: PRect, text: &str, against: bool) -> PRect {
    if text.is_empty() {
        return rect;
    }
    let radius = paint.size(RADIUS).min(rect.size.height / 2.0);
    if paint.size(HIGH) > 0.0 {
        light::label(paint, rect, radius, PLight::Rest, against, FILL);
    }
    let ink = paint.color(light::INK);
    paint.text(PPoint::new(rect.center().x - paint.measure(text) / 2.0, rect.center().y - paint.size(LINE) / 2.0), text, ink);
    rect
}
