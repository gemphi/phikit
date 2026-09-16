use super::mesh::export_mesh_gltf;
use super::point_cloud::export_point_cloud_gltf;
use super::*;
use crate::phx::{PointCloud, PointCloudError, Result, Vertex};

#[test]
fn test_gltf_buffer_writer() {
    let mut writer = GltfBufferWriter::new();
    let data = vec![1.0f32, 2.0, 3.0, 4.0, 5.0, 6.0];
    let idx = writer.write_f32_array(&data, ARRAY_BUFFER);
    assert_eq!(idx, 0);

    let (buffer, buffers, views, accessors) = writer.finalize();
    assert!(!buffer.is_empty());
    assert_eq!(buffers.len(), 1);
    assert_eq!(views.len(), 1);
    assert_eq!(accessors.len(), 1);
}

#[test]
fn test_export_point_cloud_gltf() {
    let mut cloud = PointCloud::new("test");
    cloud.add_vertex(Vertex::new(1.0, 2.0, 3.0).with_normal(0.0, 1.0, 0.0));
    cloud.add_vertex(Vertex::new(4.0, 5.0, 6.0));

    let gltf = export_point_cloud_gltf(&cloud, false).unwrap();
    assert!(gltf.contains("\"asset\""));
    assert!(gltf.contains("\"version\": \"2.0\""));
    assert!(gltf.contains("\"POSITION\""));
}

#[test]
fn test_export_mesh_gltf() {
    let positions = vec![[0.0f32, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]];
    let indices = vec![[0u32, 1, 2]];

    let gltf = export_mesh_gltf(&positions, None, None, &indices).unwrap();
    assert!(gltf.contains("\"mode\": 4")); // TRIANGLES
    assert!(gltf.contains("\"indices\""));
}
