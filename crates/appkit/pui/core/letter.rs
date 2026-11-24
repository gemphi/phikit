//! The mark of a row, a person or an app that has no picture of its own:
//! the first letter of its name, set in the window's own face at the
//! heading's size, in the middle of where the mark stands (the user, 24
//! November 2026: "those letters must be the current font"). It is in
//! the lower case, as a name in a logo is in one case; a name that
//! begins with no letter or figure shows a dot.

use super::{PColor, PPaint, PPoint, PRect};

/// What a name's mark says: its first letter or figure, small.
pub fn letter(name: &str) -> String {
    name.trim().chars().find(|one| one.is_alphanumeric()).map_or_else(|| "\u{b7}".to_string(), |one| one.to_lowercase().collect())
}

/// Set a name's mark in the middle of `rect`, in the heading's size of
/// the face the window is set in.
pub fn draw_letter(paint: &mut dyn PPaint, name: &str, rect: PRect, color: PColor) {
    let first = letter(name);
    let (head, body, line) = (paint.size("head.size"), paint.size("text.size"), paint.size(super::figure::LINE));
    let high = match head > 0.0 && body > 0.0 {
        true => line * head / body,
        false => line,
    };
    let wide = paint.measure_head(&first);
    paint.head(PPoint::new(rect.center().x - wide / 2.0, rect.center().y - high / 2.0), &first, color);
}
