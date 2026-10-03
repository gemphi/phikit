//! The drivers the Drivers editor lists are Blender's: every driven
//! object's driver F-curves — path, index, keys and the Generator
//! modifier's polynomial — against `truth/thymus_drivers.txt`
//! (`truth/blender_drivers.py`).

use phiphx::scene::SceneModel;

const FILE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/THYMUS-BOLDROID-WAVE-003.blend");
const TRUTH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/truth/thymus_drivers.txt");

#[test]
fn every_driven_object_lists_blenders_driver_curves_and_generators() {
    let bytes = std::fs::read(FILE).expect("fixture");
    let blend = phiphx::blend::BlendFile::parse_bytes(&bytes).expect("parse").extract_scene().expect("scene");
    let model = SceneModel::from_blend_owned(blend, FILE);
    let want: Vec<String> = std::fs::read_to_string(TRUTH).expect("truth").lines().map(str::to_string).collect();
    let have: Vec<String> = model
        .drivers
        .iter()
        .flat_map(|drivers| {
            let mut lines = vec![format!("object\t{}\t{}", drivers.object, drivers.curves.len())];
            lines.extend(drivers.curves.iter().map(|curve| {
                let coefficients = curve.generator.as_ref().map_or("-".to_string(), |c| c.iter().map(|v| format!("{v}")).collect::<Vec<_>>().join(" "));
                format!("driver\t{}\t{}\t{}\t{coefficients}", curve.path, curve.index, curve.keys.len())
            }));
            lines
        })
        .collect();
    let missing: Vec<&String> = want.iter().filter(|line| !have.contains(line)).collect();
    assert_eq!(have.len(), want.len(), "one line per object and driver; Blender has and phiphx lacks: {missing:#?}");
    for (index, (have, want)) in have.iter().zip(&want).enumerate() {
        assert_eq!(have, want, "line {index}");
    }
    let rig = &model.drivers[0];
    assert_eq!(rig.object, "rig");
    let inverted = rig.curves.iter().find(|c| c.generator.as_deref() == Some(&[1.0, -1.0])).expect("a `1 − x` generator");
    assert_eq!(inverted.generated(0.25), Some(0.75));
    let plain = rig.curves.iter().find(|c| c.generator.is_none()).expect("a driver with no modifier");
    assert_eq!(plain.generated(0.5), None);
}
