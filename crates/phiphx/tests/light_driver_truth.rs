//! Light and world drivers against Blender's own evaluated datablocks
//! (`fixtures/LIGHTDRIVERS.blend`, `truth/blender_light_driver_fixture.py`,
//! `truth/blender_light_driver.py`, `truth/light_driver.txt`).
//!
//! A light has no node tree, so all of its drivers are on the `Light` ID;
//! a world splits like a material, `color` on the `World` ID and the
//! Background node's sockets on `World.nodetree`. What distinguishes the
//! light properties is their **hard ranges**, which are not uniform, so
//! every driver in the fixture is pushed out of range on purpose:
//!
//! - **L_POINT** — `energy` driven negative (it is the one unbounded
//!   property, so a lamp really can go to −100), `color` R and B (0 .. ∞:
//!   clamped below, free above 1), `shadow_soft_size` driven negative, and
//!   `color` G read off a driven custom property, which is the depsgraph
//!   ordering proved again through a different ID type.
//! - **L_SPOT** — `spot_size` driven to −2, which Blender leaves at
//!   **0.017453**: its hard minimum is one degree, not zero, so driving a
//!   cone closed leaves it 1° open. `spot_blend` clamps 0 .. 1.
//! - **L_SUN** — `angle` clamps 0 .. π.
//! - **L_AREA** — `size` and `size_y` (0 .. ∞), and a **muted** driver on
//!   `energy`, so the saved 42 stands.
//! - **W_DRIVEN** — `World.color` G on the ID, the Background's `Strength`
//!   driven negative (a float socket, unclamped) and its `Color` R and B
//!   (RGBA, clamped at 0).

use phiphx::blend::BlendFile;
use phiphx::blend::BLightProp;
use phiphx::blend::{AREA_ENERGY_SCALE, LA_AREA};
use phiphx::blend::SPOT_SIZE_MIN;
use phiphx::blend::shade::world_driver::{BG_COLOR, BG_STRENGTH, BWorldWrite};
use std::path::PathBuf;

const FRAMES: [f32; 5] = [1.0, 6.0, 11.0, 16.0, 21.0];
const TOLERANCE: f32 = 1e-5;

/// A light's row: colour, then each scalar, with `None` where the lamp type
/// has no such property and Blender wrote `-`.
struct LightRow {
    color: [f32; 3],
    energy: Option<f32>,
    radius: Option<f32>,
    spot_size: Option<f32>,
    spot_blend: Option<f32>,
    size: Option<f32>,
    size_y: Option<f32>,
    angle: Option<f32>,
}

struct WorldRow {
    color: [f32; 3],
    strength: f32,
    bg_color: [f32; 3],
}

enum Row {
    Light(LightRow),
    World(WorldRow),
}

fn truth() -> Vec<(f32, String, Row)> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/truth/light_driver.txt");
    std::fs::read_to_string(path)
        .unwrap_or_default()
        .lines()
        .filter_map(|line| {
            let f: Vec<&str> = line.split(' ').collect();
            let frame: f32 = f.first()?.parse().ok()?;
            let name = f.get(2)?.to_string();
            let cell = |i: usize| -> Option<f32> { f.get(i).and_then(|c| c.parse().ok()) };
            let need = |i: usize| -> Option<f32> { cell(i) };
            match *f.get(1)? {
                "light" => Some((
                    frame,
                    name,
                    Row::Light(LightRow {
                        color: [need(3)?, need(4)?, need(5)?],
                        energy: cell(6),
                        radius: cell(7),
                        spot_size: cell(8),
                        spot_blend: cell(9),
                        size: cell(10),
                        size_y: cell(11),
                        angle: cell(12),
                    }),
                )),
                "world" => Some((
                    frame,
                    name,
                    Row::World(WorldRow {
                        color: [need(3)?, need(4)?, need(5)?],
                        strength: need(6)?,
                        bg_color: [need(7)?, need(8)?, need(9)?],
                    }),
                )),
                _ => None,
            }
        })
        .collect()
}

