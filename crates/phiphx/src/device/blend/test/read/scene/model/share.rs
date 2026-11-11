use crate::device::blend::read::scene::model::share::*;
use std::sync::Arc;

fn object(name: &str, block: u64, source: Option<u64>, verts: usize) -> BlendSceneObject {
    BlendSceneObject {
        name: name.into(),
        block_address: block,
        instance_source: source.map(|m| (m, crate::scene::IDENTITY_M4)),
        is_instance: source.is_some(),
        vertices: vec![[0.0; 3]; verts],
        ..BlendSceneObject::default()
    }
}

/// Two instances of a member in the scene share the member; two of a
/// member outside it share the first of them.
#[test]
fn an_instance_shares_its_member_or_the_first_instance_of_it() {
    let raw = [
        object("Car", 10, None, 3),
        object("Lot/Car", 0, Some(10), 3),
        object("Lot.001/Car", 0, Some(10), 3),
        object("Lot/Tree", 0, Some(20), 3),
        object("Lot.001/Tree", 0, Some(20), 3),
        object("Lot/Empty", 0, Some(30), 0),
    ];
    let wanted: HashSet<usize> = (0..raw.len()).collect();
    assert_eq!(sources(&raw, &wanted), vec![None, Some(0), Some(0), None, Some(3), None]);
    // A member not wanted (a camera, say) is nobody's source.
    let wanted: HashSet<usize> = (1..raw.len()).collect();
    assert_eq!(sources(&raw, &wanted)[..3], [None, None, Some(1)]);
}

#[test]
fn a_shared_object_holds_the_sources_arcs_under_its_own_placement() {
    let mesh = crate::scene::SceneMesh { vertices: vec![[1.0; 3]], indices: vec![0, 0, 0], normals: crate::scene::SceneNormalList::PerFace(Vec::new()), corners: Default::default() };
    let mut from = SceneObject::identity("Car", mesh);
    from.uvs = Arc::new(vec![[0.5, 0.5]]);
    let mut src = object("Lot/Car", 0, Some(10), 3);
    src.matrix_world[3] = [4.0, 5.0, 6.0, 1.0];
    let got = shared_object(&src, &from);
    assert!(Arc::ptr_eq(&got.mesh, &from.mesh) && Arc::ptr_eq(&got.uvs, &from.uvs) && Arc::ptr_eq(&got.material_indices, &from.material_indices));
    assert_eq!((got.name.as_str(), got.matrix_world[3]), ("Lot/Car", [4.0, 5.0, 6.0, 1.0]));
    let mut objects = vec![from, SceneObject::identity("Other", crate::scene::SceneMesh { vertices: Vec::new(), indices: Vec::new(), normals: crate::scene::SceneNormalList::PerFace(Vec::new()), corners: Default::default() })];
    alias(&mut objects, 0, 1);
    assert!(Arc::ptr_eq(&objects[0].mesh, &objects[1].mesh));
    alias(&mut objects, 0, 7);
}
