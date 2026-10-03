//! Task 12.2 — Click → Load → Render → Outline integration test.
//!
//! Feature: blend-file-mesh-loading
//! Validates: Requirements 1.2, 1.7, 3.4, 3.7, 5.1
//!
//! Drives the full pipeline using pure functions only (no gpui):
//!   1. apply_action sets file.selected to a .blend path.
//!   2. should_import confirms the import should start.
//!   3. A stubbed SceneModel is settled via settle_import(Loaded(model)).
//!   4. set_loaded_scene makes the model visible to scene_source.
//!   5. scene_source returns the correct Loaded objects for both the
//!      Viewport and the Outliner consumers.
//!   6. The error path retains the prior scene and leaves loaded_path unchanged.

use phiber::{
    phiphx::scene::{
        SceneLoadError, SceneMesh, SceneModel, SceneNormals, SceneObject,
    },
    v2::app::import::{ImportOutcome, settle_import, should_import},
    v2::scene::{
        BObject3D, clear_loaded_scene, error_key, loaded_path_key, loading_key, scene_source,
        selected_key, set_loaded_scene,
    },
    v2::state::{BState, action::apply_action},
    v2::view::prim::BAction,
};

// --------------------------------------------------------------------------
// Test helpers
// --------------------------------------------------------------------------

fn blend_obj(name: impl Into<String>) -> SceneObject {
    SceneObject {
        name: name.into(),
        mesh: SceneMesh {
            vertices: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            indices: vec![0, 1, 2],
            normals: SceneNormals::PerFace(vec![[0.0, 0.0, 1.0]]),
            corners: Default::default(),
        }.into(),
        location: [0.0, 0.0, 0.0],
        scale: [1.0, 1.0, 1.0],
        matrix_world: phiber::phiphx::scene::IDENTITY_M4,
        materials: Vec::new(),
        material_indices: Default::default(),
        uvs: Default::default(),
        color: [1.0; 4],
    }
}

fn model_with(path: &str, names: &[&str]) -> SceneModel {
    SceneModel {
        objects: names.iter().map(|n| blend_obj(*n)).collect(),
        source_path: Some(path.to_string()),
    ..Default::default()
    }
}

// --------------------------------------------------------------------------
// Success flow: click → should_import → settle(Loaded) → scene_source
// --------------------------------------------------------------------------

#[test]
fn success_flow_viewport_and_outliner_see_loaded_objects() {
    let mut state = BState::new();

    // 1. File Browser click: set file.selected to a .blend path.
    apply_action(
        &mut state,
        &BAction::Set(selected_key(), "/dir/model.blend".to_string()),
    );
    assert_eq!(state.get(&selected_key()), Some("/dir/model.blend"));

    // 2. should_import confirms this is a new blend (no prior loaded path).
    let selected = state.get(&selected_key());
    assert!(
        should_import(None, selected),
        "should_import must be true for a new .blend path"
    );

    // 3. Simulate spawn: set loading flag.
    state.set(&loading_key(), "true");

    // 4. Background task completes: build a stubbed SceneModel and settle it.
    let model = model_with("/dir/model.blend", &["Suzanne", "Cube"]);
    let returned = settle_import(&mut state, ImportOutcome::Loaded(model.clone()));

    // 5. settle_import clears loading, sets loaded_path, clears error, returns model.
    assert_eq!(state.get(&loading_key()), None);
    assert_eq!(state.get(&loaded_path_key()), Some("/dir/model.blend"));
    assert_eq!(state.get(&error_key()), None);
    let cached = returned.expect("Loaded outcome must return the model");

    // 6. Inject into the render-pass scope.
    set_loaded_scene(Some(cached));
    let scene = scene_source(&state);
    clear_loaded_scene();

    // 7. Both Viewport and Outliner see 2 objects in order.
    assert_eq!(scene.len(), 2);
    match &scene[0] {
        BObject3D::Loaded { name, .. } => assert_eq!(name, "Suzanne"),
        _ => panic!("expected Loaded at index 0"),
    }
    match &scene[1] {
        BObject3D::Loaded { name, .. } => assert_eq!(name, "Cube"),
        _ => panic!("expected Loaded at index 1"),
    }
}

