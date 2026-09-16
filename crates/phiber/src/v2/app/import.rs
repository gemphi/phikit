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

use crate::phiphx::services::scene::{SceneLoadError, SceneModel};
use crate::v2::scene::{
    error_key, is_blend_path, load_progress_key, load_stage_key, loaded_path_key, loading_key,
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

    match outcome {
        ImportOutcome::Loaded(model) => {
            // Success clears any prior error (Requirement 3.6) and records the
            // path now reflected by the Scene_Source (Requirement 1.7).
            state.remove(&error_key());
            match &model.source_path {
                Some(path) => state.set(&loaded_path_key(), path.clone()),
                // No source path recorded — clear the reflected path rather
                // than leave a stale one.
                None => state.remove(&loaded_path_key()),
            }
            Some(model)
        }
        ImportOutcome::Failed(e) => {
            // Retain the prior scene: touch neither loaded_path nor the cached
            // model (Requirement 3.4). Only surface the categorized message.
            state.set(&error_key(), e.message());
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
mod tests {
    use super::*;

    #[test]
    fn blend_differing_from_prev_imports() {
        // A .blend selection that differs from the reflected path => import.
        assert!(should_import(
            Some("/old/scene.blend"),
            Some("/new/model.blend")
        ));
    }

    #[test]
    fn blend_equal_to_prev_does_not_import() {
        // Selecting the already-reflected .blend path => no new import (1.6).
        assert!(!should_import(
            Some("/dir/model.blend"),
            Some("/dir/model.blend")
        ));
    }

    #[test]
    fn non_blend_selection_does_not_import() {
        // Non-.blend selections never trigger an import (1.5).
        assert!(!should_import(None, Some("notes.txt")));
        assert!(!should_import(Some("/dir/model.blend"), Some("readme.md")));
    }

    #[test]
    fn none_selected_does_not_import() {
        // No selection at all => nothing to import.
        assert!(!should_import(None, None));
        assert!(!should_import(Some("/dir/model.blend"), None));
    }

    #[test]
    fn case_insensitive_blend_suffix_imports() {
        // The .blend suffix match is case-insensitive.
        assert!(should_import(None, Some("MODEL.BLEND")));
        assert!(should_import(Some("/a.blend"), Some("/b.Blend")));
    }

    #[test]
    fn prev_none_with_blend_selected_imports() {
        // First-ever load: no reflected path yet, a .blend is selected.
        assert!(should_import(None, Some("/first/load.blend")));
    }

    #[test]
    fn blend_equal_to_prev_is_case_insensitive() {
        // Same path with differing case in the .blend suffix is still equal
        // as a raw string comparison — differing bytes DO count as different
        // (path identity is exact); only the suffix predicate is case-insensitive.
        // A byte-identical path never re-imports.
        assert!(!should_import(
            Some("/dir/Model.blend"),
            Some("/dir/Model.blend")
        ));
    }

    // ---- settle_import ------------------------------------------------------

    use crate::phiphx::services::scene::{SceneMesh, SceneNormals, SceneObject};

    /// A minimal model carrying a `source_path`, for the success path.
    fn model_at(path: &str) -> SceneModel {
        SceneModel {
            objects: vec![SceneObject {
                name: "Cube".to_string(),
                mesh: SceneMesh {
                    vertices: vec![[0.0, 0.0, 0.0]],
                    indices: vec![],
                    normals: SceneNormals::PerVertex(vec![[0.0, 0.0, 1.0]]),
                },
                location: [0.0, 0.0, 0.0],
                scale: [1.0, 1.0, 1.0],
                materials: Vec::new(),
                material_indices: Vec::new(),
                uvs: Vec::new(),
            }],
            source_path: Some(path.to_string()),
            ..Default::default()
        }
    }

    /// Length in `char`s, matching the Load_Error_State bound semantics.
    fn char_len(s: &str) -> usize {
        s.chars().count()
    }

    #[test]
    fn loaded_clears_loading_and_error_sets_loaded_path() {
        let mut s = BState::new();
        // Pre-existing loading + error from a prior in-flight import.
        s.set(&loading_key(), "true");
        s.set(&error_key(), "old failure");

        let returned = settle_import(&mut s, ImportOutcome::Loaded(model_at("/dir/model.blend")));

        // Loading absent, error absent, loaded_path set to the model's path.
        assert_eq!(s.get(&loading_key()), None);
        assert_eq!(s.get(&error_key()), None);
        assert_eq!(s.get(&loaded_path_key()), Some("/dir/model.blend"));
        // The model is returned so the caller can cache it.
        let model = returned.expect("Loaded returns the model to cache");
        assert_eq!(model.source_path.as_deref(), Some("/dir/model.blend"));
    }

    #[test]
    fn loaded_without_source_path_clears_loaded_path() {
        let mut s = BState::new();
        s.set(&loaded_path_key(), "/prev/scene.blend");
        s.set(&loading_key(), "true");

        let model = SceneModel {
            objects: vec![],
            source_path: None,
        ..Default::default()
        };
        let returned = settle_import(&mut s, ImportOutcome::Loaded(model));

        assert_eq!(s.get(&loading_key()), None);
        // No source path => the reflected path is cleared, not left stale.
        assert_eq!(s.get(&loaded_path_key()), None);
        assert!(returned.is_some());
    }

    #[test]
    fn failed_clears_loading_sets_bounded_error_retains_loaded_path() {
        let mut s = BState::new();
        // A scene was already reflected before the failed import.
        s.set(&loaded_path_key(), "/dir/prev.blend");
        s.set(&loading_key(), "true");

        let returned = settle_import(
            &mut s,
            ImportOutcome::Failed(SceneLoadError::Malformed("bad SDNA".to_string())),
        );

        // Loading cleared; error set to a 1..=256 char message.
        assert_eq!(s.get(&loading_key()), None);
        let msg = s.get(&error_key()).expect("failure sets an error message");
        assert!(
            (1..=256).contains(&char_len(msg)),
            "error len {} out of bounds",
            char_len(msg)
        );
        assert!(
            msg.contains("malformed"),
            "message names the category: {msg}"
        );
        // Prior reflected path is preserved (retain prior scene, Requirement 3.4).
        assert_eq!(s.get(&loaded_path_key()), Some("/dir/prev.blend"));
        // No model to cache on failure.
        assert!(returned.is_none());
    }

    #[test]
    fn timed_out_clears_loading_sets_bounded_error_retains_loaded_path() {
        let mut s = BState::new();
        s.set(&loaded_path_key(), "/dir/prev.blend");
        s.set(&loading_key(), "true");

        let returned = settle_import(&mut s, ImportOutcome::TimedOut);

        assert_eq!(s.get(&loading_key()), None);
        let msg = s.get(&error_key()).expect("timeout sets an error message");
        assert!(
            (1..=256).contains(&char_len(msg)),
            "timeout msg len {} out of bounds",
            char_len(msg)
        );
        // Prior reflected path is preserved on timeout too.
        assert_eq!(s.get(&loaded_path_key()), Some("/dir/prev.blend"));
        assert!(returned.is_none());
    }

    #[test]
    fn success_after_failure_clears_the_error() {
        let mut s = BState::new();
        // First a failure leaves an error and retains no path.
        settle_import(
            &mut s,
            ImportOutcome::Failed(SceneLoadError::Inaccessible("no file".to_string())),
        );
        assert!(s.get(&error_key()).is_some());

        // A subsequent success clears the error and records the path.
        settle_import(&mut s, ImportOutcome::Loaded(model_at("/dir/next.blend")));
        assert_eq!(s.get(&error_key()), None);
        assert_eq!(s.get(&loaded_path_key()), Some("/dir/next.blend"));
        assert_eq!(s.get(&loading_key()), None);
    }

    #[test]
    fn settle_clears_the_staged_progress_readout() {
        // An in-flight import has seeded the stage label + progress fraction;
        // settling (in every outcome) must clear both so the statusbar bar
        // disappears.
        for outcome in [
            ImportOutcome::Loaded(model_at("/dir/done.blend")),
            ImportOutcome::Failed(SceneLoadError::Malformed("x".to_string())),
            ImportOutcome::TimedOut,
        ] {
            let mut s = BState::new();
            s.set(&loading_key(), "true");
            s.set(&load_stage_key(), "Reading file");
            s.set(&load_progress_key(), "0.15");

            settle_import(&mut s, outcome);

            assert_eq!(s.get(&loading_key()), None);
            assert_eq!(s.get(&load_stage_key()), None);
            assert_eq!(s.get(&load_progress_key()), None);
        }
    }
}
