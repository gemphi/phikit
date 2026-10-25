//! A rig posed by NLA strips alone, against Blender 5.2
//! (`tests/fixtures/make_nla_rig.py` writes the file and
//! `tests/truth/nla_rig.txt`): two strips back to back on one track, a
//! third blending in and out over the held one, and the rig object sliding
//! on a track of its own. Each bone's saved pose is off its default, so the
//! frames no strip covers check that evaluation starts from the default.

use crate::blend::file::types::BlendFile;
use std::collections::HashMap;

const FILE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/nla_rig.blend");
const TRUTH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/truth/nla_rig.txt");

/// `(frame, bone or "object") → values`.
fn truth() -> HashMap<(u32, String), Vec<f32>> {
    std::fs::read_to_string(TRUTH)
        .expect("truth file")
        .lines()
        .filter_map(|l| {
            let mut it = l.split('\t');
            let (f, name, v) = (it.next()?, it.next()?, it.next()?);
            Some(((f.parse().ok()?, name.to_string()), v.split(' ').filter_map(|x| x.parse().ok()).collect()))
        })
        .collect()
}

#[test]
fn strips_pose_the_bones_and_move_the_rig_where_blender_does() {
    let bytes = std::fs::read(FILE).expect("fixture");
    let file = BlendFile::parse_bytes(&bytes).expect("parse");
    let dna = file.dna.as_ref().expect("dna");
    let scene = file.extract_scene().expect("scene");
    let actions = file.actions(dna);
    let sources = file.rig_sources(&scene.objects, dna, &actions);
    let source = sources.iter().find(|s| scene.objects[s.object].name == "Rig").expect("the rig");
    assert!(source.moves, "strips alone move it");
    assert!(source.curves.is_empty(), "no action assigned");
    let motion = file.motion_list(&scene.objects, &actions, dna);
    let want = truth();
    let mut frames: Vec<u32> = want.keys().map(|k| k.0).collect();
    frames.sort_unstable();
    frames.dedup();
    for f in frames {
        let state = source.rig.state_at(source.anim(), f as f32);
        for (i, bone) in source.rig.bones.iter().enumerate() {
            let theirs = &want[&(f, bone.name.clone())];
            let raw = &state.raw[i];
            let ours: Vec<f32> = raw.loc.iter().chain(&raw.quat).copied().collect();
            let worst = ours.iter().zip(theirs).map(|(a, b)| (a - b).abs()).fold(0.0, f32::max);
            assert!(worst < 1e-4, "frame {f} {}: {ours:?}, Blender {theirs:?}", bone.name);
        }
        let x = motion.list[source.object].raw_at(f as f32).loc[0];
        let theirs = want[&(f, "object".to_string())][0];
        assert!((x - theirs).abs() < 1e-4, "frame {f}: the rig at x {x}, Blender {theirs}");
    }
}
