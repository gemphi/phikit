//! Pure, gpui-free core for import orchestration.
//!
//! This module holds the small, unit-testable predicates and (later) settle
//! logic that decide *whether* and *how* a `.blend` import should run. It has
//! no gpui, no theme widgets, and no `apple::puikit`/`apple::suikit` imports,
//! so the kit boundary (steering: "One UI Kit Per App") stays intact and the
//! logic can be asserted directly in pure unit tests.
//!
//! The gpui spawn/notify shell around this core lives in `app::handle`
//! (task 7.x) — this file deliberately contains only pure functions.

use crate::phiphx::scene::{SceneLoadError, SceneModel};
use crate::v2::scene::{
    error_key, is_blend_path, load_cancel_key, load_progress_key, load_stage_key,
    loaded_path_key, loading_key,
};
use crate::v2::state::BState;

/// Decide whether a file selection should initiate a new blend import.
///
/// Returns `true` iff `selected` is `Some(path)` where `path` ends in
/// `.blend` (case-insensitive) AND it differs from the path currently
/// reflected by the Scene_Source (`prev_loaded_path`).
///
/// Returns `false` when:
/// - `selected` is `None` (nothing selected),
/// - the selected name is not a `.blend` path (Requirement 1.5), or
/// - the selected `.blend` path equals `prev_loaded_path` — the import
///   currently reflected — so no new import is initiated (Requirement 1.6).
///
/// Arguments are `Option<&str>` to match how `BState::get` returns the
/// `file.selected` and `scene.loaded_path` string values (`Option<&str>`):
/// `None` means the key is absent.
///
/// _Requirements: 1.2, 1.6, 1.5_
pub fn should_import(prev_loaded_path: Option<&str>, selected: Option<&str>) -> bool {
    match selected {
        Some(path) if is_blend_path(path) => prev_loaded_path != Some(path),
        _ => false,
    }
}

/// The settled result of an import task, handed to [`settle_import`].
///
/// The gpui spawn/notify shell (task 7.x) turns a completed background task —
/// a `Result<SceneModel, SceneLoadError>` — or an elapsed watchdog into one of
/// these variants and calls `settle_import` on the UI thread. Keeping this as a
/// plain enum (no gpui, no `Result` timeout ambiguity) lets the settle logic be
/// asserted directly in pure unit tests.
pub enum ImportOutcome {
    /// The import completed successfully with a mapped scene.
    Loaded(SceneModel),
    /// The import failed with a categorized error.
    Failed(SceneLoadError),
    /// The import did not settle within the watchdog window (Requirement 8.4).
    TimedOut,
}

/// A short, fixed timeout message set when an import exceeds the 30s watchdog.
///
/// It is a non-empty ASCII string well under the 256-char Load_Error_State
/// bound (Requirement 3.3, 8.4), so it satisfies the 1..=256 char guarantee.
const TIMEOUT_MESSAGE: &str = "timeout: blend import exceeded 30s";

/// Apply a settled import outcome to the scalar scene state keys.
///
/// This is the pure, unit-testable half of import orchestration. It mutates
/// only the small scalar keys in `BState` (`scene.loading`, `scene.error`,
/// `scene.loaded_path`); the heavy [`SceneModel`] is NOT stringified into state
/// — on success it is returned so the caller can cache it separately (keyed by
/// `loaded_path`) for the Scene_Source to read.
///
/// In every case the loading flag is cleared (Requirement 1.4, 8.5).
///
/// - [`Loaded(model)`](ImportOutcome::Loaded): clear the error, set
///   `loaded_path` to the model's `source_path` (when present), clear loading,
///   and return `Some(model)` so the caller can cache it (Requirement 1.7, 3.6).
/// - [`Failed(e)`](ImportOutcome::Failed): set `error` to `e.message()` (a
///   1..=256 char string), clear loading, and leave `loaded_path` and the
///   cached scene untouched so the prior scene is retained (Requirement 3.3,
///   3.4). Returns `None`.
/// - [`TimedOut`](ImportOutcome::TimedOut): set `error` to a short timeout
///   message, clear loading, and retain the prior scene (Requirement 8.4, 8.5).
///   Returns `None`.
///
/// _Requirements: 1.4, 3.3, 3.4, 3.6, 8.4, 8.5_
pub fn settle_import(state: &mut BState, outcome: ImportOutcome) -> Option<SceneModel> {
    // The loading flag and the staged-progress readout are cleared on every
    // settled outcome — the bar and stage label disappear once the import
    // resolves (success, failure, or timeout).
    state.remove(&loading_key());
    state.remove(&load_stage_key());
    state.remove(&load_progress_key());
    state.remove(&load_cancel_key());

    match outcome {
        ImportOutcome::Loaded(model) => {
            // Success clears any prior error (Requirement 3.6) and records the
            // path now reflected by the Scene_Source (Requirement 1.7).
            state.remove(&error_key());
            super::seed::seed_view_layer(state, &model);
            super::seed::seed_render_props(state, &model);
            super::seed::seed_output_props(state, &model);
            crate::v2::part::playback::settings::seed(state, &model.playback);
            super::seed::report_loaded(state, &model);
            super::load_ui::restore_ui(state, &model);
            // Session resume keeps the playhead you left, not the file's saved
            // frame (the session is newer than the last save). Clamped into the
            // scene's range so a resumed frame past the file's end is valid.
            if let Some(frame) = state.get_encoded(super::session::RESUME_FRAME).and_then(|f| f.parse::<i64>().ok()) {
                let lo = model.timeline.frame_start as i64;
                let hi = model.timeline.frame_end as i64;
                state.set(crate::v2::state::CtxKey::Frame, frame.clamp(lo.min(hi), lo.max(hi)).to_string());
                state.remove_encoded(super::session::RESUME_FRAME);
            }
            match &model.source_path {
                Some(path) => state.set(&loaded_path_key(), path.clone()),
                // No source path recorded — clear the reflected path rather
                // than leave a stale one.
                None => state.remove(&loaded_path_key()),
            }
            Some(model)
        }
        ImportOutcome::Failed(SceneLoadError::Cancelled) => {
            // A user cancel is not an error: retain the prior scene and show
            // nothing (Blender's job cancel is silent).
            None
        }
        ImportOutcome::Failed(e) => {
            // Retain the prior scene: touch neither loaded_path nor the cached
            // model (Requirement 3.4). Only surface the categorized message.
            state.set(&error_key(), e.message());
            let path = state.get(&crate::v2::scene::selected_key()).map(str::to_string);
            super::seed::report_failed(state, path.as_deref(), &e.message());
            None
        }
        ImportOutcome::TimedOut => {
            // Same retention rule as a failure; a short, bounded message.
            state.set(&error_key(), TIMEOUT_MESSAGE.to_string());
            None
        }
    }
}

#[cfg(test)]
#[path = "import_tests.rs"]
mod tests;
#[cfg(test)]
#[path = "settle_tests.rs"]
mod settle_tests;
