//! One object's evaluated vertices against a dump of Blender's, by index:
//! the max and mean distance. Written for the Shrinkwrap port
//! (`blend/modifier/shrinkwrap.rs`) on files too big to keep as fixtures.
//!
//! ```text
//! PHIPHX_MODIFIERS=1 cargo run --release -p phiphx --no-default-features \
//!     --example shrinkwrap_check -- file.blend "BSurfaceMesh.002" blender_verts.txt
//! ```
//!
//! The dump is one `x y z` line per vertex, in object space, as
//! `ob.evaluated_get(depsgraph).to_mesh()` gives them (with the modifiers
//! after the one under test muted in Blender, and `PHIPHX_MODIFIERS` keeping
//! the same prefix here).

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let [file, name, dump] = args.as_slice() else {
        eprintln!("usage: shrinkwrap_check <file.blend> <object> <blender_verts.txt>");
        std::process::exit(2);
    };
    let theirs: Vec<[f32; 3]> = std::fs::read_to_string(dump)
        .expect("the dump")
        .lines()
        .filter_map(|l| {
            let v: Vec<f32> = l.split_whitespace().filter_map(|x| x.parse().ok()).collect();
            (v.len() == 3).then(|| [v[0], v[1], v[2]])
        })
        .collect();
    let started = std::time::Instant::now();
    let model = phiphx::Phx::open(std::path::Path::new(file)).expect("the file opens");
    let Some(ob) = model.objects.iter().find(|o| &o.name == name) else {
        eprintln!("{name}: not in the file");
        std::process::exit(1);
    };
    let ours = &ob.mesh.vertices;
    println!("{name}: phiphx {} vertices, Blender {} (opened in {:.1?})", ours.len(), theirs.len(), started.elapsed());
    // By index only when the counts agree; the order-free measure below either way.
    let errs: Vec<f32> = ours.iter().zip(&theirs).take(match ours.len() == theirs.len() { true => ours.len(), false => 0 }).map(|(a, b)| (0..3).map(|k| (a[k] - b[k]).powi(2)).sum::<f32>().sqrt()).collect();
    let max = errs.iter().copied().fold(0.0, f32::max);
    let mean = errs.iter().sum::<f32>() / errs.len().max(1) as f32;
    println!("by index: max {max:.3e}  mean {mean:.3e}");
    // Order-free: each of ours to the nearest of Blender's and back, which
    // tells a vertex-order difference from a shape difference.
    let d = |p: &[f32; 3], q: &[f32; 3]| (0..3).map(|k| (p[k] - q[k]).powi(2)).sum::<f32>().sqrt();
    let near = |a: &[[f32; 3]], b: &[[f32; 3]]| -> Vec<f32> { a.iter().map(|p| b.iter().map(|q| d(p, q)).fold(f32::MAX, f32::min)).collect() };
    let both: Vec<f32> = near(ours, &theirs).into_iter().chain(near(&theirs, ours)).collect();
    let nmax = both.iter().copied().fold(0.0, f32::max);
    let nmean = both.iter().sum::<f32>() / both.len().max(1) as f32;
    println!("nearest: max {nmax:.3e}  mean {nmean:.3e}");
}
