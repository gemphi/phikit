//! The Geometry Nodes "Smooth by Angle" group evaluates as Blender 5.2
//! evaluates it: `fixtures/smooth_by_angle.blend` and
//! `truth/smooth_by_angle.txt` (`fixtures/make_smooth_by_angle.py`) — cubes,
//! a cylinder, a sphere, a rippled grid with loose geometry and a
//! non-manifold fan, at several angles, with "Ignore Sharpness" on and off,
//! pre-marked sharp edges and flat faces — and one production object, the
//! BOZO car's hood under Solidify then the group
//! (`truth/smooth_by_angle_bozo.txt`, `truth/blender_smooth_by_angle.py`).
//!
//! Three things are compared per object: every face's smooth flag, the set
//! of sharp edges, and the corner normals Blender draws with — each of
//! phiphx's triangle corners must carry one of Blender's corner normals at
//! that vertex, and each of Blender's must appear.

use phiphx::blend::BlendFile;
use phiphx::phx::Phx;
use std::collections::{BTreeSet, HashMap};
use std::path::{Path, PathBuf};

struct Truth {
    verts: usize,
    digest: f64,
    smooth: Vec<bool>,
    sharp: BTreeSet<(u32, u32)>,
    /// Per vertex, Blender's corner normals there.
    corners: HashMap<u32, Vec<[f32; 3]>>,
}

fn truth(path: &Path) -> Vec<(String, Truth)> {
    let text = std::fs::read_to_string(path).expect("truth file");
    let mut out: Vec<(String, Truth)> = Vec::new();
    for line in text.lines() {
        let mut f = line.split(' ');
        let tag = f.next().unwrap_or("");
        match tag {
            "O" => {
                let name = f.next().unwrap().to_string();
                let verts = f.next().unwrap().parse().unwrap();
                out.push((name, Truth { verts, digest: 0.0, smooth: Vec::new(), sharp: BTreeSet::new(), corners: HashMap::new() }));
            }
            "V" => out.last_mut().unwrap().1.digest = f.next().unwrap().parse().unwrap(),
            "F" => out.last_mut().unwrap().1.smooth = f.next().unwrap_or("").chars().map(|c| c == '1').collect(),
            "S" => {
                out.last_mut().unwrap().1.sharp = f
                    .filter(|s| !s.is_empty())
                    .map(|s| {
                        let (a, b) = s.split_once('-').unwrap();
                        (a.parse().unwrap(), b.parse().unwrap())
                    })
                    .collect()
            }
            "C" => {
                let _face = f.next();
                let v: u32 = f.next().unwrap().parse().unwrap();
                let n: Vec<f32> = f.map(|x| x.parse().unwrap()).collect();
                out.last_mut().unwrap().1.corners.entry(v).or_default().push([n[0], n[1], n[2]]);
            }
            _ => {}
        }
    }
    out
}

fn close(a: [f32; 3], b: [f32; 3]) -> bool {
    (0..3).all(|k| (a[k] - b[k]).abs() < 2e-4)
}

