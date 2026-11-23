//! A drawn letter for a name: the mark of a row, a person or an app
//! that has no picture of its own. It is an icon of the house's set
//! (`asset/icon.tsv`, `letter-a` to `letter-z`), drawn on the same
//! construction as every other icon, never type set in a font.

/// How the name of a letter's icon begins, and the icon for a name
/// that begins with no letter drawn yet.
pub const LETTER: &str = "letter-";
pub const ANY: &str = "letter-any";

/// The icon for a name: its first letter's, small or capital alike.
pub fn letter(name: &str) -> String {
    let first = name.trim().chars().next().map(|first| first.to_ascii_lowercase());
    match first.filter(char::is_ascii_lowercase) {
        Some(first) => format!("{LETTER}{first}"),
        None => ANY.to_string(),
    }
}
