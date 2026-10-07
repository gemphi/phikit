//! Material drivers against Blender's own evaluated materials
//! (`fixtures/MATDRIVERS.blend`, `truth/blender_mat_driver_fixture.py`,
//! `truth/blender_mat_driver.py`, `truth/mat_driver.txt`).
//!
//! A material's drivers live on two IDs and take three different clamps,
//! and nothing but Blender's own numbers settles which. The fixture covers:
//!
//! - **MAT_OWN** — drivers on `Material.roughness`, `metallic`,
//!   `specular_intensity` and two `diffuse_color` components, each driven
//!   well past its range, so both clamp ends are measured.
//! - **MAT_SOCK** — drivers on the Principled BSDF's sockets by index:
//!   Roughness and Metallic (which declare 0..1 in RNA and are **not**
//!   clamped), Base Color's R and B (RGBA: clamped at 0, free above 1), IOR,
//!   Emission Strength, Coat Weight, and a Sum driver over two Transform
//!   Channel variables rather than an expression.
//! - **MAT_EDGE** — the three that are easy to get wrong: a driver on a
//!   **linked** socket (Blender writes `default_value` and the link still
//!   feeds the shader), a **muted** driver (the saved value stands), a
//!   driver whose own F-curve has keys so the driver's value is the curve's
//!   X and not the frame, and a socket driven off a custom property that is
//!   itself driven — the chain that only resolves if the material pass runs
//!   after the objects'.
//! - **MAT_SLOT2** — a second slot on one object, so slot order is covered.
//!
//! The reference is the **evaluated** depsgraph copy of each material,
//! which is where a driver's result lands.

use phiphx::blend::BlendFile;
use phiphx::blend::shade::driver::{BMatProp, BMatWrite};
use std::path::PathBuf;

const FRAMES: [f32; 5] = [1.0, 6.0, 11.0, 16.0, 21.0];
/// Blender's values print to six decimals and are plain float arithmetic on
/// the same inputs, so there is no slack to give here.
const TOLERANCE: f32 = 1e-5;

/// One expected value: the frame, the material, and what was written.
#[derive(Debug, PartialEq)]
enum Row {
    /// `Material.roughness`, `metallic`, `diffuse_color[0..4]`,
    /// `specular_intensity` — the whole material-level row at once.
    Own([f32; 7]),
    /// A socket by node name, input index and component.
    Sock(String, usize, usize, f32),
}

fn truth() -> Vec<(f32, String, Row)> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/truth/mat_driver.txt");
    std::fs::read_to_string(path)
        .unwrap_or_default()
        .lines()
        .filter_map(|line| {
            let f: Vec<&str> = line.split(' ').collect();
            let frame: f32 = f.first()?.parse().ok()?;
            let material = f.get(1)?.to_string();
            match *f.get(2)? {
                "own" => {
                    let mut v = [0.0f32; 7];
                    for (i, slot) in v.iter_mut().enumerate() {
                        *slot = f.get(3 + i)?.parse().ok()?;
                    }
                    Some((frame, material, Row::Own(v)))
                }
                // `sock <node name, with spaces> <input> <component> <value>`
                "sock" => {
                    let node = f.get(3..f.len() - 3)?.join(" ");
                    let tail = f.get(f.len() - 3..)?;
                    Some((
                        frame,
                        material,
                        Row::Sock(node, tail[0].parse().ok()?, tail[1].parse().ok()?, tail[2].parse().ok()?),
                    ))
                }
                _ => None,
            }
        })
        .collect()
}

