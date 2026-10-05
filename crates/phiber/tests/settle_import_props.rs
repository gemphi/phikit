//! Property 6.4 — settle_import state transitions.
//!
//! Feature: blend-file-mesh-loading, Property 6.4: settle_import state transitions
//! Validates: Requirements 1.4, 3.3, 3.4, 3.6, 8.5
//!
//! After `settle_import`:
//! - Loading is ALWAYS cleared.
//! - Loaded: clears error, sets loaded_path, returns the model.
//! - Failed: sets 1..=256 char error, retains loaded_path, returns None.
//! - TimedOut: sets 1..=256 char error, retains loaded_path, returns None.

use phiber::{
    phiphx::scene::{
        SceneLoadError, SceneMesh, SceneModel, SceneNormals, SceneObject,
    },
    v2::app::import::{ImportOutcome, settle_import},
    v2::scene::{error_key, loaded_path_key, loading_key},
    v2::state::BState,
};
use proptest::prelude::*;

fn char_len(s: &str) -> usize {
    s.chars().count()
}

fn scene_obj(name: &str) -> SceneObject {
    SceneObject {
        name: name.to_string(),
        mesh: SceneMesh {
            vertices: vec![[0.0, 0.0, 0.0]],
            indices: vec![],
            normals: SceneNormals::PerVertex(vec![[0.0, 0.0, 1.0]]),
            corners: Default::default(),
        }.into(),
        location: [0.0, 0.0, 0.0],
        scale: [1.0, 1.0, 1.0],
        matrix_world: phiber::phiphx::scene::IDENTITY_M4,
        materials: Vec::new(),
        material_indices: Default::default(),
        uvs: Default::default(),
        color: [1.0; 4],
        shadow: Default::default(),
    }
}

fn model_at(path: &str) -> SceneModel {
    SceneModel {
        objects: vec![scene_obj("Cube")],
        source_path: Some(path.to_string()),
    ..Default::default()
    }
}

/// Strategy: whether the prior state has loading=true.
fn arb_loading_flag() -> impl Strategy<Value = bool> {
    any::<bool>()
}

/// Strategy: an optional prior loaded_path.
fn arb_prior_path() -> impl Strategy<Value = Option<String>> {
    prop::option::of(
        proptest::string::string_regex("/[a-z0-9/._-]{1,24}\\.blend")
            .expect("path regex valid"),
    )
}

/// Strategy: an optional prior error message.
fn arb_prior_error() -> impl Strategy<Value = Option<String>> {
    prop::option::of(proptest::string::string_regex("[a-z ]{1,64}").expect("err regex valid"))
}

/// Strategy: a source path for a successfully loaded model.
fn arb_source_path() -> impl Strategy<Value = Option<String>> {
    prop::option::of(
        proptest::string::string_regex("/[a-z0-9/._-]{1,24}\\.blend")
            .expect("src path regex valid"),
    )
}

/// Strategy: a SceneLoadError variant with an arbitrary detail message.
fn arb_error() -> impl Strategy<Value = SceneLoadError> {
    prop_oneof![
        proptest::string::string_regex("[a-z ]{0,64}")
            .expect("detail regex valid")
            .prop_map(SceneLoadError::Inaccessible),
        proptest::string::string_regex("[a-z ]{0,64}")
            .expect("detail regex valid")
            .prop_map(SceneLoadError::Malformed),
        proptest::string::string_regex("[a-z ]{0,64}")
            .expect("detail regex valid")
            .prop_map(SceneLoadError::Unsupported),
    ]
}

/// Build a BState with the given prior flags.
fn make_state(
    has_loading: bool,
    prior_path: Option<&str>,
    prior_error: Option<&str>,
) -> BState {
    let mut s = BState::new();
    if has_loading {
        s.set(&loading_key(), "true");
    }
    if let Some(p) = prior_path {
        s.set(&loaded_path_key(), p);
    }
    if let Some(e) = prior_error {
        s.set(&error_key(), e);
    }
    s
}

