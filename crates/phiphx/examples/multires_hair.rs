//! How far phiphx's evaluated meshes are from Blender's, for objects a truth
//! file names â€” written for the Multiresolution modifier on the CW-POSES
//! hair (`face Hair`, `face Hair.001`: Armature then Multires).
//!
//! The truth file has one line per object and mode, tab-separated: name,
//! mode, vertex count, face count, then every coordinate (object space), as
//! Blender's `to_mesh()` gives them. Lines starting with `#` are skipped.
//!
//! ```text
//! cargo run --release -p phiphx --example multires_hair -- f.blend truth.txt full
//! PHIPHX_MODIFIERS=1 cargo run ... -- f.blend truth.txt armature
//! ```
//!
//! Prints, per object: both vertex counts, then the max and mean distance
//! from each Blender vertex to the nearest of phiphx's (and back), and the
//! same by index over the first `base vertex count` vertices (the base
//! mesh's, which both put first in one order).

use std::path::Path;

fn dist(p: &[f32; 3], q: &[f32; 3]) -> f32 {
    (0..3).map(|i| (p[i] - q[i]).powi(2)).sum::<f32>().sqrt()
}

fn nearest(a: &[[f32; 3]], b: &[[f32; 3]]) -> Vec<f32> {
    a.iter().map(|p| b.iter().map(|q| dist(p, q)).fold(f32::MAX, f32::min)).collect()
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (file, truth, mode) = match &args[..] {
        [file, truth, mode, ..] => (file, truth, mode),
        _ => {
            eprintln!("usage: multires_hair <file.blend> <truth.txt> <mode> [base vertex count]");
            std::process::exit(2);
        }
    };
    let base_count: usize = args.get(3).and_then(|b| b.parse().ok()).unwrap_or(0);
    let model = phiphx::Phx::open(Path::new(file)).expect("the file opens");
    let text = std::fs::read_to_string(truth).expect("truth file");
    for line in text.lines().filter(|l| !l.starts_with('#')) {
        let f: Vec<&str> = line.split('\t').collect();
        if f.len() < 5 || f[1] != mode {
            continue;
        }
        let nums: Vec<f32> = f[4].split(' ').filter_map(|x| x.parse().ok()).collect();
        let theirs: Vec<[f32; 3]> = nums.chunks_exact(3).map(|c| [c[0], c[1], c[2]]).collect();
        let Some(ob) = model.objects.iter().find(|o| o.name == f[0]) else {
            println!("{}: not in phiphx's scene", f[0]);
            continue;
        };
        let ours = &ob.mesh.vertices;
        let there = nearest(&theirs, ours);
        let back = nearest(ours, &theirs);
        let max = there.iter().chain(&back).copied().fold(0.0, f32::max);
        let off = back.iter().filter(|&&d| d > 1e-4).count();
        let there_max = there.iter().copied().fold(0.0, f32::max);
        println!("  blender->phiphx max {there_max:.3e}; phiphx vertices over 1e-4 from any of Blender's: {off}");
        let mean = there.iter().sum::<f32>() / there.len().max(1) as f32;
        let base = ours.len().min(theirs.len()).min(base_count);
        let first: Vec<f32> = (0..base).map(|i| dist(&ours[i], &theirs[i])).collect();
        let first_max = first.iter().copied().fold(0.0, f32::max);
        let first_mean = first.iter().sum::<f32>() / base.max(1) as f32;
        println!(
            "{} [{mode}]: verts phiphx {} blender {}, triangles phiphx {} blender quads {}; nearest max {max:.3e} mean {mean:.3e}; first {base} by index max {first_max:.3e} mean {first_mean:.3e}",
            f[0],
            ours.len(),
            theirs.len(),
            ob.mesh.indices.len() / 3,
            f[3]
        );
    }
}
