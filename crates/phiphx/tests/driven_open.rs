//! Opening a file puts its driven values on the scene.
//!
//! `mat_driver_truth.rs` already proves the *values*: it drives the material
//! pass directly and compares five frames against Blender's own evaluated
//! materials to six decimals. It passed for two batches while the scene still
//! showed the saved values, because nothing called the pass — the sets were
//! read into `BlendScene::props` and left there (`anim/driver/wire.rs`).
//!
//! So this tests the other half, and only that half: that `Phx::open` — what
//! the app actually calls — leaves the driven values on the materials the
//! viewport draws from. It reads the same Blender dump
//! (`truth/mat_driver.txt`) rather than restating the numbers, so there is
//! one place they live.

use std::collections::HashMap;
use std::path::PathBuf;

/// The dump's `own` row: roughness, metallic, diffuse_color RGBA, specular.
type Own = [f32; 7];

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures").join(name)
}

/// Blender's `own` rows at `frame`, by material name.
fn truth_own(frame: u32) -> HashMap<String, Own> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/truth/mat_driver.txt");
    let text = std::fs::read_to_string(path).expect("the Blender dump is in the tree");
    let mut out = HashMap::new();
    for line in text.lines() {
        let field: Vec<&str> = line.split_whitespace().collect();
        // `<frame> <material> own r m c0 c1 c2 c3 s`
        if field.len() != 10 || field[2] != "own" || field[0].parse::<u32>() != Ok(frame) {
            continue;
        }
        let mut row: Own = [0.0; 7];
        for (slot, text) in row.iter_mut().zip(&field[3..]) {
            *slot = text.parse().expect("a float");
        }
        out.insert(field[1].to_string(), row);
    }
    out
}

/// One material slot as the opened scene carries it. The material-level
/// values and the surface's are different fields and a driver can move
/// either, so both are kept apart here rather than conflated.
#[derive(Clone, Copy, PartialEq, Debug)]
struct Slot {
    /// `Material.roughness` / `metallic` — what Solid mode shades with.
    own: [f32; 2],
    /// `Material.r/g/b/a`.
    own_color: [f32; 4],
    /// The Principled's, after a driven socket: what Material Preview and
    /// the tracer read.
    surface: [f32; 2],
    surface_color: [f32; 4],
}

/// Every material slot in the opened scene, by name. A datablock shared by
/// several objects appears once per object; they must all agree, which is
/// asserted rather than assumed.
fn opened(name: &str) -> HashMap<String, Slot> {
    let model = phiphx::Phx::open(&fixture(name)).expect("the fixture opens");
    let mut out: HashMap<String, Slot> = HashMap::new();
    for object in &model.objects {
        for slot in &object.materials {
            if slot.name.is_empty() {
                continue;
            }
            let row = Slot {
                own: slot.viewport_pbr,
                own_color: slot.viewport_color,
                surface: [slot.roughness, slot.metallic],
                surface_color: slot.base_color,
            };
            if let Some(seen) = out.get(&slot.name) {
                assert_eq!(
                    *seen, row,
                    "material {} is one datablock and must read the same on every object carrying it",
                    slot.name
                );
            }
            out.insert(slot.name.clone(), row);
        }
    }
    out
}

#[test]
fn an_open_leaves_blenders_driven_material_values_on_the_scene() {
    // MATDRIVERS is saved at frame 1, which is the row set to compare.
    let truth = truth_own(1);
    assert!(!truth.is_empty(), "the dump has frame-1 rows");
    let have = opened("MATDRIVERS.blend");

    let mut wrong = Vec::new();
    for (material, want) in &truth {
        let Some(slot) = have.get(material).copied() else {
            wrong.push(format!("{material}: not in the opened scene at all"));
            continue;
        };
        // Blender prints six decimals of plain float arithmetic, so there is
        // no slack to give — the same tolerance `mat_driver_truth.rs` uses.
        let got =
            [slot.own[0], slot.own[1], slot.own_color[0], slot.own_color[1], slot.own_color[2], slot.own_color[3]];
        for (i, (g, w)) in got.iter().zip(&want[..6]).enumerate() {
            if (g - w).abs() > 1e-5 {
                wrong.push(format!("{material} field {i}: Blender {w:.6}, opened scene {g:.6}"));
            }
        }
    }
    assert!(wrong.is_empty(), "the open did not apply {} values:\n  {}", wrong.len(), wrong.join("\n  "));
}

#[test]
fn a_driven_shader_socket_reaches_the_surface_the_viewport_draws() {
    // The material-level rows above land in `viewport_color`/`viewport_pbr`.
    // A driven *socket* has to move `base_color` too, which is what Material
    // Preview and the tracer read. MAT_SOCK drives Base Color's R and B to 0
    // (clamped there — RGBA clamps at zero) and leaves G at Blender's 0.8.
    let have = opened("MATDRIVERS.blend");
    let base = have.get("MAT_SOCK").copied().expect("MAT_SOCK is in the scene").surface_color;
    assert!(base[0].abs() < 1e-5, "Base Color R driven to 0, got {}", base[0]);
    assert!((base[1] - 0.8).abs() < 1e-5, "Base Color G undriven at 0.8, got {}", base[1]);
    assert!(base[2].abs() < 1e-5, "Base Color B driven to 0, got {}", base[2]);
}

#[test]
fn a_muted_driver_leaves_the_saved_value_standing() {
    // MAT_EDGE's roughness driver is muted, so Blender keeps the file's 0.4.
    // This is the row that would still pass if the wiring did nothing, so it
    // is here to pin the *other* direction: applying must not overwrite a
    // value whose driver is muted.
    let have = opened("MATDRIVERS.blend");
    let slot = have.get("MAT_EDGE").copied().expect("MAT_EDGE is in the scene");
    assert!((slot.own[0] - 0.4).abs() < 1e-5, "a muted driver leaves 0.4, got {}", slot.own[0]);
    // Its Metallic *socket* driver is live and constant at 0.35 — which the
    // file also saves, so this one asserts the value rather than proving the
    // pass ran (the two tests above do that). It is here because a wiring
    // that wrote the wrong field would move it.
    assert!((slot.surface[1] - 0.35).abs() < 1e-5, "the live socket driver writes 0.35, got {}", slot.surface[1]);
    assert!(slot.own[1].abs() < 1e-5, "the material-level metallic is untouched at 0.0, got {}", slot.own[1]);
}

#[test]
fn a_file_with_no_material_drivers_opens_untouched() {
    // The wiring is skipped entirely when no set has a driver in it, so this
    // also asserts the guard does not cost a file the driver pass it does not
    // need.
    let model = phiphx::Phx::open(&fixture("MATERIALS-BSDF.blend")).expect("opens");
    let slots: usize = model.objects.iter().map(|o| o.materials.len()).sum();
    assert!(slots > 0, "the fixture has materials to leave alone");
    assert!(
        model.objects.iter().flat_map(|o| &o.materials).all(|m| m.roughness.is_finite() && m.base_color[3] > 0.0),
        "every slot still carries the values the file saved"
    );
}
