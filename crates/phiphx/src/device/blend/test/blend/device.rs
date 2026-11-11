//! `BlendDevice`: the `Device` contract over a fixture `.blend`.
use crate::device::{DeviceKind, DeviceList, Source};

const FIXTURE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/actions.blend");

#[test]
fn the_list_asks_usd_then_blend() {
    let kinds: Vec<DeviceKind> = DeviceList::new().iter().map(|d| d.kind()).collect();
    assert_eq!(kinds, vec![DeviceKind::Usd, DeviceKind::Blend]);
}

#[test]
fn it_takes_blend_heads_and_the_extension() {
    let list = DeviceList::new();
    assert_eq!(list.by_extension("BLEND").map(|d| d.kind()), Some(DeviceKind::Blend));
    assert_eq!(list.detect(b"BLENDER17-01v0502").map(|d| d.kind()), Some(DeviceKind::Blend));
    assert_eq!(list.detect(&[0x28, 0xB5, 0x2F, 0xFD, 0, 0]).map(|d| d.kind()), Some(DeviceKind::Blend));
    assert!(list.detect(b"#usda 1.0").is_some_and(|d| d.kind() == DeviceKind::Usd));
    assert!(list.detect(b"not a scene").is_none());
}

#[test]
fn it_peeks_and_reads_the_fixture_from_bytes_and_from_a_path() {
    let bytes = std::fs::read(FIXTURE).expect("fixture");
    let device = DeviceList::new().get(DeviceKind::Blend).expect("blend device");
    assert!(!device.peek(&bytes).expect("peek").version.is_empty());

    let from_bytes = Source { name: "actions.blend", path: None, bytes: &bytes };
    let a = device.read(&from_bytes, &|_, _| true).expect("read bytes");
    let path = std::path::Path::new(FIXTURE);
    let from_path = Source { name: "actions.blend", path: Some(path), bytes: &bytes };
    let b = device.read(&from_path, &|_, _| true).expect("read path");
    assert!(!a.objects.is_empty());
    assert_eq!(a.objects.len(), b.objects.len());

    let (c, _deferred) = device.read_progressive(&from_path, &|_, _| true).expect("progressive");
    assert_eq!(c.objects.len(), a.objects.len());
}

#[test]
fn a_scene_is_not_written_back_as_blend() {
    let device = DeviceList::new().get(DeviceKind::Blend).expect("blend device");
    let scene = crate::scene::SceneModel::default();
    assert!(device.write(&scene).is_err());
}
