//! The mark of a row, a person or an app that has no picture of its own:
//! the first letter or figure of its name, as a drawn mark of the
//! house's set (`asset/icon-letter.tsv`: the pin's squarish frame with
//! the letter inside it, on the construction every icon is drawn on), as
//! large as the icons beside it (the user, 25 November 2026). A name
//! that begins with neither has the mark for any.

use super::{PColor, PPaint, PRect};

/// How the name of a letter's icon begins, and the icon for a name that
/// begins with no letter or figure drawn.
pub const LETTER: &str = "letter-";
pub const ANY: &str = "letter-any";

/// The icon for a name: its first letter or figure's, small or capital
/// alike.
pub fn letter(name: &str) -> String {
    let first = name.trim().chars().next().map(|first| first.to_ascii_lowercase());
    match first.filter(char::is_ascii_alphanumeric) {
        Some(first) => format!("{LETTER}{first}"),
        None => ANY.to_string(),
    }
}

/// Draw a name's mark in `rect`, the square an icon takes there.
pub fn draw_letter(paint: &mut dyn PPaint, name: &str, rect: PRect, color: PColor) {
    paint.icon(&letter(name), rect, color);
}
