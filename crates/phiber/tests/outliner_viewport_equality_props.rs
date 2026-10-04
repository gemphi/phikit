//! Property 10.3 — Outliner/Viewport set equality.
//!
//! Feature: blend-file-mesh-loading, Property 10.3: Outliner/Viewport set equality
//! Validates: Requirements 5.1, 5.2, 6.4
//!
//! For any SceneModel, the name order from `scene_source` matches
//! `model.objects[*].name` in identical order. Both the Viewport and the
//! Outliner read from the single `scene_source`, so they always agree.

use phiber::{
    phiphx::scene::{SceneMesh, SceneModel, SceneNormals, SceneObject},
    v2::scene::{BObject3D, OBJECTS, clear_loaded_scene, scene_source, set_loaded_scene},
    v2::state::BState,
};
use proptest::prelude::*;

fn obj(name: impl Into<String>) -> SceneObject {
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
        shadow: Default::default(),
    }
}

fn arb_named_objects() -> impl Strategy<Value = Vec<SceneObject>> {
    prop::collection::vec(
        proptest::string::string_regex("[A-Za-z][A-Za-z0-9_]{0,15}")
            .expect("name regex valid")
            .prop_map(|n| obj(n)),
        0..=20,
    )
}

proptest! {
    // Feature: blend-file-mesh-loading, Property 10.3: Outliner/Viewport set equality

    /// Names from scene_source match model.objects[*].name in identical order.
    #[test]
    fn scene_source_names_match_model_in_order(objects in arb_named_objects()) {
        let s = BState::new();
        let expected_names: Vec<String> = objects.iter().map(|o| o.name.clone()).collect();

        let model = SceneModel {
            objects,
            source_path: Some("/eq_test.blend".to_string()),
        ..Default::default()
        };

        set_loaded_scene(Some(model));
        let scene = scene_source(&s);
        clear_loaded_scene();

        prop_assert_eq!(scene.len(), expected_names.len());

        for (source_obj, expected_name) in scene.iter().zip(expected_names.iter()) {
            match source_obj {
                BObject3D::Loaded { name, .. } => {
                    prop_assert_eq!(name, expected_name, "name order must be preserved");
                }
                BObject3D::Primitive { .. } => {
                    prop_assert!(false, "expected Loaded, got Primitive");
                }
            }
        }
    }

    /// Default scene (None scope): names from scene_source match OBJECTS static table.
    #[test]
    fn default_scene_names_match_objects_table(_ignored in 0u8..1) {
        let s = BState::new();
        clear_loaded_scene(); // ensure clean state

        let scene = scene_source(&s);

        prop_assert_eq!(scene.len(), OBJECTS.len());
        for (source_obj, def) in scene.iter().zip(OBJECTS.iter()) {
            match source_obj {
                BObject3D::Primitive { name, .. } => {
                    prop_assert_eq!(name.as_str(), def.name);
                }
                _ => prop_assert!(false, "expected Primitive in default scene"),
            }
        }
    }

    /// Two calls to scene_source with the same scope return equal name sets.
    #[test]
    fn scene_source_is_stable_across_two_calls(objects in arb_named_objects()) {
        let s = BState::new();
        let model = SceneModel {
            objects,
            source_path: Some("/stable.blend".to_string()),
        ..Default::default()
        };

        set_loaded_scene(Some(model));
        let call1 = scene_source(&s);
        let call2 = scene_source(&s);
        clear_loaded_scene();

        let names1: Vec<&str> = call1.iter().filter_map(|o| match o {
            BObject3D::Loaded { name, .. } => Some(name.as_str()),
            _ => None,
        }).collect();
        let names2: Vec<&str> = call2.iter().filter_map(|o| match o {
            BObject3D::Loaded { name, .. } => Some(name.as_str()),
            _ => None,
        }).collect();

        prop_assert_eq!(names1, names2, "two calls with same scope must agree");
    }
}

// --- Deterministic unit cases ---

#[test]
fn three_loaded_objects_names_in_order() {
    let s = BState::new();
    let model = SceneModel {
        objects: vec![obj("Alpha"), obj("Beta"), obj("Gamma")],
        source_path: Some("/abc.blend".to_string()),
    ..Default::default()
    };

    set_loaded_scene(Some(model));
    let scene = scene_source(&s);
    clear_loaded_scene();

    assert_eq!(scene.len(), 3);
    let names: Vec<&str> = scene
        .iter()
        .filter_map(|o| match o {
            BObject3D::Loaded { name, .. } => Some(name.as_str()),
            _ => None,
        })
        .collect();
    assert_eq!(names, ["Alpha", "Beta", "Gamma"]);
}

#[test]
fn empty_model_yields_empty_name_list() {
    let s = BState::new();
    let model = SceneModel {
        objects: vec![],
        source_path: Some("/empty.blend".to_string()),
    ..Default::default()
    };

    set_loaded_scene(Some(model));
    let scene = scene_source(&s);
    clear_loaded_scene();

    assert!(scene.is_empty());
}

#[test]
fn default_scene_names_equal_static_objects() {
    let s = BState::new();
    clear_loaded_scene();

    let scene = scene_source(&s);
    let scene_names: Vec<&str> = scene
        .iter()
        .filter_map(|o| match o {
            BObject3D::Primitive { name, .. } => Some(name.as_str()),
            _ => None,
        })
        .collect();
    let expected: Vec<&str> = OBJECTS.iter().map(|o| o.name).collect();
    assert_eq!(scene_names, expected);
}
