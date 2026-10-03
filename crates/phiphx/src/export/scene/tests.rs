use super::*;

#[test]
fn test_phx_scene_roundtrip() {
    let scene = PhxScene::sample();

    // Export to bytes
    let bytes = scene.to_bytes().unwrap();

    // Import back
    let imported = PhxScene::from_bytes(&bytes).unwrap();

    assert_eq!(imported.name, scene.name);
    assert_eq!(imported.meshes.len(), scene.meshes.len());
    assert_eq!(imported.materials.len(), scene.materials.len());
}

#[test]
fn test_phx_scene_to_point_cloud() {
    let scene = PhxScene::sample();
    let cloud = scene.to_point_cloud();

    assert_eq!(cloud.vertex_count(), 3); // Triangle has 3 vertices
}

#[test]
fn test_phx_scene_to_phx() {
    let scene = PhxScene::sample();
    let ply = scene.to_ply(false).unwrap();

    assert!(ply.contains("ply"));
    assert!(ply.contains("element vertex 3"));
}

#[test]
fn test_phx_scene_magic() {
    assert_eq!(PHX_SCENE_MAGIC, b"PHX_SCENE\x01");
}
