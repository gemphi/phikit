//! Property 9.3 — Scale/render with many objects.
//!
//! Feature: blend-file-mesh-loading, Property 9.3: scale/render with many objects
//! Validates: Requirements 8.2, 4.6
//!
//! A Scene_Source of up to 500 objects returns the correct count via
//! `scene_source`. After clearing the scope the default scene is restored.

use phiber::{
    phiphx::scene::{SceneMesh, SceneModel, SceneNormals, SceneObject},
    v2::scene::{
        OBJECTS, clear_loaded_scene, scene_source, set_loaded_scene, BObject3D,
    },
    v2::state::BState,
};
use proptest::prelude::*;

fn simple_obj(name: impl Into<String>) -> SceneObject {
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
    }
}

/// Strategy: a Vec of 0..=500 scene objects.
fn arb_objects() -> impl Strategy<Value = Vec<SceneObject>> {
    prop::collection::vec(
        proptest::string::string_regex("[A-Za-z][A-Za-z0-9_]{0,15}")
            .expect("name regex valid")
            .prop_map(|name| simple_obj(name)),
        0..=500,
    )
}

proptest! {
    // Feature: blend-file-mesh-loading, Property 9.3: scale/render with many objects

    /// scene_source returns exactly the loaded object count; default scene
    /// is restored after clear.
    #[test]
    fn scene_source_count_matches_loaded_model(objects in arb_objects()) {
        let s = BState::new();
        let expected_count = objects.len();

        let model = SceneModel {
            objects,
            source_path: Some("/scale_test.blend".to_string()),
        ..Default::default()
        };

        // IMPORTANT: always clear at end of each case (thread-local scope).
        set_loaded_scene(Some(model));
        let loaded_scene = scene_source(&s);
        // Clean up before any assertion that might abort the case.
        clear_loaded_scene();

        prop_assert_eq!(loaded_scene.len(), expected_count,
            "loaded scene count should match model object count");

        // Every item is BObject3D::Loaded when a model is set.
        for obj in &loaded_scene {
            let is_loaded = matches!(obj, BObject3D::Loaded { .. });
            prop_assert!(is_loaded, "expected Loaded, got a non-Loaded variant");
        }

        // After clearing, default scene is restored.
        let default_scene = scene_source(&s);
        prop_assert_eq!(
            default_scene.len(),
            OBJECTS.len(),
            "after clear, scene_source must return Default_Scene"
        );
        for obj in &default_scene {
            let is_prim = matches!(obj, BObject3D::Primitive { .. });
            prop_assert!(is_prim, "expected Primitive after clear");
        }
    }

    /// Zero objects: empty scene, no items, default scene restores.
    #[test]
    fn zero_objects_yields_empty_then_default(_ignored in 0u8..1) {
        let s = BState::new();

        let model = SceneModel {
            objects: vec![],
            source_path: Some("/empty.blend".to_string()),
        ..Default::default()
        };

        set_loaded_scene(Some(model));
        let loaded = scene_source(&s);
        clear_loaded_scene();

        prop_assert!(loaded.is_empty());

        let default = scene_source(&s);
        prop_assert_eq!(default.len(), OBJECTS.len());
    }
}

// --- Deterministic unit cases ---

#[test]
fn loaded_500_objects_all_accessible() {
    let s = BState::new();
    let objects: Vec<SceneObject> = (0..500).map(|i| simple_obj(format!("Object{i}"))).collect();
    let model = SceneModel {
        objects,
        source_path: Some("/big_scene.blend".to_string()),
    ..Default::default()
    };

    set_loaded_scene(Some(model));
    let scene = scene_source(&s);
    clear_loaded_scene();

    assert_eq!(scene.len(), 500);
    for (i, obj) in scene.iter().enumerate() {
        match obj {
            BObject3D::Loaded { name, .. } => {
                assert_eq!(name, &format!("Object{i}"));
            }
            _ => panic!("expected Loaded at index {i}"),
        }
    }
}

#[test]
fn none_scope_yields_default_scene() {
    let s = BState::new();
    set_loaded_scene(None);
    let scene = scene_source(&s);
    clear_loaded_scene();

    assert_eq!(scene.len(), OBJECTS.len());
    for obj in &scene {
        assert!(matches!(obj, BObject3D::Primitive { .. }));
    }
}

#[test]
fn clear_after_set_restores_default() {
    let s = BState::new();
    let model = SceneModel {
        objects: vec![simple_obj("Alpha")],
        source_path: Some("/a.blend".to_string()),
    ..Default::default()
    };

    set_loaded_scene(Some(model));
    let loaded = scene_source(&s);
    assert_eq!(loaded.len(), 1);

    clear_loaded_scene();
    let default = scene_source(&s);
    assert_eq!(default.len(), OBJECTS.len());
}
