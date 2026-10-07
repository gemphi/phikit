//! Modifier drivers against Blender's own evaluated modifiers
//! (`fixtures/MODDRIVERS.blend`, `truth/blender_mod_driver_fixture.py`,
//! `truth/blender_mod_driver.py`, `truth/mod_driver.txt`).
//!
//! What this measures is the two rules a modifier setting adds over the
//! material, light and world passes:
//!
//! - **Integer settings truncate toward zero.** A driver's value is a
//!   float; an integer property takes it with a C `(int)` cast, so 0.7 → 0
//!   and 2.5 → 2, and on the negative side −0.7 → 0 and −2.5 → −2 where a
//!   floor would give −1 and −3. The fixture keys SOURCE on exactly those
//!   values with CONSTANT interpolation so each frame lands on one.
//!   `Solidify.material_offset` is in the sheet because it is the only
//!   setting here whose range reaches below zero, and so the only one that
//!   can tell truncation from floor.
//! - **The hard minimum is not always zero.** `Subsurf.quality` starts at
//!   1 and `Triangulate.min_vertices` at 4, so a driver cannot take either
//!   to zero.
//!
//! Plus the things that must keep working: a float setting clamped at 0
//! (`Weld` / `Mirror`'s thresholds) against one that is unbounded
//! (`Displace.strength`), the same property name on two different
//! modifiers, a vector setting by component, a muted driver, and a second
//! object with its own driver so the per-object split is covered.
//!
//! The values are read and typed here but not yet written into the
//! modifier stack — see `blend/modifier/driver.rs`'s header for why that
//! is a separate change.

use phiphx::blend::BlendFile;
use phiphx::blend::{BModValue, BModWrite};
use std::path::PathBuf;

const FRAMES: [f32; 5] = [1.0, 6.0, 11.0, 16.0, 21.0];
const TOLERANCE: f32 = 1e-5;

/// One expected row: frame, object, and the write with Blender's value.
struct Row {
    frame: f32,
    object: String,
    write: BModWrite,
    value: BModValue,
}

fn truth() -> Vec<Row> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/truth/mod_driver.txt");
    std::fs::read_to_string(path)
        .unwrap_or_default()
        .lines()
        .filter_map(|line| {
            // <frame> <object> <modifier> <type> <property> <component> <value>
            let f: Vec<&str> = line.split(' ').collect();
            if f.len() != 7 {
                return None;
            }
            let cell = f[6];
            // An integer prints without a point; a float always has one.
            let value = if cell.contains('.') {
                BModValue::Float(cell.parse().ok()?)
            } else {
                BModValue::Int(cell.parse().ok()?)
            };
            Some(Row {
                frame: f[0].parse().ok()?,
                object: f[1].to_string(),
                write: BModWrite {
                    modifier: f[2].to_string(),
                    kind: f[3].to_string(),
                    prop: f[4].to_string(),
                    component: f[5].parse().ok()?,
                },
                value,
            })
        })
        .collect()
}