proptest! {
    // Feature: blend-file-mesh-loading, Property 6.4: settle_import state transitions

    /// Loading is always cleared after settle, regardless of outcome.
    #[test]
    fn loading_always_cleared_after_settle(
        has_loading in arb_loading_flag(),
        prior_path in arb_prior_path(),
        prior_error in arb_prior_error(),
        outcome_tag in 0u8..3,
        source_path in arb_source_path(),
        err in arb_error(),
    ) {
        let mut s = make_state(
            has_loading,
            prior_path.as_deref(),
            prior_error.as_deref(),
        );

        let outcome = match outcome_tag % 3 {
            0 => ImportOutcome::Loaded(SceneModel {
                objects: vec![],
                source_path,
                ..Default::default()
            }),
            1 => ImportOutcome::Failed(err),
            _ => ImportOutcome::TimedOut,
        };
        settle_import(&mut s, outcome);

        prop_assert_eq!(s.get(&loading_key()), None, "loading must be cleared");
    }

    /// Loaded outcome: error cleared, loaded_path set from model source_path.
    #[test]
    fn loaded_clears_error_and_sets_path(
        has_loading in arb_loading_flag(),
        prior_path in arb_prior_path(),
        source_path in arb_source_path(),
    ) {
        let mut s = make_state(has_loading, prior_path.as_deref(), Some("prior error"));

        let model = SceneModel {
            objects: vec![],
            source_path: source_path.clone(),
        ..Default::default()
        };
        let returned = settle_import(&mut s, ImportOutcome::Loaded(model));

        prop_assert!(returned.is_some(), "Loaded must return the model");
        prop_assert_eq!(s.get(&error_key()), None, "error must be cleared on success");
        prop_assert_eq!(
            s.get(&loaded_path_key()),
            source_path.as_deref(),
            "loaded_path must match model source_path"
        );
    }

    /// Failed outcome: 1..=256 char error set, loaded_path retained.
    #[test]
    fn failed_sets_bounded_error_retains_path(
        has_loading in arb_loading_flag(),
        prior_path in arb_prior_path(),
        err in arb_error(),
    ) {
        let prior_ref = prior_path.as_deref();
        let mut s = make_state(has_loading, prior_ref, None);

        let returned = settle_import(&mut s, ImportOutcome::Failed(err));

        prop_assert!(returned.is_none(), "Failed must return None");
        let msg = s.get(&error_key());
        prop_assert!(msg.is_some(), "Failed must set an error message");
        let len = char_len(msg.unwrap());
        prop_assert!(
            (1..=256).contains(&len),
            "error length {len} not in 1..=256"
        );
        prop_assert_eq!(
            s.get(&loaded_path_key()),
            prior_ref,
            "loaded_path must be retained on failure"
        );
    }

    /// TimedOut outcome: 1..=256 char error set, loaded_path retained.
    #[test]
    fn timeout_sets_bounded_error_retains_path(
        has_loading in arb_loading_flag(),
        prior_path in arb_prior_path(),
    ) {
        let prior_ref = prior_path.as_deref();
        let mut s = make_state(has_loading, prior_ref, None);

        let returned = settle_import(&mut s, ImportOutcome::TimedOut);

        prop_assert!(returned.is_none(), "TimedOut must return None");
        let msg = s.get(&error_key());
        prop_assert!(msg.is_some(), "TimedOut must set an error message");
        let len = char_len(msg.unwrap());
        prop_assert!(
            (1..=256).contains(&len),
            "timeout error length {len} not in 1..=256"
        );
        prop_assert_eq!(
            s.get(&loaded_path_key()),
            prior_ref,
            "loaded_path must be retained on timeout"
        );
    }
}

// --- Deterministic unit cases ---

#[test]
fn loaded_returns_model_and_sets_path() {
    let mut s = BState::new();
    s.set(&loading_key(), "true");
    s.set(&error_key(), "stale error");

    let returned = settle_import(&mut s, ImportOutcome::Loaded(model_at("/dir/model.blend")));

    assert_eq!(s.get(&loading_key()), None);
    assert_eq!(s.get(&error_key()), None);
    assert_eq!(s.get(&loaded_path_key()), Some("/dir/model.blend"));
    assert!(returned.is_some());
}

#[test]
fn failed_retains_prior_loaded_path() {
    let mut s = BState::new();
    s.set(&loaded_path_key(), "/prior/scene.blend");
    s.set(&loading_key(), "true");

    let returned = settle_import(
        &mut s,
        ImportOutcome::Failed(SceneLoadError::Malformed("bad SDNA".to_string())),
    );

    assert_eq!(s.get(&loading_key()), None);
    assert_eq!(s.get(&loaded_path_key()), Some("/prior/scene.blend"));
    assert!(returned.is_none());
    let msg = s.get(&error_key()).unwrap();
    assert!((1..=256).contains(&char_len(msg)));
    assert!(msg.contains("malformed"));
}

#[test]
fn timeout_retains_prior_path_and_sets_error() {
    let mut s = BState::new();
    s.set(&loaded_path_key(), "/prior/scene.blend");
    s.set(&loading_key(), "true");

    settle_import(&mut s, ImportOutcome::TimedOut);

    assert_eq!(s.get(&loading_key()), None);
    assert_eq!(s.get(&loaded_path_key()), Some("/prior/scene.blend"));
    let msg = s.get(&error_key()).unwrap();
    assert!((1..=256).contains(&char_len(msg)));
}

#[test]
fn success_after_failure_clears_error() {
    let mut s = BState::new();
    settle_import(
        &mut s,
        ImportOutcome::Failed(SceneLoadError::Inaccessible("no file".to_string())),
    );
    assert!(s.get(&error_key()).is_some());

    settle_import(&mut s, ImportOutcome::Loaded(model_at("/next.blend")));
    assert_eq!(s.get(&error_key()), None);
    assert_eq!(s.get(&loaded_path_key()), Some("/next.blend"));
}