/// Flags and sharp edges straight off the evaluated mesh, corner normals
/// off the drawn one.
///
/// `full` says the truth lists EVERY corner, so the normals at a vertex can
/// be compared as sets both ways; a truth dumped with a stride lists only
/// some of them, and then each of Blender's must be among ours but not the
/// other way round.
fn check(path: &Path, truth_path: &Path, full: bool) -> usize {
    let file = BlendFile::parse_bytes(&std::fs::read(path).unwrap()).unwrap();
    let scene = file.extract_scene().unwrap();
    let drawn = Phx::open(path).unwrap();
    let mut checked = 0;
    for (name, want) in truth(truth_path) {
        let i = scene.objects.iter().position(|o| o.name == name).unwrap_or_else(|| panic!("{name} in the file"));
        let eval = scene.props.evaluated.get(&i).unwrap_or_else(|| panic!("{name}: evaluated by its modifier stack"));
        assert_eq!(eval.vertices.len(), want.verts, "{name}: vertex count");
        let digest: f64 = eval.vertices.iter().enumerate().map(|(i, v)| (i as f64 + 1.0) * (v[0] as f64 + 2.0 * v[1] as f64 + 3.0 * v[2] as f64)).sum();
        assert!((digest - want.digest).abs() < 1e-2 * want.digest.abs().max(1.0), "{name}: vertices {digest} vs Blender's {}", want.digest);
        assert_eq!(eval.face_smooth, want.smooth, "{name}: face smooth flags");
        let sharp: BTreeSet<(u32, u32)> = eval.sharp_edges.iter().map(|&(a, b)| (a.min(b), a.max(b))).collect();
        assert_eq!(sharp, want.sharp, "{name}: sharp edges");
        // The corner normals, per vertex.
        let o = drawn.objects.iter().find(|o| o.name == name).unwrap();
        let mut ours: HashMap<u32, Vec<[f32; 3]>> = HashMap::new();
        for (c, &v) in o.mesh.indices.iter().enumerate() {
            let n = match (&o.mesh.normals, o.mesh.corners.is_empty()) {
                (_, false) => o.mesh.corners.normal(c),
                // All flat: the triangle's own normal stands for the corner.
                (phiphx::scene::SceneNormals::PerFace(f), true) => f[c / 3],
                (phiphx::scene::SceneNormals::PerVertex(n), true) => n[v as usize],
            };
            ours.entry(v).or_default().push(n);
        }
        for (v, theirs) in &want.corners {
            let mine = ours.get(v).unwrap_or_else(|| panic!("{name}: vertex {v} drawn"));
            for t in theirs {
                assert!(mine.iter().any(|m| close(*m, *t)), "{name}: vertex {v}: Blender's corner normal {t:?} not among ours {mine:?}");
            }
            for m in mine.iter().filter(|_| full) {
                assert!(theirs.iter().any(|t| close(*m, *t)), "{name}: vertex {v}: our corner normal {m:?} not among Blender's {theirs:?}");
            }
        }
        checked += 1;
    }
    checked
}

#[test]
fn smooth_by_angle_shades_as_blender_does() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests");
    let n = check(&root.join("fixtures/smooth_by_angle.blend"), &root.join("truth/smooth_by_angle.txt"), true);
    assert_eq!(n, 11, "every case in the truth");
}

/// The BOZO car's hood: Solidify, then the "Auto Smooth" modifier Blender
/// INVENTS while loading this pre-4.2 file from the mesh's legacy
/// auto-smooth flag (`modifier/nodes_legacy.rs`) — the file itself saves
/// only the Solidify. Every face flag and all 1312 sharp edges are
/// compared exactly; the committed truth samples every 37th corner normal
/// (`--stride 37`), so each of Blender's sampled normals must be among
/// ours at that vertex. A full dump (no stride, 28,400 corners) was also
/// run and matched as sets both ways; re-check it with
///
///     blender -b FILE --python tests/truth/blender_smooth_by_angle.py -- OUT CAR-HOOD
///     PHIPHX_FULL_TRUTH=1 cargo test -p phiphx --test smooth_by_angle_truth
///
/// The file lives outside the repo; skipped when it is not here.
#[test]
fn a_production_hood_shades_as_blender_does() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests");
    let file = PathBuf::from(
        "C:/Users/phiac/Downloads/likethis/PR-LOOKS-POSES-20260704T000429Z-3-003/PR-LOOKS-POSES/SHOWCASES/PR-SHOWCASE/BRONWYN-POSES/Cars/BOZO-CAR-VERION-01/BOZO-CAR.blend",
    );
    if !file.exists() {
        eprintln!("skipped: {} is not on this machine", file.display());
        return;
    }
    assert_eq!(check(&file, &root.join("truth/smooth_by_angle_bozo.txt"), std::env::var_os("PHIPHX_FULL_TRUTH").is_some()), 1);
}
