//! Against Blender: `truth/thymus_anim.txt` holds, at frames 1 / 20 / 40 /
//! 66 / 100 / 132 of THYMUS, the metarig object's world matrix (`OB`), the
//! pose matrix of every bone whose parent chain is free of constraints and
//! drivers (`PB` — just `root` on a Rigify rig), and every bone's
//! `matrix_basis` (`PBB`), which is the F-curves and `BKE_pchan_to_mat4`
//! alone.

use super::pose::{bases_at, poses_at};
use crate::blend::file::types::BlendFile;
use glam::Mat4;
use std::collections::HashMap;

const FILE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/THYMUS-BOLDROID-WAVE-003.blend");
const TRUTH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/truth/thymus_anim.txt");

/// `(frame, kind, name) → matrix` (Blender writes column-major, as glam reads).
fn truth() -> HashMap<(u32, String, String), Mat4> {
    let text = std::fs::read_to_string(TRUTH).expect("truth file");
    text.lines()
        .filter_map(|l| {
            let mut it = l.split('\t');
            let (f, k, n, m) = (it.next()?, it.next()?, it.next()?, it.next()?);
            let v: Vec<f32> = m.split(' ').filter_map(|x| x.parse().ok()).collect();
            if v.len() != 16 {
                return None;
            }
            let m: [f32; 16] = v.try_into().ok()?;
            Some(((f.parse().ok()?, k.to_string(), n.to_string()), Mat4::from_cols_array(&m)))
        })
        .collect()
}

fn close(a: Mat4, b: Mat4) -> f32 {
    a.to_cols_array().iter().zip(b.to_cols_array()).map(|(x, y)| (x - y).abs()).fold(0.0, f32::max)
}

const FRAMES: [u32; 6] = [1, 20, 40, 66, 100, 132];

#[test]
fn the_metarig_object_moves_where_blender_moves_it() {
    let bytes = std::fs::read(FILE).expect("fixture");
    let file = BlendFile::parse_bytes(&bytes).expect("parse");
    let dna = file.dna.as_ref().expect("dna");
    let scene = file.extract_scene().expect("scene");
    let want = truth();
    let actions = file.actions(dna);
    let o = scene.objects.iter().find(|o| o.name == "Character-Metarig").expect("metarig");
    let ob = file.block_at_code(o.block_address, "OB").expect("OB");
    let b = file.binding(ob, dna).expect("the metarig is animated");
    let action = actions.get(&b.action).expect("its action was read");
    assert_eq!(action.name, "Armature.001Action");
    assert_eq!(action.curves(b.slot).len(), 9, "location, rotation_euler, scale × 3");
    let raw = file.read_object_transform_raw(ob, dna);
    // The metarig has no parent, so its basis is its world matrix. (Its
    // action holds nine curves but Blender reports the same matrix at every
    // frame — the keys are flat — so this pins agreement, not motion; the
    // motion check is the rig test below.)
    for f in FRAMES {
        let ours = super::channels::basis_at(raw, action.curves(b.slot).iter(), f as f32);
        let theirs = want[&(f, "OB".into(), "Character-Metarig".into())];
        assert!(close(ours, theirs) < 2e-3, "frame {f}: differs from Blender by {}", close(ours, theirs));
    }
    // The scene's motion list — what a player reads — says the same: the
    // metarig's world at each frame is Blender's, and the rest of the
    // scene's still objects keep the world matrices the load path solved.
    let motion = &scene.props.motion;
    assert_eq!(motion.list.len(), scene.objects.len(), "one motion per object");
    let metarig = scene.objects.iter().position(|o| o.name == "Character-Metarig").unwrap();
    assert!(motion.list[metarig].animated());
    for f in FRAMES {
        let world = motion.world_at(f as f32);
        let theirs = want[&(f, "OB".into(), "Character-Metarig".into())];
        assert!(close(Mat4::from_cols_array_2d(&world[metarig]), theirs) < 2e-3, "frame {f}: the motion list differs");
        for (i, o) in scene.objects.iter().enumerate() {
            if !motion.list[i].animated() && motion.list[i].parent.is_none_or(|p| !motion.list[p].animated()) {
                assert!(close(Mat4::from_cols_array_2d(&world[i]), Mat4::from_cols_array_2d(&o.matrix_world)) < 1e-4, "{}: a still object stays where the load put it", o.name);
            }
        }
    }
}

#[test]
fn every_bones_own_channels_match_blender_at_every_frame() {
    let bytes = std::fs::read(FILE).expect("fixture");
    let file = BlendFile::parse_bytes(&bytes).expect("parse");
    let dna = file.dna.as_ref().expect("dna");
    let scene = file.extract_scene().expect("scene");
    let want = truth();
    let actions = file.actions(dna);
    let o = scene.objects.iter().find(|o| o.name == "rig").expect("rig");
    let ob = file.block_at_code(o.block_address, "OB").expect("OB");
    let b = file.binding(ob, dna).expect("the rig is animated");
    let action = actions.get(&b.action).expect("rigAction");
    assert_eq!(action.name, "rigAction");
    assert_eq!(action.curves(b.slot).len(), 530);
    let channels = file.pose_channels(ob, dna);
    assert_eq!(channels.len(), 328, "every bone of the rig is a channel");
    let (mut n, mut driven, mut keyed_moved) = (0usize, 0usize, 0usize);
    for f in FRAMES {
        let bases = bases_at(&channels, Some(action), b.slot, f as f32);
        for (c, ours) in channels.iter().zip(&bases) {
            // A driven bone's basis is driver output — stage (c). It is
            // counted, not compared.
            let Some(theirs) = want.get(&(f, "PBB".into(), c.name.clone())) else {
                assert!(want.contains_key(&(f, "PBD".into(), c.name.clone())), "{} missing from the truth", c.name);
                driven += 1;
                continue;
            };
            assert!(close(*ours, *theirs) < 2e-3, "frame {f}, bone {}: basis differs by {}", c.name, close(*ours, *theirs));
            n += 1;
            if f != 1 && close(*theirs, want[&(1, "PBB".into(), c.name.clone())]) > 1e-3 {
                keyed_moved += 1;
            }
        }
    }
    assert_eq!(n + driven, 328 * 6);
    assert!(driven < n / 4, "most bones are keyed or still, not driven: {driven} driven of {}", n + driven);
    // 54 bones are keyed; many of their keys are flat. What does move must
    // move in the truth, or the agreement above would prove nothing.
    assert!(keyed_moved > 20, "the keyed bones must actually move in the truth ({keyed_moved})");
}

#[test]
fn a_bone_with_a_clean_parent_chain_poses_where_blender_poses_it() {
    let bytes = std::fs::read(FILE).expect("fixture");
    let file = BlendFile::parse_bytes(&bytes).expect("parse");
    let dna = file.dna.as_ref().expect("dna");
    let scene = file.extract_scene().expect("scene");
    let want = truth();
    let actions = file.actions(dna);
    let o = scene.objects.iter().find(|o| o.name == "rig").expect("rig");
    let ob = file.block_at_code(o.block_address, "OB").expect("OB");
    let b = file.binding(ob, dna).expect("bound");
    let channels = file.pose_channels(ob, dna);
    let root = channels.iter().position(|c| c.name == "root").expect("root");
    for f in FRAMES {
        let poses = poses_at(&channels, actions.get(&b.action), b.slot, f as f32);
        let theirs = want[&(f, "PB".into(), "root".into())];
        assert!(close(poses[root], theirs) < 2e-3, "frame {f}: root pose differs by {}", close(poses[root], theirs));
    }
}
