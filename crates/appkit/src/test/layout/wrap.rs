use crate::pui::layout::{fold, wrap};

/// Every letter is 10 wide.
fn gauge(text: &str) -> f32 {
    text.chars().count() as f32 * 10.0
}

#[test]
fn text_breaks_between_words_into_lines_no_wider_than_the_width() {
    assert_eq!(wrap("one two three four", 80.0, &gauge), ["one two", "three", "four"]);
    assert_eq!(wrap("short", 80.0, &gauge), ["short"]);
}

#[test]
fn a_word_wider_than_the_width_keeps_a_line_to_itself() {
    assert_eq!(wrap("a incomprehensible b", 80.0, &gauge), ["a", "incomprehensible", "b"]);
}

#[test]
fn a_line_break_in_the_text_is_kept_and_no_text_is_no_line() {
    assert_eq!(wrap("one\ntwo three", 200.0, &gauge), ["one", "two three"]);
    assert!(wrap("", 200.0, &gauge).is_empty());
}

#[test]
fn folded_a_word_too_long_for_a_line_goes_on_in_the_next() {
    // Eight letters fit a line.
    assert_eq!(fold("see C:/a/long/folder/name here", 80.0, &gauge), ["see", "C:/a/lon", "g/folder", "/name", "here"]);
    // What fits is left as it was wrapped, and nothing is nothing.
    assert_eq!(fold("one two three four", 80.0, &gauge), wrap("one two three four", 80.0, &gauge));
    assert!(fold("", 80.0, &gauge).is_empty());
    // A room narrower than a letter still takes a letter a line.
    assert_eq!(fold("abc", 5.0, &gauge), ["a", "b", "c"]);
}
