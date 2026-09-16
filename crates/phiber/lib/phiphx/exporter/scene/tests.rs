use super::*;

#[test]
fn test_phx_scene_roundtrip() {
    let scene = sample_scene();

    // Export to bytes
    let bytes = export_phx_scene_to_bytes(&scene).unwrap();

    // Import back
    let imported = import_scene_from_bytes(&bytes).unwrap();

    assert_eq!(imported.name, scene.name);
    assert_eq!(imported.meshes.len(), scene.meshes.len());
    assert_eq!(imported.materials.len(), scene.materials.len());
}

#[test]
fn test_phx_scene_to_point_cloud() {
    let scene = sample_scene();
    let cloud = scene.to_point_cloud();

    assert_eq!(cloud.vertex_count(), 3); // Triangle has 3 vertices
}

#[test]
fn test_phx_scene_to_phx() {
    let scene = sample_scene();
    let ply = phx_scene_to_phx(&scene, false).unwrap();

    assert!(ply.contains("ply"));
    assert!(ply.contains("element vertex 3"));
}

#[test]
fn test_phx_scene_magic() {
    assert_eq!(PHX_SCENE_MAGIC, b"PHX_SCENE\x01");
}
