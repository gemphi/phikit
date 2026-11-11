use super::*;
use crate::device::{Device, DeviceKind, DeviceList, Source};
fn triangle() -> SceneModel {
    let mut phx = scene::PhxScene::new("Tri");
    phx.add_mesh(scene::PhxMesh {
        name: "Tri".into(),
        vertices: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
        normals: vec![[0.0, 0.0, 1.0]; 3],
        uvs: Vec::new(),
        colors: Vec::new(),
        indices: vec![[0, 1, 2]],
        materials: Vec::new(),
        matrix_world: crate::scene::IDENTITY_M4,
        visible: true,
    });
    open::model(&phx)
}
#[test]
fn it_takes_phx_heads_and_the_extension() {
    let list = DeviceList::new();
    assert_eq!(list.by_extension("PHX").map(|d| d.kind()), Some(DeviceKind::Phx));
    assert_eq!(list.detect(b"PHX_SCENE\x01....").map(|d| d.kind()), Some(DeviceKind::Phx));
    assert_eq!(list.detect(b"PHX\0\x01\0\0\0").map(|d| d.kind()), Some(DeviceKind::Phx));
}
#[test]
fn a_scene_survives_write_and_read() {
    let model = triangle();
    let bytes = PhxDevice.write(&model).unwrap();
    assert!(PhxDevice.detect(&bytes));
    let back = PhxDevice
        .read(&Source { name: "tri.phx", path: None, bytes: &bytes }, &|_, _| true)
        .unwrap();
    assert_eq!(back.objects.len(), 1);
    assert_eq!(back.objects[0].mesh.vertices, model.objects[0].mesh.vertices);
    assert_eq!(back.objects[0].mesh.indices, vec![0, 1, 2]);
}
#[test]
fn peek_names_the_scene() {
    let bytes = PhxDevice.write(&triangle()).unwrap();
    let peek = PhxDevice.peek(&bytes).unwrap();
    assert_eq!(peek.scenes[0].scene, "Tri");
}
#[test]
fn a_point_cloud_is_not_a_scene() {
    let err = PhxDevice
        .read(&Source { name: "c.phx", path: None, bytes: b"PHX\0\x01\0\0\0" }, &|_, _| true)
        .unwrap_err();
    assert_eq!(err.category(), "unsupported");
}