//! Against Blender 5.0: `tests/truth/object_constraints.txt`
//! (`truth/blender_object_constraints.py`) holds every object's world
//! matrix at six frames — objects under each constraint kind in world,
//! local and custom spaces, a chain through a constrained camera, an
//! animated influence, and objects that follow an animated rig's bones by
//! constraints and bone parents. The file is saved at frame 13.

use crate::blend::file::types::BlendFile;
use glam::Mat4;
use std::collections::HashMap;

const FILE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/object_constraints.blend");
const TRUTH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/truth/object_constraints.txt");
const FRAMES: [u32; 6] = [1, 7, 13, 20, 28, 35];
const SAVED: u32 = 13;

/// `(frame, name) → matrix_world`.
fn truth() -> HashMap<(u32, String), Mat4> {
    std::fs::read_to_string(TRUTH)
        .expect("truth file")
        .lines()
        .filter_map(|l| {
            let mut it = l.split('\t');
            let (f, _, n, v) = (it.next()?, it.next()?, it.next()?, it.next()?);
            let m: Vec<f32> = v.split(' ').filter_map(|x| x.parse().ok()).collect();
            Some(((f.parse().ok()?, n.to_string()), Mat4::from_cols_slice(&m)))
        })
        .collect()
}

fn off(ours: &[[f32; 4]; 4], theirs: &Mat4) -> f32 {
    Mat4::from_cols_array_2d(ours).to_cols_array().iter().zip(theirs.to_cols_array()).map(|(a, b)| (a - b).abs()).fold(0.0, f32::max)
}

#[test]
fn every_following_object_stands_where_blender_puts_it() {
    let file = BlendFile::parse_bytes(&std::fs::read(FILE).expect("fixture")).expect("parse");
    let scene = file.extract_scene().expect("scene");
    let (motion, rigs) = (&scene.props.motion, &scene.props.rigs.rigs);
    assert!(motion.follows(), "the fixture's objects follow others");
    assert_eq!(rigs.len(), 1, "the rig is posed");
    let want = truth();
    let mut bad = Vec::new();
    for f in FRAMES {
        let at = motion.solve(f as f32, rigs, false);
        for (m, world) in motion.list.iter().zip(&at.world) {
            let d = off(world, &want[&(f, m.name.clone())]);
            if d > 1e-4 {
                bad.push((d, f, m.name.clone()));
            }
        }
    }
    bad.sort_by(|a, b| b.0.total_cmp(&a.0));
    assert!(bad.is_empty(), "{} object matrices differ from Blender's: {:?}", bad.len(), &bad[..bad.len().min(30)]);
}

#[test]
fn a_still_read_places_followers_at_the_saved_frame() {
    let file = BlendFile::parse_bytes(&std::fs::read(FILE).expect("fixture")).expect("parse");
    let scene = file.extract_scene().expect("scene");
    let want = truth();
    let bad: Vec<_> = scene
        .objects
        .iter()
        .map(|o| (off(&o.matrix_world, &want[&(SAVED, o.name.clone())]), o.name.clone()))
        .filter(|(d, _)| *d > 1e-4)
        .collect();
    assert!(bad.is_empty(), "read at the saved frame, off: {bad:?}");
}

#[test]
fn the_scene_model_films_the_tracking_camera_where_blender_does() {
    let file = BlendFile::parse_bytes(&std::fs::read(FILE).expect("fixture")).expect("parse");
    let model = crate::scene::SceneModel::from_blend(&file.extract_scene().expect("scene"), FILE);
    assert!(model.motion.animated());
    let want = truth();
    let cam = model.cameras.iter().position(|c| c.name == "Cam").expect("the camera");
    let held = model.empties.iter().position(|e| e.name == "Held").expect("the bone's child");
    for f in FRAMES {
        let at = model.motion.at(f as f32);
        let camera = at.camera(cam).expect("the camera moves");
        assert!(off(&camera, &want[&(f, "Cam".into())]) < 1e-4, "frame {f}: the camera");
        let empty = at.empty(held).expect("the bone's child moves");
        assert!(off(&empty, &want[&(f, "Held".into())]) < 1e-4, "frame {f}: the bone's child");
    }
}
