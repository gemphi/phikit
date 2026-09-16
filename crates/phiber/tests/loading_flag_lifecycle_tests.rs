//! Task 7.3 — Loading-flag lifecycle unit tests.
//!
//! Feature: blend-file-mesh-loading
//! Validates: Requirements 1.3, 1.4, 8.1, 8.5
//!
//! Tests the pure `settle_import` function for each outcome: spawn → success,
//! spawn → failure, spawn → timeout. Verifies that `scene.loading` is
//! true before settlement (simulated by manually setting the flag) and
//! absent after, on every outcome.

use phiber::{
    phiphx::services::scene::{
        SceneLoadError, SceneMesh, SceneModel, SceneNormals, SceneObject,
    },
    v2::app::import::{ImportOutcome, settle_import},
    v2::scene::{error_key, loaded_path_key, loading_key},
    v2::state::BState,
};

fn char_len(s: &str) -> usize {
    s.chars().count()
}

fn minimal_model(path: &str) -> SceneModel {
    SceneModel {
        objects: vec![SceneObject {
            name: "Cube".to_string(),
            mesh: SceneMesh {
                vertices: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
                indices: vec![0, 1, 2],
                normals: SceneNormals::PerFace(vec![[0.0, 0.0, 1.0]]),
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

// --- spawn → success ---

#[test]
fn spawn_success_loading_cleared() {
    let mut s = BState::new();
    // Simulate "spawn": loading=true was set before background task started.
    s.set(&loading_key(), "true");
    assert_eq!(s.get(&loading_key()), Some("true"), "precondition: loading set");

    settle_import(&mut s, ImportOutcome::Loaded(minimal_model("/dir/model.blend")));

    assert_eq!(s.get(&loading_key()), None, "loading must be absent after success");
}

#[test]
fn spawn_success_error_cleared() {
    let mut s = BState::new();
    s.set(&loading_key(), "true");
    s.set(&error_key(), "prior error");

    settle_import(&mut s, ImportOutcome::Loaded(minimal_model("/dir/model.blend")));

    assert_eq!(s.get(&error_key()), None, "error must be cleared on success");
}

#[test]
fn spawn_success_loaded_path_set() {
    let mut s = BState::new();
    s.set(&loading_key(), "true");

    let returned = settle_import(&mut s, ImportOutcome::Loaded(minimal_model("/dir/model.blend")));

    assert_eq!(s.get(&loaded_path_key()), Some("/dir/model.blend"));
    assert!(returned.is_some(), "success settle returns the model for caching");
}

// --- spawn → failure ---

#[test]
fn spawn_failure_loading_cleared() {
    let mut s = BState::new();
    s.set(&loading_key(), "true");
    assert_eq!(s.get(&loading_key()), Some("true"), "precondition: loading set");

    settle_import(
        &mut s,
        ImportOutcome::Failed(SceneLoadError::Malformed("bad block".to_string())),
    );

    assert_eq!(s.get(&loading_key()), None, "loading must be absent after failure");
}

#[test]
fn spawn_failure_error_message_bounded() {
    let mut s = BState::new();
    s.set(&loading_key(), "true");

    settle_import(
        &mut s,
        ImportOutcome::Failed(SceneLoadError::Malformed("bad block".to_string())),
    );

    let msg = s.get(&error_key()).expect("failure must set an error");
    let len = char_len(msg);
    assert!(
        (1..=256).contains(&len),
        "error length {len} must be 1..=256"
    );
}

#[test]
fn spawn_failure_returns_none_and_retains_prior_path() {
    let mut s = BState::new();
    s.set(&loaded_path_key(), "/prior/scene.blend");
    s.set(&loading_key(), "true");

    let returned = settle_import(
        &mut s,
        ImportOutcome::Failed(SceneLoadError::Inaccessible("missing".to_string())),
    );

    assert!(returned.is_none(), "failure settle must return None");
    // Prior path is preserved — the caller's scene cache is untouched.
    assert_eq!(s.get(&loaded_path_key()), Some("/prior/scene.blend"));
}

#[test]
fn spawn_failure_inaccessible_error_names_category() {
    let mut s = BState::new();
    s.set(&loading_key(), "true");
    settle_import(
        &mut s,
        ImportOutcome::Failed(SceneLoadError::Inaccessible("file gone".to_string())),
    );
    let msg = s.get(&error_key()).expect("error set");
    assert!(msg.contains("inaccessible"), "message must name the category: {msg}");
}

#[test]
fn spawn_failure_unsupported_error_names_category() {
    let mut s = BState::new();
    s.set(&loading_key(), "true");
    settle_import(
        &mut s,
        ImportOutcome::Failed(SceneLoadError::Unsupported("version 2.4".to_string())),
    );
    let msg = s.get(&error_key()).expect("error set");
    assert!(msg.contains("unsupported"), "message must name the category: {msg}");
}

// --- spawn → timeout ---

#[test]
fn spawn_timeout_loading_cleared() {
    let mut s = BState::new();
    s.set(&loading_key(), "true");
    assert_eq!(s.get(&loading_key()), Some("true"), "precondition: loading set");

    settle_import(&mut s, ImportOutcome::TimedOut);

    assert_eq!(s.get(&loading_key()), None, "loading must be absent after timeout");
}

#[test]
fn spawn_timeout_error_message_bounded() {
    let mut s = BState::new();
    s.set(&loading_key(), "true");

    settle_import(&mut s, ImportOutcome::TimedOut);

    let msg = s.get(&error_key()).expect("timeout must set an error");
    let len = char_len(msg);
    assert!(
        (1..=256).contains(&len),
        "timeout error length {len} must be 1..=256"
    );
}

#[test]
fn spawn_timeout_retains_prior_path() {
    let mut s = BState::new();
    s.set(&loaded_path_key(), "/prior/scene.blend");
    s.set(&loading_key(), "true");

    let returned = settle_import(&mut s, ImportOutcome::TimedOut);

    assert!(returned.is_none(), "timeout settle must return None");
    assert_eq!(s.get(&loaded_path_key()), Some("/prior/scene.blend"));
}

// --- loading absent before settle stays absent ---

#[test]
fn loading_absent_before_settle_stays_absent_after_success() {
    let mut s = BState::new();
    // No loading flag set (e.g. spurious late callback).
    assert_eq!(s.get(&loading_key()), None, "precondition: loading absent");

    settle_import(&mut s, ImportOutcome::Loaded(minimal_model("/dir/a.blend")));

    assert_eq!(s.get(&loading_key()), None, "loading must stay absent");
}

#[test]
fn loading_absent_before_settle_stays_absent_after_failure() {
    let mut s = BState::new();
    settle_import(
        &mut s,
        ImportOutcome::Failed(SceneLoadError::Malformed("oops".to_string())),
    );
    assert_eq!(s.get(&loading_key()), None);
}

#[test]
fn loading_absent_before_settle_stays_absent_after_timeout() {
    let mut s = BState::new();
    settle_import(&mut s, ImportOutcome::TimedOut);
    assert_eq!(s.get(&loading_key()), None);
}
