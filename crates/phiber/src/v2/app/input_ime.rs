//! GPUI's `EntityInputHandler` on `BPhiberApp` — the platform text-input
//! bridge: what the IME reads from the field, where it is painted, which
//! character the pointer hit. Split from `input.rs` so each file sits
//! under the 240-line read cap (`tests/level_structure.rs`); the caret
//! model and the key-level editing stay there.

use super::BPhiberApp;
use super::text_edit::{self, BCaret};
use crate::v2::state::{BKey as _, FileKey};
use gpui::{App, Bounds, Context, EntityInputHandler, Pixels, Point, UTF16Selection, Window};
use std::ops::Range;



impl EntityInputHandler for BPhiberApp {
    fn text_for_range(
        &mut self,
        range_utf16: Range<usize>,
        actual_range: &mut Option<Range<usize>>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<String> {
        let content = self.edit_content();
        let range = text_edit::range_from_utf16(&content, &range_utf16);
        let range = range.start.min(content.len())..range.end.min(content.len());
        actual_range.replace(text_edit::range_to_utf16(&content, &range));
        content.get(range).map(str::to_string)
    }

    fn selected_text_range(
        &mut self,
        _ignore_disabled_input: bool,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<UTF16Selection> {
        self.edit_path()?;
        let content = self.fresh_content();
        Some(UTF16Selection {
            range: text_edit::range_to_utf16(&content, &self.input.caret.selected),
            reversed: self.input.caret.reversed,
        })
    }

    fn marked_text_range(
        &self,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<Range<usize>> {
        let content = self.edit_content();
        self.input
            .caret
            .marked
            .as_ref()
            .map(|r| text_edit::range_to_utf16(&content, r))
    }

    fn unmark_text(&mut self, _window: &mut Window, _cx: &mut Context<Self>) {
        self.input.caret.marked = None;
    }

    fn replace_text_in_range(
        &mut self,
        range_utf16: Option<Range<usize>>,
        new_text: &str,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(path) = self.edit_path() else { return };
        let content = self.fresh_content();
        let range = range_utf16
            .map(|r| text_edit::range_from_utf16(&content, &r))
            .or_else(|| self.input.caret.marked.clone())
            .unwrap_or_else(|| self.input.caret.selected.clone());
        let range = text_edit::floor_boundary(&content, range.start)
            ..text_edit::floor_boundary(&content, range.end);
        let next = text_edit::splice(&content, range.clone(), new_text);
        let caret = range.start + new_text.len();
        self.input.caret.place(caret);
        self.input.caret.marked = None;
        self.state.set_encoded(&path, next);
        cx.notify();
    }

    fn replace_and_mark_text_in_range(
        &mut self,
        range_utf16: Option<Range<usize>>,
        new_text: &str,
        new_selected_range_utf16: Option<Range<usize>>,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(path) = self.edit_path() else { return };
        let content = self.fresh_content();
        let range = range_utf16
            .map(|r| text_edit::range_from_utf16(&content, &r))
            .or_else(|| self.input.caret.marked.clone())
            .unwrap_or_else(|| self.input.caret.selected.clone());
        let range = text_edit::floor_boundary(&content, range.start)
            ..text_edit::floor_boundary(&content, range.end);
        let next = text_edit::splice(&content, range.clone(), new_text);
        self.input.caret.marked =
            (!new_text.is_empty()).then(|| range.start..range.start + new_text.len());
        self.input.caret.reversed = false;
        self.input.caret.selected = new_selected_range_utf16
            .map(|r| text_edit::range_from_utf16(&next, &r))
            .map(|nr| nr.start + range.start..nr.end + range.end)
            .unwrap_or_else(|| {
                let c = range.start + new_text.len();
                c..c
            });
        self.state.set_encoded(&path, next);
        cx.notify();
    }

    fn bounds_for_range(
        &mut self,
        range_utf16: Range<usize>,
        element_bounds: Bounds<Pixels>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<Bounds<Pixels>> {
        let content = self.edit_content();
        let line = self.input.last_line.as_ref()?;
        let range = text_edit::range_from_utf16(&content, &range_utf16);
        Some(Bounds::from_corners(
            gpui::point(element_bounds.left() + line.x_for_index(range.start), element_bounds.top()),
            gpui::point(element_bounds.left() + line.x_for_index(range.end), element_bounds.bottom()),
        ))
    }

    fn character_index_for_point(
        &mut self,
        point: Point<Pixels>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<usize> {
        let bounds = self.input.last_bounds?;
        let line = self.input.last_line.as_ref()?;
        let content = self.edit_content();
        let x = point.x - bounds.left();
        let utf8 = line.index_for_x(x).unwrap_or_else(|| {
            if point.x <= bounds.left() { 0 } else { content.len() }
        });
        Some(text_edit::offset_to_utf16(&content, utf8))
    }
}
