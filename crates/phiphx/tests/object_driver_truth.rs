//! Object drivers against Blender's own evaluated matrices
//! (`fixtures/OBJDRIVERS.blend`, `truth/blender_object_drivers.py`,
//! `truth/object_drivers.txt`): a cube keyed moving in X, and three objects
//! whose own transform channels are driven from it —
//!
//! - **Wheel**: `rotation_euler[2]` = Average of one variable, through a
//!   Generator of `0.5 + 2x`, which is what makes the driver's **F-curve**
//!   (not the frame) the thing being mapped.
//! - **Scaled**: `scale[0]` = the Python expression `1 + x / 10`.
//! - **Summed**: `location[2]` = the Sum of the driver's X and Y.
//! - **Still**: nothing, so the list carries an undriven object too.
//!
//! The reference is taken from the **evaluated** depsgraph copy, which is
//! where a driver's result lands; the matrices are decomposed so each
//! channel can be compared on its own.

use phiphx::blend::BlendFile;
use std::path::PathBuf;

const FRAMES: [f32; 5] = [1.0, 6.0, 11.0, 16.0, 21.0];

fn truth() -> Vec<(f32, String, [f32; 9])> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/truth/object_drivers.txt");
    std::fs::read_to_string(path)
        .unwrap_or_default()
        .lines()
        .filter_map(|line| {
            let f: Vec<&str> = line.split('\t').collect();
            if f.len() < 11 {
                return None;
            }
            let mut v = [0.0f32; 9];
            for (i, slot) in v.iter_mut().enumerate() {
                *slot = f[2 + i].parse().ok()?;
            }
            Some((f[0].parse().ok()?, f[1].to_string(), v))
        })
        .collect()
}

/// A world matrix decomposed the way Blender's `matrix_world.decompose()`
/// does: translation, XYZ Euler, scale.
fn decompose(m: [[f32; 4]; 4]) -> [f32; 9] {
    let m = glam::Mat4::from_cols_array_2d(&m);
    let (scale, rot, loc) = m.to_scale_rotation_translation();
    let (x, y, z) = rot.to_euler(glam::EulerRot::XYZ);
    [loc.x, loc.y, loc.z, x, y, z, scale.x, scale.y, scale.z]
}

#[test]
fn a_driven_objects_channels_are_blenders_at_every_frame() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/OBJDRIVERS.blend");
    let Ok(bytes) = std::fs::read(&path) else { return eprintln!("no OBJDRIVERS.blend; skipped") };
    let want = truth();
    assert!(!want.is_empty(), "the truth file is there");
    let blend = BlendFile::parse_bytes(&bytes).expect("parse");
    let scene = blend.extract_scene().expect("scene");
    let motion = &scene.props.motion;
    assert!(motion.animated(), "something moves");

    // The drivers reached the reader at all.
    let driven: Vec<&str> = motion.list.iter().filter(|m| !m.drivers.is_empty()).map(|m| m.name.as_str()).collect();
    let mut sorted = driven.clone();
    sorted.sort_unstable();
    assert_eq!(sorted, ["Scaled", "Summed", "Wheel"], "three driven objects, and Still is not one: {driven:?}");

    let mut worst = (String::new(), 0.0f32, 0usize);
    for frame in FRAMES {
        let world = motion.world_at(frame);
        for (i, m) in motion.list.iter().enumerate() {
            let Some((_, _, blender)) = want.iter().find(|(f, name, _)| *f == frame && *name == m.name) else {
                panic!("no reference for {} at frame {frame}", m.name);
            };
            let ours = decompose(world[i]);
            for (k, (a, b)) in ours.iter().zip(blender.iter()).enumerate() {
                let off = (a - b).abs();
                if off > worst.1 {
                    worst = (format!("{} at frame {frame}", m.name), off, k);
                }
                assert!(off < 2e-4, "{} at frame {frame}, channel {k}: ours {a}, Blender's {b}", m.name);
            }
        }
    }
    eprintln!("worst: {} channel {} off by {:.2e}", worst.0, worst.2, worst.1);
}

