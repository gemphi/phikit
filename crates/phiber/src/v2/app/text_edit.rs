//! The caret and selection for the field being typed into — the model behind
//! `EntityInputHandler`. Ported from gpui's `examples/input.rs`, adapted so the
//! edited text itself lives in `BState` (under the field's own key) and only
//! the caret travels here.
//!
//! It lives at the app level, beside `app/input.rs`, because the caret lives
//! on `BPhiberApp` and nothing in `state/` reads it: the state level holds the
//! store, the keys and the rules, and `state/edit.rs` — which field HAS the
//! caret and what each key does to the value — is the part of this that does
//! belong there (`tests/rule_budget.rs`).
//!
//! Offsets are byte indices into the field's current value. The platform text
//! system speaks UTF-16, so the conversions here bridge the two; boundaries
//! step one grapheme cluster at a time (as gpui's example does, via
//! `unicode-segmentation`), so ←/→, Backspace and Delete treat a combining
//! mark, an emoji ZWJ sequence or a regional-indicator flag as the single
//! character the reader sees, never splitting it mid-cluster.

use std::ops::Range;
use unicode_segmentation::GraphemeCursor;

/// The caret/selection for the active field. An empty `selected` range is a
/// plain caret at that offset; a non-empty one is a highlighted span whose
/// moving end is `start` when `reversed`, else `end`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BCaret {
    pub selected: Range<usize>,
    pub reversed: bool,
    /// The IME's in-progress (marked) text while composing, as a byte range.
    pub marked: Option<Range<usize>>,
}

impl BCaret {
    /// A caret resting after all of `content` — where a field opens for typing.
    pub fn at_end(content: &str) -> Self {
        Self { selected: content.len()..content.len(), reversed: false, marked: None }
    }

    /// The moving end of the selection — where the next character lands.
    pub fn cursor(&self) -> usize {
        if self.reversed { self.selected.start } else { self.selected.end }
    }

    /// Collapse the selection to a plain caret at `offset` (a move).
    pub fn place(&mut self, offset: usize) {
        self.selected = offset..offset;
        self.reversed = false;
    }

    /// Drag the moving end to `offset` (a shift-move), flipping which end moves
    /// when it crosses the anchor.
    pub fn extend(&mut self, offset: usize) {
        if self.reversed {
            self.selected.start = offset;
        } else {
            self.selected.end = offset;
        }
        if self.selected.end < self.selected.start {
            self.reversed = !self.reversed;
            self.selected = self.selected.end..self.selected.start;
        }
    }

    /// Keep the caret inside `content` and on char boundaries — the content can
    /// change under it (a filter is cleared, the key is set from elsewhere).
    pub fn clamp(&mut self, content: &str) {
        let s = floor_boundary(content, self.selected.start);
        let e = floor_boundary(content, self.selected.end);
        self.selected = s.min(e)..s.max(e);
        self.marked = self.marked.take().and_then(|m| {
            let ms = floor_boundary(content, m.start);
            let me = floor_boundary(content, m.end);
            (ms < me).then_some(ms..me)
        });
    }
}

/// The char boundary at or before `offset` (and never past the end).
pub fn floor_boundary(content: &str, offset: usize) -> usize {
    let mut b = offset.min(content.len());
    while b > 0 && !content.is_char_boundary(b) {
        b -= 1;
    }
    b
}

/// The grapheme boundary one cluster before `offset` — the step Left and
/// Backspace take, so the caret clears a whole combining/emoji cluster at once.
pub fn prev_boundary(content: &str, offset: usize) -> usize {
    let o = floor_boundary(content, offset);
    GraphemeCursor::new(o, content.len(), true)
        .prev_boundary(content, 0)
        .ok()
        .flatten()
        .unwrap_or(0)
}

/// The grapheme boundary one cluster after `offset` — the step Right and
/// Delete take.
pub fn next_boundary(content: &str, offset: usize) -> usize {
    let o = floor_boundary(content, offset);
    GraphemeCursor::new(o, content.len(), true)
        .next_boundary(content, 0)
        .ok()
        .flatten()
        .unwrap_or(content.len())
}

/// Splice `text` into `content` over the byte `range`, returning the result.
pub fn splice(content: &str, range: Range<usize>, text: &str) -> String {
    let mut out = String::with_capacity(content.len() + text.len());
    out.push_str(&content[..range.start]);
    out.push_str(text);
    out.push_str(&content[range.end..]);
    out
}

/// Byte offset for a UTF-16 offset into `content` (what the platform hands us).
pub fn offset_from_utf16(content: &str, target: usize) -> usize {
    let mut utf8 = 0;
    let mut utf16 = 0;
    for ch in content.chars() {
        if utf16 >= target {
            break;
        }
        utf16 += ch.len_utf16();
        utf8 += ch.len_utf8();
    }
    utf8
}

