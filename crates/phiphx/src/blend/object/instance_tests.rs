//! Collection instances against Blender 5.2: two empties instancing a
//! collection kept out of the scene, a member parented to another, the
//! collection's instance offset, the empties turned and scaled
//! (`tests/fixtures/make_collection_instances.py`, Blender's
//! `depsgraph.object_instances` in `tests/truth/collection_instances.txt`).

use crate::blend::file::types::BlendFile;

const FILE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/collection_instances.blend");
const TRUTH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/truth/collection_instances.txt");

#[test]
fn every_instance_stands_where_blender_draws_it() {
    let file = BlendFile::parse_bytes(&std::fs::read(FILE).expect("fixture")).expect("parse");
    let scene = file.extract_scene().expect("scene");
    let truth = std::fs::read_to_string(TRUTH).expect("truth");
    let mut seen = 0;
    for line in truth.lines() {
        let (name, values) = line.split_once('\t').expect("name and matrix");
        let want: Vec<f32> = values.split(' ').map(|v| v.parse().expect("a number")).collect();
        let found = scene.objects.iter().find(|o| o.name == name).unwrap_or_else(|| panic!("{name} is drawn"));
        assert!(found.is_instance && found.block_address == 0, "{name} is an instance");
        assert!(!found.vertices.is_empty(), "{name} carries its member's mesh");
        let ours: Vec<f32> = found.matrix_world.iter().flatten().copied().collect();
        let worst = ours.iter().zip(&want).map(|(a, b)| (a - b).abs()).fold(0.0, f32::max);
        assert!(worst < 1e-5, "{name}: {ours:?}, Blender {want:?}");
        seen += 1;
    }
    assert_eq!(seen, 4, "two members, two instancers");
    // The collection is not in the scene: its objects are seen only as
    // instances.
    assert!(!scene.objects.iter().any(|o| o.name == "Pan" || o.name == "Back"));
}
