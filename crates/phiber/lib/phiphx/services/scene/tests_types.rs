//! Basic scene-type invariants (Default, Clone, PartialEq). Split from the
//! traversal tests so both files stay under the 200-line cap.

use super::{SceneMesh, SceneModel, SceneNormals, SceneObject};

#[test]
fn scene_model_default_is_empty() {
    let m = SceneModel::default();
    assert!(m.objects.is_empty());
    assert_eq!(m.source_path, None);
}

#[test]
fn scene_object_clone_eq() {
    let obj = SceneObject {
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
    };
    assert_eq!(obj.clone(), obj);
}

#[test]
fn scene_normals_variants_distinct() {
    let per_vertex = SceneNormals::PerVertex(vec![[0.0, 1.0, 0.0]]);
    let per_face = SceneNormals::PerFace(vec![[0.0, 1.0, 0.0]]);
    assert_ne!(per_vertex, per_face);
}

#[test]
fn scene_model_holds_objects_and_path() {
    let m = SceneModel {
        objects: vec![SceneObject {
            name: "Suzanne".to_string(),
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
        source_path: Some("/tmp/monkey.blend".to_string()),
        ..Default::default()
    };
    assert_eq!(m.objects.len(), 1);
    assert_eq!(m.objects[0].name, "Suzanne");
    assert_eq!(m.source_path.as_deref(), Some("/tmp/monkey.blend"));
}
