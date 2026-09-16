//! Property 5.4 — Scene_Source default vs loaded selection.
//!
//! Feature: blend-file-mesh-loading, Property 5.4: current_scene default vs loaded
//! Validates: Requirements 6.1, 6.2, 5.1
//!
//! `current_scene(s, None)` returns exactly `OBJECTS.len()` items all as
//! `BObject3D::Primitive`; `current_scene(s, Some(&m))` returns exactly
//! `m.objects.len()` items all as `BObject3D::Loaded` in the same name order.

use phiber::{
    phiphx::services::scene::{SceneMesh, SceneModel, SceneNormals, SceneObject},
    v2::scene::{BObject3D, OBJECTS, current_scene},
    v2::state::BState,
};
use proptest::prelude::*;

/// Build a minimal `SceneObject` carrying the given name.
fn scene_obj(name: impl Into<String>) -> SceneObject {
    SceneObject {
        name: name.into(),
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
    }
}

/// Strategy: a list of 0..=20 non-empty object names.
fn arb_names() -> impl Strategy<Value = Vec<String>> {
    prop::collection::vec(
        proptest::string::string_regex("[A-Za-z][A-Za-z0-9_]{0,15}")
            .expect("name regex valid"),
        0..=20,
    )
}

proptest! {
    // Feature: blend-file-mesh-loading, Property 5.4: current_scene default vs loaded

    /// None -> Default_Scene: exactly OBJECTS.len() Primitives.
    #[test]
    fn default_scene_returns_primitives_when_no_model_loaded(
        _ignored in 0u8..1,  // force proptest structure; no actual input needed
    ) {
        let s = BState::new();
        let scene = current_scene(&s, None);
        prop_assert_eq!(scene.len(), OBJECTS.len());
        for obj in &scene {
            let is_prim = matches!(obj, BObject3D::Primitive { .. });
            prop_assert!(is_prim, "expected Primitive, got a Loaded variant");
        }
    }

    /// Some(model) -> one BObject3D::Loaded per object, in the same name order.
    #[test]
    fn loaded_scene_returns_loaded_in_order(names in arb_names()) {
        let s = BState::new();
        let model = SceneModel {
            objects: names.iter().map(|n| scene_obj(n)).collect(),
            source_path: Some("/dir/model.blend".to_string()),
        ..Default::default()
        };

        let scene = current_scene(&s, Some(&model));

        prop_assert_eq!(scene.len(), names.len());
        for (obj, expected_name) in scene.iter().zip(names.iter()) {
            match obj {
                BObject3D::Loaded { name, .. } => {
                    prop_assert_eq!(name, expected_name);
                }
                BObject3D::Primitive { .. } => {
                    prop_assert!(false, "expected Loaded, got Primitive");
                }
            }
        }
    }

    /// None gives Default_Scene regardless of what BState contains.
    #[test]
    fn none_always_yields_default_regardless_of_state(
        key_suffix in "[a-z]{1,8}",
        value in "[a-z0-9]{1,8}",
    ) {
        let mut s = BState::new();
        s.set_encoded(&format!("ctx.test.{key_suffix}"), value);
        let scene = current_scene(&s, None);
        prop_assert_eq!(scene.len(), OBJECTS.len());
        for obj in &scene {
            let is_prim = matches!(obj, BObject3D::Primitive { .. });
            prop_assert!(is_prim, "expected Primitive after state entries");
        }
    }
}

// --- Deterministic unit cases ---

#[test]
fn default_scene_count_matches_static_objects() {
    let s = BState::new();
    let scene = current_scene(&s, None);
    assert_eq!(scene.len(), OBJECTS.len());
    for (obj, def) in scene.iter().zip(OBJECTS.iter()) {
        match obj {
            BObject3D::Primitive { name, .. } => assert_eq!(name, def.name),
            _ => panic!("expected Primitive"),
        }
    }
}

#[test]
fn loaded_zero_objects_yields_empty_scene() {
    let s = BState::new();
    let model = SceneModel {
        objects: vec![],
        source_path: Some("/empty.blend".to_string()),
    ..Default::default()
    };
    let scene = current_scene(&s, Some(&model));
    assert!(scene.is_empty());
}

#[test]
fn loaded_single_object_name_preserved() {
    let s = BState::new();
    let model = SceneModel {
        objects: vec![scene_obj("Suzanne")],
        source_path: Some("/monkey.blend".to_string()),
    ..Default::default()
    };
    let scene = current_scene(&s, Some(&model));
    assert_eq!(scene.len(), 1);
    match &scene[0] {
        BObject3D::Loaded { name, .. } => assert_eq!(name, "Suzanne"),
        _ => panic!("expected Loaded"),
    }
}
