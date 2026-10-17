//! Material, UV and colour-attribute traversal tests (split from
//! `tests_traversal`, which keeps the geometry half).

use super::SceneModel;
use super::tests_fixture::{blend_obj, blend_scene};

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
    let model = SceneModel::from_blend(&blend_scene(vec![obj]), "m.blend");
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
    let model = SceneModel::from_blend(&blend_scene(vec![obj]), "bare.blend");
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
    let model = SceneModel::from_blend(&blend_scene(vec![obj]), "uv.blend");
    let scene_obj = &model.objects[0];
    assert_eq!(*scene_obj.uvs, vec![[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]]);
}

#[test]
fn scene_from_blend_leaves_uvs_empty_without_a_uv_layer() {
    let obj = blend_obj(
        "NoUv",
        vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
        vec![vec![0, 1, 2]],
        Vec::new(),
    );
    let model = SceneModel::from_blend(&blend_scene(vec![obj]), "nouv.blend");
    assert!(model.objects[0].uvs.is_empty());
}

#[test]
fn scene_from_blend_carries_the_colour_attribute_to_the_corners() {
    // A quad (two triangles) with a colour a loop: the triangles' corners
    // take their loops' colours; without a layer, no colours at all.
    let mut obj = blend_obj(
        "Painted",
        vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 1.0, 0.0], [0.0, 1.0, 0.0]],
        vec![vec![0, 1, 2, 3]],
        Vec::new(),
    );
    obj.loop_colors = vec![[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0], [1.0, 1.0, 1.0]];
    let model = SceneModel::from_blend(&blend_scene(vec![obj.clone()]), "col.blend");
    let mesh = &model.objects[0].mesh;
    assert_eq!(mesh.indices.len(), 6);
    assert_eq!(mesh.corners.colors.len(), 6, "one colour per triangle corner");
    for (c, &v) in mesh.indices.iter().enumerate() {
        assert_eq!(mesh.corners.colors[c], obj.loop_colors[v as usize], "corner {c} is loop {v}'s colour");
    }
    obj.loop_colors.clear();
    let model = SceneModel::from_blend(&blend_scene(vec![obj]), "col.blend");
    assert!(model.objects[0].mesh.corners.colors.is_empty());
}
