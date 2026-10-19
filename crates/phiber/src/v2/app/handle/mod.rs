//! GPUI action dispatch. State transitions are pure and shared with tests;
//! this layer is responsible only for notifying GPUI after a transition.
//!
//! Three concerns, each one file: this module is the dispatcher proper —
//! `handle_action`, the window-in-hand `handle_action_in`, and `seat_caret`
//! that keeps the field with the caret matching whatever the state got to.
//! Keystrokes into a field (`key.rs`) and the area/zone drag seam
//! (`resize.rs`) are their own files.

use super::BPhiberApp;
use super::import::should_import;
use crate::v2::scene::{loaded_path_key, selected_key};
use crate::v2::state::{BKey as _, CtxKey, action::apply_action};
use crate::v2::view::prim::BAction;
use gpui::Context;

mod key;
mod resize;

impl BPhiberApp {
    /// An action from the window itself: the window's own operators (quit,
    /// fullscreen) are taken here, everything else goes the usual way.
    pub fn handle_action_in(&mut self, action: &BAction, window: &mut gpui::Window, cx: &mut Context<Self>) {
        // Unsaved changes: quit, new and open wait for the dialog's answer,
        // and the answer itself is taken here, with the window in hand.
        if self.asks_to_save_first(action) || self.answers_file_close(action, window, cx) {
            cx.notify();
            return;
        }
        if self.handle_window_op(action, window) {
            crate::v2::state::menu::close_overlays(&mut self.state);
            cx.notify();
            return;
        }
        self.handle_action(action, cx);
    }

    /// The caret follows the field that holds it. A field takes the caret
    /// however the state got there — an action that sets `AppKey::Edit`, or a
    /// rule that calls `edit::start` while the action is applied — so this
    /// asks what CHANGED rather than watching one action's shape. It was
    /// seated only on the action path, and F2's rename popup takes the caret
    /// the other way: the field opened editing at offset 0, so four
    /// backspaces deleted nothing and the typed name landed in front of the
    /// old one ("CrateCube"). A field opens with the caret after all its
    /// text, so the first keystroke appends and the arrows have somewhere to
    /// move from.
    fn seat_caret(&mut self, before: &crate::v2::state::BState) {
        let field = crate::v2::state::AppKey::Edit;
        if before.get(field) == self.state.get(field) {
            return;
        }
        let content = crate::v2::state::edit::editing(&self.state)
            .and_then(|key| self.state.get_encoded(&key).map(str::to_string))
            .unwrap_or_default();
        self.input.caret = crate::v2::app::text_edit::BCaret::at_end(&content);
    }

    pub fn handle_action(&mut self, action: &BAction, cx: &mut Context<Self>) {
        // 0. `ctx.link = <url>` is Blender's `wm.url_open`: hand the URL to
        //    the system browser, then let the action run its course so the
        //    menu closes and the status bar says what was opened; the key
        //    itself is not kept.
        if let BAction::Set(path, url) = action
            && *path == crate::v2::state::AppKey::Link.path()
            && !url.is_empty()
        {
            cx.open_url(url);
            let before = self.state.clone();
            apply_action(&mut self.state, action);
            let _ = crate::v2::state::log::record(action, &before, &self.state);
            self.state.remove(crate::v2::state::AppKey::Link);
            cx.notify();
            return;
        }
        // 0a. Render ▸ Lock Interface, while a render runs: an edit to the
        //     document does not go through (`wm->is_interface_locked`,
        //     `wm_operator_check_locked_interface`); the view, the menus and
        //     the render's own Cancel still do.
        if self.interface_locked_against(action) {
            crate::v2::state::log::push("Interface is locked while rendering (Render ▸ Lock Interface)".to_string());
            cx.notify();
            return;
        }
        // 0b. Unsaved changes: quit, new and open wait for the dialog.
        if self.asks_to_save_first(action) {
            cx.notify();
            return;
        }
        // 0c. The app's own operators (`app/op.rs`) — undo, new, revert,
        //     render — run instead of being stored.
        if self.handle_app_op(action, cx) {
            cx.notify();
            return;
        }
        // What the file browser was opened for, read before the commit's
        // rule can clear it; `.blend` opening is the only commit the old
        // path took, exports and imports are finished below.
        let commit = match action {
            BAction::Set(path, file) if *path == selected_key() && !file.is_empty() => Some((file.clone(), super::op::purpose_of(&self.state))),
            _ => None,
        };
        // 1. Apply the pure state transition and log what it changed; the
        //    document before it is an undo step when the action changed it.
        let before = self.state.clone();
        let document = super::undo::snapshot(&self.state);
        apply_action(&mut self.state, action);
        self.seat_caret(&before);
        // Share this edit with any connected peers (a no-op offline, and the
        // local-only keys — caret, camera, workspace — are filtered out).
        super::collab::BCollab::broadcast(action);
        let name = crate::v2::state::log::record(action, &before, &self.state).unwrap_or_else(|| "Edit".to_string());
        if self.history.record_held(document, &super::undo::snapshot(&self.state), &name, super::undo::holding(&self.state)) {
            super::op::close::tag_modified(&mut self.state);
        }
        // The Rendering workspace opens on a render (`op/render.rs`).
        if before.get(CtxKey::Space) != self.state.get(CtxKey::Space) {
            self.render_on_entry();
        }

        // 2. Import spawn/settle shell around the pure core. If the just-applied
        //    action moved `file.selected` to a `.blend` path differing from the
        //    path the Scene_Source currently reflects (`scene.loaded_path`), we
        //    kick off exactly one import off the UI thread — for an Open, not
        //    for a Save As, Link or Export that happened to name a `.blend`.
        match commit {
            Some((file, crate::v2::part::file::purpose::BFilePurpose::Open)) => {
                if should_import(self.state.get(&loaded_path_key()), Some(&file)) {
                    self.start_import(file, cx);
                }
            }
            Some((file, purpose)) => self.handle_file_commit(&file, purpose, cx),
            None => {}
        }

        // 3. Playback starts its ticker when Play turns on.
        self.ensure_playback(cx);

        // 4. Always notify after the synchronous part so the loading flag (and
        //    any other state change) shows on this render pass (as before).
        cx.notify();
    }
}
