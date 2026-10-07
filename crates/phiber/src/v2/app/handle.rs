//! GPUI action dispatch. State transitions are pure and shared with tests;
//! this layer is responsible only for notifying GPUI after a transition.

use super::BPhiberApp;
use super::import::should_import;
use crate::v2::scene::{loaded_path_key, selected_key};
use crate::v2::space::BSpaceType;
use crate::v2::area::size::{apply_delta, apply_drag_with_min_extent, normalize};
use crate::v2::state::{BKey as _, CtxKey, action::apply_action};
use crate::v2::zone::size::apply_zone_drag_with_min_extent;
use crate::v2::view::prim::BAction;
use crate::v2::zone::BZoneType;
use gpui::Context;

impl BPhiberApp {
    /// One key press: to the field that holds the caret, else to the keymap
    /// (`app/keymap`) — the window's own operators with the window in hand.
    /// Returns true when the window should re-render.
    pub fn handle_key(
        &mut self,
        key: &str,
        typed: Option<&str>,
        stroke: Option<(super::keymap::BKeystroke, &mut gpui::Window)>,
        cx: &mut Context<Self>,
    ) -> bool {
        use crate::v2::state::edit::{BEdit, editing};
        let Some(target) = editing(&self.state) else {
            let Some((stroke, window)) = stroke else { return false };
            let Some((action, name)) = super::keymap::action_for(&stroke, &self.state) else { return false };
            crate::v2::state::log::push(name);
            self.handle_action_in(&action, window, cx);
            return true;
        };
        match stroke {
            // A real window is behind the pass: the platform's text-input
            // handler (`app::input`) owns printable text and IME; the caret
            // model owns the navigation, selection and delete keys.
            Some((ks, _window)) => {
                let handled = self.caret_key(key, &ks, &target, cx);
                if handled {
                    cx.notify();
                }
                handled
            }
            // No window (headless renders and tests): the original append/pop
            // path, so typing a value and Return/Escape work without the
            // platform text system.
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
        ks: &super::keymap::BKeystroke,
        target: &str,
        cx: &mut Context<Self>,
    ) -> bool {
        use crate::v2::state::edit::BEdit;
        match key {
            "escape" | "enter" | "return" => {
                match crate::v2::state::edit::key(&mut self.state, key, None) {
                    BEdit::Confirmed => {
                        self.commit_edit(target, cx);
                        true
                    }
                    BEdit::Cancelled => true,
                    _ => false,
                }
            }
            "left" if ks.shift => {
                self.caret_select_left();
                true
            }
            "left" => {
                self.caret_left();
                true
            }
            "right" if ks.shift => {
                self.caret_select_right();
                true
            }
            "right" => {
                self.caret_right();
                true
            }
            "home" => {
                self.caret_home();
                true
            }
            "end" => {
                self.caret_end();
                true
            }
            "a" if ks.control => {
                self.caret_select_all();
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

    /// Return on a typed field re-applies its value through the normal action
    /// path so everything watching the key still fires (a typed folder path
    /// navigates, a picked file imports).
    fn commit_edit(&mut self, target: &str, cx: &mut Context<Self>) {
        if let Some(path) = crate::v2::state::edit::field_path(target) {
            let value = self.state.get_encoded(target).unwrap_or("").to_string();
            self.handle_action(&BAction::Set(path, value), cx);
        }
    }

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

    pub fn handle_action(&mut self, action: &BAction, cx: &mut Context<Self>) {
        // A field taking the caret opens with the caret after all its text
        // (`AppKey::Edit = <field key>`), so the first keystroke appends and
        // the arrow keys have somewhere to move from.
        if let BAction::Set(path, field) = action
            && *path == crate::v2::state::AppKey::Edit.path()
            && !field.is_empty()
        {
            let content = self.state.get_encoded(field).unwrap_or("").to_string();
            self.input.caret = crate::v2::state::text_edit::BCaret::at_end(&content);
        }
        // 0. `ctx.link = <url>` is Blender's `wm.url_open`: hand the URL to
        //    the system browser, then let the action run its course so the
        //    menu closes and the status bar says what was opened; the key
        //    itself is not kept.
        if let BAction::Set(path, url) = action {
            if *path == crate::v2::state::AppKey::Link.path() && !url.is_empty() {
                cx.open_url(url);
                let before = self.state.clone();
                apply_action(&mut self.state, action);
                let _ = crate::v2::state::log::record(action, &before, &self.state);
                self.state.remove(crate::v2::state::AppKey::Link);
                cx.notify();
                return;
            }
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

    /// Apply an incremental area-resize delta (legacy seam kept for the
    /// `BResizeHandler` callback) and notify only after a valid change.
    pub fn handle_resize(
        &mut self,
        space: BSpaceType,
        zone: BZoneType,
        index: usize,
        dpx: f32,
        extent: f32,
        cx: &mut Context<Self>,
    ) {
        if apply_delta(&mut self.state, space, zone, index, dpx, extent).is_ok() {
            normalize(&mut self.state, space, zone);
            cx.notify();
        }
    }

    /// Apply an *absolute* area-resize drag from a fixed anchor. The
    /// interactive divider uses this so the resize is idempotent in the total
    /// pixel delta — re-dispatching the same move converges to the same
    /// fractions regardless of how many move events gpui emits.
    #[allow(clippy::too_many_arguments)]
    pub fn handle_resize_drag(
        &mut self,
        space: BSpaceType,
        zone: BZoneType,
        index: usize,
        anchor_i: f32,
        anchor_j: f32,
        total_dpx: f32,
        extent: f32,
        min_extent: f32,
        cx: &mut Context<Self>,
    ) {
        if apply_drag_with_min_extent(
            &mut self.state,
            space,
            zone,
            index,
            anchor_i,
            anchor_j,
            total_dpx,
            extent,
            min_extent,
        )
        .is_ok()
        {
            normalize(&mut self.state, space, zone);
            cx.notify();
        }
    }

    /// Apply an absolute zone-seam drag (whole Left/Right column width or
    /// Bottom strip height). `anchor` is the zone size at drag start,
    /// `total` the cursor travel since then, `sign` the direction mapping.
    pub fn handle_zone_drag(
        &mut self,
        space: BSpaceType,
        zone: BZoneType,
        anchor: f32,
        total: f32,
        sign: f32,
        extent: f32,
        min_extent: f32,
        cx: &mut Context<Self>,
    ) {
        if apply_zone_drag_with_min_extent(
            &mut self.state,
            space,
            zone,
            anchor,
            total,
            sign,
            extent,
            min_extent,
        ) {
            cx.notify();
        }
    }
}
