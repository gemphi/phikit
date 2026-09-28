//! GPUI action dispatch. State transitions are pure and shared with tests;
//! this layer is responsible only for notifying GPUI after a transition.

use super::BPhiberApp;
use super::import::should_import;
use crate::v2::scene::{loaded_path_key, selected_key};
use crate::v2::space::BSpaceType;
use crate::v2::area::size::{apply_delta, apply_drag_with_min_extent, normalize};
use crate::v2::state::{BKey as _, action::apply_action};
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
        let outcome = crate::v2::state::edit::key(&mut self.state, key, typed);
        if outcome == BEdit::Ignored {
            return false;
        }
        // Return commits: run the value back through the normal action path so
        // everything that watches the key still fires (a typed folder path
        // navigates, a picked file imports).
        if outcome == BEdit::Confirmed {
            if let Some(path) = crate::v2::state::edit::field_path(&target) {
                let value = self.state.get_encoded(&target).unwrap_or("").to_string();
                self.handle_action(&BAction::Set(path, value), cx);
            }
        }
        cx.notify();
        true
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
        // 0a. Unsaved changes: quit, new and open wait for the dialog.
        if self.asks_to_save_first(action) {
            cx.notify();
            return;
        }
        // 0b. The app's own operators (`app/op.rs`) — undo, new, revert,
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

        // 1. Apply the pure state transition (unchanged behavior), and log
        //    what it changed for the status bar's action log. The document
        //    before it is an undo step when the action changed it.
        let before = self.state.clone();
        let document = super::undo::snapshot(&self.state);
        apply_action(&mut self.state, action);
        let name = crate::v2::state::log::record(action, &before, &self.state).unwrap_or_else(|| "Edit".to_string());
        let holding = crate::v2::scene::grab::BMarkerGrab::active(&self.state) || crate::v2::scene::xform::BTransformModal::active(&self.state);
        if self.history.record_held(document, &super::undo::snapshot(&self.state), &name, holding) {
            super::op::close::tag_modified(&mut self.state);
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
            Some((file, purpose)) => self.handle_file_commit(&file, purpose),
            None => {}
        }

        // 3. Playback starts its ticker when Play turns on.
        self.ensure_playback(cx);

        // 4. Opening the render window is Blender's F12: it starts the render
        //    as well as showing the place the result will appear.
        self.ensure_render();

        // 5. Always notify after the synchronous part so the loading flag (and
        //    any other state change) shows on this render pass (as before).
        cx.notify();
    }

    /// Start a final render when the render window has just been opened
    /// (`Render ▸ Render Image`, F12) and one is not already under way.
    ///
    /// Blender renders on opening that window rather than making the user ask
    /// twice, and leaves the result there until the next F12 — so a window
    /// that is already showing a finished render is not re-rendered.
    fn ensure_render(&mut self) {
        use crate::v2::scene::render_job;
        let open = self.state.get(crate::v2::state::ModalKey::Open) == Some(crate::v2::state::val::V_MODAL_RENDER);
        if !open || render_job::rendering() {
            return;
        }
        if render_job::shot().0.is_some_and(|s| s.done) {
            return;
        }
        self.render_now();
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
