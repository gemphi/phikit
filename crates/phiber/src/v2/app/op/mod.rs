//! The operators only the app can run — the ones that need the window, the
//! loaded file, the undo history or a background job: `wm.quit_blender`,
//! `wm.window_fullscreen_toggle`, `ed.undo` / `ed.redo`, `wm.read_homefile`,
//! `wm.revert_mainfile` here; `render.render` in `render.rs`, `render.view_show`,
//! `render.play_rendered_anim` and `sound.mixdown` in `render_view.rs`; the export and
//! import that a file-browser commit finishes in `file.rs`. Everything else
//! a menu fires is a rule (`state/rule.rs`); this is the rest, in one place.

pub(crate) mod close;
mod file;
mod render;
pub(crate) mod render_open;
mod image;
mod render_view;

use super::BPhiberApp;
use crate::v2::part::file::purpose::{BFilePurpose, purpose};
use crate::v2::state::{BKey as _, BState, CtxKey, ModalKey, OpKey, SceneKey, log};
use crate::v2::view::prim::BAction;
use gpui::{Context, Window};

impl BPhiberApp {
    /// The window's own operators: quit and fullscreen. `true` when taken.
    pub(crate) fn handle_window_op(&mut self, action: &BAction, window: &mut Window) -> bool {
        let BAction::Set(path, _) = action else { return false };
        match OpKey::of(path) {
            Some(OpKey::Quit) => {
                self.closing = true;
                // Flush the session now so the camera and playhead as they rest
                // at exit survive the throttle (`app::session`).
                super::session::BSession::flush(&self.state);
                window.remove_window();
            }
            Some(OpKey::Fullscreen) => {
                window.toggle_fullscreen();
                log::push(
                    if window.is_fullscreen() { "Left fullscreen" } else { "Fullscreen" }
                        .to_string(),
                );
            }
            _ => return false,
        }
        true
    }

    /// The app's operators, before the state sees the action. `true` when
    /// the action was taken here and must not be applied as a `Set`.
    pub(crate) fn handle_app_op(&mut self, action: &BAction, cx: &mut Context<Self>) -> bool {
        let BAction::Set(path, value) = action else { return false };
        // A match on the op, not a family check and then ten string
        // comparisons against ten freshly encoded paths. `OpKey::of` reads the
        // path once — family and tail together — where the chain this replaces
        // encoded up to ten keys per press to answer a question the key
        // already knows, and read as a ladder rather than as the table of
        // operators it is.
        match OpKey::of(path) {
            Some(OpKey::Undo) => self.undo(),
            Some(OpKey::Redo) => self.redo(),
            Some(OpKey::NewFile) => self.new_file(value, cx),
            Some(OpKey::Revert) => self.revert(value, cx),
            Some(OpKey::Save) => self.save_document(value == "incremental"),
            Some(OpKey::FileClose) => self.handle_file_close(value, None, cx),
            Some(OpKey::UndoHistory) => self.undo_to(value.parse().unwrap_or(0)),
            Some(OpKey::Render) => match value.as_str() {
                "animation" => self.render_animation(cx),
                "turnaround" => self.render_turnaround(cx),
                "cancel" => crate::v2::scene::render_job::cancel(),
                "pause" => {
                    crate::v2::scene::render_job::toggle_pause();
                    log::push(if crate::v2::scene::render_job::paused() { "Render paused" } else { "Render resumed" }.to_string());
                }
                "view" => self.render_view(),
                "play" => self.render_play(cx),
                _ => self.render_image(),
            },
            Some(OpKey::ImageCopy) => self.image_copy(cx),
            // Only the window can do these; without one they are reported.
            Some(OpKey::Quit | OpKey::Fullscreen) => log::push("This needs the window".to_string()),
            _ => return false,
        }
        crate::v2::state::menu::close_overlays(&mut self.state);
        if self.state.get(ModalKey::Open) == Some(crate::v2::state::val::V_MODAL_COMMAND) {
            self.state.set(ModalKey::Open, crate::v2::state::val::V_EMPTY);
        }
        true
    }

    fn undo(&mut self) {
        let now = super::undo::snapshot(&self.state);
        match self.history.undo(now) {
            Some(back) => {
                super::undo::restore(&mut self.state, &back);
                log::push(format!("Undo ({} left)", self.history.depth().0));
            }
            None => log::push("Nothing to undo".to_string()),
        }
    }