fn fixture() -> Option<Vec<u8>> {
    std::fs::read(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/MODDRIVERS.blend")).ok()
}

#[test]
fn every_modifier_driver_writes_what_blender_writes() {
    let Some(bytes) = fixture() else { return eprintln!("no MODDRIVERS.blend; skipped") };
    let want = truth();
    assert!(!want.is_empty(), "the truth file is there");
    let scene = BlendFile::parse_bytes(&bytes).expect("parse").extract_scene().expect("scene");
    let sets = &scene.props.modifier_drivers;
    assert_eq!(
        sets.len(),
        2,
        "two objects carry modifier drivers: {:?}",
        sets.iter().map(|s| &s.object).collect::<Vec<_>>()
    );

    // The one muted driver: `Lat.strength`, saved at 0.625.
    const MUTED: (&str, &str) = ("Lat", "strength");

    let mut checked = 0usize;
    let mut worst = (String::new(), 0.0f32);
    for frame in FRAMES {
        let state = scene.props.motion.driver_state_at(frame);
        let ds = state.scene();
        for set in sets {
            let written = set.values(&ds, frame);
            for row in want.iter().filter(|r| r.frame == frame && r.object == set.object) {
                let what = format!(
                    "{} {}({}).{}[{}] at frame {frame}",
                    set.object, row.write.modifier, row.write.kind, row.write.prop, row.write.component
                );
                let Some((_, ours)) = written.iter().find(|(w, _)| *w == row.write) else {
                    // Nothing written means a muted driver, and Blender's
                    // row is then the value the file saved.
                    assert_eq!(
                        (row.write.modifier.as_str(), row.write.prop.as_str()),
                        MUTED,
                        "{what}: Blender wrote {:?} and nothing here claims it",
                        row.value
                    );
                    assert_eq!(row.value, BModValue::Float(0.625), "{what}: the saved value stands");
                    checked += 1;
                    continue;
                };
                // The type must match, not just the number: an integer
                // setting read as a float would pass a numeric compare at
                // whole values and hide the truncation.
                match (*ours, row.value) {
                    (BModValue::Int(a), BModValue::Int(b)) => {
                        assert_eq!(a, b, "{what}: ours {a}, Blender's {b}");
                    }
                    (BModValue::Float(a), BModValue::Float(b)) => {
                        let off = (a - b).abs();
                        if off > worst.1 {
                            worst = (what.clone(), off);
                        }
                        assert!(off < TOLERANCE, "{what}: ours {a}, Blender's {b}");
                    }
                    (a, b) => panic!("{what}: kind differs — ours {a:?}, Blender's {b:?}"),
                }
                checked += 1;
            }
        }
    }
    assert!(checked > 70, "only {checked} values compared");
    eprintln!("{checked} values compared; worst float off by {:.2e} ({})", worst.1, worst.0);
}

/// The truncation, named against the sheet, so a regression says which
/// rule broke rather than only that a number moved.
#[test]
fn an_integer_setting_truncates_toward_zero_in_the_file_too() {
    let Some(bytes) = fixture() else { return eprintln!("no MODDRIVERS.blend; skipped") };
    let scene = BlendFile::parse_bytes(&bytes).expect("parse").extract_scene().expect("scene");
    let set = scene.props.modifier_drivers.iter().find(|s| s.object == "OB_MODS").expect("OB_MODS");

    let at = |frame: f32, modifier: &str, prop: &str| -> Option<BModValue> {
        let state = scene.props.motion.driver_state_at(frame);
        let ds = state.scene();
        set.values(&ds, frame)
            .into_iter()
            .find(|(w, _)| w.modifier == modifier && w.prop == prop && w.component == 0)
            .map(|(_, v)| v)
    };

    // Frame 1 is z = −0.7, frame 6 is 2.5, frame 11 is 0.5, frame 16 is
    // 11.9, frame 21 is −2.5 — each chosen so a round or a floor differs.
    assert_eq!(at(1.0, "Subsurf", "levels"), Some(BModValue::Int(0)), "−0.7 truncates to 0 and clamps there");
    assert_eq!(at(6.0, "Subsurf", "levels"), Some(BModValue::Int(2)), "2.5 truncates to 2, it does not round to 3");
    assert_eq!(at(11.0, "Subsurf", "levels"), Some(BModValue::Int(0)), "0.5 truncates to 0");
    assert_eq!(at(16.0, "Subsurf", "levels"), Some(BModValue::Int(11)), "11.9 truncates to 11");

    // `material_offset` reaches below zero, which is where floor differs.
    assert_eq!(at(1.0, "Sol", "material_offset"), Some(BModValue::Int(0)), "−0.7 → 0; a floor would give −1");
    assert_eq!(at(21.0, "Sol", "material_offset"), Some(BModValue::Int(-2)), "−2.5 → −2; a floor would give −3");

    // The non-zero minima.
    assert_eq!(at(11.0, "Subsurf", "quality"), Some(BModValue::Int(1)), "quality cannot go below one");
    assert_eq!(at(1.0, "Tri", "min_vertices"), Some(BModValue::Int(4)), "min_vertices cannot go below four");
    assert_eq!(at(16.0, "Tri", "min_vertices"), Some(BModValue::Int(11)));

    // A float beside them, to show the kinds are not confused.
    assert_eq!(at(1.0, "Disp", "strength"), Some(BModValue::Float(-0.7)), "an unbounded float keeps its sign and fraction");
    assert_eq!(at(1.0, "Weld", "merge_threshold"), Some(BModValue::Float(0.0)), "a distance clamps at zero");
}

/// The setting table is keyed by modifier type, because the name is not
/// enough — and a driver on something the table has no entry for is read
/// and dropped rather than guessed at.
#[test]
fn a_modifiers_type_decides_what_its_property_name_means() {
    let Some(bytes) = fixture() else { return eprintln!("no MODDRIVERS.blend; skipped") };
    let scene = BlendFile::parse_bytes(&bytes).expect("parse").extract_scene().expect("scene");
    let set = scene.props.modifier_drivers.iter().find(|s| s.object == "OB_MODS").expect("OB_MODS");

    // Every driver resolved its modifier's type from the object's stack.
    for d in &set.drivers {
        assert!(!d.write.kind.is_empty(), "{} has no type", d.write.modifier);
    }
    let kind_of = |modifier: &str| set.drivers.iter().find(|d| d.write.modifier == modifier).map(|d| d.write.kind.clone());
    assert_eq!(kind_of("Subsurf").as_deref(), Some("SUBSURF"));
    assert_eq!(kind_of("Disp").as_deref(), Some("DISPLACE"));
    assert_eq!(kind_of("Sol").as_deref(), Some("SOLIDIFY"));
    assert_eq!(kind_of("Mirror").as_deref(), Some("MIRROR"));
    assert_eq!(kind_of("Weld").as_deref(), Some("WELD"));
    assert_eq!(kind_of("Array").as_deref(), Some("ARRAY"));

    // `merge_threshold` is driven on both Weld and Mirror, and both are
    // read — the name alone would have collided.
    let thresholds: Vec<&str> = set
        .drivers
        .iter()
        .filter(|d| d.write.prop == "merge_threshold")
        .map(|d| d.write.modifier.as_str())
        .collect();
    assert_eq!(thresholds.len(), 2, "both merge_thresholds: {thresholds:?}");

    // The vector setting came through by component.
    let mut components: Vec<usize> = set
        .drivers
        .iter()
        .filter(|d| d.write.prop == "constant_offset_displace")
        .map(|d| d.write.component)
        .collect();
    components.sort();
    assert_eq!(components, vec![0, 2], "the two components the fixture drives");
}
