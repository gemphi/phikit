//! Curve objects against Blender's own meshes of them (`blend/curve`'s
//! display list through `blend/text/sweep`).
//!
//! `fixtures/curve_mesh.blend` (`fixtures/make_curve_mesh.py`) holds 3D
//! and flat Béziers, polys and NURBS with round, profile and object bevels,
//! extrusion, every fill mode, caps, a taper and an Array.
//! `truth/curve_mesh.txt` (`truth/blender_curve_mesh.py`, Blender 5.2) has,
//! for each, the mesh before its modifiers — counts, bounding box, every
//! vertex in order — and after them — counts and box.
//!
//! The vertex and triangle counts must agree exactly, every vertex within
//! 1e-4 of Blender's at the same index, and the boxes too. The triangles
//! themselves are not compared: Blender's scanfill picks its own diagonals.

use phiphx::Phx;
use std::collections::BTreeMap;
use std::path::PathBuf;

const TOL: f32 = 1e-4;

#[derive(Default, Debug)]
struct Truth {
    verts: usize,
    tris: usize,
    lo: [f32; 3],
    hi: [f32; 3],
    points: Vec<[f32; 3]>,
}

fn here(rel: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests").join(rel)
}

/// `OBJ name`, then `BASE` / `EVAL` lines, BASE with its `V` points.
fn truth() -> BTreeMap<String, (Truth, Truth)> {
    let text = std::fs::read_to_string(here("truth/curve_mesh.txt")).expect("truth/curve_mesh.txt");
    let mut out: BTreeMap<String, (Truth, Truth)> = BTreeMap::new();
    let (mut name, mut eval) = (String::new(), false);
    for line in text.lines() {
        let mut w = line.split_whitespace();
        let tag = w.next().unwrap_or("");
        let nums: Vec<f32> = w.filter_map(|s| s.parse().ok()).collect();
        match tag {
            "OBJ" => {
                name = line[4..].to_string();
                eval = false;
            }
            "BASE" | "EVAL" => {
                eval = tag == "EVAL";
                let entry = out.entry(name.clone()).or_default();
                let t = if eval { &mut entry.1 } else { &mut entry.0 };
                (t.verts, t.tris) = (nums[0] as usize, nums[1] as usize);
                t.lo = [nums[2], nums[3], nums[4]];
                t.hi = [nums[5], nums[6], nums[7]];
            }
            "V" if !eval => out.entry(name.clone()).or_default().0.points.push([nums[0], nums[1], nums[2]]),
            _ => {}
        }
    }
    out
}

fn bounds(v: &[[f32; 3]]) -> ([f32; 3], [f32; 3]) {
    v.iter().fold(([f32::MAX; 3], [f32::MIN; 3]), |(lo, hi), p| (std::array::from_fn(|i| lo[i].min(p[i])), std::array::from_fn(|i| hi[i].max(p[i]))))
}

fn box_error(ours: &[[f32; 3]], t: &Truth) -> f32 {
    let (lo, hi) = bounds(ours);
    (0..3).map(|i| (lo[i] - t.lo[i]).abs().max((hi[i] - t.hi[i]).abs())).fold(0.0, f32::max)
}

/// The largest distance between our vertex and Blender's at each index.
fn in_order(ours: &[[f32; 3]], theirs: &[[f32; 3]]) -> (f32, usize) {
    ours.iter()
        .zip(theirs)
        .enumerate()
        .map(|(i, (p, q))| (((p[0] - q[0]).powi(2) + (p[1] - q[1]).powi(2) + (p[2] - q[2]).powi(2)).sqrt(), i))
        .fold((0.0, 0), |a, b| if b.0 > a.0 { b } else { a })
}

#[test]
fn curve_objects_match_blenders_meshes() {
    let path = here("fixtures/curve_mesh.blend");
    let truth = truth();
    assert_eq!(truth.len(), 13, "thirteen curve objects in the truth");
    let (base, _) = Phx::open_progressive(&path, &|_, _| true).expect("progressive open");
    let eval = Phx::open(&path).expect("open");
    let mut failures = Vec::new();
    for (name, (tb, te)) in &truth {
        let ob = base.objects.iter().find(|o| &o.name == name).unwrap_or_else(|| panic!("{name} opened"));
        let ours = &ob.mesh.vertices;
        let tris = ob.mesh.indices.len() / 3;
        let (err, at) = in_order(ours, &tb.points);
        let boxed = box_error(ours, tb);
        eprintln!("{name}: base {} verts / {tris} tris (Blender {} / {}), vertices within {err:.2e} (worst #{at}), box {boxed:.2e}", ours.len(), tb.verts, tb.tris);
        if ours.len() != tb.verts || tris != tb.tris || err > TOL || boxed > TOL {
            // `CURVE_DEBUG=1` prints the vertices either side about the worst.
            if std::env::var("CURVE_DEBUG").is_ok() {
                let n = ours.len().min(tb.points.len());
                for i in (0..3.min(n)).chain(at.saturating_sub(4)..(at + 4).min(n)).chain(n.saturating_sub(3)..n) {
                    eprintln!("  #{i}: ours {:?} Blender {:?}", ours[i], tb.points[i]);
                }
            }
            failures.push(format!("{name} base"));
        }
        let ob = eval.objects.iter().find(|o| &o.name == name).expect("evaluated");
        let (n, tris) = (ob.mesh.vertices.len(), ob.mesh.indices.len() / 3);
        let boxed = box_error(&ob.mesh.vertices, te);
        eprintln!("{name}: evaluated {n} verts / {tris} tris (Blender {} / {}), box {boxed:.2e}", te.verts, te.tris);
        if n != te.verts || tris != te.tris || boxed > TOL {
            failures.push(format!("{name} evaluated"));
        }
    }
    assert!(failures.is_empty(), "differ from Blender: {failures:?}");
}

/// BC-15's two profile-bevelled roofs (`F1_Right_Roof_Split_GEO.013` and
/// `.018`, a closed 702-point poly each): Blender 5.2's evaluated counts.
/// Skipped where the file is absent; slow, so run it by name with
/// `--ignored --release`.
#[test]
#[ignore]
fn bc15_roofs_match_blender() {
    let path = PathBuf::from(r"C:\Users\phiac\Downloads\BC-15.blend");
    if !path.exists() {
        return;
    }
    let scene = Phx::open(&path).expect("open");
    for name in ["F1_Right_Roof_Split_GEO.013", "F1_Right_Roof_Split_GEO.018"] {
        let ob = scene.objects.iter().find(|o| o.name == name).unwrap_or_else(|| panic!("{name}"));
        let (n, t) = (ob.mesh.vertices.len(), ob.mesh.indices.len() / 3);
        eprintln!("{name}: {n} verts / {t} tris (Blender 14040 / 28080)");
        assert_eq!((n, t), (14040, 28080), "{name}: counts");
    }
}