/// What each driver is, as read: the kind, the variables and the curve the
/// driver's value is mapped through. A driver read with the wrong kind
/// still moves something, so the kinds are pinned by name.
#[test]
fn each_driver_is_read_with_its_kind_its_variables_and_its_curve() {
    use phiphx::blend::BlendDriverRead;
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/OBJDRIVERS.blend");
    let Ok(bytes) = std::fs::read(&path) else { return eprintln!("no OBJDRIVERS.blend; skipped") };
    let scene = BlendFile::parse_bytes(&bytes).expect("parse").extract_scene().expect("scene");
    let of = |name: &str| scene.props.motion.named(name).map(|m| m.drivers.channels.clone()).unwrap_or_default();

    // `DRIVER_TYPE_*`: Average 0, Python 1, Sum 2.
    let wheel = of("Wheel");
    assert_eq!(wheel.len(), 1);
    assert_eq!((wheel[0].property.as_str(), wheel[0].index, wheel[0].driver.kind), ("rotation_euler", 2, 0), "Average on rotation_euler[2]");
    assert_eq!(wheel[0].driver.vars.len(), 1);
    assert_eq!(wheel[0].driver.vars[0].name, "x");
    // The variable reads the Driver's `location[0]`.
    let BlendDriverRead::Channel { property, index, .. } = &wheel[0].driver.vars[0].read else { panic!("a channel variable: {:?}", wheel[0].driver.vars[0].read) };
    assert_eq!((property.as_str(), *index), ("location", 0));
    // `driver_add` gives the curve **two default keys** — a 1:1 ramp — and
    // the Generator sits over them, as an F-curve modifier does
    // (`evaluate_fcurve_ex` runs the modifiers after the keys). So the
    // shape is keys *and* a generator, and what matters is the value the
    // two come to together: `0.5 + 2x`, not the ramp's `x`.
    assert_eq!(wheel[0].driver.curve.keys.len(), 2, "the default ramp `driver_add` leaves");
    assert_eq!(wheel[0].driver.curve.generator.as_deref(), Some([0.5f32, 2.0].as_slice()), "the Generator over it");

    let scaled = of("Scaled");
    assert_eq!((scaled[0].property.as_str(), scaled[0].index, scaled[0].driver.kind), ("scale", 0, 1), "Python on scale[0]");
    assert_eq!(scaled[0].driver.expression, "1 + x / 10");
    assert!(scaled[0].driver.parsed.is_some(), "and the expression parses");

    // And the mapping, measured: the driver's value through the curve.
    // `Driver.location[0]` is 0 at frame 1 and 2.5 at frame 6, so the
    // Wheel's `rotation_euler[2]` is 0.5 and 5.5 — which is the Generator,
    // not the 1:1 ramp underneath it.
    // `BlendMotionList::raw_at` is the channels with the drivers run, so
    // the angle can be read as the driver wrote it — a matrix decomposition
    // would wrap 5.5 radians to 5.5 − 2π and say nothing about the mapping.
    let motion = &scene.props.motion;
    let i = motion.list.iter().position(|m| m.name == "Wheel").expect("the Wheel");
    let rot_z = |frame: f32| motion.raw_at(frame)[i].rot[2];
    assert!((rot_z(1.0) - 0.5).abs() < 1e-4, "frame 1: {}", rot_z(1.0));
    assert!((rot_z(6.0) - 5.5).abs() < 1e-3, "frame 6: {}", rot_z(6.0));
    assert!((rot_z(21.0) - 20.5).abs() < 1e-3, "frame 21: {}", rot_z(21.0));

    let summed = of("Summed");
    assert_eq!((summed[0].property.as_str(), summed[0].index, summed[0].driver.kind), ("location", 2, 2), "Sum on location[2]");
    assert_eq!(summed[0].driver.vars.len(), 2, "two variables");
    assert!(of("Still").is_empty() && of("Driver").is_empty(), "neither of those is driven");
}
