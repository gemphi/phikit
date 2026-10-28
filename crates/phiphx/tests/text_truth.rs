//! Text objects against Blender's own meshes of them (`blend/text`).
//!
//! `fixtures/text.blend` (`fixtures/make_text.py`) holds three texts in the
//! builtin font: extruded and bevelled, flat under a Solidify, underlined
//! with small caps. `truth/text.txt` (`truth/blender_text.py`, Blender
//! 5.2.3) has, for each, the mesh before its modifiers — counts, bounding
//! box, every vertex — and after them — counts and box.
//!
//! The outline points and the box must agree within 1e-4, the vertex and
//! triangle counts exactly. The triangles themselves are not compared:
//! Blender's scanfill picks its own diagonals. (`TEXT_DEBUG=1` prints the
//! points either side has that the other lacks.)

use phiphx::Phx;
use std::collections::HashMap;
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

/// `OBJ name`, then `BASE` / `EVAL` lines each with their `V` points.
fn truth() -> HashMap<String, (Truth, Truth)> {
    let text = std::fs::read_to_string(here("truth/text.txt")).expect("truth/text.txt");
    let mut out: HashMap<String, (Truth, Truth)> = HashMap::new();
    let (mut name, mut eval) = (String::new(), false);
    for line in text.lines() {
        let mut w = line.split_whitespace();
        let tag = w.next().unwrap_or("");
        let nums: Vec<f32> = w.clone().filter_map(|s| s.parse().ok()).collect();
        let entry = out.entry(name.clone()).or_default();
        let t = if eval { &mut entry.1 } else { &mut entry.0 };
        match tag {
            "OBJ" => name = line[4..].to_string(),
            "BASE" | "EVAL" => {
                eval = tag == "EVAL";
                let t = if eval { &mut entry.1 } else { &mut entry.0 };
                (t.verts, t.tris) = (nums[0] as usize, nums[1] as usize);
                t.lo = [nums[2], nums[3], nums[4]];
                t.hi = [nums[5], nums[6], nums[7]];
            }
            "V" => t.points.push([nums[0], nums[1], nums[2]]),
            _ => {}
        }
    }
    out.remove("");
    out
}

fn bounds(v: &[[f32; 3]]) -> ([f32; 3], [f32; 3]) {
    v.iter().fold(([f32::MAX; 3], [f32::MIN; 3]), |(lo, hi), p| (std::array::from_fn(|i| lo[i].min(p[i])), std::array::from_fn(|i| hi[i].max(p[i]))))
}

/// The largest distance from a point of `a` to the nearest point of `b`.
fn reach(a: &[[f32; 3]], b: &[[f32; 3]]) -> f32 {
    let cell = |p: &[f32; 3]| p.map(|x| (x / 0.01).floor() as i32);
    let mut grid: HashMap<[i32; 3], Vec<[f32; 3]>> = HashMap::new();
    for p in b {
        grid.entry(cell(p)).or_default().push(*p);
    }
    let d = |p: &[f32; 3], q: &[f32; 3]| ((p[0] - q[0]).powi(2) + (p[1] - q[1]).powi(2) + (p[2] - q[2]).powi(2)).sqrt();
    a.iter()
        .map(|p| {
            let c = cell(p);
            let mut best = f32::MAX;
            for dx in -1..=1 {
                for dy in -1..=1 {
                    for dz in -1..=1 {
                        for q in grid.get(&[c[0] + dx, c[1] + dy, c[2] + dz]).into_iter().flatten() {
                            best = best.min(d(p, q));
                        }
                    }
                }
            }
            best
        })
        .fold(0.0, f32::max)
}

fn check_box(name: &str, stage: &str, ours: &[[f32; 3]], t: &Truth) {
    let (lo, hi) = bounds(ours);
    for i in 0..3 {
        assert!((lo[i] - t.lo[i]).abs() < TOL && (hi[i] - t.hi[i]).abs() < TOL, "{name} {stage} box {lo:?} {hi:?} vs Blender {:?} {:?}", t.lo, t.hi);
    }
}