// --------------------------------------------------------------------------
// Error path: prior scene retained, loaded_path unchanged
// --------------------------------------------------------------------------

#[test]
fn error_path_retains_prior_scene() {
    let mut state = BState::new();

    // Step 1: establish a successful prior load at model_a.
    let model_a = model_with("/dir/prev.blend", &["Alpha", "Beta"]);
    let returned_a = settle_import(&mut state, ImportOutcome::Loaded(model_a.clone()));
    let cached_a = returned_a.expect("first load must succeed");
    assert_eq!(state.get(&loaded_path_key()), Some("/dir/prev.blend"));

    // Step 2: simulate a new import attempt starting.
    state.set(&loading_key(), "true");

    // Step 3: the new import fails.
    settle_import(
        &mut state,
        ImportOutcome::Failed(SceneLoadError::Malformed("corrupt block".to_string())),
    );

    // loading cleared, error set, loaded_path NOT changed.
    assert_eq!(state.get(&loading_key()), None);
    assert!(state.get(&error_key()).is_some(), "error must be set on failure");
    assert_eq!(
        state.get(&loaded_path_key()),
        Some("/dir/prev.blend"),
        "loaded_path must be retained after failure"
    );

    // Step 4: the prior scene is still accessible via scene_source.
    set_loaded_scene(Some(cached_a));
    let scene = scene_source(&state);
    clear_loaded_scene();

    assert_eq!(scene.len(), 2);
    match &scene[0] {
        BObject3D::Loaded { name, .. } => assert_eq!(name, "Alpha"),
        _ => panic!("expected Loaded at index 0"),
    }
}

// --------------------------------------------------------------------------
// should_import not triggered for same path
// --------------------------------------------------------------------------

#[test]
fn should_import_not_triggered_for_same_loaded_path() {
    let mut state = BState::new();

    // Establish a prior loaded path.
    let model = model_with("/dir/model.blend", &["Cube"]);
    settle_import(&mut state, ImportOutcome::Loaded(model));
    assert_eq!(state.get(&loaded_path_key()), Some("/dir/model.blend"));

    // Selecting the same .blend again.
    apply_action(
        &mut state,
        &BAction::Set(selected_key(), "/dir/model.blend".to_string()),
    );

    let loaded_path = state.get(&loaded_path_key());
    let selected = state.get(&selected_key());
    assert!(
        !should_import(loaded_path, selected),
        "should_import must be false when selected matches loaded_path"
    );
}

// --------------------------------------------------------------------------
// Timeout path
// --------------------------------------------------------------------------

#[test]
fn timeout_path_retains_prior_scene_and_sets_error() {
    let mut state = BState::new();

    // Establish a prior load.
    let model = model_with("/prior.blend", &["Mesh"]);
    settle_import(&mut state, ImportOutcome::Loaded(model));
    assert_eq!(state.get(&loaded_path_key()), Some("/prior.blend"));

    // Simulate import start + timeout.
    state.set(&loading_key(), "true");
    settle_import(&mut state, ImportOutcome::TimedOut);

    assert_eq!(state.get(&loading_key()), None);
    assert!(state.get(&error_key()).is_some());
    assert_eq!(state.get(&loaded_path_key()), Some("/prior.blend"));
}

// --------------------------------------------------------------------------
// Fresh start: no prior scene, failure leaves everything absent
// --------------------------------------------------------------------------

#[test]
fn fresh_failure_leaves_loaded_path_absent() {
    let mut state = BState::new();
    assert_eq!(state.get(&loaded_path_key()), None, "precondition");

    settle_import(
        &mut state,
        ImportOutcome::Failed(SceneLoadError::Inaccessible("no file".to_string())),
    );

    // loaded_path stays absent since there was no prior successful load.
    assert_eq!(state.get(&loaded_path_key()), None);
    assert_eq!(state.get(&loading_key()), None);
    assert!(state.get(&error_key()).is_some());
}
