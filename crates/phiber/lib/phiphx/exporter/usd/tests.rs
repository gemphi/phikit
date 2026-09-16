use super::mesh::export_mesh_usda;
use super::point_cloud::export_point_cloud_usda;
use super::*;
use crate::phx::{PointCloud, Vertex};

#[test]
fn test_usda_writer() {
    let mut writer = UsdaWriter::new();
    writer.push_scope("Test", "Xform");
    writer.attr_float3("translation", [1.0, 2.0, 3.0]);
    writer.pop_scope();

    let text = writer.get_text();
    assert!(text.contains("#usda 1.0"));
    assert!(text.contains("def Xform \"Test\""));
    assert!(text.contains("float3 translation = (1.000000, 2.000000, 3.000000)"));
}

#[test]
fn test_export_point_cloud_usda() {
    let mut cloud = PointCloud::new("test");
    cloud.add_vertex(Vertex::new(1.0, 2.0, 3.0).with_color(255, 0, 0, 255));
    cloud.add_vertex(Vertex::new(4.0, 5.0, 6.0));

    let usda = export_point_cloud_usda(&cloud).unwrap();
    assert!(usda.contains("#usda 1.0"));
    assert!(usda.contains("def PointInstancer \"PHXPointCloud\""));
    assert!(usda.contains("point3f[] positions"));
}

#[test]
fn test_export_mesh_usda() {
    let positions = vec![[0.0f32, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]];
    let face_counts = vec![3i32];
    let face_indices = vec![0i32, 1, 2];

    let usda = export_mesh_usda(
        "TestMesh",
        &positions,
        None,
        None,
        None,
        &face_counts,
        &face_indices,
        None,
    )
    .unwrap();

    assert!(usda.contains("def Mesh \"mesh\""));
    assert!(usda.contains("point3f[] points"));
    assert!(usda.contains("int[] faceVertexCounts"));
}
