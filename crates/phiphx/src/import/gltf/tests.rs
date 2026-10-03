//! Tests for the glTF importer.

use super::importer::GltfImporter;
use super::transform::transform_point;

// Minimal valid glTF JSON
const MINIMAL_GLTF: &str = r#"{
    "asset": {"version": "2.0"},
    "scene": 0,
    "scenes": [{"nodes": []}],
    "nodes": [],
    "meshes": []
}"#;

#[test]
fn test_can_import_json_gltf() {
    let data = MINIMAL_GLTF.as_bytes();
    assert!(GltfImporter::can_import(data));
}

#[test]
fn test_cannot_import_random_data() {
    let data = b"not a gltf file";
    assert!(!GltfImporter::can_import(data));
}

#[test]
fn test_trs_to_matrix() {
    // Translation only
    let matrix =
        GltfImporter::trs_to_matrix([1.0, 2.0, 3.0], [0.0, 0.0, 0.0, 1.0], [1.0, 1.0, 1.0]);
    assert_eq!(matrix[12], 1.0);
    assert_eq!(matrix[13], 2.0);
    assert_eq!(matrix[14], 3.0);
}

#[test]
fn test_transform_point() {
    let identity = [
        1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
    ];
    let point = [1.0, 2.0, 3.0];
    let result = transform_point(&point, &identity);
    assert_eq!(result, point);
}
