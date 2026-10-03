//! Tests for `settle_import`.
use super::*;

    // ---- settle_import ------------------------------------------------------

    use crate::phiphx::scene::{SceneMesh, SceneNormals, SceneObject};

    /// A minimal model carrying a `source_path`, for the success path.
    fn model_at(path: &str) -> SceneModel {
        SceneModel {
            objects: vec![SceneObject {
                name: "Cube".to_string(),
                mesh: SceneMesh {
                    vertices: vec![[0.0, 0.0, 0.0]],
                    indices: vec![],
                    normals: SceneNormals::PerVertex(vec![[0.0, 0.0, 1.0]]),
                    corners: Default::default(),
                }.into(),
                location: [0.0, 0.0, 0.0],
                scale: [1.0, 1.0, 1.0],
                matrix_world: crate::phiphx::scene::IDENTITY_M4,
                materials: Vec::new(),
                material_indices: Default::default(),
                uvs: Default::default(),
                color: [1.0; 4],
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
