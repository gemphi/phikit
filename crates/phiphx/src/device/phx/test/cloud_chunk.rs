use super::*;

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

    let phx_data = PhxChunkList::mesh(&mesh, "test_mesh").unwrap();
    assert!(!phx_data.is_empty());
    assert_eq!(&phx_data[0..4], PHX_MAGIC);
}
