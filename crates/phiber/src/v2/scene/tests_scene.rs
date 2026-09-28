//! `current_scene` + `BObject3D` tests. Split from `mod.rs` so all test
//! files stay under the 200-line cap.

use super::*;
use crate::phiphx::scene::{SceneMesh, SceneNormals, SceneObject};

fn loaded_object(name: &str) -> SceneObject {
    SceneObject {
        name: name.to_string(),
        mesh: SceneMesh {
            vertices: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            indices: vec![0, 1, 2],
            normals: SceneNormals::PerFace(vec![[0.0, 0.0, 1.0]]),
            corners: Default::default(),
        }.into(),
        location: [0.0, 0.0, 0.0],
        scale: [1.0, 1.0, 1.0],
        matrix_world: crate::phiphx::scene::IDENTITY_M4,
        materials: Vec::new(),
        material_indices: Default::default(),
        uvs: Default::default(),
    }
}

#[test]
fn current_scene_none_yields_default_primitives() {
    let s = BState::new();
    let scene = current_scene(&s, None);
    // One Primitive per OBJECTS entry, matching name/shape/x/y in order.
    assert_eq!(scene.len(), OBJECTS.len());
    for (obj, def) in scene.iter().zip(OBJECTS.iter()) {
        match obj {
            BObject3D::Primitive { name, shape, x, y } => {
                assert_eq!(name, def.name);
                assert_eq!(*shape, def.shape);
                assert!((*x - def.x).abs() < 1e-6);
                assert!((*y - def.y).abs() < 1e-6);
            }
            BObject3D::Loaded { .. } => panic!("expected Primitive for default scene"),
        }
    }
}

#[test]
fn current_scene_some_yields_loaded_in_order() {
    let s = BState::new();
    let model = SceneModel {
        objects: vec![loaded_object("Alpha"), loaded_object("Beta")],
        source_path: Some("/dir/model.blend".to_string()),
        ..Default::default()
    };
    let scene = current_scene(&s, Some(&model));
    assert_eq!(scene.len(), model.objects.len());
    for (obj, src) in scene.iter().zip(model.objects.iter()) {
        match obj {
            BObject3D::Loaded {
                name,
                mesh,
                location,
                scale,
                matrix_world,
                materials,
                material_indices,
                uvs,
            } => {
                assert_eq!(name, &src.name);
                assert_eq!(matrix_world, &src.matrix_world);
                assert!(std::sync::Arc::ptr_eq(mesh, &src.mesh), "mesh is shared, not copied");
                assert_eq!(location, &src.location);
                assert_eq!(scale, &src.scale);
                assert_eq!(materials, &src.materials);
                assert_eq!(material_indices, &src.material_indices);
                assert_eq!(uvs, &src.uvs);
            }
            BObject3D::Primitive { .. } => panic!("expected Loaded for loaded scene"),
        }
    }
    // Order preserved.
    if let BObject3D::Loaded { name, .. } = &scene[0] {
        assert_eq!(name, "Alpha");
    }
    if let BObject3D::Loaded { name, .. } = &scene[1] {
        assert_eq!(name, "Beta");
    }
}

#[test]
fn current_scene_zero_object_model_yields_empty() {
    let s = BState::new();
    let model = SceneModel {
        objects: vec![],
        source_path: Some("/dir/empty.blend".to_string()),
        ..Default::default()
    };
    let scene = current_scene(&s, Some(&model));
    assert!(scene.is_empty());
}
