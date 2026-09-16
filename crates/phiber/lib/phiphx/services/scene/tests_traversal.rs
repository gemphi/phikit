//! `scene_from_blend` traversal tests. Covers fan-triangulation, out-of-range
//! index dropping, degenerate-face skipping, and normal synthesis. Basic
//! type invariants live in the sibling `tests_types` file so both stay
//! under the 200-line cap.

use super::{SceneNormals, scene_from_blend};
use crate::phiphx::importer::blend::{BlendScene, BlendSceneObject};

/// Minimal mesh object carrying only the fields the traversal reads
/// (`name`, `vertices`, `faces`, `loop_normals`); everything else inert.
fn blend_obj(
    name: &str,
    vertices: Vec<[f32; 3]>,
    faces: Vec<Vec<u32>>,
    loop_normals: Vec<[f32; 3]>,
) -> BlendSceneObject {
    BlendSceneObject {
        name: name.to_string(),
        object_type: "MESH".to_string(),
        location: [0.0; 3],
        rotation: [0.0; 3],
        quaternion: [1.0, 0.0, 0.0, 0.0],
        scale: [1.0; 3],
        matrix_world: [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ],
        vertices,
        faces,
        loop_normals,
        loop_uvs: Vec::new(),
        modifiers: Vec::new(),
        instance_targets: Vec::new(),
        is_instance: false,
        base_color: [0.8, 0.8, 0.8, 1.0],
        roughness: 0.5,
        metallic: 0.0,
        materials: Vec::new(),
        texture_paths: Vec::new(),
    }
}

fn blend_scene(objects: Vec<BlendSceneObject>) -> BlendScene {
    BlendScene {
        name: "test".to_string(),
        collections: Vec::new(),
        objects,
        active_object: None,
        selected: Vec::new(),
    }
}

#[test]
fn scene_from_blend_sets_source_path() {
    let scene = blend_scene(Vec::new());
    let model = scene_from_blend(&scene, "/tmp/foo.blend");
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
    let model = scene_from_blend(&blend_scene(vec![obj]), "q.blend");
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
    let model = scene_from_blend(&blend_scene(vec![obj]), "b.blend");
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
    let model = scene_from_blend(&blend_scene(vec![obj]), "l.blend");
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
    let model = scene_from_blend(&blend_scene(vec![obj]), "t.blend");
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
    let model = scene_from_blend(&blend_scene(vec![obj]), "t.blend");
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
    let model = scene_from_blend(&blend_scene(vec![obj]), "q.blend");
    let mesh = &model.objects[0].mesh;
    let tri_count = mesh.indices.len() / 3;
    match &mesh.normals {
        SceneNormals::PerFace(ns) => assert_eq!(ns.len(), tri_count),
        other => panic!("expected PerFace, got {other:?}"),
    }
}

// ── Material + UV threading (blend → SceneObject) ────────────────────────

#[test]
fn scene_from_blend_carries_the_single_material_slot() {
    // `blend_obj` seeds a non-default base_color/roughness, so the fallback
    // single-material path should produce exactly one slot mirroring it.
    let obj = blend_obj(
        "Tri",
        vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
        vec![vec![0, 1, 2]],
        Vec::new(),
    );
    let model = scene_from_blend(&blend_scene(vec![obj]), "m.blend");
    let scene_obj = &model.objects[0];
    assert_eq!(scene_obj.materials.len(), 1);
    assert_eq!(scene_obj.materials[0].base_color, [0.8, 0.8, 0.8, 1.0]);
    assert_eq!(scene_obj.materials[0].roughness, 0.5);
    // With one slot, triangle 0's base colour resolves to it.
    assert_eq!(
        scene_obj.triangle_base_color(0),
        Some([0.8, 0.8, 0.8, 1.0])
    );
}

#[test]
fn scene_from_blend_drops_material_when_object_is_fully_default() {
    // An object with an all-zero base_color and zero PBR scalars is treated
    // as "no material" so the renderer uses its neutral loaded colour.
    let mut obj = blend_obj(
        "Bare",
        vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
        vec![vec![0, 1, 2]],
        Vec::new(),
    );
    obj.base_color = [0.0, 0.0, 0.0, 0.0];
    obj.roughness = 0.0;
    obj.metallic = 0.0;
    let model = scene_from_blend(&blend_scene(vec![obj]), "bare.blend");
    let scene_obj = &model.objects[0];
    assert!(scene_obj.materials.is_empty());
    assert_eq!(scene_obj.triangle_base_color(0), None);
}

#[test]
fn scene_from_blend_builds_per_vertex_uvs_from_loop_uvs() {
    // One triangle with a per-corner UV each; the first loop referencing a
    // vertex sets that vertex's UV.
    let mut obj = blend_obj(
        "UvTri",
        vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
        vec![vec![0, 1, 2]],
        Vec::new(),
    );
    obj.loop_uvs = vec![[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]];
    let model = scene_from_blend(&blend_scene(vec![obj]), "uv.blend");
    let scene_obj = &model.objects[0];
    assert_eq!(scene_obj.uvs, vec![[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]]);
}

#[test]
fn scene_from_blend_leaves_uvs_empty_without_a_uv_layer() {
    let obj = blend_obj(
        "NoUv",
        vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
        vec![vec![0, 1, 2]],
        Vec::new(),
    );
    let model = scene_from_blend(&blend_scene(vec![obj]), "nouv.blend");
    assert!(model.objects[0].uvs.is_empty());
}
