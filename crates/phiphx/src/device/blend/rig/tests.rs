//! Against Blender 5.0: `tests/truth/thymus_rig.txt` (`truth/blender_rig.py`)
//! holds every one of THYMUS's 328 rig bones' pose matrix at frames 1, 20,
//! 40, 66, 100 and 132 — after its drivers, its 330 constraints and the
//! legs' IK — and every driver's value.

use super::BRig;
use crate::device::blend::file::types::BlendFile;
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
fn thymus() -> (BRig, Vec<crate::device::blend::anim::fcurve::BFCurve>, Mat4) {
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

/// Not a fixture test: point `RIG_DEFORM` at a `.blend` and `RIG_FRAMES` at
/// frames (`5,10`), and this checks what the Armature modifier is handed. A
/// point at the head, middle and tail of each deforming bone's rest, moved by
/// that bone's deform (B-Bone segments included), should land on the same point
/// of the posed bone; it prints the bones that miss most. Does nothing without
/// the var.
#[test]
fn rig_deform_of_env_file() {
    use super::math::BMat4;
    use glam::Vec3;
    let Ok(path) = std::env::var("RIG_DEFORM") else { return };
    let frames: Vec<f32> = std::env::var("RIG_FRAMES").unwrap_or_else(|_| "1".into()).split(',').filter_map(|f| f.trim().parse().ok()).collect();
    let bytes = std::fs::read(&path).expect("read");
    let file = BlendFile::parse_bytes(&bytes).expect("parse");
    let dna = file.dna.as_ref().expect("dna");
    let scene = file.extract_scene().expect("scene");
    let actions = file.actions(dna);
    for src in file.rig_sources(&scene.objects, dna, &actions) {
        let world = Mat4::from_cols_array_2d(&scene.objects[src.object].matrix_world);
        for &f in &frames {
            let pose = src.pose_at(f, world, &super::BEmptyScene);
            let mats = src.deform_bones(&pose);
            let mut worst: Vec<(f32, String, f32)> = Vec::new();
            for (i, bone) in src.rig.bones.iter().enumerate() {
                let Some(d) = mats.get(&bone.name) else { continue };
                let head = bone.arm.loc();
                let along = bone.arm.y_axis.truncate().normalize() * bone.length;
                for t in [0.0f32, 0.5, 1.0] {
                    let co = head + along * t;
                    let got = d.matrix(co).transform_point3(co);
                    let want = pose.pose[i].transform_point3(Vec3::new(0.0, bone.length * t, 0.0));
                    worst.push(((got - want).length(), bone.name.clone(), t));
                }
            }
            worst.sort_by(|a, b| b.0.total_cmp(&a.0));
            for (miss, name, t) in worst.iter().take(8) {
                println!("DEFORM frame {f} {name} at {t}: misses by {miss:.3}");
            }
        }
    }
}

fn moved_for_report(v: &[[f32; 3]], d: &[Vec<(u32, f32)>], g: &[String], m: &HashMap<String, crate::device::blend::modifier::armature::BDeformBone>, to_arm: Mat4) -> Vec<[f32; 3]> {
    crate::device::blend::modifier::armature::deform(v, d, g, m, to_arm).unwrap_or_else(|| v.to_vec())
}

/// Not a fixture test: with `RIG_DEFORM` and `RIG_FRAMES` as above and
/// `RIG_SKIN` naming a mesh object, this poses that mesh the way a frame is
/// rendered — the base before its Armature, skinned by the solved pose, then
/// the modifiers after it — and prints the vertices that land far from the
/// rest of the mesh, with their weights.
#[test]
fn rig_skin_of_env_file() {
    use glam::Vec3;
    let (Ok(path), Ok(name)) = (std::env::var("RIG_DEFORM"), std::env::var("RIG_SKIN")) else { return };
    let frames: Vec<f32> = std::env::var("RIG_FRAMES").unwrap_or_else(|_| "1".into()).split(',').filter_map(|f| f.trim().parse().ok()).collect();
    let bytes = std::fs::read(&path).expect("read");
    let file = BlendFile::parse_bytes(&bytes).expect("parse");
    let scene = file.extract_scene().expect("scene");
    let rigs = &scene.props.rigs;
    for skin in rigs.skins.iter().filter(|s| scene.objects[s.object].name == name) {
        let rig = rigs.rigs.iter().find(|r| r.address == skin.armature).expect("its rig");
        let rig_world = Mat4::from_cols_array_2d(&scene.objects[rig.object].matrix_world);
        let world = Mat4::from_cols_array_2d(&scene.objects[skin.object].matrix_world);
        println!("SKIN {name}: base {} verts, dverts {}, groups {}, tail {}", skin.base.vertices.len(), skin.base.dverts.len(), skin.groups.len(), skin.tail.len());
        for &f in &frames {
            let pose = rig.pose_at(f, rig_world, &super::BEmptyScene);
            let mats = rig.deform_bones(&pose);
            let to_arm = rig_world.inverse() * world;
            let moved = crate::device::blend::modifier::armature::deform(&skin.base.vertices, &skin.base.dverts, &skin.groups, &mats, to_arm).unwrap_or_else(|| skin.base.vertices.clone());
            let pts: Vec<Vec3> = moved.iter().map(|v| world.transform_point3(Vec3::from(*v))).collect();
            let mut sorted: Vec<Vec3> = pts.clone();
            let mut mid = |k: usize| {
                sorted.sort_by(|a, b| a[k].total_cmp(&b[k]));
                sorted[sorted.len() / 2][k]
            };
            let centre = Vec3::new(mid(0), mid(1), mid(2));
            let mut far: Vec<(f32, usize)> = pts.iter().enumerate().map(|(i, p)| ((*p - centre).length(), i)).collect();
            far.sort_by(|a, b| b.0.total_cmp(&a.0));
            println!("SKIN frame {f}: centre {centre:?}, farthest {:.2} m", far[0].0);
            for &(d, i) in far.iter().take(6) {
                let named: Vec<(String, f32)> = skin.base.dverts.get(i).map(|dv| dv.iter().map(|&(g, w)| (skin.groups.get(g as usize).cloned().unwrap_or(format!("#{g}")), w)).collect()).unwrap_or_default();
                println!("  vertex {i} at {d:.2} m: rest {:?} weights {named:?}", skin.base.vertices[i]);
            }
            let evaluated = skin.evaluate(moved, world);
            let tail_far = evaluated.vertices.iter().map(|v| (world.transform_point3(Vec3::from(*v)) - centre).length()).fold(0.0f32, f32::max);
            println!("SKIN frame {f}: after the tail {} verts, farthest {tail_far:.2} m", evaluated.vertices.len());
            // The same tail over the whole base mesh, every attribute kept.
            let mut full = skin.base.clone();
            full.vertices = crate::device::blend::modifier::armature::deform(&skin.base.vertices, &skin.base.dverts, &skin.groups, &mats, to_arm).unwrap_or_else(|| skin.base.vertices.clone());
            let worlds = HashMap::new();
            let ctx = crate::device::blend::modifier::EvalCtx { world, worlds: &worlds };
            for m in &skin.tail {
                m.apply(&mut full, &ctx);
            }
            let full_far = full.vertices.iter().map(|v| (world.transform_point3(Vec3::from(*v)) - centre).length()).fold(0.0f32, f32::max);
            println!("SKIN frame {f}: tail over the full mesh {} verts, farthest {full_far:.2} m; base vert_creases {} edge_creases {} loose {} sharp {} uvs {} smooth {}", full.vertices.len(), skin.base.vert_creases.len(), skin.base.edge_creases.len(), skin.base.loose_edges.len(), skin.base.sharp_edges.len(), skin.base.loop_uvs.len(), skin.base.face_smooth.len());
            // Which subdivided vertices fly off, and the input vertex each starts from.
            let mut off: Vec<(f32, usize)> = full.vertices.iter().enumerate().map(|(i, v)| ((world.transform_point3(Vec3::from(*v)) - centre).length(), i)).collect();
            off.sort_by(|a, b| b.0.total_cmp(&a.0));
            let n = skin.base.vertices.len();
            let faces = &skin.base.faces;
            for &(d, i) in off.iter().take(4) {
                let mut near = None;
                let p = Vec3::from(full.vertices[i]);
                let mut best = f32::MAX;
                for (k, v) in moved_for_report(&skin.base.vertices, &skin.base.dverts, &skin.groups, &mats, to_arm).iter().enumerate() {
                    let e = (Vec3::from(*v) - p).length();
                    if e < best { best = e; near = Some(k); }
                }
                let k = near.unwrap_or(0);
                let around: Vec<usize> = (0..faces.len()).filter(|&fi| faces.get(fi).contains(&(k as u32))).collect();
                let corners: Vec<Vec<u32>> = around.iter().map(|&fi| faces.get(fi).to_vec()).collect();
                println!("  out {i} ({}) at {d:.2} m; nearest input {k} {best:.3} m away, in {} faces: {corners:?}", if i < n { "an input vertex" } else { "a new point" }, around.len());
            }
        }
    }
}
