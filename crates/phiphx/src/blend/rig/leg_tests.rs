//! Against Blender 5.2: `tests/truth/rigify_leg.txt` holds every bone of
//! the small Rigify-like leg `tests/fixtures/make_rigify_leg.py` builds —
//! an FK and an IK chain (two-bone IK with a pole and stretch) blended onto
//! the ORG chain by a driven Copy Transforms, a foot roll clamped by a
//! legacy Limit Rotation, tweaks with Damped Track and a half-influence
//! local Copy Transforms, DEF bones with Stretch To — at frames 1, 4, 7
//! and 10, with the driver's value.

use super::{BEmptyScene, BRig, BRigAnim};
use crate::blend::anim::fcurve::BFCurve;
use crate::blend::file::types::BlendFile;
use glam::Mat4;
use std::collections::HashMap;

const FILE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/rigify_leg.blend");
const TRUTH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/truth/rigify_leg.txt");
const FRAMES: [u32; 4] = [1, 4, 7, 10];

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

/// The rig, its action's curves and its world matrix.
fn leg() -> (BRig, Vec<BFCurve>, Mat4) {
    let bytes = std::fs::read(FILE).expect("fixture");
    let file = BlendFile::parse_bytes(&bytes).expect("parse");
    let dna = file.dna.as_ref().expect("dna");
    let scene = file.extract_scene().expect("scene");
    let o = scene.objects.iter().find(|o| o.name == "leg").expect("leg");
    let ob = file.block_at_code(o.block_address, "OB").expect("OB");
    let rig = file.rig(ob).expect("a rig");
    let b = file.binding(ob, dna).expect("bound");
    let curves = file.actions(dna).get(&b.action).map(|a| a.curves(b.slot).to_vec()).unwrap_or_default();
    (rig, curves, Mat4::from_cols_array_2d(&o.matrix_world))
}

fn close(a: Mat4, b: &[f32]) -> f32 {
    a.to_cols_array().iter().zip(b).map(|(x, y)| (x - y).abs()).fold(0.0, f32::max)
}

#[test]
fn the_ik_fk_switch_drives_what_blender_drives() {
    let (rig, curves, _) = leg();
    let want = truth();
    assert_eq!(rig.drivers.len(), 3);
    for f in FRAMES {
        let state = rig.state_at(BRigAnim::action(&curves), f as f32);
        for d in &rig.drivers {
            let path = format!("{}#{}", d.curve.path, d.curve.index);
            let v = want[&(f, "DV".into(), path.clone())][0];
            let got = d.value(&rig, &state, f as f32);
            assert!((got - v).abs() < 1e-4, "frame {f}: {path} = {got}, Blender {v}");
        }
    }
}

#[test]
fn every_leg_bone_poses_where_blender_poses_it() {
    let (rig, curves, ob) = leg();
    let want = truth();
    let mut worst: Vec<(f32, u32, String)> = Vec::new();
    for f in FRAMES {
        let pose = rig.pose_at(BRigAnim::action(&curves), f as f32, ob, &BEmptyScene);
        for (i, bone) in rig.bones.iter().enumerate() {
            let theirs = &want[&(f, "PM".into(), bone.name.clone())];
            worst.push((close(pose.pose[i], theirs), f, bone.name.clone()));
        }
    }
    worst.sort_by(|a, b| b.0.total_cmp(&a.0));
    let off: Vec<_> = worst.iter().filter(|w| w.0 > 1e-4).collect();
    assert!(off.is_empty(), "{} of {} bone poses differ by more than 1e-4; worst: {:?}", off.len(), worst.len(), &off[..off.len().min(25)]);
}

/// The roll bone's Limit Rotation carries the legacy clamp the fixture set,
/// as a file saved by Blender 5.2 stores it; an older file gets it from
/// the 4.2 versioning (`read.rs`), which THYMUS (3.6) exercises.
#[test]
fn the_roll_limit_keeps_its_legacy_clamp() {
    let (rig, _, _) = leg();
    let roll = rig.bone("MCH-roll2").expect("roll bone");
    let legacy = rig.bones[roll].constraints.iter().any(|c| matches!(c.kind, super::BConstraintKind::LimitRotation { flag, .. } if flag & (1 << 3) != 0));
    assert!(legacy);
}
