//! THYMUS waves: its rig posed at a frame deforms the character's meshes
//! where Blender 5.0 puts them — every 25th vertex of each mesh the rig
//! deforms, as the Armature modifier leaves it, object space, at frames 1
//! and 66 (`truth/thymus_rig.txt`, `truth/blender_rig.py`).

use phiphx::phx::Phx;
use std::collections::HashMap;
use std::path::Path;

const FILE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/THYMUS-BOLDROID-WAVE-003.blend");
const TRUTH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/truth/thymus_rig.txt");

/// `(frame, mesh) → (vertex count, every 25th vertex)`.
fn truth() -> HashMap<(u32, String), (usize, Vec<[f32; 3]>)> {
    std::fs::read_to_string(TRUTH)
        .expect("truth file")
        .lines()
        .filter(|l| l.contains("\tMV\t"))
        .filter_map(|l| {
            let mut it = l.split('\t');
            let (f, _, name, v) = (it.next()?, it.next()?, it.next()?, it.next()?);
            let nums: Vec<f32> = v.split(' ').filter_map(|x| x.parse().ok()).collect();
            let count = nums[0] as usize;
            Some(((f.parse().ok()?, name.to_string()), (count, nums[1..].chunks_exact(3).map(|c| [c[0], c[1], c[2]]).collect())))
        })
        .collect()
}

/// For each of Blender's vertices, the nearest of ours, and the farthest
/// such pair.
fn pairs(ours: &[[f32; 3]], theirs: &[[f32; 3]]) -> (Vec<usize>, f32) {
    let d2 = |a: &[f32; 3], b: &[f32; 3]| (0..3).map(|i| (a[i] - b[i]).powi(2)).sum::<f32>();
    let mut worst = 0.0f32;
    let pairs = theirs
        .iter()
        .map(|t| {
            let (j, d) = ours.iter().enumerate().map(|(j, o)| (j, d2(o, t))).min_by(|a, b| a.1.total_cmp(&b.1)).expect("vertices");
            worst = worst.max(d.sqrt());
            j
        })
        .collect();
    (pairs, worst)
}

fn max_diff(a: [f32; 3], b: [f32; 3]) -> f32 {
    (0..3).map(|i| (a[i] - b[i]).abs()).fold(0.0, f32::max)
}

/// Two meshes deform before subdividing (the torso, the upper legs): their
/// vertices are the mesh's own, compared one for one. The nine subdivided
/// first number their vertices as the subdivision does, so each of
/// Blender's sampled vertices must have one of ours where it is, at both
/// frames. The subdivision itself sits within 0.01 mm of Blender's on every
/// mesh, the head's 136 creased edges included (`subsurf_creases.rs`).
#[test]
fn the_character_waves_where_blender_waves_it() {
    let model = Phx::open(Path::new(FILE)).expect("THYMUS opens");
    assert!(model.motion.animated(), "the rig moves");
    assert_eq!(model.motion.rigs.skins.len(), 11, "the eleven meshes the rig deforms");
    let want = truth();
    let first = model.motion.skinned_at(1.0);
    let at = std::time::Instant::now();
    let later = model.motion.skinned_at(66.0);
    eprintln!("frame 66: {} meshes skinned in {:.1} ms", later.len(), at.elapsed().as_secs_f64() * 1e3);
    let mut moved = 0;
    for (frame, skinned) in [(1u32, &first), (66, &later)] {
        for (object, ours) in skinned.iter() {
            let name = model.objects[*object].name.clone();
            let (count, theirs) = &want[&(frame, name.clone())];
            let (_, at_one) = &want[&(1, name.clone())];
            assert_eq!(ours.len(), *count, "{name}: vertex count");
            if frame == 66 {
                moved += theirs.iter().zip(at_one).filter(|(a, b)| max_diff(**a, **b) > 0.01).count();
            }
            if matches!(name.as_str(), "torso_geo" | "leg_upper_geo") {
                let worst = theirs.iter().enumerate().map(|(k, v)| max_diff(ours[k * 25], *v)).fold(0.0, f32::max);
                assert!(worst < 1e-4, "{name} at frame {frame}: a vertex {worst} off Blender's");
            } else {
                let (_, worst) = pairs(ours, theirs);
                assert!(worst < 1e-4, "{name} at frame {frame}: a vertex {worst} from the nearest of ours");
            }
        }
    }
    assert!(moved > 100, "the arm waves between frames 1 and 66 ({moved} sampled vertices move)");
    // Same triangles every frame, so the tracer takes a frame as a new pose.
    let a = model.motion.meshes_at(1.0);
    let at = std::time::Instant::now();
    let b = model.motion.meshes_at(66.0);
    eprintln!("frame 66: {} meshes posed and subdivided in {:.1} ms", b.len(), at.elapsed().as_secs_f64() * 1e3);
    assert!(a.iter().zip(&b).all(|((_, x), (_, y))| x.indices == y.indices && x.vertices.len() == y.vertices.len()));
    // Frame 1 is the pose the file was saved in: the meshes as read, the
    // subdivision after the Armature included, normals and all.
    for (object, mesh) in &a {
        let read = &model.objects[*object].mesh;
        let off = mesh.vertices.iter().zip(&read.vertices).map(|(p, q)| max_diff(*p, *q)).fold(0.0, f32::max);
        assert!(off < 1e-4 && mesh.indices == read.indices, "{}: {off} from the mesh as read", model.objects[*object].name);
        assert_eq!(mesh.corners.ids, read.corners.ids);
    }
}
