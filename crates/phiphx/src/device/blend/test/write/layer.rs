use super::*;

#[test]
fn the_scene_its_bases_and_its_collections_are_found_by_index() {
    let path = format!("{}/tests/fixtures/nla_cube.blend", env!("CARGO_MANIFEST_DIR"));
    let file = BlendFile::parse_bytes(&std::fs::read(path).expect("fixture")).expect("parse");
    let scene = file.current_scene_index().expect("a scene");
    assert_eq!(file.blocks[scene].code_str, "SC");
    let view_layer = file.view_layer_index(scene).expect("a view layer");
    let bases = file.bases(view_layer);
    assert!(!bases.is_empty(), "the cube has a base");
    let cube = file.find_id("OB", "Cube").expect("the cube");
    assert!(bases.iter().any(|b| file.index_at(b.object_address) == Some(cube)), "a base points at the cube");
    let collections = file.collections_at(scene, view_layer);
    assert_eq!(collections[0].path, "Scene Collection");
    assert!(collections.iter().skip(1).all(|c| c.path.starts_with("Scene Collection/")), "{:?}", collections.iter().map(|c| &c.path).collect::<Vec<_>>());
    assert!(collections.iter().all(|c| c.layer_collection.is_some()), "every collection has its layer collection");
}
