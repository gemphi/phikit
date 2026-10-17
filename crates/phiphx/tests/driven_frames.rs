//! A driven modifier setting, at every frame — not just the saved one.
//!
//! `mod_driver_truth.rs` drives the reader directly. `driven_open.rs` proves
//! the saved frame reaches the scene. This is the third thing and the one the
//! others cannot show: that an opened scene can be asked for **another**
//! frame's settings and gives Blender's.
//!
//! It matters because a mesh is evaluated once at open, so every driven
//! modifier value was frozen at `frame_current` until `SceneRigs::driven_at`
//! existed. `MODDRIVERS` drives a Subdivision's level 0 → 2 → 0 → 11 → 0
//! across its five frames, which is the clearest possible witness: a setting
//! that both switches the modifier off and takes it past its cap.

use std::collections::HashMap;
use std::path::PathBuf;

/// `<frame> <object> <modifier> <TYPE> <prop> <component> <value>`.
///
/// The OBJECT is part of the key, and finding that out was the point of
/// writing this: `MODDRIVERS` gives two different objects a modifier both
/// called `Disp`, so a key without the object silently compares one object's
/// driven strength against the other's.
type Key = (String, String, String, usize);

fn truth(frame: u32) -> HashMap<Key, f32> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/truth/mod_driver.txt");
    let text = std::fs::read_to_string(path).expect("the Blender dump is in the tree");
    let mut out = HashMap::new();
    for line in text.lines() {
        let f: Vec<&str> = line.split_whitespace().collect();
        if f.len() != 7 || f[0].parse::<u32>() != Ok(frame) {
            continue;
        }
        let (Ok(component), Ok(value)) = (f[5].parse::<usize>(), f[6].parse::<f32>()) else { continue };
        out.insert((f[1].to_string(), f[2].to_string(), f[4].to_string(), component), value);
    }
    out
}

const FRAMES: [u32; 5] = [1, 6, 11, 16, 21];

#[test]
fn an_opened_scene_gives_blenders_driven_settings_at_any_frame() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/MODDRIVERS.blend");
    let model = phiphx::Phx::open(&path).expect("the fixture opens");
    let rigs = &model.motion.rigs;
    assert!(!rigs.drivers.is_empty(), "the scene carries the modifier driver sets");

    let mut wrong = Vec::new();
    let mut checked = 0;
    for frame in FRAMES {
        let want = truth(frame);
        assert!(!want.is_empty(), "the dump has frame-{frame} rows");
        for (address, values) in rigs.driven_at(&model.motion.motion, frame as f32) {
            // The sets are keyed by the object's saved address; the dump
            // names the object, so one maps to the other.
            let object = rigs
                .drivers
                .iter()
                .find(|set| set.address == address)
                .map(|set| set.object.clone())
                .expect("every driven address belongs to a set the scene carries");
            for (write, value) in &values {
                let key = (object.clone(), write.modifier.clone(), write.prop.clone(), write.component);
                let Some(&expected) = want.get(&key) else {
                    wrong.push(format!("frame {frame}: {key:?} is driven here and not in Blender's dump"));
                    continue;
                };
                checked += 1;
                // Integers are exact — they are a C `(int)` cast of the
                // driver's float, so a near-miss is a different number.
                if (value.as_f32() - expected).abs() > 1e-5 {
                    wrong.push(format!("frame {frame} {key:?}: Blender {expected}, scene {}", value.as_f32()));
                }
            }
        }
    }
    assert!(wrong.is_empty(), "{} values differ:\n  {}", wrong.len(), wrong.join("\n  "));
    assert!(checked >= 60, "five frames of a dozen-odd live drivers; checked {checked}");
}

#[test]
fn the_subdivision_level_really_does_move_between_frames() {
    // The witness. If this ever reads the same number at every frame, the
    // per-frame path has been lost again and every other test here would
    // still pass, because they compare one frame at a time.
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/MODDRIVERS.blend");
    let model = phiphx::Phx::open(&path).expect("opens");
    let level_at = |frame: f32| -> Option<f32> {
        model
            .motion
            .rigs
            .driven_at(&model.motion.motion, frame)
            .values()
            .flatten()
            .find(|(w, _)| w.modifier == "Subsurf" && w.prop == "levels")
            .map(|(_, v)| v.as_f32())
    };
    let seen: Vec<Option<f32>> = FRAMES.iter().map(|&f| level_at(f as f32)).collect();
    assert_eq!(
        seen,
        vec![Some(0.0), Some(2.0), Some(0.0), Some(11.0), Some(0.0)],
        "Blender drives this level 0, 2, 0, 11, 0 across the five frames"
    );
}
