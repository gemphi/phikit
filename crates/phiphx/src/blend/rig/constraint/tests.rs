//! Against Blender 5.0: `tests/truth/constraints.txt`
//! (`truth/blender_constraints.py`) holds the pose matrix of every bone of a
//! rig with one bone under each constraint batch 89 ports, at six frames,
//! with two animated control bones and an animated Empty as targets.

use crate::blend::file::types::BlendFile;
use crate::blend::rig::BRig;
use glam::Mat4;
use std::collections::HashMap;

const FILE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/constraints.blend");
const TRUTH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/truth/constraints.txt");
const FRAMES: [u32; 6] = [1, 7, 13, 20, 28, 35];

/// `(frame, kind, name) → matrix`.
fn truth() -> HashMap<(u32, String, String), Mat4> {
    std::fs::read_to_string(TRUTH)
        .expect("truth file")
        .lines()
        .filter_map(|l| {
            let mut it = l.split('\t');
            let (f, k, n, v) = (it.next()?, it.next()?, it.next()?, it.next()?);
            let m: Vec<f32> = v.split(' ').filter_map(|x| x.parse().ok()).collect();
            Some(((f.parse().ok()?, k.to_string(), n.to_string()), Mat4::from_cols_slice(&m)))
        })
        .collect()
}

#[test]
fn every_constrained_bone_poses_where_blender_poses_it() {
    let bytes = std::fs::read(FILE).expect("fixture");
    let file = BlendFile::parse_bytes(&bytes).expect("parse");
    let dna = file.dna.as_ref().expect("dna");
    let scene = file.extract_scene().expect("scene");
    let o = scene.objects.iter().find(|o| o.name == "Rig").expect("the rig");
    let aim = scene.objects.iter().find(|o| o.name == "Aim").expect("the Empty").block_address;
    let ob = file.block_at_code(o.block_address, "OB").expect("OB");
    let rig: BRig = file.rig(ob).expect("a rig");
    let b = file.binding(ob, dna).expect("bound");
    let curves = file.actions(dna).get(&b.action).map(|a| a.curves(b.slot).to_vec()).unwrap_or_default();
    let world = Mat4::from_cols_array_2d(&o.matrix_world);
    let want = truth();
    let mut off = Vec::new();
    for f in FRAMES {
        let aim_world = want[&(f, "OB".into(), "Aim".into())];
        let others = |address: u64| (address == aim).then_some(aim_world);
        let pose = rig.pose_at(&curves, f as f32, world, &others);
        for (i, bone) in rig.bones.iter().enumerate() {
            let theirs = want[&(f, "PM".into(), bone.name.clone())];
            let d = pose.pose[i].to_cols_array().iter().zip(theirs.to_cols_array()).map(|(a, b)| (a - b).abs()).fold(0.0, f32::max);
            if d > 1e-4 {
                off.push((d, f, bone.name.clone()));
            }
        }
    }
    off.sort_by(|a, b| b.0.total_cmp(&a.0));
    assert!(off.is_empty(), "{} bone poses differ from Blender's: {:?}", off.len(), &off[..off.len().min(30)]);
}
