//! Every object of a file against a dump of Blender's (`verts` lines:
//! evaluated vertices in object space, `world` lines: world matrices), as
//! `tests/truth/blender_eval_dump.py` writes them — the largest and the
//! mean distance between same-index vertices, and the largest matrix
//! element difference. Written for the deform modifiers and vertex
//! parenting (`modifier/simple_deform.rs`, `surface_deform.rs`,
//! `cloth.rs`, `object/vertex_parent.rs`), whose real uses are in files
//! too big to be tests.
//!
//! ```text
//! cargo run --release -p phiphx --no-default-features --example truth_check -- \
//!     f.blend truth.tsv [tolerance] [--progressive] [--only name,name] [--nearest]
//! ```
//!
//! `--nearest` measures each of Blender's vertices to the closest of ours
//! rather than to the same index — the comparison
//! `tests/solidify_creases_truth.rs` makes, for a mesh whose generative
//! modifiers may emit the same surface in another order.
//!
//! Exits 1 when any named object is missing, differs in vertex count, or
//! is off by more than the tolerance (default 1e-4).

use glam::Mat4;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (mut path, mut truth): (Option<&str>, Option<&str>) = (None, None);
    let (mut tol, mut progressive, mut only): (f32, bool, Vec<String>) = (1e-4, false, Vec::new());
    let mut nearest = false;
    let mut k = 0;
    while k < args.len() {
        match args[k].as_str() {
            "--progressive" => progressive = true,
            "--nearest" => nearest = true,
            "--only" => {
                k += 1;
                only = args.get(k).map(|s| s.split(',').map(str::to_string).collect()).unwrap_or_default();
            }
            a if path.is_none() => path = Some(a),
            a if truth.is_none() => truth = Some(a),
            a => tol = a.parse().expect("a tolerance"),
        }
        k += 1;
    }
    let (Some(path), Some(truth)) = (path, truth) else {
        eprintln!("usage: truth_check <file.blend> <truth.tsv> [tolerance] [--progressive] [--only a,b]");
        std::process::exit(2);
    };
    let text = std::fs::read_to_string(truth).expect("truth file");
    let p = std::path::Path::new(path);
    let model = match progressive {
        true => {
            let (mut base, deferred) = phiphx::Phx::open_progressive(p, &|_, _| true).expect("the file opens");
            if let Some(d) = deferred {
                base.apply_deferred(d.finish());
            }
            base
        }
        false => phiphx::Phx::open(p).expect("the file opens"),
    };
    let mut bad = 0;
    let mut n = 0;
    for line in text.lines() {
        let mut it = line.split('\t');
        let (Some(kind), Some(name), Some(count), Some(nums)) = (it.next(), it.next(), it.next(), it.next()) else { continue };
        if !only.is_empty() && !only.iter().any(|o| o == name) {
            continue;
        }
        let nums: Vec<f32> = nums.split(' ').filter_map(|x| x.parse().ok()).collect();
        n += 1;
        match kind {
            "verts" => {
                let Some(ours) = model.objects.iter().find(|o| o.name == name).map(|o| &o.mesh.vertices) else {
                    println!("{name}: MISSING ({count} vertices in Blender)");
                    bad += 1;
                    continue;
                };
                let theirs: Vec<[f32; 3]> = nums.chunks_exact(3).map(|c| [c[0], c[1], c[2]]).collect();
                if ours.len() != theirs.len() {
                    println!("{name}: {} vertices here, {} in Blender", ours.len(), theirs.len());
                    bad += 1;
                    continue;
                }
                let d: Vec<f32> = match nearest {
                    false => ours.iter().zip(&theirs).map(|(a, b)| (0..3).map(|i| (a[i] - b[i]).powi(2)).sum::<f32>().sqrt()).collect(),
                    true => theirs.iter().map(|b| ours.iter().map(|a| (0..3).map(|i| (a[i] - b[i]).powi(2)).sum::<f32>().sqrt()).fold(f32::MAX, f32::min)).collect(),
                };
                let max = d.iter().copied().fold(0.0, f32::max);
                let mean = d.iter().sum::<f32>() / d.len().max(1) as f32;
                let worst = d.iter().position(|&x| x == max).unwrap_or(0);
                let flag = match max > tol {
                    true => {
                        bad += 1;
                        " <-- off"
                    }
                    false => "",
                };
                println!("{name}: {} vertices, max {max:.2e} (vertex {worst}) mean {mean:.2e}{flag}", ours.len());
            }
            "world" => {
                let ours = model
                    .objects
                    .iter()
                    .find(|o| o.name == name)
                    .map(|o| o.matrix_world)
                    .or_else(|| model.empties.iter().find(|e| e.name == name).map(|e| e.matrix_world))
                    .or_else(|| model.cameras.iter().find(|c| c.name == name).map(|c| c.matrix_world))
                    .or_else(|| model.lights.iter().find(|l| l.name == name).map(|l| l.matrix_world));
                let Some(ours) = ours else {
                    println!("{name}: world MISSING");
                    bad += 1;
                    continue;
                };
                // Blender's dump is row-major; ours is `m[col][row]`.
                let theirs = Mat4::from_cols_array_2d(&std::array::from_fn(|c| std::array::from_fn(|r| nums.get(r * 4 + c).copied().unwrap_or(0.0))));
                let ours = Mat4::from_cols_array_2d(&ours);
                let max = ours.to_cols_array().iter().zip(theirs.to_cols_array()).map(|(a, b)| (a - b).abs()).fold(0.0, f32::max);
                let flag = match max > tol {
                    true => {
                        bad += 1;
                        " <-- off"
                    }
                    false => "",
                };
                println!("{name}: world max {max:.2e}{flag}");
            }
            _ => {}
        }
    }
    println!("{n} checked, {bad} off (tolerance {tol:e})");
    std::process::exit((bad > 0) as i32);
}
