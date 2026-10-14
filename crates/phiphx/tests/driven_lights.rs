//! Opening a file puts its driven lamp and world values on the scene.
//!
//! The lamp-and-world half of `driven_open.rs`, split off at this project's
//! 240-line ceiling; that file's header explains why an open needs a test of
//! its own at all, and `driver-parity.md` why these two needed a datablock
//! address before they could be wired.
//!
//! `light_driver_truth.rs` proves the values. This proves `Phx::open` leaves
//! them on the scene — which it did not until batch 186, because a set is
//! keyed by its `Light` or `World` datablock and neither carried one to match
//! on. Both read the same Blender dump, `truth/light_driver.txt`, so the
//! numbers live in one place.

use std::path::PathBuf;

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures").join(name)
}

/// One `light` row of `truth/light_driver.txt`, which is
/// `<frame> light <name> r g b energy shadow_soft_size spot_size spot_blend
/// size size_y angle` — the order the dump's `PROPS` list builds, not the
/// order its header comment claims (the comment puts energy first; the data
/// puts the colour first). A column Blender has no property for prints `-`.
struct Lamp {
    name: String,
    color: [f32; 3],
    energy: f32,
    radius: Option<f32>,
    spot_size: Option<f32>,
    spot_blend: Option<f32>,
    size: Option<f32>,
    angle: Option<f32>,
}

fn truth_lights(frame: u32) -> (Vec<Lamp>, Option<([f32; 3], f32)>) {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/truth/light_driver.txt");
    let text = std::fs::read_to_string(path).expect("the Blender dump is in the tree");
    let num = |cell: &str| -> Option<f32> { cell.parse().ok() };
    let mut lamps = Vec::new();
    let mut world = None;
    for line in text.lines() {
        let f: Vec<&str> = line.split_whitespace().collect();
        if f.len() < 3 || f[0].parse::<u32>() != Ok(frame) {
            continue;
        }
        match f[1] {
            "light" if f.len() >= 13 => lamps.push(Lamp {
                name: f[2].to_string(),
                color: [num(f[3]).unwrap_or(0.0), num(f[4]).unwrap_or(0.0), num(f[5]).unwrap_or(0.0)],
                energy: num(f[6]).unwrap_or(0.0),
                radius: num(f[7]),
                spot_size: num(f[8]),
                spot_blend: num(f[9]),
                size: num(f[10]),
                angle: num(f[12]),
            }),
            // `<frame> world <name> color rgb, bg strength, bg color rgb`.
            // `BlendWorld::color` is the **Background node's** colour (the
            // one that lights the render), not the World ID's own `color`,
            // so the last three columns are the comparison and the first
            // three are not.
            "world" if f.len() >= 10 => {
                world = Some((
                    [num(f[7]).unwrap_or(0.0), num(f[8]).unwrap_or(0.0), num(f[9]).unwrap_or(0.0)],
                    num(f[6]).unwrap_or(0.0),
                ))
            }
            _ => {}
        }
    }
    (lamps, world)
}

#[test]
fn an_open_leaves_blenders_driven_lamp_values_on_the_scene() {
    let (lamps, _) = truth_lights(1);
    assert_eq!(lamps.len(), 4, "the dump's four frame-1 lamps");
    let model = phiphx::Phx::open(&fixture("LIGHTDRIVERS.blend")).expect("opens");

    let mut wrong: Vec<String> = Vec::new();
    for lamp in &lamps {
        // The dump names the `Light` datablock (`L_SPOT`); the scene names
        // the OBJECT (`OB_L_SPOT`). Matching on the suffix is what this
        // fixture's naming allows, and one match is asserted so a rename
        // that made it ambiguous would fail here rather than pass quietly.
        let found: Vec<_> = model.lights.iter().filter(|l| l.name.ends_with(&lamp.name)).collect();
        assert_eq!(found.len(), 1, "exactly one scene light for {}", lamp.name);
        let got = found[0];

        for (c, want) in lamp.color.iter().enumerate() {
            if (got.color[c] - want).abs() > 1e-5 {
                wrong.push(format!("{} color[{c}]: Blender {want:.6}, scene {:.6}", lamp.name, got.color[c]));
            }
        }

        // An area light's stored energy is 4/pi of the watts Blender's RNA
        // reports, and `SceneLight::energy` is deliberately the stored
        // number. Converting here rather than loosening the tolerance keeps
        // the factor asserted instead of absorbed.
        let watts = if got.kind == "AREA" { got.energy * std::f32::consts::FRAC_PI_4 } else { got.energy };
        if (watts - lamp.energy).abs() > 1e-4 {
            wrong.push(format!("{} energy: Blender {:.6} W, scene {watts:.6} W", lamp.name, lamp.energy));
        }

        // `size` is overloaded by kind, exactly as Blender's three separate
        // properties are: a point's and spot's radius, an area light's edge,
        // a sun's angular diameter.
        let want_size = match got.kind.as_str() {
            "AREA" => lamp.size,
            "SUN" => lamp.angle,
            _ => lamp.radius,
        };
        if let Some(want) = want_size {
            if (got.size - want).abs() > 1e-5 {
                wrong.push(format!("{} size: Blender {want:.6}, scene {:.6}", lamp.name, got.size));
            }
        }

        if got.kind == "SPOT" {
            for (label, want, have) in [
                ("spot_size", lamp.spot_size, got.spot_size),
                ("spot_blend", lamp.spot_blend, got.spot_blend),
            ] {
                if let Some(want) = want {
                    if (have - want).abs() > 1e-5 {
                        wrong.push(format!("{} {label}: Blender {want:.6}, scene {have:.6}", lamp.name));
                    }
                }
            }
        }
    }
    assert!(wrong.is_empty(), "the open did not apply {} lamp values:\n  {}", wrong.len(), wrong.join("\n  "));
}

#[test]
fn the_two_rules_no_uniform_clamp_would_give_survive_the_open() {
    // These are the two `light_driver_truth.rs` singled out as unguessable,
    // and they are the ones a careless clamp in the wiring would destroy.
    let model = phiphx::Phx::open(&fixture("LIGHTDRIVERS.blend")).expect("opens");
    let point = model.lights.iter().find(|l| l.name.ends_with("L_POINT")).expect("the point lamp");
    assert!(
        point.energy < -99.0,
        "`energy` is unbounded, so a lamp driven to -100 W stays there; got {}",
        point.energy
    );
    let spot = model.lights.iter().find(|l| l.name.ends_with("L_SPOT")).expect("the spot lamp");
    assert!(
        (spot.spot_size - 0.017453).abs() < 1e-5,
        "`spot_size` stops at 1 degree, so a cone driven shut stays open; got {}",
        spot.spot_size
    );
}

#[test]
fn an_open_leaves_blenders_driven_world_on_the_scene() {
    let (_, world) = truth_lights(1);
    let (color, strength) = world.expect("the dump's frame-1 world row");
    let model = phiphx::Phx::open(&fixture("LIGHTDRIVERS.blend")).expect("opens");
    let have = &model.world.background;
    assert!(
        (have.strength - strength).abs() < 1e-5,
        "Background Strength: Blender {strength:.6}, scene {:.6}",
        have.strength
    );
    assert!(strength < 0.0, "and it is negative, which is the float socket's own rule");
    for c in 0..3 {
        assert!(
            (have.color[c] - color[c]).abs() < 1e-5,
            "Background Color[{c}]: Blender {:.6}, scene {:.6}",
            color[c],
            have.color[c]
        );
    }
}