fn fixture() -> Option<Vec<u8>> {
    std::fs::read(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/MATDRIVERS.blend")).ok()
}

/// What the material's own row should read, given the values this frame's
/// drivers wrote. Blender's row is the material as evaluated, so an
/// undriven field keeps the value the file saved.
fn own_row(saved: [f32; 7], written: &[(BMatWrite, f32)]) -> [f32; 7] {
    let mut row = saved;
    for (write, value) in written {
        let BMatWrite::Own(prop) = write else { continue };
        match prop {
            BMatProp::Roughness => row[0] = *value,
            BMatProp::Metallic => row[1] = *value,
            BMatProp::DiffuseColor(c) => row[2 + c] = *value,
            BMatProp::SpecularIntensity => row[6] = *value,
        }
    }
    row
}

#[test]
fn every_material_driver_writes_what_blender_writes() {
    let Some(bytes) = fixture() else { return eprintln!("no MATDRIVERS.blend; skipped") };
    let want = truth();
    assert!(!want.is_empty(), "the truth file is there");
    let scene = BlendFile::parse_bytes(&bytes).expect("parse").extract_scene().expect("scene");
    let sets = &scene.props.material_drivers;
    assert_eq!(sets.len(), 4, "four of the file's materials carry a driver: {:?}", sets.iter().map(|s| &s.material).collect::<Vec<_>>());

    // The defaults a node-less material is read with, which an undriven
    // field in Blender's row keeps: `Material.roughness`/`metallic` are
    // 0.4 / 0, `r/g/b/a` 0.8 grey, `specular_intensity` 0.5.
    const SAVED: [f32; 7] = [0.4, 0.0, 0.8, 0.8, 0.8, 1.0, 0.5];

    let mut checked = 0usize;
    let mut worst = (String::new(), 0.0f32);
    for frame in FRAMES {
        let state = scene.props.motion.driver_state_at(frame);
        let driver_scene = state.scene();
        for set in sets {
            let written = set.values(&driver_scene, frame);
            for (f, material, row) in want.iter().filter(|(f, m, _)| *f == frame && *m == set.material) {
                let (ours, theirs, what): (f32, f32, String) = match row {
                    Row::Own(blender) => {
                        let ours = own_row(SAVED, &written);
                        for (i, (a, b)) in ours.iter().zip(blender.iter()).enumerate() {
                            let off = (a - b).abs();
                            if off > worst.1 {
                                worst = (format!("{material} own[{i}] at frame {f}"), off);
                            }
                            assert!(off < TOLERANCE, "{material} own[{i}] at frame {f}: ours {a}, Blender's {b}");
                            checked += 1;
                        }
                        continue;
                    }
                    Row::Sock(node, input, component, blender) => {
                        let found = written.iter().find(|(w, _)| {
                            matches!(w, BMatWrite::Socket { node: n, input: i, component: c }
                                if n == node && i == input && c == component)
                        });
                        match found {
                            Some((_, ours)) => (*ours, *blender, format!("{material} {node}.inputs[{input}][{component}]")),
                            // No value written: a muted driver. Blender's
                            // row then holds the value the file saved, and
                            // the reader must agree by writing nothing.
                            None => {
                                assert!(
                                    set.drivers.iter().any(|d| matches!(&d.write, BMatWrite::Socket { node: n, input: i, component: c }
                                        if n == node && i == input && c == component)
                                        && d.driver.muted()),
                                    "{material} {node}.inputs[{input}][{component}] at frame {f}: \
                                     Blender wrote {blender} and nothing here claims that socket"
                                );
                                continue;
                            }
                        }
                    }
                };
                let off = (ours - theirs).abs();
                if off > worst.1 {
                    worst = (format!("{what} at frame {f}"), off);
                }
                assert!(off < TOLERANCE, "{what} at frame {f}: ours {ours}, Blender's {theirs}");
                checked += 1;
            }
        }
    }
    assert!(checked > 150, "only {checked} values compared");
    eprintln!("{checked} values compared; worst: {} off by {:.2e}", worst.0, worst.1);
}

/// The clamps, named one by one against the sheet — so a regression says
/// *which* rule broke rather than only that a number moved.
#[test]
fn the_three_clamp_rules_are_blenders_own() {
    let Some(bytes) = fixture() else { return eprintln!("no MATDRIVERS.blend; skipped") };
    let scene = BlendFile::parse_bytes(&bytes).expect("parse").extract_scene().expect("scene");
    let state = scene.props.motion.driver_state_at(1.0);
    let driver_scene = state.scene();
    let at = |material: &str| {
        scene.props.material_drivers.iter().find(|s| s.material == material).map(|s| s.values(&driver_scene, 1.0)).unwrap_or_default()
    };
    let socket = |written: &[(BMatWrite, f32)], input: usize, component: usize| {
        written
            .iter()
            .find(|(w, _)| matches!(w, BMatWrite::Socket { input: i, component: c, .. } if *i == input && *c == component))
            .map(|(_, v)| *v)
    };
    let own = |written: &[(BMatWrite, f32)], prop: BMatProp| {
        written.iter().find(|(w, _)| matches!(w, BMatWrite::Own(p) if *p == prop)).map(|(_, v)| *v)
    };

    // At frame 1 SOURCE's Z is -2, so every driver here is driven out of
    // its RNA range in one direction or the other.
    let o = at("MAT_OWN");
    assert_eq!(own(&o, BMatProp::Roughness), Some(0.0), "`roughness` is -1.0 driven, and clamps at its hard min");
    assert_eq!(own(&o, BMatProp::Metallic), Some(1.0), "`metallic` is 3.0 driven, and clamps at its hard max");
    assert_eq!(own(&o, BMatProp::DiffuseColor(0)), Some(0.0), "a colour clamps at 0 below");
    assert_eq!(own(&o, BMatProp::SpecularIntensity), Some(0.0), "`specular_intensity` clamps like the rest");

    let s = at("MAT_SOCK");
    assert_eq!(socket(&s, 2, 0), Some(-1.0), "the Roughness socket says 0..1 in RNA and takes -1 from a driver");
    assert_eq!(socket(&s, 1, 0), Some(3.0), "and the Metallic socket takes 3");
    assert_eq!(socket(&s, 0, 0), Some(0.0), "Base Color is RGBA: its R clamps at 0");
    assert_eq!(socket(&s, 0, 2), Some(0.0), "and so does its B");
    assert_eq!(socket(&s, 28, 0), Some(-4.0), "Emission Strength is unbounded either way");

    let e = at("MAT_EDGE");
    assert_eq!(socket(&e, 1, 0), None, "a muted driver writes nothing at all");
    assert_eq!(socket(&e, 2, 0), Some(-2.0), "a driver on a linked socket still writes it, as Blender does");
    assert_eq!(socket(&e, 28, 0), Some(-1.5), "the custom-property chain resolves in one frame");
    // The driver's own F-curve: keys (0, 0.9) (1, 1.0) (2, 0.1) with linear
    // extrapolation, so a driver value of -2 reads 0.9 - 2 × 0.1.
    let shaped = socket(&e, 20, 0).expect("Coat Roughness is driven");
    assert!((shaped - 0.7).abs() < TOLERANCE, "the curve maps the driver's value, not the frame: {shaped}");
}

/// A driven socket reaches the surface only when it can: the fold respects
/// the link and the Material Output's own node.
#[test]
fn a_driven_socket_moves_the_surface_it_actually_feeds() {
    let Some(bytes) = fixture() else { return eprintln!("no MATDRIVERS.blend; skipped") };
    let scene = BlendFile::parse_bytes(&bytes).expect("parse").extract_scene().expect("scene");
    let state = scene.props.motion.driver_state_at(11.0);
    let driver_scene = state.scene();
    let set = scene.props.material_drivers.iter().find(|s| s.material == "MAT_SOCK").expect("MAT_SOCK");
    assert_eq!(set.surface.as_deref(), Some("Principled BSDF"), "the Output reaches the Principled");

    let object = scene.objects.iter().find(|o| o.name == "OB_SOCK").expect("OB_SOCK");
    let mut material = object.materials.iter().find(|m| m.name == "MAT_SOCK").expect("the slot").clone();
    set.apply(&mut material, &driver_scene, 11.0);

    // At frame 11 SOURCE's Z is 0.6: Roughness ← 0.3, Metallic ← 0.4.
    assert!((material.bsdf.roughness - 0.3).abs() < TOLERANCE, "roughness {}", material.bsdf.roughness);
    assert!((material.bsdf.metallic - 0.4).abs() < TOLERANCE, "metallic {}", material.bsdf.metallic);
    // `roughness`/`metallic` are the surface's, so they move with it.
    assert!((material.roughness - 0.3).abs() < TOLERANCE, "the material's own roughness follows the socket");

    // MAT_EDGE's Roughness socket is fed by a Noise Texture, so the driver
    // writes the socket and the surface keeps what the link gives it.
    let edge = scene.props.material_drivers.iter().find(|s| s.material == "MAT_EDGE").expect("MAT_EDGE");
    let object = scene.objects.iter().find(|o| o.name == "OB_EDGE").expect("OB_EDGE");
    let mut material = object.materials.iter().find(|m| m.name == "MAT_EDGE").expect("the slot").clone();
    let before = material.bsdf.roughness;
    edge.apply(&mut material, &driver_scene, 11.0);
    assert_eq!(material.bsdf.roughness, before, "a linked socket's driver changes no shading");
}
