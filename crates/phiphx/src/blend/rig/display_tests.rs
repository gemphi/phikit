//! Against Blender 5.0 (`truth/blender_armature_display.py`): which bones an
//! armature shows in Object mode and where each bone's shape is drawn —
//! `fixtures/armature_display.blend` (nested and soloed bone collections,
//! a hidden pose bone, a bone hidden only in Edit mode, a bone in none,
//! custom shapes moved, turned, scaled, drawn at another bone and not scaled
//! by the bone, a Stick armature) at four frames, and THYMUS's Rigify rig (a
//! 3.6 file: layers, bone groups and hidden bones, its widgets' meshes read
//! in their own scope) at three — every custom shape's edges too.

use crate::blend::file::types::BlendFile;
use glam::Mat4;
use std::collections::HashMap;

const DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests");

/// `(frame, rig, bone) → (shown, shape, world matrix)`.
fn truth(name: &str) -> HashMap<(u32, String, String), (bool, String, Vec<f32>)> {
    std::fs::read_to_string(format!("{DIR}/truth/{name}"))
        .expect("truth file")
        .lines()
        .filter_map(|l| {
            let v: Vec<&str> = l.split('\t').collect();
            let m = v.get(5)?.split(' ').filter_map(|x| x.parse().ok()).collect();
            Some(((v[0].parse().ok()?, v[1].to_string(), v[2].to_string()), (v[3] == "1", v[4].to_string(), m)))
        })
        .collect()
}

/// Every bone of every rig the truth names, against it: `(rows, faults)`.
fn check(fixture: &str, truth_file: &str) -> (usize, Vec<String>) {
    let file = BlendFile::parse_bytes(&std::fs::read(format!("{DIR}/fixtures/{fixture}")).expect("fixture")).expect("parse");
    let scene = file.extract_scene().expect("scene");
    let want = truth(truth_file);
    let rigs = &scene.props.rigs.rigs;
    let mut frames: Vec<u32> = want.keys().map(|k| k.0).collect();
    frames.sort_unstable();
    frames.dedup();
    let mut bad = Vec::new();
    for f in frames {
        let at = scene.props.motion.solve(f as f32, rigs, true);
        for r in rigs {
            let name = &scene.objects[r.object].name;
            let look = r.look.as_ref().expect("a rig's look");
            let pose = &at.poses.get(&r.object).expect("every rig posed").pose;
            let world = Mat4::from_cols_array_2d(&at.world[r.object]);
            let shown: HashMap<usize, _> = look.shown(&r.rig, pose).into_iter().map(|s| (s.bone, s)).collect();
            for (i, b) in r.rig.bones.iter().enumerate() {
                let Some((vis, shape, m)) = want.get(&(f, name.clone(), b.name.clone())) else { continue };
                match (shown.get(&i), vis) {
                    (None, false) => {}
                    (Some(s), true) => {
                        let ours = match s.draw {
                            crate::blend::rig::BBoneDraw::Shape(a) => look.shapes[&a].name.clone(),
                            _ => "-".to_string(),
                        };
                        let d = (world * s.disp).to_cols_array().iter().zip(m).map(|(x, y)| (x - y).abs()).fold(0.0, f32::max);
                        if ours != *shape || d > 1e-4 {
                            bad.push(format!("{f} {name} {}: {ours} vs {shape}, off {d}", b.name));
                        }
                    }
                    (s, v) => bad.push(format!("{f} {name} {}: shown {} vs Blender {v}", b.name, s.is_some())),
                }
            }
        }
    }
    // Each custom shape a shown bone draws: its edges, by count and sum.
    let text = std::fs::read_to_string(format!("{DIR}/truth/{truth_file}")).expect("truth file");
    let ours: HashMap<&str, &crate::blend::rig::BBoneShape> = rigs.iter().flat_map(|r| r.look.as_ref().into_iter().flat_map(|l| l.shapes.values())).map(|s| (s.name.as_str(), s.as_ref())).collect();
    for l in text.lines().filter(|l| l.starts_with("SHAPE\t")) {
        let v: Vec<&str> = l.split('\t').collect();
        let (count, sum): (usize, f32) = (v[2].parse().unwrap(), v[3].parse().unwrap());
        match ours.get(v[1]) {
            Some(s) => {
                let total: f32 = s.edges.iter().map(|[a, b]| a.x + a.y + a.z + b.x + b.y + b.z).sum();
                if s.edges.len() != count || (total - sum).abs() > 1e-3 {
                    bad.push(format!("{}: {} edges summing {total} vs {count} summing {sum}", v[1], s.edges.len()));
                }
            }
            None => bad.push(format!("{}: not read", v[1])),
        }
    }
    (want.len(), bad)
}

#[test]
fn each_armature_shows_the_bones_blender_shows_where_it_draws_them() {
    let (rows, bad) = check("armature_display.blend", "armature_display.txt");
    assert!(rows >= 40, "{rows}");
    assert!(bad.is_empty(), "{bad:#?}");
}

#[test]
fn thymus_shows_its_54_controls_where_blender_draws_them() {
    let (rows, bad) = check("THYMUS-BOLDROID-WAVE-003.blend", "thymus_bones.txt");
    assert!(rows > 900, "{rows}");
    assert!(bad.is_empty(), "{} of {rows} differ: {:#?}", bad.len(), &bad[..bad.len().min(20)]);
}

#[test]
fn a_custom_shape_is_its_mesh_s_edges() {
    let file = BlendFile::parse_bytes(&std::fs::read(format!("{DIR}/fixtures/armature_display.blend")).expect("fixture")).expect("parse");
    let scene = file.extract_scene().expect("scene");
    let rig = scene.props.rigs.rigs.iter().find(|r| scene.objects[r.object].name == "Rig").expect("Rig");
    let look = rig.look.as_ref().expect("look");
    let edges: HashMap<&str, usize> = look.shapes.values().map(|s| (s.name.as_str(), s.edges.len())).collect();
    assert_eq!(edges.get("WGT-circle"), Some(&16), "a 16-vertex circle");
    assert_eq!(edges.get("WGT-cube"), Some(&12), "a cube's twelve edges");
    assert!(look.in_front == false && look.custom);
}
