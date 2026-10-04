//! A `.blend` written back is the `.blend` that was read — byte for byte
//! when nothing changed, and with exactly the session's edits when
//! something did — in every layout Blender has written: the legacy
//! `SmallBHead8` file (3.x), the 5.0 `LargeBHead8` file, and a
//! zstd-wrapped one, each re-read through phiphx's own reader.

use phiphx::blend::write::mesh::BlendNewMesh;
use phiphx::blend::write::{BlendCompression, BlendDuplicate, BlendEdits, BlendNewObject, BlendObjectEdit};
use phiphx::blend::{BlendFile, BlendVisibility};

const FIXTURES: &[&str] = &["THYMUS-BOLDROID-WAVE-003.blend", "nla_cube.blend", "DEPTH-OF-FIELD.blend"];

fn fixture(name: &str) -> Vec<u8> {
    std::fs::read(format!("{}/tests/fixtures/{name}", env!("CARGO_MANIFEST_DIR"))).expect("fixture")
}

#[test]
fn unchanged_files_write_back_as_the_bytes_they_were_read_from() {
    for name in FIXTURES {
        let bytes = fixture(name);
        let compression = BlendCompression::of(&bytes);
        let file = BlendFile::parse_bytes(&bytes).expect("parse");
        let raw = file.to_bytes();
        if compression == BlendCompression::None {
            assert!(raw == bytes, "{name}: byte for byte");
        } else {
            // A compressed file's bytes depend on the encoder; its content does not.
            let again = BlendFile::parse_bytes(&compression.wrap(raw.clone()).expect("wrap")).expect("parse");
            assert!(again.to_bytes() == raw, "{name}: the same file inside the wrapping");
        }
    }
}

#[test]
fn the_edits_are_the_only_difference_and_read_back_in_every_layout() {
    for name in FIXTURES {
        let bytes = fixture(name);
        let compression = BlendCompression::of(&bytes);
        let mut file = BlendFile::parse_bytes(&bytes).expect("parse");
        let before = file.extract_scene().expect("scene");
        let first = before.objects.first().map(|o| o.name.clone()).expect("an object");
        let edits = BlendEdits {
            frame: Some(42),
            frame_start: Some(3),
            frame_end: Some(77),
            objects: vec![BlendObjectEdit {
                name: first.clone(),
                rename: Some("Renamed".into()),
                visibility: Some(BlendVisibility { hide_viewport: false, hide_select: false, hide_render: true }),
                hidden: Some(false),
                selected: Some(true),
                transform: None,
                shadow: None,
            }],
            active: Some(first.clone()),
            markers: Some(vec![(5, "A".into()), (50, "B".into())]),
            ..Default::default()
        };
        let report = edits.apply(&mut file);
        assert!(report.warnings.is_empty(), "{name}: {:?}", report.warnings);
        let written = compression.wrap(file.to_bytes()).expect("wrap");
        let after = BlendFile::parse_bytes(&written).expect("parse the written file").extract_scene().expect("scene");
        assert_eq!((after.props.frame_current, after.props.frame_start, after.props.frame_end), (42, 3, 77), "{name}");
        let index = after.objects.iter().position(|o| o.name == "Renamed").unwrap_or_else(|| panic!("{name}: the rename"));
        assert!(!after.objects.iter().any(|o| o.name == first), "{name}: the old name is gone");
        assert!(after.objects[index].visibility.hide_render, "{name}: the camera toggle");
        assert!(after.selected.contains(&index) && !after.hidden.contains(&index), "{name}: the base flags");
        assert_eq!(after.active_object, Some(index), "{name}: the active object");
        assert_eq!(after.props.markers.iter().map(|m| (m.frame, m.name.as_str())).collect::<Vec<_>>(), [(5, "A"), (50, "B")], "{name}");
        // Everything else is as it was — but an object that follows another
        // (a camera on a path, a tracker) stands where frame 42 puts it.
        assert_eq!(after.objects.len(), before.objects.len(), "{name}");
        assert_eq!(after.collections.len(), before.collections.len(), "{name}");
        let at_42 = before.props.motion.solve(42.0, &before.props.rigs.rigs, false).world;
        for (i, (a, b)) in after.objects.iter().zip(&before.objects).enumerate() {
            assert_eq!(a.vertices.len(), b.vertices.len(), "{name}: {}", b.name);
            assert_eq!(a.matrix_local, b.matrix_local, "{name}: {}", b.name);
            let world = if before.props.motion.follower(i) { at_42[i] } else { b.matrix_world };
            assert_eq!(a.matrix_world, world, "{name}: {}", b.name);
        }
    }
}

