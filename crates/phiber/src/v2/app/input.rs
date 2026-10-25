//! Real text input for the field that holds the caret — gpui's
//! `EntityInputHandler` on the one window entity.
//!
//! The edited text lives in `BState` under the field's own key; the caret and
//! the IME's marked range live here on [`BInput`], and the field's shaped line
//! and painted bounds are captured each frame (`draw::text_input`) so the
//! platform can place an IME popup and map a click to an offset. Printable text
//! and IME composition arrive through this handler; the arrow / Home / End /
//! Backspace / Delete / select keys come through `handle_key` and drive the
//! same caret (`caret_*` below). Ported from gpui's `examples/input.rs`.

use std::ops::Range;

use gpui::{Bounds, Pixels, Point, ShapedLine};

use crate::v2::state::edit;
use crate::v2::app::text_edit::{self, BCaret};

use super::BPhiberApp;

/// The caret and the geometry the input handler needs, carried on the window
/// entity. `last_line` / `last_bounds` are set by the field element as it
/// paints and read back for IME placement and click-to-offset.
#[derive(Default)]
pub struct BInput {
    pub caret: BCaret,
    pub last_line: Option<ShapedLine>,
    pub last_bounds: Option<Bounds<Pixels>>,
}

impl BPhiberApp {
    /// The encoded key of the field being typed into, if any.
    pub(crate) fn edit_path(&self) -> Option<String> {
        edit::editing(&self.state)
    }

    /// The current text of the edited field (empty when none is editing).
    pub(crate) fn edit_content(&self) -> String {
        self.edit_path()
            .and_then(|p| self.state.get_encoded(&p).map(str::to_string))
            .unwrap_or_default()
    }

    /// Clamp the caret to the current content and return that content — run at
    /// the top of every edit so a value changed elsewhere can't strand it.
    fn fresh_content(&mut self) -> String {
        let content = self.edit_content();
        self.input.caret.clamp(&content);
        content
    }

    // ── Caret keys (driven from `handle_key`) ───────────────────────────────

    pub(crate) fn caret_left(&mut self) {
        let c = self.fresh_content();
        if self.input.caret.selected.is_empty() {
            let p = text_edit::prev_boundary(&c, self.input.caret.cursor());
            self.input.caret.place(p);
        } else {
            let s = self.input.caret.selected.start;
            self.input.caret.place(s);
        }
    }

    pub(crate) fn caret_right(&mut self) {
        let c = self.fresh_content();
        if self.input.caret.selected.is_empty() {
            let n = text_edit::next_boundary(&c, self.input.caret.cursor());
            self.input.caret.place(n);
        } else {
            let e = self.input.caret.selected.end;
            self.input.caret.place(e);
        }
    }

    pub(crate) fn caret_select_left(&mut self) {
        let c = self.fresh_content();
        let p = text_edit::prev_boundary(&c, self.input.caret.cursor());
        self.input.caret.extend(p);
    }

    pub(crate) fn caret_select_right(&mut self) {
        let c = self.fresh_content();
        let n = text_edit::next_boundary(&c, self.input.caret.cursor());
        self.input.caret.extend(n);
    }

    pub(crate) fn caret_home(&mut self) {
        let _ = self.fresh_content();
        self.input.caret.place(0);
    }

    pub(crate) fn caret_end(&mut self) {
        let c = self.fresh_content();
        self.input.caret.place(c.len());
    }

    /// Put `text` in place of the selection (or at the caret), the caret
    /// after it: what a paste, a cut (with `""`) and a typed replacement
    /// all are. The same splice `replace_text_in_range` makes for the
    /// platform's own inserts, without the UTF-16 round trip.
    pub(crate) fn caret_insert(&mut self, text: &str) {
        let Some(path) = self.edit_path() else { return };
        let content = self.fresh_content();
        let range = self.input.caret.selected.clone();
        let range = text_edit::floor_boundary(&content, range.start)..text_edit::floor_boundary(&content, range.end);
        let next = text_edit::splice(&content, range.clone(), text);
        self.input.caret.place(range.start + text.len());
        self.input.caret.marked = None;
        self.state.set_encoded(&path, next);
    }

    /// The selected text, `None` when nothing is selected.
    pub(crate) fn caret_selected_text(&mut self) -> Option<String> {
        let content = self.fresh_content();
        let range = self.input.caret.selected.clone();
        if range.is_empty() {
            return None;
        }
        let range = text_edit::floor_boundary(&content, range.start)..text_edit::floor_boundary(&content, range.end);
        Some(content[range].to_string())
    }

    pub(crate) fn caret_select_all(&mut self) {
        let c = self.fresh_content();
        self.input.caret.place(0);
        self.input.caret.extend(c.len());
    }

    /// Delete the selection, or the char before the caret when it is empty.
    pub(crate) fn caret_backspace(&mut self) -> bool {
        self.delete_span(|c, caret| {
            if caret.selected.is_empty() {
                let prev = text_edit::prev_boundary(c, caret.cursor());
                (prev != caret.cursor()).then_some(prev..caret.cursor())
            } else {
                Some(caret.selected.clone())
            }
        })
    }

    /// Delete the selection, or the char after the caret when it is empty.
    pub(crate) fn caret_delete(&mut self) -> bool {
        self.delete_span(|c, caret| {
            if caret.selected.is_empty() {
                let next = text_edit::next_boundary(c, caret.cursor());
                (next != caret.cursor()).then_some(caret.cursor()..next)
            } else {
                Some(caret.selected.clone())
            }
        })
    }

    /// Place the caret at a clicked point, using the field's captured line and
    /// bounds; falls back to the end when nothing has painted yet. `extend`
    /// drags the selection to the point instead (shift-click).
    pub(crate) fn place_caret_at_point(&mut self, pos: Point<Pixels>, extend: bool) {
        let content = self.fresh_content();
        let offset = match (self.input.last_bounds, self.input.last_line.as_ref()) {
            (Some(bounds), Some(line)) => {
                let x = pos.x - bounds.left();
                let idx = line
                    .index_for_x(x)
                    .unwrap_or(if pos.x <= bounds.left() { 0 } else { content.len() });
                text_edit::floor_boundary(&content, idx)
            }
            _ => content.len(),
        };
        if extend {
            self.input.caret.extend(offset);
        } else {
            self.input.caret.place(offset);
        }
    }

    fn delete_span(&mut self, span: impl Fn(&str, &BCaret) -> Option<Range<usize>>) -> bool {
        let Some(path) = self.edit_path() else { return false };
        let content = self.fresh_content();
        let Some(range) = span(&content, &self.input.caret) else { return false };
        let next = text_edit::splice(&content, range.clone(), "");
        self.input.caret.place(range.start);
        self.input.caret.marked = None;
        self.state.set_encoded(&path, next);
        true
    }
}

#[path = "input_ime.rs"]
mod input_ime;


#[cfg(test)]
#[path = "input_tests.rs"]
mod tests;
