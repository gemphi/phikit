//! Every driver **variable** kind against Blender's own evaluated matrices
//! (`fixtures/DRIVERVARS.blend`, `truth/blender_driver_var.py`,
//! `truth/driver_var.txt`).
//!
//! `OBJDRIVERS.blend` covers the driver *types* — Average, Python, Sum —
//! over one Single Property variable. This covers the other side: what a
//! variable may read.
//!
//! - **SOURCE** is keyed in location, rotation and scale; everything else
//!   reads it.
//! - **TRANSCHAN** is driven by Transform Channel variables: a location
//!   axis, a scale axis, the average scale (the cube root of the volume),
//!   a rotation in Auto mode, in Quaternion mode and in Swing/Twist Z, and
//!   a parented child's X in all three spaces — World, Transform and Local
//!   are three different answers there, which is what says the space flag
//!   is read and read right.
//! - **DIFFS** is driven by Loc Diff (world and local) and Rot Diff.
//! - **PROPHOST** carries a custom property driven off SOURCE, and
//!   **READER** reads that property — the chain the depsgraph orders, and
//!   the reason custom-property drivers run before channel drivers. READER
//!   is also driven by Min, Max and Average over two Transform Channel
//!   variables, one of which goes negative, which is why the reference is
//!   the matrix itself and not a decomposition.
//!
//! The reference is taken from the **evaluated** depsgraph copy, which is
//! where a driver's result lands.

use phiphx::blend::BlendFile;
use std::path::PathBuf;

const FRAMES: [f32; 5] = [1.0, 6.0, 11.0, 16.0, 21.0];
/// Blender's own matrices print to six decimals; the worst element over the
/// whole sheet comes out under 1e-6, so there is no slack to give here.
const TOLERANCE: f32 = 1e-5;

fn truth() -> Vec<(f32, String, [f32; 16])> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/truth/driver_var.txt");
    std::fs::read_to_string(path)
        .unwrap_or_default()
        .lines()
        .filter_map(|line| {
            let f: Vec<&str> = line.split('\t').collect();
            if f.len() < 18 {
                return None;
            }
            let mut v = [0.0f32; 16];
            for (i, slot) in v.iter_mut().enumerate() {
                *slot = f[2 + i].parse().ok()?;
            }
            Some((f[0].parse().ok()?, f[1].to_string(), v))
        })
        .collect()
}

fn flat(m: [[f32; 4]; 4]) -> [f32; 16] {
    let mut out = [0.0f32; 16];
    for (c, col) in m.iter().enumerate() {
        out[c * 4..c * 4 + 4].copy_from_slice(col);
    }
    out
}