fn fixture() -> Option<Vec<u8>> {
    std::fs::read(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/LIGHTDRIVERS.blend")).ok()
}

#[test]
fn every_light_driver_writes_what_blender_writes() {
    let Some(bytes) = fixture() else { return eprintln!("no LIGHTDRIVERS.blend; skipped") };
    let want = truth();
    assert!(!want.is_empty(), "the truth file is there");
    let scene = BlendFile::parse_bytes(&bytes).expect("parse").extract_scene().expect("scene");
    assert_eq!(scene.props.light_drivers.len(), 4, "all four lamps carry a driver");
    assert_eq!(scene.props.world_drivers.len(), 1, "and the world does");

    // What an undriven or muted property keeps: the value the file saved,
    // taken from the reader itself rather than guessed — a Sun's default
    // energy is 1.0 (irradiance in W/m²) where a Point's is 1000 (W), so a
    // single hard-coded default would be wrong for one of them. The
    // fixture names each lamp's object `OB_<light>`.
    // An area light's stored energy is 4/π of the watts RNA reports, so
    // the saved value is converted back before it is compared with
    // Blender's row — the same scale `apply` uses in the other direction.
    let saved = |name: &str| -> (f32, [f32; 3]) {
        let object = scene.objects.iter().find(|o| o.name == format!("OB_{name}")).expect("the lamp's object");
        match &object.data {
            phiphx::blend::BlendObjectData::Light(light) => {
                let watts = if light.kind == LA_AREA { light.energy / AREA_ENERGY_SCALE } else { light.energy };
                (watts, light.color)
            }
            other => panic!("{name} is not a light: {other:?}"),
        }
    };

    let mut checked = 0usize;
    let mut worst = (String::new(), 0.0f32);
    let mut note = |what: String, ours: f32, theirs: f32, checked: &mut usize, worst: &mut (String, f32)| {
        let off = (ours - theirs).abs();
        if off > worst.1 {
            *worst = (what.clone(), off);
        }
        assert!(off < TOLERANCE, "{what}: ours {ours}, Blender's {theirs}");
        *checked += 1;
    };

    for frame in FRAMES {
        let state = scene.props.motion.driver_state_at(frame);
        let ds = state.scene();
        for (f, name, row) in want.iter().filter(|(f, _, _)| *f == frame) {
            match row {
                Row::Light(blender) => {
                    let set = scene.props.light_drivers.iter().find(|s| s.light == *name).expect("the light");
                    let written = set.values(&ds, frame);
                    let got = |prop: BLightProp| written.iter().find(|(p, _)| *p == prop).map(|(_, v)| *v);
                    let (saved_energy, saved_color) = saved(name);

                    for c in 0..3 {
                        let ours = got(BLightProp::Color(c)).unwrap_or(saved_color[c]);
                        note(format!("{name} color[{c}] at frame {f}"), ours, blender.color[c], &mut checked, &mut worst);
                    }
                    for (prop, theirs, label) in [
                        (BLightProp::Energy, blender.energy, "energy"),
                        (BLightProp::Radius, blender.radius, "shadow_soft_size"),
                        (BLightProp::SpotSize, blender.spot_size, "spot_size"),
                        (BLightProp::SpotBlend, blender.spot_blend, "spot_blend"),
                        (BLightProp::AreaSize, blender.size, "size"),
                        (BLightProp::AreaSizeY, blender.size_y, "size_y"),
                        (BLightProp::SunAngle, blender.angle, "angle"),
                    ] {
                        // Blender wrote `-` for a property this lamp type
                        // has not got; there is nothing to compare.
                        let Some(theirs) = theirs else { continue };
                        match got(prop) {
                            Some(ours) => note(format!("{name} {label} at frame {f}"), ours, theirs, &mut checked, &mut worst),
                            // Nothing written: either no driver, or a muted
                            // one. Blender's value is then the saved one.
                            None if prop == BLightProp::Energy => {
                                note(format!("{name} {label} (muted) at frame {f}"), saved_energy, theirs, &mut checked, &mut worst);
                            }
                            None => {}
                        }
                    }
                }
                Row::World(blender) => {
                    let set = scene.props.world_drivers.iter().find(|s| s.world == *name).expect("the world");
                    let written = set.values(&ds, frame);
                    let sock = |input: usize, component: usize| {
                        written
                            .iter()
                            .find(|(w, _)| {
                                matches!(w, BWorldWrite::Socket { input: i, component: c, .. } if *i == input && *c == component)
                            })
                            .map(|(_, v)| *v)
                    };
                    // `World.color`: only G is driven; R and B keep the
                    // default 0.05 the world was made with.
                    for c in 0..3 {
                        let ours = written
                            .iter()
                            .find(|(w, _)| matches!(w, BWorldWrite::Color(i) if *i == c))
                            .map(|(_, v)| *v)
                            .unwrap_or(0.05);
                        note(format!("{name} World.color[{c}] at frame {f}"), ours, blender.color[c], &mut checked, &mut worst);
                    }
                    let strength = sock(BG_STRENGTH, 0).expect("Strength is driven");
                    note(format!("{name} Background.Strength at frame {f}"), strength, blender.strength, &mut checked, &mut worst);
                    for c in 0..3 {
                        // Only R and B are driven; G keeps the Background's
                        // own default of 0.05.
                        let ours = sock(BG_COLOR, c).unwrap_or(0.05);
                        note(format!("{name} Background.Color[{c}] at frame {f}"), ours, blender.bg_color[c], &mut checked, &mut worst);
                    }
                }
            }
        }
    }
    assert!(checked > 100, "only {checked} values compared");
    eprintln!("{checked} values compared; worst: {} off by {:.2e}", worst.0, worst.1);
}

/// The ranges, named one by one, so a regression says which rule broke.
#[test]
fn the_hard_ranges_are_blenders_own() {
    let Some(bytes) = fixture() else { return eprintln!("no LIGHTDRIVERS.blend; skipped") };
    let scene = BlendFile::parse_bytes(&bytes).expect("parse").extract_scene().expect("scene");
    let state = scene.props.motion.driver_state_at(1.0);
    let ds = state.scene();
    let at = |name: &str| {
        scene.props.light_drivers.iter().find(|s| s.light == name).map(|s| s.values(&ds, 1.0)).unwrap_or_default()
    };
    let got = |written: &[(BLightProp, f32)], prop: BLightProp| written.iter().find(|(p, _)| *p == prop).map(|(_, v)| *v);

    // At frame 1 SOURCE's Z is −2, so every driver is out of range.
    let p = at("L_POINT");
    assert_eq!(got(&p, BLightProp::Energy), Some(-100.0), "`energy` is unbounded — a driven lamp may go negative");
    assert_eq!(got(&p, BLightProp::Color(0)), Some(0.0), "a colour clamps at 0");
    assert_eq!(got(&p, BLightProp::Color(2)), Some(1.5), "and is free above 1");
    assert_eq!(got(&p, BLightProp::Radius), Some(0.0), "a radius cannot go negative");
    assert_eq!(got(&p, BLightProp::Color(1)), Some(0.0), "the custom-property chain, clamped at 0");

    let s = at("L_SPOT");
    assert_eq!(got(&s, BLightProp::SpotSize), Some(SPOT_SIZE_MIN), "a cone driven closed stays one degree open");
    assert_eq!(got(&s, BLightProp::SpotBlend), Some(0.0), "`spot_blend` clamps at 0");

    let sun = at("L_SUN");
    assert_eq!(got(&sun, BLightProp::SunAngle), Some(0.0), "a sun's angle clamps at 0");

    let a = at("L_AREA");
    assert_eq!(got(&a, BLightProp::AreaSize), Some(0.0), "an area edge clamps at 0");
    assert_eq!(got(&a, BLightProp::AreaSizeY), Some(2.0), "and is free above 1");
    assert_eq!(got(&a, BLightProp::Energy), None, "the muted driver writes nothing");
}

/// `apply` puts the driven values on the datablocks the renderer reads.
#[test]
fn applying_moves_the_light_and_the_world_the_renderer_reads() {
    let Some(bytes) = fixture() else { return eprintln!("no LIGHTDRIVERS.blend; skipped") };
    let scene = BlendFile::parse_bytes(&bytes).expect("parse").extract_scene().expect("scene");
    let state = scene.props.motion.driver_state_at(11.0);
    let ds = state.scene();

    // At frame 11 SOURCE's Z is 0.6: energy ← 30, radius ← 0.12.
    let object = scene.objects.iter().find(|o| o.name == "OB_L_POINT").expect("OB_L_POINT");
    let phiphx::blend::BlendObjectData::Light(mut light) = object.data.clone() else { panic!("not a light") };
    let set = scene.props.light_drivers.iter().find(|s| s.light == "L_POINT").expect("L_POINT");
    set.apply(&mut light, &ds, 11.0);
    assert!((light.energy - 30.0).abs() < 1e-3, "energy {}", light.energy);
    assert!((light.radius - 0.12).abs() < TOLERANCE, "radius {}", light.radius);
    assert!((light.color[0] - 0.6).abs() < TOLERANCE, "color R {}", light.color[0]);

    // An area light: the driven watts land in the field's stored units.
    let object = scene.objects.iter().find(|o| o.name == "OB_L_AREA").expect("OB_L_AREA");
    let phiphx::blend::BlendObjectData::Light(mut area) = object.data.clone() else { panic!("not a light") };
    assert_eq!(area.kind, LA_AREA);
    let set = scene.props.light_drivers.iter().find(|s| s.light == "L_AREA").expect("L_AREA");
    set.apply(&mut area, &ds, 11.0);
    // At frame 11 `size` ← 0.6 and `size_y` ← 0.7, both plain values.
    assert!((area.area_size - 0.6).abs() < TOLERANCE, "size {}", area.area_size);
    assert!((area.area_sizey - 0.7).abs() < TOLERANCE, "size_y {}", area.area_sizey);
    // Its energy driver is muted, so the saved value stands untouched.
    assert!((area.energy - 42.0 * AREA_ENERGY_SCALE).abs() < 1e-3, "energy {}", area.energy);

    // The world's Background: Strength ← 0.6, Color R ← 0.6, B ← 0.15.
    let mut world = scene.props.world.clone();
    let set = scene.props.world_drivers.iter().find(|s| s.world == "W_DRIVEN").expect("W_DRIVEN");
    set.apply(&mut world, &ds, 11.0);
    assert!((world.strength - 0.6).abs() < TOLERANCE, "strength {}", world.strength);
    assert!((world.color[0] - 0.6).abs() < TOLERANCE, "color R {}", world.color[0]);
    assert!((world.color[2] - 0.15).abs() < TOLERANCE, "color B {}", world.color[2]);
}
