//! Against Blender 5.0: `tests/truth/thymus_rig.txt` (`truth/blender_rig.py`)
//! holds every one of THYMUS's 328 rig bones' pose matrix at frames 1, 20,
//! 40, 66, 100 and 132 — after its drivers, its 330 constraints and the
//! legs' IK — and every driver's value.

use super::BRig;
use crate::blend::file::types::BlendFile;
use glam::Mat4;
use std::collections::HashMap;

const FILE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/THYMUS-BOLDROID-WAVE-003.blend");
const TRUTH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/truth/thymus_rig.txt");
const FRAMES: [u32; 6] = [1, 20, 40, 66, 100, 132];

/// `(frame, kind, name) → values`.
fn truth() -> HashMap<(u32, String, String), Vec<f32>> {
    std::fs::read_to_string(TRUTH)
        .expect("truth file")
        .lines()
        .filter_map(|l| {
            let mut it = l.split('\t');
            let (f, k, n, v) = (it.next()?, it.next()?, it.next()?, it.next()?);
            Some(((f.parse().ok()?, k.to_string(), n.to_string()), v.split(' ').filter_map(|x| x.parse().ok()).collect()))
        })
        .collect()
}

fn close(a: Mat4, b: &[f32]) -> f32 {
    a.to_cols_array().iter().zip(b).map(|(x, y)| (x - y).abs()).fold(0.0, f32::max)
}

/// The rig, its action's curves and its world matrix.
fn thymus() -> (BRig, Vec<crate::blend::anim::fcurve::BFCurve>, Mat4) {
    let bytes = std::fs::read(FILE).expect("fixture");
    let file = BlendFile::parse_bytes(&bytes).expect("parse");
    let dna = file.dna.as_ref().expect("dna");
    let scene = file.extract_scene().expect("scene");
    let o = scene.objects.iter().find(|o| o.name == "rig").expect("rig");
    let ob = file.block_at_code(o.block_address, "OB").expect("OB");
    let rig = file.rig(ob).expect("a rig");
    let b = file.binding(ob, dna).expect("bound");
    let curves = file.actions(dna).get(&b.action).map(|a| a.curves(b.slot).to_vec()).unwrap_or_default();
    (rig, curves, Mat4::from_cols_array_2d(&o.matrix_world))
}

/// Every driver the rig object carries evaluates to Blender's value. Blender
/// 5.0 also moves the armature's eight `bones["…"].hide` drivers onto the
/// object when it opens a file this old (`version_bone_hide_property_driver`);
/// they hide bones in the viewport and move nothing.
#[test]
fn every_driver_reads_what_blender_reads() {
    let (rig, curves, _) = thymus();
    let want = truth();
    let ours: Vec<String> = rig.drivers.iter().map(|d| format!("{}#{}", d.curve.path, d.curve.index)).collect();
    let theirs: Vec<&String> = want.keys().filter(|k| k.0 == 1 && k.1 == "DV").map(|k| &k.2).collect();
    let missing: Vec<&&String> = theirs.iter().filter(|p| !ours.contains(p)).collect();
    assert_eq!((ours.len(), theirs.len()), (93, 101));
    assert!(missing.iter().all(|p| p.ends_with(".hide#0")), "{missing:?}");
    for f in FRAMES {
        let state = rig.state_at(super::BRigAnim::action(&curves), f as f32);
        for (d, path) in rig.drivers.iter().zip(&ours) {
            let v = want[&(f, "DV".into(), path.clone())][0];
            let got = d.value(&rig, &state, f as f32);
            assert!((got - v).abs() < 1e-4, "frame {f}: {path} = {got}, Blender {v}");
        }
    }
}

#[test]
fn every_bone_poses_where_blender_poses_it() {
    let (rig, curves, ob) = thymus();
    let want = truth();
    let mut worst: Vec<(f32, u32, String)> = Vec::new();
    for f in FRAMES {
        let pose = rig.pose_at(super::BRigAnim::action(&curves), f as f32, ob, &super::BEmptyScene);
        for (i, bone) in rig.bones.iter().enumerate() {
            let theirs = &want[&(f, "PM".into(), bone.name.clone())];
            worst.push((close(pose.pose[i], theirs), f, bone.name.clone()));
        }
    }
    worst.sort_by(|a, b| b.0.total_cmp(&a.0));
    let off: Vec<_> = worst.iter().filter(|w| w.0 > 1e-3).collect();
    assert!(off.is_empty(), "{} of {} bone poses differ by more than 1e-3; worst: {:?}", off.len(), worst.len(), &off[..off.len().min(25)]);
}

/// Not a fixture test: point `RIG_DIGEST` at a `.blend` and this prints
/// every bone's rest (`arm`), pose and chan matrix at frame 1, for
/// checking a file whose pose comes out wrong. Does nothing without the var.
#[test]
fn rig_digest_of_env_file() {
    let Ok(path) = std::env::var("RIG_DIGEST") else { return };
    let bytes = std::fs::read(&path).expect("read");
    let file = BlendFile::parse_bytes(&bytes).expect("parse");
    let dna = file.dna.as_ref().expect("dna");
    let scene = file.extract_scene().expect("scene");
    for o in scene.objects.iter().filter(|o| o.object_type == "ARMATURE") {
        let ob = file.block_at_code(o.block_address, "OB").expect("OB");
        let rig = file.rig(ob).expect("a rig");
        let b = file.binding(ob, dna).expect("bound");
        let curves = file.actions(dna).get(&b.action).map(|a| a.curves(b.slot).to_vec()).unwrap_or_default();
        let world = Mat4::from_cols_array_2d(&o.matrix_world);
        let pose = rig.pose_at(super::BRigAnim::action(&curves), 1.0, world, &super::BEmptyScene);
        for (i, bone) in rig.bones.iter().enumerate() {
            println!(
                "RIG {} / {}: raw rot {:?} rotmode {} arm loc {:?} pose loc {:?} chan loc {:?} chan x-col {:?}",
                o.name,
                bone.name,
                bone.raw.rot,
                bone.raw.rotmode,
                bone.arm.w_axis.truncate(),
                pose.pose[i].w_axis.truncate(),
                pose.chan[i].w_axis.truncate(),
                pose.chan[i].x_axis.truncate(),
            );
        }
    }
}