fn fixture() -> Option<Vec<u8>> {
    std::fs::read(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/DRIVERVARS.blend")).ok()
}

#[test]
fn every_driver_variable_kind_puts_its_object_where_blender_puts_it() {
    let Some(bytes) = fixture() else { return eprintln!("no DRIVERVARS.blend; skipped") };
    let want = truth();
    assert!(!want.is_empty(), "the truth file is there");
    let scene = BlendFile::parse_bytes(&bytes).expect("parse").extract_scene().expect("scene");
    let motion = &scene.props.motion;

    let mut worst = (String::new(), 0.0f32, 0usize);
    for frame in FRAMES {
        let world = motion.world_at(frame);
        for (i, m) in motion.list.iter().enumerate() {
            let Some((_, _, blender)) = want.iter().find(|(f, name, _)| *f == frame && *name == m.name) else {
                panic!("no reference for {} at frame {frame}", m.name);
            };
            for (k, (a, b)) in flat(world[i]).iter().zip(blender.iter()).enumerate() {
                let off = (a - b).abs();
                if off > worst.1 {
                    worst = (format!("{} at frame {frame}", m.name), off, k);
                }
                assert!(off < TOLERANCE, "{} at frame {frame}, element {k}: ours {a}, Blender's {b}", m.name);
            }
        }
    }
    eprintln!("worst: {} element {} off by {:.2e}", worst.0, worst.2, worst.1);
}

/// What the reader made of each variable. A variable read as Unsupported
/// still leaves the object somewhere — at 0 — so the kinds are pinned by
/// name rather than left to the matrices alone.
#[test]
fn each_variable_is_read_as_the_kind_the_file_gives_it() {
    use phiphx::blend::{BDriverSpace, BlendDriverRead};
    let Some(bytes) = fixture() else { return eprintln!("no DRIVERVARS.blend; skipped") };
    let scene = BlendFile::parse_bytes(&bytes).expect("parse").extract_scene().expect("scene");
    let set = |name: &str| scene.props.motion.named(name).map(|m| m.drivers.clone()).unwrap_or_default();

    let reads: Vec<BlendDriverRead> =
        set("TRANSCHAN").channels.iter().flat_map(|d| d.driver.vars.iter().map(|v| v.read.clone())).collect();
    assert!(!reads.is_empty(), "TRANSCHAN is driven");
    assert!(
        reads.iter().all(|r| matches!(r, BlendDriverRead::Transform { .. })),
        "every one of its variables is a Transform Channel: {reads:?}"
    );
    for space in [BDriverSpace::World, BDriverSpace::Transform, BDriverSpace::Local] {
        assert!(
            reads.iter().any(|r| matches!(r, BlendDriverRead::Transform { space: s, .. } if *s == space)),
            "and all three spaces are read: {space:?} is missing from {reads:?}"
        );
    }
    assert!(
        reads.iter().any(|r| matches!(r, BlendDriverRead::Transform { chan, .. } if *chan == phiphx::blend::TRANSCHAN_SCALE_AVG)),
        "and one of them is the average scale"
    );

    let diffs: Vec<BlendDriverRead> = set("DIFFS").channels.iter().flat_map(|d| d.driver.vars.iter().map(|v| v.read.clone())).collect();
    assert_eq!(diffs.iter().filter(|r| matches!(r, BlendDriverRead::LocDiff { .. })).count(), 2, "two Loc Diffs: {diffs:?}");
    assert_eq!(diffs.iter().filter(|r| matches!(r, BlendDriverRead::RotDiff { .. })).count(), 1, "one Rot Diff");

    // The chain: a driver on PROPHOST's custom property, and a variable on
    // READER that reads it.
    let host = set("PROPHOST");
    assert_eq!(host.props.len(), 1, "one custom-property driver");
    assert_eq!(host.props[0].name, "gain");
    assert!(host.channels.is_empty(), "and nothing on its own channels");
    assert!(host.saved.get("gain").is_some(), "and the property itself is read off the datablock");

    let reader = set("READER");
    let prop = reader.channels.iter().flat_map(|d| d.driver.vars.iter()).find(|v| matches!(v.read, BlendDriverRead::Prop { .. }));
    let Some(var) = prop else { panic!("READER reads a custom property: {reader:?}") };
    let BlendDriverRead::Prop { name, .. } = &var.read else { unreachable!() };
    assert_eq!(name, "gain");
}

/// The custom-property chain, measured: `PROPHOST["gain"]` is twice
/// `SOURCE.location[0]`, and `READER.location[2]` is that property. A pass
/// that ran the channel drivers first would leave READER at the property's
/// saved 0 for every frame.
#[test]
fn a_driven_custom_property_reaches_the_driver_that_reads_it_in_the_same_frame() {
    let Some(bytes) = fixture() else { return eprintln!("no DRIVERVARS.blend; skipped") };
    let scene = BlendFile::parse_bytes(&bytes).expect("parse").extract_scene().expect("scene");
    let motion = &scene.props.motion;
    let at = |name: &str| motion.list.iter().position(|m| m.name == name).unwrap_or_else(|| panic!("no {name}"));
    let (source, reader) = (at("SOURCE"), at("READER"));
    for frame in FRAMES {
        let raw = motion.raw_at(frame);
        let want = raw[source].loc[0] * 2.0;
        let got = raw[reader].loc[2];
        assert!((got - want).abs() < 1e-4, "frame {frame}: READER.z {got}, twice SOURCE.x {want}");
    }
    // And it is not trivially zero everywhere.
    assert!(motion.raw_at(11.0)[reader].loc[2].abs() > 1.0, "the chain carries a real value at frame 11");
}