/// UTF-16 offset for a byte offset into `content` (what the platform expects).
pub fn offset_to_utf16(content: &str, target: usize) -> usize {
    let mut utf16 = 0;
    let mut utf8 = 0;
    for ch in content.chars() {
        if utf8 >= target {
            break;
        }
        utf8 += ch.len_utf8();
        utf16 += ch.len_utf16();
    }
    utf16
}

pub fn range_to_utf16(content: &str, r: &Range<usize>) -> Range<usize> {
    offset_to_utf16(content, r.start)..offset_to_utf16(content, r.end)
}

pub fn range_from_utf16(content: &str, r: &Range<usize>) -> Range<usize> {
    offset_from_utf16(content, r.start)..offset_from_utf16(content, r.end)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn placing_and_extending_track_the_moving_end() {
        let mut c = BCaret::at_end("cube");
        assert_eq!(c.cursor(), 4);
        c.place(2);
        assert_eq!((c.selected.clone(), c.reversed), (2..2, false));
        c.extend(4); // select right to the end
        assert_eq!((c.selected.clone(), c.reversed, c.cursor()), (2..4, false, 4));
        c.extend(1); // drag back past the anchor — the start now moves
        assert_eq!((c.selected.clone(), c.reversed, c.cursor()), (1..2, true, 1));
    }

    #[test]
    fn boundaries_step_one_grapheme_and_stop_at_the_ends() {
        let s = "ab";
        assert_eq!(prev_boundary(s, 0), 0);
        assert_eq!(prev_boundary(s, 2), 1);
        assert_eq!(next_boundary(s, 2), 2);
        assert_eq!(next_boundary(s, 0), 1);
        // A multi-byte char is one step, not one byte.
        let u = "é"; // two bytes
        assert_eq!(u.len(), 2);
        assert_eq!(next_boundary(u, 0), 2);
        assert_eq!(prev_boundary(u, 2), 0);
    }

    #[test]
    fn a_grapheme_cluster_is_one_step_not_one_char() {
        // "e" + combining acute (U+0301): one cluster, three bytes.
        let combined = "e\u{0301}";
        assert_eq!(combined.len(), 3);
        assert_eq!(next_boundary(combined, 0), 3, "the mark rides with its base letter");
        assert_eq!(prev_boundary(combined, 3), 0, "Backspace clears the whole cluster");

        // A regional-indicator flag (🇿🇦): two code points, one grapheme, 8 bytes.
        let flag = "🇿🇦";
        assert_eq!(flag.len(), 8);
        assert_eq!(next_boundary(flag, 0), 8, "the flag is a single character to the caret");
        assert_eq!(prev_boundary(flag, 8), 0);

        // A ZWJ family emoji: one cluster, stepped whole from either end.
        let family = "👨‍👩‍👧";
        assert_eq!(next_boundary(family, 0), family.len());
        assert_eq!(prev_boundary(family, family.len()), 0);

        // Stepping through "a" + flag + "b": a, then the whole flag, then b.
        let mixed = "a🇿🇦b";
        assert_eq!(next_boundary(mixed, 0), 1); // past 'a'
        assert_eq!(next_boundary(mixed, 1), 9); // past the 8-byte flag as one step
        assert_eq!(next_boundary(mixed, 9), 10); // past 'b'
        assert_eq!(prev_boundary(mixed, 9), 1); // back over the flag in one step
    }

    #[test]
    fn splice_inserts_replaces_and_deletes() {
        assert_eq!(splice("cube", 4..4, "s"), "cubes"); // insert at end
        assert_eq!(splice("cube", 0..0, "a "), "a cube"); // insert at start
        assert_eq!(splice("cube", 1..3, "X"), "cXe"); // replace a span
        assert_eq!(splice("cube", 1..2, ""), "cbe"); // delete one
    }

    #[test]
    fn utf16_round_trips_through_bytes() {
        // An astral char is 2 UTF-16 units and 4 bytes.
        let s = "a😀b";
        assert_eq!(offset_to_utf16(s, 0), 0);
        assert_eq!(offset_to_utf16(s, 1), 1); // after 'a'
        assert_eq!(offset_to_utf16(s, 5), 3); // after the emoji (1 + 4 bytes)
        assert_eq!(offset_from_utf16(s, 3), 5);
        assert_eq!(range_from_utf16(s, &range_to_utf16(s, &(1..5))), 1..5);
    }

    #[test]
    fn clamp_pulls_a_stale_caret_back_inside_the_content() {
        let mut c = BCaret { selected: 3..9, reversed: false, marked: Some(5..7) };
        c.clamp("hi"); // content shrank under the caret
        assert!(c.selected.end <= 2 && c.selected.start <= c.selected.end);
        assert_eq!(c.marked, None, "a marked range past the end is dropped");
    }
}