#[test]
fn text_objects_match_blenders_meshes() {
    let path = here("fixtures/text.blend");
    let truth = truth();
    assert_eq!(truth.len(), 3, "three text objects in the truth");
    let (base, _) = Phx::open_progressive(&path, &|_, _| true).expect("progressive open");
    let eval = Phx::open(&path).expect("open");
    for (name, (tb, te)) in &truth {
        let ob = base.objects.iter().find(|o| &o.name == name).unwrap_or_else(|| panic!("{name} opened"));
        let ours = &ob.mesh.vertices;
        let tris = ob.mesh.indices.len() / 3;
        eprintln!("{name}: base {} verts / {tris} tris (Blender {} / {})", ours.len(), tb.verts, tb.tris);
        if std::env::var("TEXT_DEBUG").is_ok() {
            let far: Vec<_> = tb.points.iter().filter(|p| reach(&[**p], ours) > TOL).take(40).collect();
            eprintln!("unmatched Blender points: {far:?}");
            let far: Vec<_> = ours.iter().filter(|p| reach(&[**p], &tb.points) > TOL).take(40).collect();
            eprintln!("unmatched ours: {far:?}");
        }
        assert_eq!(ours.len(), tb.verts, "{name}: base vertex count");
        check_box(name, "base", ours, tb);
        let (there, back) = (reach(&tb.points, ours), reach(ours, &tb.points));
        eprintln!("{name}: outline points within {there:.2e} / {back:.2e}");
        assert!(there < TOL && back < TOL, "{name}: Blender's points to ours {there}, ours to Blender's {back}");
        assert_eq!(tris, tb.tris, "{name}: base triangle count");

        let ob = eval.objects.iter().find(|o| &o.name == name).expect("evaluated");
        let (n, tris) = (ob.mesh.vertices.len(), ob.mesh.indices.len() / 3);
        eprintln!("{name}: evaluated {n} verts / {tris} tris (Blender {} / {})", te.verts, te.tris);
        assert_eq!(n, te.verts, "{name}: evaluated vertex count");
        check_box(name, "evaluated", &ob.mesh.vertices, te);
        assert_eq!(tris, te.tris, "{name}: evaluated triangle count");
    }
}

/// The CW-POSES scene's two text objects (`Boldlandmain-Regular.ttf`, an
/// on-disk TrueType beside the `.blend`'s folders, under a Solidify):
/// Blender 5.2.3's evaluated counts and boxes, from
/// `truth/blender_text.py` run over the file. Skipped where the file is
/// absent; slow (281 MB), so run it by name with `--ignored --release`.
#[test]
#[ignore]
fn cw_poses_texts_match_blender() {
    let path = PathBuf::from(r"C:\Users\phiac\Downloads\phiber\CW-POSES\CHATTY-POSES-RENDER-SCENE-03_Duo pose 26_1.blend");
    if !path.exists() {
        return;
    }
    let scene = Phx::open(&path).expect("open");
    let blender = [
        ("Text.004", 5910, 11804, [-1.623144, 0.006987, -0.1], [1.628384, 0.654149, 0.0]),
        ("Text.005", 4096, 8184, [-0.844542, 0.016594, -0.1], [0.847162, 0.645415, 0.0]),
    ];
    for (name, verts, tris, lo, hi) in blender {
        let ob = scene.objects.iter().find(|o| o.name == name).unwrap_or_else(|| panic!("{name}"));
        let (n, t) = (ob.mesh.vertices.len(), ob.mesh.indices.len() / 3);
        let (l, h) = bounds(&ob.mesh.vertices);
        let err = (0..3).map(|i| (l[i] - lo[i]).abs().max((h[i] - hi[i]).abs())).fold(0.0, f32::max);
        eprintln!("{name}: {n} verts / {t} tris (Blender {verts} / {tris}), box {l:?} {h:?}, off by {err:.2e}");
        assert_eq!((n, t), (verts, tris), "{name}: counts");
        assert!(err < TOL, "{name}: box off by {err}");
    }
}
