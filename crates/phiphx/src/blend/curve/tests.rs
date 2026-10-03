//! Against Blender 5.0: `tests/truth/curve_paths.txt`
//! (`truth/blender_curve_paths.py`) holds every object's world matrix at
//! six frames — children riding a closed 3D Bézier ring (its time a
//! Generator), an open NURBS path with tilt and radius, a poly line, a flat
//! closed Bézier and an open Bézier with a vector corner, and objects under
//! Follow Path (animated, fixed, with radius, without following). The file
//! is saved at frame 12.

use crate::blend::file::types::BlendFile;
use glam::Mat4;
use std::collections::HashMap;

const FILE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/curve_paths.blend");
const TRUTH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/truth/curve_paths.txt");
const FRAMES: [u32; 6] = [1, 9, 12, 20, 33, 47];
const THYMUS: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/THYMUS-BOLDROID-WAVE-003.blend");
const THYMUS_TRUTH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/truth/thymus_turntable.txt");

/// `(frame, name) → matrix_world`.
fn truth() -> HashMap<(u32, String), Mat4> {
    read_truth(TRUTH)
}

fn read_truth(path: &str) -> HashMap<(u32, String), Mat4> {
    std::fs::read_to_string(path)
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
fn every_object_on_a_path_stands_where_blender_puts_it() {
    let file = BlendFile::parse_bytes(&std::fs::read(FILE).expect("fixture")).expect("parse");
    let scene = file.extract_scene().expect("scene");
    let motion = &scene.props.motion;
    let paths = motion.list.iter().filter(|m| m.path.is_some()).count();
    assert_eq!(paths, 5, "every curve reads as a path");
    let want = truth();
    let mut bad = Vec::new();
    for f in FRAMES {
        let at = motion.solve(f as f32, &[], false);
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
fn a_still_read_puts_path_riders_where_the_saved_frame_has_them() {
    let file = BlendFile::parse_bytes(&std::fs::read(FILE).expect("fixture")).expect("parse");
    let scene = file.extract_scene().expect("scene");
    let want = truth();
    let bad: Vec<_> = scene.objects.iter().map(|o| (off(&o.matrix_world, &want[&(12, o.name.clone())]), o.name.clone())).filter(|(d, _)| *d > 1e-4).collect();
    assert!(bad.is_empty(), "read at the saved frame, off: {bad:?}");
}

#[test]
fn thymus_turntable_camera_rides_its_path_as_in_blender() {
    let file = BlendFile::parse_bytes(&std::fs::read(THYMUS).expect("THYMUS")).expect("parse");
    let scene = file.extract_scene().expect("scene");
    let motion = &scene.props.motion;
    let cam = motion.list.iter().position(|m| m.name == "turn_cam_02").expect("the turntable camera");
    for ((frame, _), theirs) in read_truth(THYMUS_TRUTH) {
        let at = motion.solve(frame as f32, &scene.props.rigs.rigs, false);
        let d = off(&at.world[cam], &theirs);
        assert!(d < 1e-4, "frame {frame}: the turntable camera is {d} off Blender's");
    }
    let still = scene.objects.iter().find(|o| o.name == "turn_cam_02").expect("read");
    assert!(off(&still.matrix_world, &read_truth(THYMUS_TRUTH)[&(1, "turn_cam_02".into())]) < 1e-4, "read at the saved frame");
}

/// Clamp To (`truth/blender_clamp_to.py`): owners keyed before, through and
/// past the bounds of an open Bézier rail (the automatic axis, and Y), a
/// NURBS tower under a turning parent, a poly line whose radii widen its
/// box, and a closed loop clamped flat and cyclically, one after Copy
/// Location — every object at six frames, and as read at the saved frame.
#[test]
fn every_object_under_clamp_to_stands_where_blender_puts_it() {
    const CLAMP: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/clamp_to.blend");
    const CLAMP_TRUTH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/truth/clamp_to.txt");
    let file = BlendFile::parse_bytes(&std::fs::read(CLAMP).expect("fixture")).expect("parse");
    let scene = file.extract_scene().expect("scene");
    let motion = &scene.props.motion;
    let want = read_truth(CLAMP_TRUTH);
    let mut bad = Vec::new();
    for f in [1u32, 7, 13, 20, 27, 35] {
        let at = motion.solve(f as f32, &[], false);
        for (m, world) in motion.list.iter().zip(&at.world) {
            let d = off(world, &want[&(f, m.name.clone())]);
            if d > 1e-4 {
                bad.push((d, f, m.name.clone()));
            }
        }
    }
    bad.sort_by(|a, b| b.0.total_cmp(&a.0));
    assert!(bad.is_empty(), "{} object matrices differ from Blender's: {:?}", bad.len(), &bad[..bad.len().min(30)]);
    let still: Vec<_> = scene.objects.iter().map(|o| (off(&o.matrix_world, &want[&(13, o.name.clone())]), o.name.clone())).filter(|(d, _)| *d > 1e-4).collect();
    assert!(still.is_empty(), "read at the saved frame, off: {still:?}");
}
