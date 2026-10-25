//! Keystrokes into a field: the window-backed caret path and the window-less
//! append/pop path, forking only on whether a real window stands behind the
//! keystroke. The platform's text-input handler (`app::input`) owns printable
//! text and IME when a window is there; the caret model owns the navigation,
//! selection and delete keys. Without a window — headless renders, tests —
//! `state::edit` runs through its own simpler path so typing a value and
//! Return/Escape work with no platform text system under them.
//!
//! Return on a typed field re-applies its value through the normal action
//! path so everything watching the key still fires (a typed folder path
//! navigates, a picked file imports) — the one entry back into the
//! dispatcher.

use super::super::BPhiberApp;
use crate::v2::view::prim::BAction;
use gpui::Context;

impl BPhiberApp {
    pub fn handle_key(
        &mut self,
        key: &str,
        typed: Option<&str>,
        stroke: Option<(super::super::keymap::BKeystroke, &mut gpui::Window)>,
        cx: &mut Context<Self>,
    ) -> bool {
        use crate::v2::state::edit::{BEdit, editing};
        // Escape stops a render under way whoever has the keyboard — a
        // field left in edit mode (the Save As name, typed into before the
        // render) used to take it, and the render ran on. Blender's
        // `wm_event_do_handlers` sets `G.is_break` before any handler sees
        // the key: "we can always cancel operations such as rendering".
        if key == "escape" && crate::v2::scene::render_job::rendering() {
            crate::v2::state::edit::stop(&mut self.state);
            crate::v2::scene::render_job::cancel();
            crate::v2::state::log::push("Render cancelled (Escape)".to_string());
            cx.notify();
            return true;
        }
        let Some(target) = editing(&self.state) else {
            let Some((stroke, window)) = stroke else { return false };
            let Some((action, name)) = super::super::keymap::action_for(&stroke, &self.state) else { return false };
            crate::v2::state::log::push(name);
            self.handle_action_in(&action, window, cx);
            return true;
        };
        match stroke {
            Some((ks, _window)) => {
                let handled = self.caret_key(key, &ks, &target, cx);
                if handled {
                    cx.notify();
                }
                handled
            }
            None => {
                let outcome = crate::v2::state::edit::key(&mut self.state, key, typed);
                if outcome == BEdit::Ignored {
                    return false;
                }
                if outcome == BEdit::Confirmed {
                    self.commit_edit(&target, cx);
                }
                cx.notify();
                true
            }
        }
    }

    /// One editing key against the caret model, for a window-backed pass.
    /// Returns whether it was consumed; a printable key is left unconsumed so
    /// the text-input handler inserts it (and IME composes it).
    fn caret_key(
        &mut self,
        key: &str,
        ks: &super::super::keymap::BKeystroke,
        target: &str,
        cx: &mut Context<Self>,
    ) -> bool {
        use crate::v2::state::edit::BEdit;
        match key {
            "escape" | "enter" | "return" => match crate::v2::state::edit::key(&mut self.state, key, None) {
                BEdit::Confirmed => {
                    self.commit_edit(target, cx);
                    true
                }
                BEdit::Cancelled => true,
                _ => false,
            },
            "left" if ks.shift => self.consume(|a| a.caret_select_left()),
            "left" => self.consume(|a| a.caret_left()),
            "right" if ks.shift => self.consume(|a| a.caret_select_right()),
            "right" => self.consume(|a| a.caret_right()),
            "home" => self.consume(|a| a.caret_home()),
            "end" => self.consume(|a| a.caret_end()),
            "a" if ks.control => self.consume(|a| a.caret_select_all()),
            // The clipboard, Blender's `ui_textedit_copypaste`: Ctrl V
            // pastes over the selection (or at the caret), Ctrl C copies the
            // selection, Ctrl X cuts it. None of these existed — a key the
            // caret ladder did not name fell through to the platform text
            // handler as a printable, and "v" with Control held types
            // nothing, so Ctrl V did nothing in any field.
            "v" if ks.control => {
                let pasted = cx.read_from_clipboard().and_then(|item| item.text());
                match pasted {
                    Some(text) if !text.is_empty() => {
                        // One line: a field holds one line, as Blender's text
                        // button does (`ui_textedit_insert_buf` cuts at '\n').
                        let line: String = text.lines().next().unwrap_or("").to_string();
                        self.caret_insert(&line);
                        true
                    }
                    _ => true,
                }
            }
            "c" if ks.control => {
                if let Some(text) = self.caret_selected_text() {
                    cx.write_to_clipboard(gpui::ClipboardItem::new_string(text));
                }
                true
            }
            "x" if ks.control => {
                if let Some(text) = self.caret_selected_text() {
                    cx.write_to_clipboard(gpui::ClipboardItem::new_string(text));
                    self.caret_insert("");
                }
                true
            }
            "backspace" => {
                self.caret_backspace();
                true
            }
            "delete" => {
                self.caret_delete();
                true
            }
            // Printable: the text-input handler inserts it.
            _ => false,
        }
    }

    /// Run one caret move and report it as consumed. The ladder above was a
    /// branch per key with the same `true` return; the move itself is the
    /// only thing that differs.
    fn consume(&mut self, mv: impl FnOnce(&mut Self)) -> bool {
        mv(self);
        true
    }

    pub(super) fn commit_edit(&mut self, target: &str, cx: &mut Context<Self>) {
        if let Some(path) = crate::v2::state::edit::field_path(target) {
            let value = self.state.get_encoded(target).unwrap_or("").to_string();
            self.handle_action(&BAction::Set(path.clone(), value.clone()), cx);
            // Return in Menu Search runs the first item found, as Blender's
            // search menu does with its highlighted row.
            use crate::v2::modal::command::{find, palette};
            use crate::v2::state::{AppKey, BKey as _, ModalKey};
            let open = self.state.get(ModalKey::Open) == Some(crate::v2::state::val::V_MODAL_COMMAND);
            if path == AppKey::CommandQuery.path() && open && !value.trim().is_empty() {
                let kind = find::BFindKind::of(&self.state);
                if let Some(hit) = find::hits(&value, kind, palette::host(&self.state)).into_iter().next() {
                    self.state.set(ModalKey::Open, crate::v2::state::val::V_EMPTY);
                    self.handle_action(&hit.action, cx);
                }
            }
        }
    }
}