/// A cube added, a linked duplicate of a file object, and a file object
/// deleted: the saved file has them so, the rest exactly as it was, in
/// the 3.6 and the 5.0 layouts.
#[test]
fn new_objects_are_written_and_read_back_with_their_geometry() {
    let cube = BlendNewMesh {
        name: "Crate".into(),
        vertices: vec![[-1., -1., -1.], [1., -1., -1.], [1., 1., -1.], [-1., 1., -1.], [-1., -1., 1.], [1., -1., 1.], [1., 1., 1.], [-1., 1., 1.]],
        faces: vec![vec![0, 1, 2, 3], vec![4, 7, 6, 5], vec![0, 4, 5, 1], vec![1, 5, 6, 2], vec![2, 6, 7, 3], vec![3, 7, 4, 0]],
    };
    for name in ["THYMUS-BOLDROID-WAVE-003.blend", "nla_cube.blend"] {
        let bytes = fixture(name);
        let mut file = BlendFile::parse_bytes(&bytes).expect("parse");
        let before = file.extract_scene().expect("scene");
        let first = before.objects[0].name.clone();
        let last = before.objects.last().unwrap().name.clone();
        let edits = BlendEdits {
            added: vec![BlendNewObject { name: "Crate".into(), mesh: cube.clone(), location: [1.0, 2.0, 3.0], scale: [1.0; 3], collection: String::new(), selected: true, visibility: None }],
            duplicated: vec![BlendDuplicate { copy: format!("{first}.001"), source: first.clone(), selected: false }],
            deleted: vec![last.clone()],
            active: Some("Crate".into()),
            ..Default::default()
        };
        let report = edits.apply(&mut file);
        assert!(report.warnings.is_empty(), "{name}: {:?}", report.warnings);
        let after = BlendFile::parse_bytes(&file.to_bytes()).expect("the written file parses").extract_scene().expect("scene");
        assert_eq!(after.objects.len(), before.objects.len() + 1, "{name}: one added, one duplicated, one deleted");
        let crate_ = after.objects.iter().find(|o| o.name == "Crate").unwrap_or_else(|| panic!("{name}: the crate"));
        assert_eq!((crate_.vertices.len(), crate_.faces.len()), (8, 6), "{name}");
        assert_eq!(crate_.location, [1.0, 2.0, 3.0], "{name}: where it was put");
        let index = after.objects.iter().position(|o| o.name == "Crate").unwrap();
        assert!(after.selected.contains(&index), "{name}: selected");
        assert_eq!(after.active_object, Some(index), "{name}: active");
        let copy = after.objects.iter().find(|o| o.name == format!("{first}.001")).unwrap_or_else(|| panic!("{name}: the duplicate"));
        let source = after.objects.iter().find(|o| o.name == first).unwrap();
        assert_eq!(copy.vertices.len(), source.vertices.len(), "{name}: the duplicate shares the mesh");
        assert_eq!(copy.location, source.location, "{name}: in place");
        if first != last {
            assert!(!after.objects.iter().any(|o| o.name == last), "{name}: {last} is gone");
        }
        for b in &before.objects {
            if b.name == last {
                continue;
            }
            let a = after.objects.iter().find(|o| o.name == b.name).unwrap_or_else(|| panic!("{name}: {} kept", b.name));
            assert_eq!((a.vertices.len(), a.faces.len()), (b.vertices.len(), b.faces.len()), "{name}: {}", b.name);
        }
    }
}

/// The startup files phiber's New File reads (`phiber/assets/blend/
/// startup-{3,4,5}.blend`) are Blender's own factory scenes, one per major
/// version, each written by that Blender (`assets/blend/make_startup.py`
/// with `bpy` 4.2 / 5.0; 3.6's is `release/datafiles/startup.blend` at the
/// `v3.6.23` tag, zstd-wrapped). This checks each one saves through the
/// writer in its own layout: a cube added to it reads back as 8 vertices
/// and 6 faces beside the factory Cube, Light and Camera.
#[test]
fn every_startup_template_saves_through_in_its_own_layout() {
    let assets = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../phiber/assets/blend");
    for (file, version) in [("startup-3.blend", "3.6"), ("startup-4.blend", "4.2"), ("startup-5.blend", "5.0")] {
        let mut blend = BlendFile::parse_bytes(&std::fs::read(assets.join(file)).unwrap()).unwrap();
        assert_eq!(blend.header.version_label(), version, "{file}");
        let cube = BlendNewMesh {
            name: "Added".into(),
            vertices: vec![[-1., -1., -1.], [1., -1., -1.], [1., 1., -1.], [-1., 1., -1.], [-1., -1., 1.], [1., -1., 1.], [1., 1., 1.], [-1., 1., 1.]],
            faces: vec![vec![0, 1, 2, 3], vec![4, 7, 6, 5], vec![0, 4, 5, 1], vec![1, 5, 6, 2], vec![2, 6, 7, 3], vec![3, 7, 4, 0]],
        };
        let added = BlendNewObject { name: "Added".into(), mesh: cube, location: [3.0, 0.0, 0.0], scale: [1.0; 3], collection: String::new(), selected: true, visibility: None };
        let edits = BlendEdits { added: vec![added], ..Default::default() };
        let report = edits.apply(&mut blend);
        assert!(report.warnings.is_empty(), "{file}: {:?}", report.warnings);
        let again = BlendFile::parse_bytes(&blend.to_bytes()).unwrap().extract_scene().unwrap();
        let mut names: Vec<&str> = again.objects.iter().map(|o| o.name.as_str()).collect();
        names.sort();
        assert_eq!(names, ["Added", "Camera", "Cube", "Light"], "{file}");
        let added = again.objects.iter().find(|o| o.name == "Added").unwrap();
        assert_eq!((added.vertices.len(), added.faces.len()), (8, 6), "{file}");
    }
}
