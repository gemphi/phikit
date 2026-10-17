//! `SceneModel::from_blend` geometry traversal tests: fan-triangulation,
//! out-of-range index dropping, degenerate-face skipping, and normal
//! synthesis. Material / UV / colour tests live in `tests_material`.

use super::{SceneModel, SceneNormals};
use super::tests_fixture::{blend_obj, blend_scene};

#[test]
fn scene_from_blend_sets_source_path() {
    let scene = blend_scene(Vec::new());
    let model = SceneModel::from_blend(&scene, "/tmp/foo.blend");
    assert_eq!(model.source_path.as_deref(), Some("/tmp/foo.blend"));
    assert!(model.objects.is_empty());
}

#[test]
fn scene_from_blend_fan_triangulates_quad() {
    let obj = blend_obj(
        "Quad",
        vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ],
        vec![vec![0, 1, 2, 3]],
        Vec::new(),
    );
    let model = SceneModel::from_blend(&blend_scene(vec![obj]), "q.blend");
    let mesh = &model.objects[0].mesh;
    assert_eq!(mesh.indices, vec![0, 1, 2, 0, 2, 3]);
    assert_eq!(mesh.indices.len() % 3, 0);
}

#[test]
fn scene_from_blend_drops_out_of_range_triangles() {
    let obj = blend_obj(
        "Bad",
        vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
        vec![vec![0, 1, 9]],
        Vec::new(),
    );
    let model = SceneModel::from_blend(&blend_scene(vec![obj]), "b.blend");
    let mesh = &model.objects[0].mesh;
    assert!(mesh.indices.is_empty());
    assert_eq!(mesh.indices.len() % 3, 0);
    assert!(
        mesh.indices
            .iter()
            .all(|&i| (i as usize) < mesh.vertices.len())
    );
}

#[test]
fn scene_from_blend_skips_degenerate_faces() {
    let obj = blend_obj(
        "Line",
        vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]],
        vec![vec![0, 1]],
        Vec::new(),
    );
    let model = SceneModel::from_blend(&blend_scene(vec![obj]), "l.blend");
    assert!(model.objects[0].mesh.indices.is_empty());
}

#[test]
fn scene_from_blend_synthesizes_per_face_normals() {
    let obj = blend_obj(
        "Tri",
        vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
        vec![vec![0, 1, 2]],
        Vec::new(),
    );
    let model = SceneModel::from_blend(&blend_scene(vec![obj]), "t.blend");
    match &model.objects[0].mesh.normals {
        SceneNormals::PerFace(ns) => {
            assert_eq!(ns.len(), 1);
            let n = ns[0];
            assert!((n[0]).abs() < 1e-5 && (n[1]).abs() < 1e-5 && (n[2] - 1.0).abs() < 1e-5);
        }
        other => panic!("expected PerFace, got {other:?}"),
    }
}

#[test]
fn scene_from_blend_uses_loop_normals_when_present() {
    let obj = blend_obj(
        "Tri",
        vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
        vec![vec![0, 1, 2]],
        vec![[0.0, 1.0, 0.0], [0.0, 1.0, 0.0], [0.0, 1.0, 0.0]],
    );
    let model = SceneModel::from_blend(&blend_scene(vec![obj]), "t.blend");
    match &model.objects[0].mesh.normals {
        SceneNormals::PerFace(ns) => {
            assert_eq!(ns.len(), 1);
            assert_eq!(ns[0], [0.0, 1.0, 0.0]);
        }
        other => panic!("expected PerFace, got {other:?}"),
    }
}

#[test]
fn scene_from_blend_normals_cardinality_matches_triangles() {
    let obj = blend_obj(
        "Quad",
        vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ],
        vec![vec![0, 1, 2, 3]],
        Vec::new(),
    );
    let model = SceneModel::from_blend(&blend_scene(vec![obj]), "q.blend");
    let mesh = &model.objects[0].mesh;
    let tri_count = mesh.indices.len() / 3;
    match &mesh.normals {
        SceneNormals::PerFace(ns) => assert_eq!(ns.len(), tri_count),
        other => panic!("expected PerFace, got {other:?}"),
    }
}
