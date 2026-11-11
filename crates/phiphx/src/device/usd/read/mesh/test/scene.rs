use super::*;

fn mesh_named(name: &str, verts: usize) -> Mesh {
    let mut m = Mesh::new();
    m.name = name.to_string();
    m.positions = vec![[0.0, 0.0, 0.0]; verts];
    m
}

#[test]
fn empty_scene_has_no_primary() {
    let scene = UsdScene::from_meshes(vec![]);
    assert!(scene.primary_object().is_none());
}

#[test]
fn falls_back_to_largest_mesh() {
    let scene = UsdScene::from_meshes(vec![mesh_named("small", 3), mesh_named("big", 10)]);
    assert_eq!(scene.primary_object().unwrap().name, "big");
}

#[test]
fn active_takes_precedence() {
    let mut scene = UsdScene::from_meshes(vec![mesh_named("small", 3), mesh_named("big", 10)]);
    scene.active_object = Some(0);
    assert_eq!(scene.primary_object().unwrap().name, "small");
}

#[test]
fn out_of_range_active_falls_back() {
    let mut scene = UsdScene::from_meshes(vec![mesh_named("only", 5)]);
    scene.active_object = Some(9);
    assert_eq!(scene.primary_object().unwrap().name, "only");
}

#[test]
fn tie_break_by_name_is_order_independent() {
    let a = UsdScene::from_meshes(vec![mesh_named("beta", 4), mesh_named("alpha", 4)]);
    let b = UsdScene::from_meshes(vec![mesh_named("alpha", 4), mesh_named("beta", 4)]);
    assert_eq!(a.primary_object().unwrap().name, "alpha");
    assert_eq!(b.primary_object().unwrap().name, "alpha");
}

#[test]
fn with_active_named_sets_index() {
    let scene = UsdScene::from_meshes(vec![mesh_named("small", 3), mesh_named("hero", 5)])
        .with_active_named("hero");
    assert_eq!(scene.active_object, Some(1));
    assert_eq!(scene.primary_object().unwrap().name, "hero");
}
