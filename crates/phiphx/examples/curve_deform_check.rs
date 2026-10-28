//! Compare phiphx's evaluated vertices with a Blender dump, object by object:
//! max and mean distance, vertex by vertex when the counts agree.
//!
//! The dump is one line per object — `name \t count \t x y z x y z …` in
//! object space, from `ob.evaluated_get(depsgraph).to_mesh()` — the shape
//! `tests/fixtures/make_curve_deform.py` writes. Written to check the Curve
//! modifier on the CW-POSES zips and stitching:
//!
//! ```text
//! cargo run --release -p phiphx --no-default-features --example curve_deform_check -- f.blend dump.txt
//! ```

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let [file, dump] = &args[..] else {
        eprintln!("usage: curve_deform_check <file.blend> <dump.txt>");
        std::process::exit(2);
    };
    let t = std::time::Instant::now();
    let model = phiphx::phx::Phx::open(std::path::Path::new(file)).expect("open");
    eprintln!("opened in {:?}", t.elapsed());
    for line in std::fs::read_to_string(dump).expect("dump").lines() {
        let mut it = line.split('\t');
        let (Some(name), Some(_), Some(v)) = (it.next(), it.next(), it.next()) else { continue };
        let nums: Vec<f32> = v.split(' ').filter_map(|x| x.parse().ok()).collect();
        let theirs: Vec<[f32; 3]> = nums.chunks_exact(3).map(|c| [c[0], c[1], c[2]]).collect();
        let Some(o) = model.objects.iter().find(|o| o.name == name) else {
            println!("{name}: not in phiphx's scene");
            continue;
        };
        let ours = &o.mesh.vertices;
        if ours.len() != theirs.len() {
            println!("{name}: {} vertices, Blender {}", ours.len(), theirs.len());
            continue;
        }
        let d = |p: &[f32; 3], q: &[f32; 3]| (0..3).map(|i| (p[i] - q[i]).powi(2)).sum::<f32>().sqrt();
        let errors: Vec<f32> = ours.iter().zip(&theirs).map(|(a, b)| d(a, b)).collect();
        let max = errors.iter().copied().fold(0.0, f32::max);
        let mean = errors.iter().sum::<f32>() / errors.len().max(1) as f32;
        let (mut lo, mut hi) = ([f32::MAX; 3], [f32::MIN; 3]);
        for p in &theirs {
            for i in 0..3 {
                lo[i] = lo[i].min(p[i]);
                hi[i] = hi[i].max(p[i]);
            }
        }
        println!("{name}: {} vertices, max {max:.3e}, mean {mean:.3e}, size {:.4}", ours.len(), d(&lo, &hi));
    }
}
