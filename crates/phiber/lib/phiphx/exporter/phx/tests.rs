use super::import_point_cloud_from_phx;
use crate::phx::Mesh as ImportMesh;
use crate::phx::Vertex;
use crate::phx::exporter::blend::chunks::{
    PHX_MAGIC, PHX_VERSION, PhxChunkType, PhxHeader, export_mesh_to_phx, export_point_cloud_to_phx,
};

#[test]
fn test_phx_header_write_read() {
    let header = PhxHeader::new(5);
    let mut buf = Vec::new();
    header.write(&mut buf).unwrap();

    let mut reader = &buf[..];
    let read_header = PhxHeader::read(&mut reader).unwrap();

    assert_eq!(read_header.magic, *PHX_MAGIC);
    assert_eq!(read_header.version, PHX_VERSION);
    assert_eq!(read_header.chunk_count, 5);
}

#[test]
fn test_export_import_mesh_roundtrip() {
    let mut mesh = ImportMesh::new();
    mesh.positions = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]];
    mesh.normals = Some(vec![[0.0, 0.0, 1.0], [0.0, 0.0, 1.0], [0.0, 0.0, 1.0]]);
    mesh.indices = vec![[0, 1, 2]];

    let phx_data = export_mesh_to_phx(&mesh, "test_mesh").unwrap();
    assert!(!phx_data.is_empty());
    assert_eq!(&phx_data[0..4], PHX_MAGIC);
}

#[test]
fn test_export_point_cloud_roundtrip() {
    use crate::phx::PointCloud;

    let mut cloud = PointCloud::new("test_cloud");
    cloud.add_vertex(Vertex::new(1.0, 2.0, 3.0).with_color(255, 128, 64, 255));
    cloud.add_vertex(Vertex::new(4.0, 5.0, 6.0).with_normal(0.0, 1.0, 0.0));

    let phx_data = export_point_cloud_to_phx(&cloud).unwrap();
    assert!(!phx_data.is_empty());
    assert_eq!(&phx_data[0..4], PHX_MAGIC);

    // Read back
    let imported = import_point_cloud_from_phx(&phx_data).unwrap();
    assert_eq!(imported.vertex_count(), 2);
}

#[test]
fn test_phx_chunk_type_values() {
    assert_eq!(PhxChunkType::Mesh as u32, 0x4D455348);
    assert_eq!(PhxChunkType::PointCloud as u32, 0x5054434C);
    assert_eq!(PhxChunkType::Material as u32, 0x4D41544C);
    assert_eq!(PhxChunkType::Scene as u32, 0x53434E45);
    assert_eq!(PhxChunkType::End as u32, 0x454E4421);
}
