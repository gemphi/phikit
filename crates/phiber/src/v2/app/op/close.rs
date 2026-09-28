//! Unsaved changes — Blender's `wm->file_saved`. A document is marked
//! changed when an action records an undo step (`WM_file_tag_modified` from
//! `ED_undo_push`) and clean again when it is read or written; the window's
//! title carries the `* ` (`wm_window_title`). Quit, New File and Open ask
//! first while it is changed (`wm_operator_close_file_dialog_if_needed`):
//! the dialog is `modal::file_close`, what to do after it is answered is
//! `ctx.modal.file_close.then`, and this is where the answer lands.

use super::super::BPhiberApp;
use crate::v2::modal::file_close::BCloseAnswer;
use crate::v2::part::file::purpose::BFilePurpose;
use crate::v2::state::{BKey as _, BState, FileKey, ModalKey, OpKey, SceneKey, val::{V_EMPTY, V_MODAL_FILE_BROWSER, V_MODAL_FILE_CLOSE, V_TRUE}};
use crate::v2::view::prim::BAction;
use gpui::{Context, Window};

/// Blender's `wm_file_or_session_data_has_unsaved_changes`.
pub fn has_unsaved_changes(state: &BState) -> bool {
    state.get(SceneKey::Unsaved) == Some(V_TRUE)
}

/// `WM_file_tag_modified`.
pub fn tag_modified(state: &mut BState) {
    state.set(SceneKey::Unsaved, V_TRUE);
}

/// The document was read or written: `wm->file_saved = 1`.
pub fn tag_saved(state: &mut BState) {
    state.remove(SceneKey::Unsaved);
}

/// What the window's title bar says (`wm_window_title`): `* ` while the
/// document has unsaved changes, its name and, in brackets, its path — or
/// `(Unsaved)` for a document never saved — then the app.
pub fn window_title(state: &BState) -> String {
    let mut title = if has_unsaved_changes(state) { "* ".to_string() } else { String::new() };
    match state.get(&crate::v2::scene::loaded_path_key()).filter(|p| !p.is_empty()) {
        Some(path) => {
            let name = path.rsplit(['/', '\\']).next().unwrap_or(path);
            title.push_str(name.rsplit_once('.').map_or(name, |(stem, _)| stem));
            title.push_str(&format!(" [{path}]"));
        }
        None => title.push_str("(Unsaved)"),
    }
    title.push_str(" - Phiber");
    title
}

impl BPhiberApp {
    /// The action that must wait for the dialog, as `then` spells it: quit,
    /// a new file, opening the browser for a file, or opening a named one
    /// (Recent). A commit from the browser that Open already asked for is
    /// not asked again.
    fn asks_first(&self, action: &BAction) -> Option<String> {
        let BAction::Set(path, value) = action else { return None };
        if *path == OpKey::Quit.path() {
            return Some("quit".to_string());
        }
        if *path == OpKey::NewFile.path() {
            return Some(format!("new_file:{value}"));
        }
        if *path == FileKey::Purpose.path() && value == BFilePurpose::Open.slug() {
            return Some("open".to_string());
        }
        if *path == crate::v2::scene::selected_key()
            && !value.is_empty()
            && self.state.get(ModalKey::Open) != Some(V_MODAL_FILE_BROWSER)
            && super::purpose_of(&self.state) == BFilePurpose::Open
            && super::super::import::should_import(self.state.get(&crate::v2::scene::loaded_path_key()), Some(value))
        {
            return Some(format!("open:{value}"));
        }
        None
    }

    /// `wm_operator_close_file_dialog_if_needed`: `true` when the dialog
    /// took the action, to be run again once it is answered.
    pub(crate) fn asks_to_save_first(&mut self, action: &BAction) -> bool {
        if self.answered_close || !has_unsaved_changes(&self.state) {
            return false;
        }
        let Some(then) = self.asks_first(action) else { return false };
        self.state.set(ModalKey::CloseThen, then);
        crate::v2::state::menu::close_overlays(&mut self.state);
        self.state.set(ModalKey::Open, V_MODAL_FILE_CLOSE);
        true
    }

    /// `ctx.op.file_close` with the window in hand: `true` when taken.
    pub(crate) fn answers_file_close(&mut self, action: &BAction, window: &mut Window, cx: &mut Context<Self>) -> bool {
        let BAction::Set(path, answer) = action else { return false };
        if *path != OpKey::FileClose.path() {
            return false;
        }
        self.handle_file_close(answer, Some(window), cx);
        true
    }

    /// The dialog's answer. Save writes the file first — an untitled one
    /// asks where instead and stops there, as Blender does — Don't Save
    /// goes straight on, Cancel stays.
    pub(crate) fn handle_file_close(&mut self, answer: &str, window: Option<&mut Window>, cx: &mut Context<Self>) {
        let then = self.state.get(ModalKey::CloseThen).unwrap_or("").to_string();
        self.state.remove(ModalKey::CloseThen);
        self.state.set(ModalKey::Open, V_EMPTY);
        match BCloseAnswer::from_slug(answer) {
            Some(BCloseAnswer::Discard) => {}
            Some(BCloseAnswer::Save) => {
                if self.state.get(&crate::v2::scene::loaded_path_key()).filter(|p| !p.is_empty()).is_none() {
                    self.save_document(false);
                    return;
                }
                self.save_document(false);
                if has_unsaved_changes(&self.state) {
                    return;
                }
            }
            _ => return,
        }
        let again = match then.split_once(':') {
            None if then == "quit" => BAction::Set(OpKey::Quit.path(), String::new()),
            None if then == "open" => BAction::Set(FileKey::Purpose.path(), BFilePurpose::Open.slug().to_string()),
            Some(("new_file", spec)) => BAction::Set(OpKey::NewFile.path(), spec.to_string()),
            Some(("open", path)) => BAction::Set(crate::v2::scene::selected_key(), path.to_string()),
            _ => return,
        };
        self.answered_close = true;
        match window {
            Some(window) => self.handle_action_in(&again, window, cx),
            None => self.handle_action(&again, cx),
        }
        self.answered_close = false;
    }
}