    /// `ed.undo_history`: undo or redo until the document stands at step
    /// `at` of the listing ("Original" is 0).
    fn undo_to(&mut self, at: usize) {
        let listing = self.history.listing();
        let at = at.min(listing.names.len().saturating_sub(1));
        while self.history.depth().0 > at {
            let now = super::undo::snapshot(&self.state);
            match self.history.undo(now) {
                Some(back) => super::undo::restore(&mut self.state, &back),
                None => break,
            }
        }
        while self.history.depth().0 < at {
            let now = super::undo::snapshot(&self.state);
            match self.history.redo(now) {
                Some(ahead) => super::undo::restore(&mut self.state, &ahead),
                None => break,
            }
        }
        log::push(format!("Undo History: {}", listing.names.get(at).cloned().unwrap_or_default()));
    }

    fn redo(&mut self) {
        let now = super::undo::snapshot(&self.state);
        match self.history.redo(now) {
            Some(ahead) => {
                super::undo::restore(&mut self.state, &ahead);
                log::push(format!("Redo ({} ahead)", self.history.depth().1));
            }
            None => log::push("Nothing to redo".to_string()),
        }
    }

    /// `wm.read_homefile`: the startup file read as an untitled document
    /// (`scene::template`) — the one in the open document's version, or the
    /// major `spec` names (`Layout:3`) — the workspace it names opened.
    pub(crate) fn new_file(&mut self, spec: &str, cx: &mut Context<Self>) {
        use crate::v2::scene::template::BTemplate;
        let open = self.scene_cache.as_ref().map(|(_, m)| m.report.file_version.clone());
        let (template, startup) = BTemplate::pick(spec, open.as_deref());
        self.scene_cache = None;
        crate::v2::scene::source::set_loaded_shared(None);
        crate::v2::scene::source::clear_scene_objects();
        forget_document(&mut self.state);
        self.state.remove(&crate::v2::scene::loaded_path_key());
        self.history.clear();
        if let Some(space) = crate::v2::space::BSpaceType::from_slug(template) {
            crate::v2::space::switch::switch_space(&mut self.state, space);
        }
        self.state.set(ModalKey::Open, crate::v2::state::val::V_EMPTY);
        let name = crate::v2::space::BSpaceType::from_slug(template).map_or(template.to_string(), |s| s.label().to_string());
        log::push(format!("New file ({name}, Blender {} layout)", startup.blender()));
        match startup.startup_path() {
            Ok(path) => {
                self.untitled = true;
                self.start_import(path.to_string_lossy().to_string(), cx);
            }
            Err(why) => log::push(format!("The startup file could not be written: {why}")),
        }
    }

    /// `wm.revert_mainfile`: asked first (`modal::revert`), then the file
    /// read again and the edits dropped.
    fn revert(&mut self, answer: &str, cx: &mut Context<Self>) {
        let Some((path, _)) = self.scene_cache.clone() else {
            log::push("Nothing to revert: no file is open".to_string());
            return;
        };
        if answer != crate::v2::modal::revert::CONFIRMED {
            self.state.set(ModalKey::Open, crate::v2::state::val::V_MODAL_REVERT);
            return;
        }
        self.state.set(ModalKey::Open, crate::v2::state::val::V_EMPTY);
        forget_document(&mut self.state);
        self.history.clear();
        log::push("Reverting".to_string());
        self.start_import(path, cx);
    }
}

/// What a file commit was for, read before the commit's own rule may clear it.
pub(crate) fn purpose_of(state: &BState) -> BFilePurpose {
    purpose(state)
}

/// Drop everything the document held: the selection, the edits, the range.
fn forget_document(state: &mut BState) {
    for key in [SceneKey::Added, SceneKey::Deleted, SceneKey::Copied, SceneKey::Renamed] {
        state.remove(key);
    }
    for key in [CtxKey::Object, CtxKey::Selected, CtxKey::Frame, CtxKey::FrameStart, CtxKey::FrameEnd, CtxKey::Fps, CtxKey::Playing, CtxKey::Markers] {
        state.remove(key);
    }
    let outliner = {
        let encoded = crate::v2::state::OutlinerKey::Search.path().encode();
        let mut parts: Vec<&str> = encoded.split('.').collect();
        parts.truncate(2);
        format!("{}.", parts.join("."))
    };
    let restrictions: Vec<String> = state.iter_keys().filter(|k| k.starts_with(&outliner)).cloned().collect();
    for key in restrictions {
        state.remove_encoded(&key);
    }
}

